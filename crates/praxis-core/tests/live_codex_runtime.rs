use std::env;
use std::fs;
use std::path::PathBuf;

use praxis_core::{
    augment_draft, benchmark_source, create_draft, doctor_workspace_with_executor, init_workspace,
    list_workspace, BenchmarkRunRequest, CreateDraftRequest, DraftAugmentRequest,
    ExternalExecutorConfig, ExternalExecutorKind, Scope,
};
use tempfile::tempdir;

fn demo_source_path() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/sources/demo-cards")
        .to_string_lossy()
        .to_string()
}

fn live_executor_from_env() -> Option<ExternalExecutorConfig> {
    if env::var("PRAXIS_LIVE_CODEX_RUNTIME").ok().as_deref() != Some("1") {
        eprintln!(
            "skipping live codex-runtime test because PRAXIS_LIVE_CODEX_RUNTIME=1 is not set"
        );
        return None;
    }

    Some(ExternalExecutorConfig {
        provider: ExternalExecutorKind::CodexRuntime,
        model: env::var("PRAXIS_LIVE_CODEX_RUNTIME_MODEL").ok(),
    })
}

#[test]
#[ignore = "requires PRAXIS_LIVE_CODEX_RUNTIME=1 and a locally authenticated codex CLI"]
fn live_codex_runtime_drives_ai_judge_and_augment_end_to_end() {
    let Some(executor) = live_executor_from_env() else {
        return;
    };

    let temp = tempdir().expect("tempdir");
    let repo_root = temp.path().join("repo");
    fs::create_dir_all(&repo_root).expect("repo root");
    let repo_root_str = repo_root.to_string_lossy().to_string();

    init_workspace(Scope::Repo, Some(repo_root_str.clone())).expect("init");
    let doctor = doctor_workspace_with_executor(
        Scope::Repo,
        Some(repo_root_str.clone()),
        Some(executor.clone()),
    )
    .expect("doctor");
    assert!(
        doctor.ok,
        "expected doctor readiness to pass before live run: {:?}",
        doctor.checks
    );

    let source = demo_source_path();
    let run = benchmark_source(BenchmarkRunRequest {
        scope: Scope::Repo,
        root: Some(repo_root_str.clone()),
        suite_id: "runtime-conformance".to_string(),
        source: source.clone(),
        mode: Some("ai-judge".to_string()),
        executor: Some(executor.clone()),
    })
    .expect("live benchmark");

    assert_eq!(run.status, "succeeded");
    assert!(run.evidence_path.is_some());
    let evidence_path = PathBuf::from(run.evidence_path.expect("evidence path"));
    assert!(evidence_path.is_file());
    assert!(!fs::read_to_string(&evidence_path)
        .expect("read evidence")
        .trim()
        .is_empty());

    let preview = create_draft(CreateDraftRequest {
        scope: Scope::Repo,
        root: Some(repo_root_str.clone()),
        name: "live-augment-skill".to_string(),
        description: "Live augment draft".to_string(),
        preset: "skill".to_string(),
    })
    .expect("create draft");
    let prompt = "Tighten the purpose and add one concrete usage example.".to_string();
    let augmented = augment_draft(DraftAugmentRequest {
        scope: Scope::Repo,
        root: Some(repo_root_str.clone()),
        draft_id: preview.draft.id.clone(),
        prompt: prompt.clone(),
        executor: Some(executor),
    })
    .expect("live augment");

    assert_eq!(augmented.draft.lineage.origin_kind, "augment");
    assert_eq!(
        augmented.draft.lineage.augmentation_prompt.as_deref(),
        Some(prompt.as_str())
    );
    assert_eq!(
        augmented.draft.lineage.parent_version_id.as_deref(),
        Some(preview.draft.version_id.as_str())
    );

    let workspace = list_workspace(Scope::Repo, Some(repo_root_str)).expect("workspace");
    assert_eq!(workspace.jobs.failed, 0);
    assert!(workspace
        .jobs
        .recent_jobs
        .iter()
        .any(|job| job.kind == "benchmark-ai-judge" && job.status == "succeeded"));
    assert!(workspace
        .jobs
        .recent_jobs
        .iter()
        .any(|job| job.kind == "augment-draft" && job.status == "succeeded"));
}
