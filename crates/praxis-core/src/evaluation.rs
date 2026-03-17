use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::fs;

use crate::library::{ensure_library_store, sync_catalog_to_library};
use crate::model::{BenchmarkRunSummary, BenchmarkSuiteSummary, EvaluationSnapshot, SourceCatalog};
use crate::workspace::WorkspacePaths;

pub fn ensure_evaluation_store(paths: &WorkspacePaths) -> Result<()> {
    ensure_library_store(paths)?;
    let conn = open_connection(paths)?;
    init_schema(&conn)?;
    ensure_default_suites(&conn)?;
    Ok(())
}

pub fn read_evaluation_snapshot(paths: &WorkspacePaths) -> Result<EvaluationSnapshot> {
    ensure_evaluation_store(paths)?;
    let conn = open_connection(paths)?;

    let mut suite_stmt = conn
        .prepare(
            "SELECT suite_id, title, description, case_count, suite_kind
             FROM benchmark_suites
             ORDER BY title ASC",
        )
        .context("prepare benchmark suites query")?;
    let suites = suite_stmt
        .query_map([], |row| {
            Ok(BenchmarkSuiteSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                case_count: row.get::<_, i64>(3)? as usize,
                suite_kind: row.get(4)?,
            })
        })
        .context("query benchmark suites")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("collect benchmark suites")?;

    let mut run_stmt = conn
        .prepare(
            "SELECT run_id, suite_id, candidate_source_id, baseline_source_id, status,
                    mode, recommendation, score, summary, reviewer_note, review_decision,
                    job_id, evidence_path, created_at, finished_at
             FROM benchmark_runs
             ORDER BY created_at DESC
             LIMIT 10",
        )
        .context("prepare benchmark runs query")?;
    let recent_runs = run_stmt
        .query_map([], |row| {
            Ok(BenchmarkRunSummary {
                id: row.get(0)?,
                suite_id: row.get(1)?,
                candidate_source_id: row.get(2)?,
                baseline_source_id: row.get(3)?,
                status: row.get(4)?,
                mode: row.get(5)?,
                recommendation: row.get(6)?,
                score: row.get(7)?,
                summary: row.get(8)?,
                reviewer_note: row.get(9)?,
                review_decision: row.get(10)?,
                job_id: row.get(11)?,
                evidence_path: row.get(12)?,
                created_at: row.get(13)?,
                finished_at: row.get(14)?,
            })
        })
        .context("query benchmark runs")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("collect benchmark runs")?;

    Ok(EvaluationSnapshot {
        suites,
        recent_runs,
    })
}

pub fn run_benchmark(
    paths: &WorkspacePaths,
    suite_id: &str,
    catalog: &SourceCatalog,
    baseline_source_id: Option<&str>,
    mode: &str,
) -> Result<BenchmarkRunSummary> {
    ensure_evaluation_store(paths)?;
    sync_catalog_to_library(paths, catalog, "benchmark")?;

    let mut conn = open_connection(paths)?;
    let tx = conn.transaction().context("start benchmark transaction")?;
    let suite: (String, i64) = tx
        .query_row(
            "SELECT title, case_count FROM benchmark_suites WHERE suite_id = ?1",
            params![suite_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| anyhow!("benchmark suite '{}' not found", suite_id))?;

    let created_at = Utc::now().to_rfc3339();
    let run_id = benchmark_run_id(suite_id, &catalog.source_id, &created_at);
    let warning_penalty = (catalog.warnings.len() as f64) * 7.0;
    let score = (catalog.skills.len() as f64 * 10.0)
        + (catalog.agent_file_templates.len() as f64 * 6.0)
        + (catalog.decks.len() as f64 * 4.0)
        + (suite.1 as f64 * 2.0)
        - warning_penalty;
    let recommendation = if catalog.skills.is_empty() && catalog.agent_file_templates.is_empty() {
        "reject"
    } else if !catalog.warnings.is_empty() {
        "hold"
    } else if score >= 15.0 {
        "promote"
    } else {
        "manual_review"
    };
    let summary = format!(
        "{}: {} skills, {} decks, {} agent file templates, {} warnings",
        suite.0,
        catalog.skills.len(),
        catalog.decks.len(),
        catalog.agent_file_templates.len(),
        catalog.warnings.len()
    );
    let (status, recommendation, score, finished_at, reviewer_note, review_decision, summary): (
        String,
        String,
        f64,
        String,
        Option<String>,
        Option<String>,
        String,
    ) = if mode == "human-review" {
        (
            "awaiting_human".to_string(),
            "manual_review".to_string(),
            0.0,
            String::new(),
            None,
            None,
            format!("{summary}. Awaiting human review decision."),
        )
    } else {
        (
            "succeeded".to_string(),
            recommendation.to_string(),
            score,
            created_at.clone(),
            None,
            None,
            summary,
        )
    };

    tx.execute(
        "INSERT INTO benchmark_runs (
            run_id,
            suite_id,
            candidate_source_id,
            baseline_source_id,
            status,
            mode,
            recommendation,
            score,
            summary,
            reviewer_note,
            review_decision,
            job_id,
            evidence_path,
            created_at,
            finished_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            &run_id,
            suite_id,
            &catalog.source_id,
            baseline_source_id,
            &status,
            mode,
            &recommendation,
            score,
            &summary,
            reviewer_note,
            review_decision,
            Option::<String>::None,
            Option::<String>::None,
            &created_at,
            &finished_at,
        ],
    )
    .context("insert benchmark run")?;
    tx.commit().context("commit benchmark transaction")?;

    Ok(BenchmarkRunSummary {
        id: run_id,
        suite_id: suite_id.to_string(),
        candidate_source_id: catalog.source_id.clone(),
        baseline_source_id: baseline_source_id.map(ToOwned::to_owned),
        job_id: None,
        mode: mode.to_string(),
        status,
        recommendation,
        score,
        summary,
        reviewer_note: None,
        review_decision: None,
        evidence_path: None,
        created_at,
        finished_at,
    })
}

