use std::fs;
use std::path::PathBuf;

use praxis_core::{
    augment_draft, benchmark_source, cancel_job, create_draft, init_workspace,
    inspect_source_input, install_source, list_workspace, plan_install, retry_job, Agent,
    BenchmarkRunRequest, CreateDraftRequest, DraftAugmentRequest, ExternalExecutorConfig,
    InstallRequest, JobCancelRequest, JobRetryRequest, Scope,
};
use tempfile::tempdir;

fn demo_source_path() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/sources/demo-cards")
        .to_string_lossy()
        .to_string()
}

#[test]
fn repo_scope_install_flow_works_against_local_source() {
    let temp = tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    fs::create_dir_all(&repo_root).expect("repo root");
    let source = demo_source_path();

    let snapshot =
        init_workspace(Scope::Repo, Some(repo_root.to_string_lossy().to_string())).expect("init");
    assert!(snapshot.manifest.installs.is_empty());

    let catalog = inspect_source_input(
        Scope::Repo,
        Some(repo_root.to_string_lossy().to_string()),
        &source,
    )
    .expect("inspect");
    assert!(!catalog.skills.is_empty());

    let plan = plan_install(InstallRequest {
        scope: Scope::Repo,
        root: Some(repo_root.to_string_lossy().to_string()),
        source: source.clone(),
        all: true,
        decks: Vec::new(),
        skills: Vec::new(),
        exclude_skills: Vec::new(),
        agent_file_templates: vec!["codex-project-root".to_string()],
        targets: vec![Agent::Codex],
    })
    .expect("plan");
    assert!(!plan.skills.is_empty());

    let applied = install_source(InstallRequest {
        scope: Scope::Repo,
        root: Some(repo_root.to_string_lossy().to_string()),
        source,
        all: true,
        decks: Vec::new(),
        skills: Vec::new(),
        exclude_skills: Vec::new(),
        agent_file_templates: vec!["codex-project-root".to_string()],
        targets: vec![Agent::Codex],
    })
    .expect("install");
    assert_eq!(applied.manifest.installs.len(), 1);
    assert!(repo_root.join(".agents/skills").is_dir());
}

#[test]
fn ai_judge_failure_is_persisted_as_run_and_job_state() {
    let temp = tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    fs::create_dir_all(&repo_root).expect("repo root");
    let source = demo_source_path();

    init_workspace(Scope::Repo, Some(repo_root.to_string_lossy().to_string())).expect("init");
    let run = benchmark_source(BenchmarkRunRequest {
        scope: Scope::Repo,
        root: Some(repo_root.to_string_lossy().to_string()),
        suite_id: "runtime-conformance".to_string(),
        source,
        mode: Some("ai-judge".to_string()),
        executor: Some(ExternalExecutorConfig::default()),
    })
    .expect("benchmark");

    assert_eq!(run.status, "failed");
    assert!(run.job_id.is_some());

    let workspace = list_workspace(Scope::Repo, Some(repo_root.to_string_lossy().to_string()))
        .expect("list workspace");
    assert_eq!(workspace.jobs.failed, 1);
    assert_eq!(workspace.evaluation.recent_runs[0].status, "failed");
}

#[test]
fn augment_jobs_can_be_retried_and_cancelled() {
    let temp = tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    fs::create_dir_all(&repo_root).expect("repo root");

    init_workspace(Scope::Repo, Some(repo_root.to_string_lossy().to_string())).expect("init");
    let preview = create_draft(CreateDraftRequest {
        scope: Scope::Repo,
        root: Some(repo_root.to_string_lossy().to_string()),
        name: "Scenario Draft".to_string(),
        description: "Scenario draft".to_string(),
        preset: "skill".to_string(),
    })
    .expect("create draft");

    let _ = augment_draft(DraftAugmentRequest {
        scope: Scope::Repo,
        root: Some(repo_root.to_string_lossy().to_string()),
        draft_id: preview.draft.id.clone(),
        prompt: "Tighten the skill purpose".to_string(),
        executor: Some(ExternalExecutorConfig::default()),
    })
    .expect("augment call should return preview");

    let workspace = list_workspace(Scope::Repo, Some(repo_root.to_string_lossy().to_string()))
        .expect("list workspace");
    let failed_job = workspace
        .jobs
        .recent_jobs
        .iter()
        .find(|job| job.subject_id == preview.draft.id)
        .expect("failed augment job");
    assert_eq!(failed_job.status, "failed");

    let retried = retry_job(JobRetryRequest {
        scope: Scope::Repo,
        root: Some(repo_root.to_string_lossy().to_string()),
        job_id: failed_job.id.clone(),
    })
    .expect("retry job");
    assert!(retried.recent_jobs.iter().any(|job| job.status == "queued"));

    let cancelled = cancel_job(JobCancelRequest {
        scope: Scope::Repo,
        root: Some(repo_root.to_string_lossy().to_string()),
        job_id: failed_job.id.clone(),
    })
    .expect("cancel job");
    assert!(cancelled
        .recent_jobs
        .iter()
        .any(|job| job.id == failed_job.id && job.status == "cancelled"));
}

#[test]
fn empty_draft_id_is_rejected_before_job_enqueue() {
    let temp = tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    fs::create_dir_all(&repo_root).expect("repo root");

    init_workspace(Scope::Repo, Some(repo_root.to_string_lossy().to_string())).expect("init");
    let error = augment_draft(DraftAugmentRequest {
        scope: Scope::Repo,
        root: Some(repo_root.to_string_lossy().to_string()),
        draft_id: "   ".to_string(),
        prompt: "Tighten purpose".to_string(),
        executor: Some(ExternalExecutorConfig::default()),
    })
    .expect_err("blank draft id should be rejected");
    assert!(error.to_string().contains("draft id must not be empty"));

    let workspace = list_workspace(Scope::Repo, Some(repo_root.to_string_lossy().to_string()))
        .expect("list workspace");
    assert!(workspace.jobs.recent_jobs.is_empty());
}
