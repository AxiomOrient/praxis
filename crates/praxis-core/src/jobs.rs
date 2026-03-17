use anyhow::{anyhow, Context, Result};
use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::create::{augment_draft_with_response, draft_root_path};
use crate::evaluation::{
    complete_ai_benchmark_run, fail_benchmark_run, mark_benchmark_run_running, submit_human_review,
};
use crate::executor::run_prompt;
use crate::library::{ensure_library_store, sync_catalog_to_library};
use crate::model::{ExternalExecutorConfig, JobSnapshot, JobSummary};
use crate::parser::parse_source_input;
use crate::source::scan_source;
use crate::workspace::WorkspacePaths;

const LEASE_SECONDS: i64 = 300;
const KIND_AI_BENCHMARK: &str = "benchmark-ai-judge";
const KIND_HUMAN_REVIEW: &str = "benchmark-human-review";
const KIND_AUGMENT_DRAFT: &str = "augment-draft";

#[derive(Debug, Serialize, Deserialize)]
pub struct AiBenchmarkJobPayload {
    pub run_id: String,
    pub suite_id: String,
    pub source: String,
    pub baseline_source_id: Option<String>,
    pub executor: ExternalExecutorConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HumanReviewJobPayload {
    pub run_id: String,
    pub decision: String,
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AugmentDraftJobPayload {
    pub draft_id: String,
    pub prompt: String,
    pub executor: ExternalExecutorConfig,
}

#[derive(Debug)]
struct JobRecord {
    id: String,
    kind: String,
    status: String,
    subject_id: String,
    summary: String,
    leased_by_session: Option<String>,
    lease_expires_at: Option<String>,
    attempts: usize,
    payload_json: String,
    last_error: Option<String>,
    log_path: String,
    created_at: String,
    updated_at: String,
}

pub fn ensure_jobs_store(paths: &WorkspacePaths) -> Result<()> {
    ensure_library_store(paths)?;
    fs::create_dir_all(&paths.jobs_dir)
        .with_context(|| format!("failed to create {}", paths.jobs_dir.display()))?;
    let conn = open_connection(paths)?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS jobs (
            job_id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            status TEXT NOT NULL,
            subject_id TEXT NOT NULL,
            summary TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            leased_by_session TEXT,
            lease_expires_at TEXT,
            attempts INTEGER NOT NULL DEFAULT 0,
            last_error TEXT,
            log_path TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_jobs_status_updated
            ON jobs(status, updated_at DESC);
        ",
    )
    .context("initialize jobs schema")?;
    Ok(())
}

pub fn read_job_snapshot(paths: &WorkspacePaths) -> Result<JobSnapshot> {
    ensure_jobs_store(paths)?;
    let conn = open_connection(paths)?;
    let queued = count_status(&conn, "queued")?;
    let leased = count_status(&conn, "leased")?;
    let running = count_status(&conn, "running")?;
    let failed = count_status(&conn, "failed")?;
    let mut stmt = conn
        .prepare(
            "SELECT job_id, kind, status, subject_id, summary, leased_by_session, lease_expires_at,
                    attempts, last_error, log_path, created_at, updated_at
             FROM jobs
             ORDER BY updated_at DESC
             LIMIT 12",
        )
        .context("prepare recent jobs query")?;
    let recent_jobs = stmt
        .query_map([], |row| {
            Ok(JobSummary {
                id: row.get(0)?,
                kind: row.get(1)?,
                status: row.get(2)?,
                subject_id: row.get(3)?,
                summary: row.get(4)?,
                leased_by_session: row.get(5)?,
                lease_expires_at: row.get(6)?,
                attempts: row.get::<_, i64>(7)? as usize,
                last_error: row.get(8)?,
                log_path: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .context("query recent jobs")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("collect recent jobs")?;

    Ok(JobSnapshot {
        queued: queued + leased,
        running,
        failed,
        recent_jobs,
    })
}

pub fn enqueue_ai_benchmark_job(
    paths: &WorkspacePaths,
    subject_id: &str,
    suite_id: &str,
    source: &str,
    baseline_source_id: Option<&str>,
    executor: ExternalExecutorConfig,
) -> Result<JobSummary> {
    enqueue_job(
        paths,
        KIND_AI_BENCHMARK,
        subject_id,
        &format!("AI judge queued for benchmark run {subject_id}"),
        &AiBenchmarkJobPayload {
            run_id: subject_id.to_string(),
            suite_id: suite_id.to_string(),
            source: source.to_string(),
            baseline_source_id: baseline_source_id.map(ToOwned::to_owned),
            executor,
        },
    )
}

pub fn enqueue_human_review_job(
    paths: &WorkspacePaths,
    run_id: &str,
    decision: &str,
    note: &str,
) -> Result<JobSummary> {
    enqueue_job(
        paths,
        KIND_HUMAN_REVIEW,
        run_id,
        &format!("Human review queued for benchmark run {run_id}"),
        &HumanReviewJobPayload {
            run_id: run_id.to_string(),
            decision: decision.to_string(),
            note: note.to_string(),
        },
    )
}

pub fn enqueue_augment_draft_job(
    paths: &WorkspacePaths,
    draft_id: &str,
    prompt: &str,
    executor: ExternalExecutorConfig,
) -> Result<JobSummary> {
    enqueue_job(
        paths,
        KIND_AUGMENT_DRAFT,
        draft_id,
        &format!("Augment draft queued for {draft_id}"),
        &AugmentDraftJobPayload {
            draft_id: draft_id.to_string(),
            prompt: prompt.to_string(),
            executor,
        },
    )
}

pub fn cancel_job(paths: &WorkspacePaths, job_id: &str) -> Result<JobSummary> {
    ensure_jobs_store(paths)?;
    let conn = open_connection(paths)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE jobs
         SET status = 'cancelled',
             summary = 'Cancelled by operator',
             updated_at = ?2,
             leased_by_session = NULL,
             lease_expires_at = NULL
         WHERE job_id = ?1",
        params![job_id, &now],
    )
    .with_context(|| format!("cancel job '{job_id}'"))?;
    read_job(paths, job_id)
}

pub fn retry_job(paths: &WorkspacePaths, job_id: &str) -> Result<JobSummary> {
    ensure_jobs_store(paths)?;
    let conn = open_connection(paths)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE jobs
         SET status = 'queued',
             summary = 'Retry queued',
             updated_at = ?2,
             leased_by_session = NULL,
             lease_expires_at = NULL,
             last_error = NULL
         WHERE job_id = ?1",
        params![job_id, &now],
    )
    .with_context(|| format!("retry job '{job_id}'"))?;
    read_job(paths, job_id)
}

pub fn work_jobs(
    paths: &WorkspacePaths,
    session_id: Option<&str>,
    max_jobs: Option<usize>,
) -> Result<JobSnapshot> {
    ensure_jobs_store(paths)?;
    let session_id = session_id
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("worker-{}", Utc::now().timestamp()));
    let max_jobs = max_jobs.unwrap_or(1).max(1);

    for _ in 0..max_jobs {
        let Some(job) = lease_next_job(paths, &session_id)? else {
            break;
        };
        let _ = process_job(paths, &session_id, job);
    }

    read_job_snapshot(paths)
}

fn enqueue_job<T: Serialize>(
    paths: &WorkspacePaths,
    kind: &str,
    subject_id: &str,
    summary: &str,
    payload: &T,
) -> Result<JobSummary> {
    ensure_jobs_store(paths)?;
    let job_id = new_job_id(kind, subject_id);
    let log_path = paths.jobs_dir.join(format!("{job_id}.log"));
    let now = Utc::now().to_rfc3339();
    fs::write(&log_path, format!("[{now}] queued {kind}\n"))
        .with_context(|| format!("failed to write {}", log_path.display()))?;
    let conn = open_connection(paths)?;
    conn.execute(
        "INSERT INTO jobs (
            job_id, kind, status, subject_id, summary, payload_json,
            leased_by_session, lease_expires_at, attempts, last_error, log_path, created_at, updated_at
        ) VALUES (?1, ?2, 'queued', ?3, ?4, ?5, NULL, NULL, 0, NULL, ?6, ?7, ?7)",
        params![
            &job_id,
            kind,
            subject_id,
            summary,
            serde_json::to_string(payload).context("serialize job payload")?,
            log_path.to_string_lossy().to_string(),
            &now,
        ],
    )
    .with_context(|| format!("insert job '{job_id}'"))?;
    read_job(paths, &job_id)
}

fn process_job(paths: &WorkspacePaths, session_id: &str, job: JobRecord) -> Result<()> {
    append_log(paths, &job.log_path, &format!("leased by {session_id}"))?;
    set_job_status(
        paths,
        &job.id,
        "running",
        &job.summary,
        None,
        Some(session_id),
    )?;

    let outcome = match job.kind.as_str() {
        KIND_AI_BENCHMARK => process_ai_benchmark_job(paths, &job),
        KIND_HUMAN_REVIEW => process_human_review_job(paths, &job),
        KIND_AUGMENT_DRAFT => process_augment_draft_job(paths, &job),
        other => Err(anyhow!("unsupported job kind '{other}'")),
    };

    match outcome {
        Ok(summary) => {
            append_log(paths, &job.log_path, &summary)?;
            set_job_status(paths, &job.id, "succeeded", &summary, None, None)?;
            Ok(())
        }
        Err(err) => {
            let message = err.to_string();
            append_log(paths, &job.log_path, &format!("failed: {message}"))?;
            set_job_status(paths, &job.id, "failed", &job.summary, Some(&message), None)?;
            if job.kind == KIND_AI_BENCHMARK {
                let payload: AiBenchmarkJobPayload = serde_json::from_str(&job.payload_json)
                    .context("decode ai benchmark payload")?;
                let _ = fail_benchmark_run(paths, &payload.run_id, &message, Some(&job.log_path));
            }
            Err(err)
        }
    }
}

fn process_ai_benchmark_job(paths: &WorkspacePaths, job: &JobRecord) -> Result<String> {
    let payload: AiBenchmarkJobPayload =
        serde_json::from_str(&job.payload_json).context("decode ai benchmark payload")?;
    mark_benchmark_run_running(paths, &payload.run_id)?;

    let source = parse_source_input(&payload.source)?;
    let catalog = scan_source(&source, &paths.cache_dir)?;
    sync_catalog_to_library(paths, &catalog, "ai-judge")?;
    let prompt = format!(
        "Review this skill source for promotion readiness.\nReturn strict JSON with keys recommendation, score, summary.\nRecommendation must be one of promote, hold, reject, manual_review.\nSkills: {}\nDecks: {}\nAgent file templates: {}\nWarnings: {}\nNotes: {}\n",
        catalog.skills.len(),
        catalog.decks.len(),
        catalog.agent_file_templates.len(),
        catalog.warnings.join("; "),
        catalog.notes.join("; "),
    );
    let assistant_text = run_prompt(
        PathBuf::from(&catalog.checkout_root).as_path(),
        &prompt,
        Some(&payload.executor),
    )?;
    let evidence_path = paths.jobs_dir.join(format!("{}.evidence.md", job.id));
    fs::write(&evidence_path, &assistant_text)
        .with_context(|| format!("failed to write {}", evidence_path.display()))?;
    let parsed = parse_ai_judge_output(&assistant_text);
    let updated = complete_ai_benchmark_run(
        paths,
        &payload.run_id,
        &parsed.recommendation,
        parsed.score,
        &parsed.summary,
        Some(&evidence_path.to_string_lossy()),
    )?;
    Ok(format!(
        "{} -> {} ({:.1})",
        updated.id, updated.recommendation, updated.score
    ))
}

fn process_human_review_job(paths: &WorkspacePaths, job: &JobRecord) -> Result<String> {
    let payload: HumanReviewJobPayload =
        serde_json::from_str(&job.payload_json).context("decode human review payload")?;
    let run = submit_human_review(paths, &payload.run_id, &payload.decision, &payload.note)?;
    Ok(format!(
        "{} -> {} ({})",
        run.id,
        run.recommendation,
        run.review_decision.unwrap_or_else(|| "none".to_string())
    ))
}

fn process_augment_draft_job(paths: &WorkspacePaths, job: &JobRecord) -> Result<String> {
    let payload: AugmentDraftJobPayload =
        serde_json::from_str(&job.payload_json).context("decode augment payload")?;
    let prompt = format!(
        "Review the current draft in this working directory and propose one augmentation.\nReturn plain markdown with concrete additions only.\nTask: {}\n",
        payload.prompt.trim()
    );
    let draft_root = draft_root_path(paths, &payload.draft_id)?;
    let assistant_text = run_prompt(&draft_root, &prompt, Some(&payload.executor))?;
    let preview =
        augment_draft_with_response(paths, &payload.draft_id, &payload.prompt, &assistant_text)?;
    Ok(format!("created augmented draft {}", preview.draft.id))
}

fn lease_next_job(paths: &WorkspacePaths, session_id: &str) -> Result<Option<JobRecord>> {
    ensure_jobs_store(paths)?;
    let mut conn = open_connection(paths)?;
    let tx = conn.transaction().context("start lease transaction")?;
    let now = Utc::now();
    let now_rfc = now.to_rfc3339();
    let reclaimed_cutoff = now.to_rfc3339();
    let job = tx
        .query_row(
            "SELECT job_id, kind, status, subject_id, summary, leased_by_session, lease_expires_at,
                    attempts, payload_json, last_error, log_path, created_at, updated_at
             FROM jobs
             WHERE status = 'queued'
                OR (status IN ('leased', 'running') AND lease_expires_at IS NOT NULL AND lease_expires_at <= ?1)
             ORDER BY created_at ASC
             LIMIT 1",
            params![&reclaimed_cutoff],
            |row| {
                Ok(JobRecord {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    status: row.get(2)?,
                    subject_id: row.get(3)?,
                    summary: row.get(4)?,
                    leased_by_session: row.get(5)?,
                    lease_expires_at: row.get(6)?,
                    attempts: row.get::<_, i64>(7)? as usize,
                    payload_json: row.get(8)?,
                    last_error: row.get(9)?,
                    log_path: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            },
        )
        .optional()
        .context("select leaseable job")?;

    let Some(job) = job else {
        tx.commit().context("commit empty lease transaction")?;
        return Ok(None);
    };

    let lease_expires_at = (now + Duration::seconds(LEASE_SECONDS)).to_rfc3339();
    tx.execute(
        "UPDATE jobs
         SET status = 'leased',
             leased_by_session = ?2,
             lease_expires_at = ?3,
             attempts = attempts + 1,
             updated_at = ?4
         WHERE job_id = ?1",
        params![&job.id, session_id, &lease_expires_at, &now_rfc],
    )
    .with_context(|| format!("lease job '{}'", job.id))?;
    tx.commit().context("commit lease transaction")?;

    read_job_record(paths, &job.id).map(Some)
}

fn set_job_status(
    paths: &WorkspacePaths,
    job_id: &str,
    status: &str,
    summary: &str,
    last_error: Option<&str>,
    lease_owner: Option<&str>,
) -> Result<()> {
    ensure_jobs_store(paths)?;
    let conn = open_connection(paths)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE jobs
         SET status = ?2,
             summary = ?3,
             last_error = ?4,
             leased_by_session = ?5,
             lease_expires_at = CASE WHEN ?2 IN ('leased', 'running') THEN lease_expires_at ELSE NULL END,
             updated_at = ?6
         WHERE job_id = ?1",
        params![job_id, status, summary, last_error, lease_owner, &now],
    )
    .with_context(|| format!("set job '{job_id}' status"))?;
    Ok(())
}

fn read_job(paths: &WorkspacePaths, job_id: &str) -> Result<JobSummary> {
    let record = read_job_record(paths, job_id)?;
    Ok(JobSummary {
        id: record.id,
        kind: record.kind,
        status: record.status,
        subject_id: record.subject_id,
        summary: record.summary,
        leased_by_session: record.leased_by_session,
        lease_expires_at: record.lease_expires_at,
        attempts: record.attempts,
        last_error: record.last_error,
        log_path: record.log_path,
        created_at: record.created_at,
        updated_at: record.updated_at,
    })
}

fn read_job_record(paths: &WorkspacePaths, job_id: &str) -> Result<JobRecord> {
    ensure_jobs_store(paths)?;
    let conn = open_connection(paths)?;
    conn.query_row(
        "SELECT job_id, kind, status, subject_id, summary, leased_by_session, lease_expires_at,
                attempts, payload_json, last_error, log_path, created_at, updated_at
         FROM jobs
         WHERE job_id = ?1",
        params![job_id],
        |row| {
            Ok(JobRecord {
                id: row.get(0)?,
                kind: row.get(1)?,
                status: row.get(2)?,
                subject_id: row.get(3)?,
                summary: row.get(4)?,
                leased_by_session: row.get(5)?,
                lease_expires_at: row.get(6)?,
                attempts: row.get::<_, i64>(7)? as usize,
                payload_json: row.get(8)?,
                last_error: row.get(9)?,
                log_path: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        },
    )
    .map_err(|_| anyhow!("job '{}' not found", job_id))
}

fn count_status(conn: &Connection, status: &str) -> Result<usize> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE status = ?1",
            params![status],
            |row| row.get(0),
        )
        .with_context(|| format!("count jobs with status '{status}'"))?;
    Ok(count as usize)
}

fn open_connection(paths: &WorkspacePaths) -> Result<Connection> {
    if let Some(parent) = paths.library_db_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    Connection::open(&paths.library_db_path)
        .with_context(|| format!("failed to open {}", paths.library_db_path.display()))
}

fn append_log(paths: &WorkspacePaths, log_path: &str, message: &str) -> Result<()> {
    fs::create_dir_all(&paths.jobs_dir)
        .with_context(|| format!("failed to create {}", paths.jobs_dir.display()))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .with_context(|| format!("failed to open job log {log_path}"))?;
    writeln!(file, "[{}] {}", Utc::now().to_rfc3339(), message).context("append job log")?;
    Ok(())
}

fn new_job_id(kind: &str, subject_id: &str) -> String {
    let mut hasher = Sha256::new();
    let now = Utc::now().to_rfc3339();
    hasher.update(kind.as_bytes());
    hasher.update(subject_id.as_bytes());
    hasher.update(now.as_bytes());
    let digest = hex::encode(hasher.finalize());
    format!("job_{}", &digest[..16])
}

struct ParsedJudge {
    recommendation: String,
    score: f64,
    summary: String,
}

fn parse_ai_judge_output(output: &str) -> ParsedJudge {
    if let Ok(value) = serde_json::from_str::<Value>(output) {
        let recommendation = value
            .get("recommendation")
            .and_then(Value::as_str)
            .unwrap_or("manual_review")
            .to_string();
        let score = value.get("score").and_then(Value::as_f64).unwrap_or(50.0);
        let summary = value
            .get("summary")
            .and_then(Value::as_str)
            .unwrap_or("AI judge completed.")
            .to_string();
        return ParsedJudge {
            recommendation,
            score,
            summary,
        };
    }

    ParsedJudge {
        recommendation: "manual_review".to_string(),
        score: 50.0,
        summary: output.lines().take(3).collect::<Vec<_>>().join(" "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create::create_skill_draft;
    use crate::evaluation::{queue_benchmark_run, read_benchmark_run};
    use crate::model::{ExternalExecutorKind, Scope};
    use crate::workspace::{ensure_workspace, WorkspacePaths};
    use std::path::Path;
    use tempfile::tempdir;

    fn sample_paths(root: &Path) -> WorkspacePaths {
        let state_dir = root.join(".praxis");
        WorkspacePaths {
            scope: Scope::Repo,
            repo_root: Some(root.to_path_buf()),
            state_dir: state_dir.clone(),
            jobs_dir: state_dir.join("jobs"),
            db_dir: state_dir.join("db"),
            library_dir: state_dir.join("library"),
            library_db_path: state_dir.join("db").join("praxis.db"),
            library_imports_dir: state_dir.join("library").join("imports"),
            library_drafts_dir: state_dir.join("library").join("drafts"),
            library_skills_dir: state_dir.join("library").join("skills"),
            library_decks_dir: state_dir.join("library").join("decks"),
            library_agent_files_dir: state_dir.join("library").join("agent-files"),
            library_bundles_dir: state_dir.join("library").join("bundles"),
            cache_dir: state_dir.join("cache"),
            manifest_path: state_dir.join("manifest.toml"),
            lock_path: state_dir.join("lock.json"),
            codex_skills_dir: root.join(".agents").join("skills"),
            claude_skills_dir: root.join(".claude").join("skills"),
            gemini_skills_dir: root.join(".gemini").join("skills"),
            codex_user_agents_path: root.join(".codex").join("AGENTS.md"),
            codex_user_override_path: root.join(".codex").join("AGENTS.override.md"),
            codex_project_agents_path: root.join("AGENTS.md"),
            codex_project_override_path: root.join("AGENTS.override.md"),
            codex_agent_alias_path: root.join("AGENT.md"),
            claude_user_root_path: root.join(".claude-home").join("CLAUDE.md"),
            claude_project_root_path: root.join("CLAUDE.md"),
            claude_project_dot_path: root.join(".claude").join("CLAUDE.md"),
            gemini_user_root_path: root.join(".gemini-home").join("GEMINI.md"),
            gemini_project_root_path: root.join("GEMINI.md"),
        }
    }

    #[test]
    fn queued_jobs_are_visible_in_snapshot() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        fs::create_dir_all(&repo_root).expect("repo root");
        let paths = sample_paths(&repo_root);
        ensure_workspace(&paths).expect("ensure workspace");

        let draft = create_skill_draft(&paths, "Queue Draft", "Queue", "skill").expect("draft");
        enqueue_augment_draft_job(
            &paths,
            &draft.draft.id,
            "tighten the purpose section",
            ExternalExecutorConfig::default(),
        )
        .expect("enqueue augment");

        let snapshot = read_job_snapshot(&paths).expect("job snapshot");
        assert_eq!(snapshot.queued, 1);
        assert_eq!(snapshot.recent_jobs.len(), 1);
    }

    #[test]
    fn failed_jobs_can_be_retried() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        fs::create_dir_all(&repo_root).expect("repo root");
        let paths = sample_paths(&repo_root);
        ensure_workspace(&paths).expect("ensure workspace");

        let job = enqueue_augment_draft_job(
            &paths,
            "draft_missing",
            "tighten the purpose section",
            ExternalExecutorConfig {
                provider: ExternalExecutorKind::CodexRuntime,
                model: None,
            },
        )
        .expect("enqueue");
        let _ = work_jobs(&paths, Some("test-worker"), Some(1));
        let failed = read_job(&paths, &job.id).expect("failed job");
        assert_eq!(failed.status, "failed");

        let retried = retry_job(&paths, &job.id).expect("retry");
        assert_eq!(retried.status, "queued");
    }

    #[test]
    fn stale_leased_jobs_are_reclaimed() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        fs::create_dir_all(&repo_root).expect("repo root");
        let paths = sample_paths(&repo_root);
        ensure_workspace(&paths).expect("ensure workspace");
        ensure_jobs_store(&paths).expect("ensure jobs");

        let queued = queue_benchmark_run(
            &paths,
            "runtime-conformance",
            "github:owner/repo@main#root",
            None,
            "ai-judge",
            "job_stale",
            "queued",
        )
        .expect("queue run");
        let conn = open_connection(&paths).expect("open conn");
        let stale_time = (Utc::now() - Duration::seconds(30)).to_rfc3339();
        conn.execute(
            "INSERT INTO jobs (
                job_id, kind, status, subject_id, summary, payload_json, leased_by_session,
                lease_expires_at, attempts, last_error, log_path, created_at, updated_at
            ) VALUES (?1, ?2, 'leased', ?3, 'stale', ?4, 'dead-worker', ?5, 1, NULL, ?6, ?7, ?7)",
            params![
                "job_stale",
                KIND_HUMAN_REVIEW,
                queued.id,
                serde_json::to_string(&HumanReviewJobPayload {
                    run_id: queued.id.clone(),
                    decision: "hold".to_string(),
                    note: "retry".to_string(),
                })
                .expect("serialize"),
                &stale_time,
                paths
                    .jobs_dir
                    .join("job_stale.log")
                    .to_string_lossy()
                    .to_string(),
                Utc::now().to_rfc3339(),
            ],
        )
        .expect("insert stale job");

        let leased = lease_next_job(&paths, "new-worker")
            .expect("lease")
            .expect("job");
        assert_eq!(leased.id, "job_stale");
        let run = read_benchmark_run(&paths, &queued.id).expect("queued run still readable");
        assert_eq!(run.status, "queued");
    }
}