pub fn submit_human_review(
    paths: &WorkspacePaths,
    run_id: &str,
    decision: &str,
    note: &str,
) -> Result<BenchmarkRunSummary> {
    ensure_evaluation_store(paths)?;
    let conn = open_connection(paths)?;

    let (suite_id, candidate_source_id, baseline_source_id, status, mode, created_at): (
        String,
        String,
        Option<String>,
        String,
        String,
        String,
    ) = conn
        .query_row(
            "SELECT suite_id, candidate_source_id, baseline_source_id, status, mode, created_at
             FROM benchmark_runs WHERE run_id = ?1",
            params![run_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .map_err(|_| anyhow!("benchmark run '{}' not found", run_id))?;

    if mode != "human-review" {
        return Err(anyhow!(
            "benchmark run '{}' is not a human-review run",
            run_id
        ));
    }
    if status != "awaiting_human" {
        return Err(anyhow!(
            "benchmark run '{}' is not awaiting human review",
            run_id
        ));
    }

    let normalized_decision = match decision {
        "promote" | "hold" | "reject" | "manual_review" => decision,
        _ => {
            return Err(anyhow!(
                "invalid human review decision '{}'; expected promote|hold|reject|manual_review",
                decision
            ))
        }
    };
    let score = match normalized_decision {
        "promote" => 100.0,
        "hold" => 60.0,
        "manual_review" => 50.0,
        _ => 20.0,
    };
    let finished_at = Utc::now().to_rfc3339();
    let summary = format!(
        "Human review resolved with decision '{}'{}",
        normalized_decision,
        if note.trim().is_empty() {
            String::new()
        } else {
            format!(": {}", note.trim())
        }
    );

    conn.execute(
        "UPDATE benchmark_runs
         SET status = ?2,
             recommendation = ?3,
             score = ?4,
             summary = ?5,
             reviewer_note = ?6,
             review_decision = ?7,
             finished_at = ?8
         WHERE run_id = ?1",
        params![
            run_id,
            "succeeded",
            normalized_decision,
            score,
            &summary,
            note.trim(),
            normalized_decision,
            &finished_at,
        ],
    )
    .context("update human review result")?;

    Ok(BenchmarkRunSummary {
        id: run_id.to_string(),
        suite_id,
        candidate_source_id,
        baseline_source_id,
        job_id: None,
        mode,
        status: "succeeded".to_string(),
        recommendation: normalized_decision.to_string(),
        score,
        summary,
        reviewer_note: if note.trim().is_empty() {
            None
        } else {
            Some(note.trim().to_string())
        },
        review_decision: Some(normalized_decision.to_string()),
        evidence_path: None,
        created_at,
        finished_at,
    })
}

pub fn read_benchmark_run(paths: &WorkspacePaths, run_id: &str) -> Result<BenchmarkRunSummary> {
    ensure_evaluation_store(paths)?;
    let conn = open_connection(paths)?;
    conn.query_row(
        "SELECT run_id, suite_id, candidate_source_id, baseline_source_id, status, mode,
                recommendation, score, summary, reviewer_note, review_decision, job_id,
                evidence_path, created_at, finished_at
         FROM benchmark_runs
         WHERE run_id = ?1",
        params![run_id],
        |row| {
            Ok(BenchmarkRunSummary {
                id: row.get(0)?,
                suite_id: row.get(1)?,
                candidate_source_id: row.get(2)?,
                baseline_source_id: row.get(3)?,
                status: row.get(4)?,
                mode: row.get(5)?,
                recommendation: row.get(6)?,
                score: row.get(7)?,
                summary: row.get(8)?,
                reviewer_note: row.get(9)?,
                review_decision: row.get(10)?,
                job_id: row.get(11)?,
                evidence_path: row.get(12)?,
                created_at: row.get(13)?,
                finished_at: row.get(14)?,
            })
        },
    )
    .map_err(|_| anyhow!("benchmark run '{}' not found", run_id))
}

pub fn queue_benchmark_run(
    paths: &WorkspacePaths,
    suite_id: &str,
    candidate_source_id: &str,
    baseline_source_id: Option<&str>,
    mode: &str,
    job_id: &str,
    summary: &str,
) -> Result<BenchmarkRunSummary> {
    ensure_evaluation_store(paths)?;
    let conn = open_connection(paths)?;
    let created_at = Utc::now().to_rfc3339();
    let run_id = benchmark_run_id(suite_id, candidate_source_id, &created_at);
    conn.execute(
        "INSERT INTO benchmark_runs (
            run_id,
            suite_id,
            candidate_source_id,
            baseline_source_id,
            status,
            mode,
            recommendation,
            score,
            summary,
            reviewer_note,
            review_decision,
            job_id,
            evidence_path,
            created_at,
            finished_at
        ) VALUES (?1, ?2, ?3, ?4, 'queued', ?5, 'manual_review', 0.0, ?6, NULL, NULL, ?7, NULL, ?8, '')",
        params![&run_id, suite_id, candidate_source_id, baseline_source_id, mode, summary, job_id, &created_at],
    )
    .context("insert queued benchmark run")?;

    read_benchmark_run(paths, &run_id)
}

pub fn attach_job_to_benchmark_run(
    paths: &WorkspacePaths,
    run_id: &str,
    job_id: &str,
) -> Result<()> {
    ensure_evaluation_store(paths)?;
    let conn = open_connection(paths)?;
    conn.execute(
        "UPDATE benchmark_runs SET job_id = ?2 WHERE run_id = ?1",
        params![run_id, job_id],
    )
    .with_context(|| format!("attach job '{job_id}' to benchmark run '{run_id}'"))?;
    Ok(())
}

pub fn mark_benchmark_run_running(paths: &WorkspacePaths, run_id: &str) -> Result<()> {
    ensure_evaluation_store(paths)?;
    let conn = open_connection(paths)?;
    conn.execute(
        "UPDATE benchmark_runs SET status = 'running' WHERE run_id = ?1",
        params![run_id],
    )
    .with_context(|| format!("mark benchmark run '{run_id}' running"))?;
    Ok(())
}

pub fn complete_ai_benchmark_run(
    paths: &WorkspacePaths,
    run_id: &str,
    recommendation: &str,
    score: f64,
    summary: &str,
    evidence_path: Option<&str>,
) -> Result<BenchmarkRunSummary> {
    ensure_evaluation_store(paths)?;
    let conn = open_connection(paths)?;
    let finished_at = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE benchmark_runs
         SET status = 'succeeded',
             recommendation = ?2,
             score = ?3,
             summary = ?4,
             evidence_path = ?5,
             finished_at = ?6
         WHERE run_id = ?1",
        params![
            run_id,
            recommendation,
            score,
            summary,
            evidence_path,
            &finished_at
        ],
    )
    .with_context(|| format!("complete ai benchmark run '{run_id}'"))?;
    read_benchmark_run(paths, run_id)
}

pub fn fail_benchmark_run(
    paths: &WorkspacePaths,
    run_id: &str,
    summary: &str,
    evidence_path: Option<&str>,
) -> Result<BenchmarkRunSummary> {
    ensure_evaluation_store(paths)?;
    let conn = open_connection(paths)?;
    let finished_at = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE benchmark_runs
         SET status = 'failed',
             recommendation = 'manual_review',
             score = 0.0,
             summary = ?2,
             evidence_path = ?3,
             finished_at = ?4
         WHERE run_id = ?1",
        params![run_id, summary, evidence_path, &finished_at],
    )
    .with_context(|| format!("fail benchmark run '{run_id}'"))?;
    read_benchmark_run(paths, run_id)
}

fn open_connection(paths: &WorkspacePaths) -> Result<Connection> {
    if let Some(parent) = paths.library_db_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    Connection::open(&paths.library_db_path)
        .with_context(|| format!("failed to open {}", paths.library_db_path.display()))
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS benchmark_suites (
            suite_id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            case_count INTEGER NOT NULL,
            suite_kind TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS benchmark_runs (
            run_id TEXT PRIMARY KEY,
            suite_id TEXT NOT NULL REFERENCES benchmark_suites(suite_id) ON DELETE CASCADE,
            candidate_source_id TEXT NOT NULL,
            baseline_source_id TEXT,
            status TEXT NOT NULL,
            mode TEXT NOT NULL DEFAULT 'deterministic',
            recommendation TEXT NOT NULL,
            score REAL NOT NULL,
            summary TEXT NOT NULL,
            reviewer_note TEXT,
            review_decision TEXT,
            job_id TEXT,
            evidence_path TEXT,
            created_at TEXT NOT NULL,
            finished_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_benchmark_runs_suite
            ON benchmark_runs(suite_id, created_at DESC);
        ",
    )
    .context("initialize evaluation schema")?;
    ensure_column(
        conn,
        "benchmark_runs",
        "mode",
        "TEXT NOT NULL DEFAULT 'deterministic'",
    )?;
    ensure_column(conn, "benchmark_runs", "reviewer_note", "TEXT")?;
    ensure_column(conn, "benchmark_runs", "review_decision", "TEXT")?;
    ensure_column(conn, "benchmark_runs", "job_id", "TEXT")?;
    ensure_column(conn, "benchmark_runs", "evidence_path", "TEXT")?;
    Ok(())
}

fn ensure_default_suites(conn: &Connection) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    for (suite_id, title, description, case_count, suite_kind) in [
        (
            "runtime-conformance",
            "Runtime Conformance",
            "Checks whether a candidate source has enough structured artifacts to safely project into runtime outputs.",
            3_i64,
            "deterministic",
        ),
        (
            "import-quality",
            "Import Quality",
            "Checks whether imported source structure is rich enough to become a reusable library candidate.",
            4_i64,
            "deterministic",
        ),
    ] {
        conn.execute(
            "INSERT INTO benchmark_suites (
                suite_id,
                title,
                description,
                case_count,
                suite_kind,
                created_at,
                updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
            ON CONFLICT(suite_id) DO UPDATE SET
                title = excluded.title,
                description = excluded.description,
                case_count = excluded.case_count,
                suite_kind = excluded.suite_kind,
                updated_at = excluded.updated_at",
            params![suite_id, title, description, case_count, suite_kind, &now],
        )
        .with_context(|| format!("upsert benchmark suite '{suite_id}'"))?;
    }
    Ok(())
}

fn benchmark_run_id(suite_id: &str, candidate_source_id: &str, created_at: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(suite_id.as_bytes());
    hasher.update(candidate_source_id.as_bytes());
    hasher.update(created_at.as_bytes());
    let digest = hex::encode(hasher.finalize());
    format!("br_{}", &digest[..16])
}

fn ensure_column(conn: &Connection, table: &str, column: &str, definition: &str) -> Result<()> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .with_context(|| format!("prepare table info for {table}"))?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .with_context(|| format!("query columns for {table}"))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .with_context(|| format!("collect columns for {table}"))?;

    if !columns.iter().any(|existing| existing == column) {
        conn.execute(
            &format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"),
            [],
        )
        .with_context(|| format!("add column {table}.{column}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        AgentFileSlot, AgentFileTemplate, AgentFileTemplateOrigin, DeckInfo, SkillInfo, SourceRef,
    };
    use crate::workspace::{ensure_workspace, WorkspacePaths};
    use std::path::Path;
    use tempfile::tempdir;

    fn sample_paths(root: &Path) -> WorkspacePaths {
        let state_dir = root.join(".praxis");
        WorkspacePaths {
            scope: crate::model::Scope::Repo,
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

    fn sample_catalog(root: &Path) -> SourceCatalog {
        let checkout_root = root.join("fixture");
        fs::create_dir_all(checkout_root.join("demo-skill")).expect("skill dir");
        fs::write(
            checkout_root.join("demo-skill").join("SKILL.md"),
            "# Demo\n",
        )
        .expect("skill");
        fs::write(checkout_root.join("AGENTS.md"), "System instructions\n").expect("agent file");

        SourceCatalog {
            source_id: "github:owner/repo@main#root".to_string(),
            label: "owner/repo".to_string(),
            source: SourceRef::Github {
                owner: "owner".to_string(),
                repo: "repo".to_string(),
                reference: Some("main".to_string()),
                subdir: None,
            },
            checkout_root: checkout_root.to_string_lossy().to_string(),
            resolved_reference: Some("main".to_string()),
            source_hash: "abc123".to_string(),
            decks: vec![DeckInfo {
                id: "all".to_string(),
                name: "All".to_string(),
                description: "Every skill".to_string(),
                skills: vec!["demo-skill".to_string()],
                synthesized: true,
            }],
            skills: vec![SkillInfo {
                name: "demo-skill".to_string(),
                description: "Demo skill".to_string(),
                relative_path: "demo-skill".to_string(),
                root_component: "SKILL.md".to_string(),
                display_name: Some("Demo Skill".to_string()),
                category: Some("workflow".to_string()),
                tags: vec!["demo".to_string()],
            }],
            agent_file_templates: vec![AgentFileTemplate {
                id: "codex-project-root".to_string(),
                title: "Codex Root".to_string(),
                description: "Project instructions".to_string(),
                relative_path: "AGENTS.md".to_string(),
                slots: vec![AgentFileSlot::CodexProjectRoot],
                priority: 100,
                origin: AgentFileTemplateOrigin::Declared,
            }],
            recipe: None,
            warnings: Vec::new(),
            notes: Vec::new(),
        }
    }

    #[test]
    fn evaluation_snapshot_contains_default_suites() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        fs::create_dir_all(&repo_root).expect("repo root");
        let paths = sample_paths(&repo_root);

        ensure_workspace(&paths).expect("ensure workspace");
        let snapshot = read_evaluation_snapshot(&paths).expect("evaluation snapshot");

        assert_eq!(snapshot.suites.len(), 2);
        assert!(snapshot.recent_runs.is_empty());
    }

    #[test]
    fn run_benchmark_persists_evidence() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        fs::create_dir_all(&repo_root).expect("repo root");
        let paths = sample_paths(&repo_root);

        ensure_workspace(&paths).expect("ensure workspace");
        let catalog = sample_catalog(&repo_root);
        let run = run_benchmark(
            &paths,
            "runtime-conformance",
            &catalog,
            None,
            "deterministic",
        )
        .expect("run benchmark");
        let snapshot = read_evaluation_snapshot(&paths).expect("evaluation snapshot");

        assert_eq!(run.recommendation, "promote");
        assert_eq!(snapshot.recent_runs.len(), 1);
        assert_eq!(snapshot.recent_runs[0].suite_id, "runtime-conformance");
    }

    #[test]
    fn submit_human_review_resolves_awaiting_run() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        fs::create_dir_all(&repo_root).expect("repo root");
        let paths = sample_paths(&repo_root);

        ensure_workspace(&paths).expect("ensure workspace");
        let catalog = sample_catalog(&repo_root);
        let run = run_benchmark(
            &paths,
            "runtime-conformance",
            &catalog,
            None,
            "human-review",
        )
        .expect("run benchmark");
        assert_eq!(run.status, "awaiting_human");

        let resolved = submit_human_review(&paths, &run.id, "promote", "Looks production ready")
            .expect("submit review");
        assert_eq!(resolved.status, "succeeded");
        assert_eq!(resolved.recommendation, "promote");
        assert_eq!(resolved.review_decision.as_deref(), Some("promote"));
    }
}
