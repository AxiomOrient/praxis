#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use praxis_core::{
    augment_draft, benchmark_source, cancel_job, create_draft, doctor_workspace, fork_draft,
    inspect_source_input, install_source, jobs_work, list_workspace, plan_install, preview_draft,
    promote_draft, read_agent_file_state, remove_from_source, retry_job,
    submit_human_review_action, sync_workspace, update_draft, update_workspace,
    write_agent_file_user_content, AgentFileWriteRequest, BenchmarkRunRequest, CreateDraftRequest,
    DraftAugmentRequest, DraftPreviewRequest, DraftUpdateRequest, ForkDraftRequest,
    HumanReviewRequest, InstallRequest, JobCancelRequest, JobRetryRequest, JobWorkRequest,
    PromoteDraftRequest, RemoveRequest, Scope,
};

fn parse_scope(scope: &str) -> Result<Scope, String> {
    match scope {
        "repo" => Ok(Scope::Repo),
        "user" => Ok(Scope::User),
        _ => Err(format!("invalid scope: {scope}")),
    }
}

#[tauri::command]
fn workspace(scope: &str, root: Option<String>) -> Result<snapshot::WorkspaceSnapshot, String> {
    let scope = parse_scope(scope)?;
    list_workspace(scope, root).map_err(|e| e.to_string())
}

#[tauri::command]
fn inspect(
    scope: &str,
    root: Option<String>,
    source: String,
) -> Result<snapshot::SourceCatalog, String> {
    let scope = parse_scope(scope)?;
    inspect_source_input(scope, root, &source).map_err(|e| e.to_string())
}

#[tauri::command]
fn plan(payload: InstallRequest) -> Result<snapshot::InstallPlan, String> {
    plan_install(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn install(payload: InstallRequest) -> Result<snapshot::WorkspaceSnapshot, String> {
    install_source(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_install(payload: RemoveRequest) -> Result<snapshot::WorkspaceSnapshot, String> {
    remove_from_source(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn sync(scope: &str, root: Option<String>) -> Result<snapshot::WorkspaceSnapshot, String> {
    let scope = parse_scope(scope)?;
    sync_workspace(scope, root).map_err(|e| e.to_string())
}

#[tauri::command]
fn update(scope: &str, root: Option<String>) -> Result<snapshot::WorkspaceSnapshot, String> {
    let scope = parse_scope(scope)?;
    update_workspace(scope, root).map_err(|e| e.to_string())
}

#[tauri::command]
fn doctor(scope: &str, root: Option<String>) -> Result<snapshot::DoctorReport, String> {
    let scope = parse_scope(scope)?;
    doctor_workspace(scope, root).map_err(|e| e.to_string())
}

#[tauri::command]
fn agent_files(scope: &str, root: Option<String>) -> Result<snapshot::AgentFileSnapshot, String> {
    let scope = parse_scope(scope)?;
    read_agent_file_state(scope, root).map_err(|e| e.to_string())
}

#[tauri::command]
fn agent_files_write(payload: AgentFileWriteRequest) -> Result<snapshot::AgentFileSnapshot, String> {
    write_agent_file_user_content(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn benchmark_run(payload: BenchmarkRunRequest) -> Result<snapshot::BenchmarkRunSummary, String> {
    benchmark_source(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn submit_human_review(
    payload: HumanReviewRequest,
) -> Result<snapshot::BenchmarkRunSummary, String> {
    submit_human_review_action(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_skill_draft(payload: CreateDraftRequest) -> Result<snapshot::DraftPreview, String> {
    create_draft(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn preview_create_draft(payload: DraftPreviewRequest) -> Result<snapshot::DraftPreview, String> {
    preview_draft(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn promote_create_draft(payload: PromoteDraftRequest) -> Result<snapshot::DraftPreview, String> {
    promote_draft(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn fork_create_draft(payload: ForkDraftRequest) -> Result<snapshot::DraftPreview, String> {
    fork_draft(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_create_draft(payload: DraftUpdateRequest) -> Result<snapshot::DraftPreview, String> {
    update_draft(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn augment_create_draft(payload: DraftAugmentRequest) -> Result<snapshot::DraftPreview, String> {
    augment_draft(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn work_jobs(payload: JobWorkRequest) -> Result<snapshot::JobSnapshot, String> {
    jobs_work(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn cancel_job_command(payload: JobCancelRequest) -> Result<snapshot::JobSnapshot, String> {
    cancel_job(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn retry_job_command(payload: JobRetryRequest) -> Result<snapshot::JobSnapshot, String> {
    retry_job(payload).map_err(|e| e.to_string())
}

mod snapshot {
    pub use praxis_core::model::{
        AgentFileSnapshot, BenchmarkRunSummary, DoctorReport, DraftPreview, InstallPlan,
        JobSnapshot, SourceCatalog, WorkspaceSnapshot,
    };
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            workspace,
            inspect,
            plan,
            install,
            remove_install,
            sync,
            update,
            doctor,
            agent_files,
            agent_files_write,
            benchmark_run,
            submit_human_review,
            create_skill_draft,
            preview_create_draft,
            promote_create_draft,
            fork_create_draft,
            update_create_draft,
            augment_create_draft,
            work_jobs,
            cancel_job_command,
            retry_job_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running praxis desktop");
}

#[cfg(test)]
mod tests {
    use super::snapshot::{AgentFileSnapshot, InstallPlan, WorkspaceSnapshot};
    use praxis_core::model::{
        Agent, AgentFileSlot, AgentFileState, BenchmarkRunSummary, BenchmarkSuiteSummary,
        CreateSnapshot, DraftDocument, DraftLineage, DraftPreview, DraftSummary,
        EvaluationSnapshot, InstallSelection, JobSnapshot, JobSummary, LibraryStats,
        LibraryStoreSnapshot, ManagedAgentFileBlock, PlanSummary, PlannedAgentFileAction,
        PromotionReviewSummary, SourceInstall, SourceRef, TargetPaths, TargetProfile,
        WorkspaceLock, WorkspaceManifest, WorkspaceSettings,
    };
    use serde_json::Value;

    #[test]
    fn workspace_snapshot_serializes_current_contract_keys() {
        let snapshot = WorkspaceSnapshot {
            manifest: WorkspaceManifest {
                version: 1,
                settings: WorkspaceSettings {
                    target_profile: TargetProfile::MultiRuntimeDefault,
                    write_codex_agent_alias: true,
                },
                installs: vec![SourceInstall {
                    id: "github:owner/repo@default#root".to_string(),
                    source: SourceRef::Github {
                        owner: "owner".to_string(),
                        repo: "repo".to_string(),
                        reference: None,
                        subdir: None,
                    },
                    targets: vec![Agent::Codex],
                    selection: InstallSelection {
                        all: false,
                        decks: vec!["workflow".to_string()],
                        skills: vec!["inspect".to_string()],
                        exclude_skills: Vec::new(),
                        agent_file_templates: vec!["codex-project-root".to_string()],
                    },
                }],
            },
            lock: WorkspaceLock {
                version: 1,
                generated_at: "2026-01-01T00:00:00Z".to_string(),
                installs: Vec::new(),
            },
            targets: TargetPaths {
                codex_skills: "/tmp/.agents/skills".to_string(),
                claude_skills: "/tmp/.claude/skills".to_string(),
                codex_agents: "/tmp/AGENTS.md".to_string(),
                codex_override: "/tmp/AGENTS.override.md".to_string(),
                codex_agent_alias: "/tmp/AGENT.md".to_string(),
                claude_root: "/tmp/CLAUDE.md".to_string(),
                claude_dot: "/tmp/.claude/CLAUDE.md".to_string(),
            },
            library: LibraryStoreSnapshot {
                db_path: "/tmp/.praxis/db/praxis.db".to_string(),
                artifact_root: "/tmp/.praxis/library".to_string(),
                stats: LibraryStats {
                    sources: 1,
                    snapshots: 1,
                    artifacts: 2,
                },
            },
            evaluation: EvaluationSnapshot {
                suites: vec![BenchmarkSuiteSummary {
                    id: "runtime-conformance".to_string(),
                    title: "Runtime Conformance".to_string(),
                    description: "Checks runtime readiness.".to_string(),
                    case_count: 3,
                    suite_kind: "deterministic".to_string(),
                }],
                recent_runs: vec![BenchmarkRunSummary {
                    id: "br_1234567890abcdef".to_string(),
                    suite_id: "runtime-conformance".to_string(),
                    candidate_source_id: "github:owner/repo@default#root".to_string(),
                    baseline_source_id: None,
                    job_id: Some("job_1234567890abcdef".to_string()),
                    mode: "deterministic".to_string(),
                    status: "succeeded".to_string(),
                    recommendation: "promote".to_string(),
                    score: 27.0,
                    summary:
                        "Runtime Conformance: 1 skills, 0 decks, 1 agent file templates, 0 warnings"
                            .to_string(),
                    reviewer_note: None,
                    review_decision: None,
                    evidence_path: Some(
                        "/tmp/.praxis/jobs/job_1234567890abcdef.evidence.md".to_string(),
                    ),
                    created_at: "2026-01-01T00:00:00Z".to_string(),
                    finished_at: "2026-01-01T00:00:00Z".to_string(),
                }],
            },
            jobs: JobSnapshot {
                queued: 1,
                running: 0,
                failed: 0,
                recent_jobs: vec![JobSummary {
                    id: "job_1234567890abcdef".to_string(),
                    kind: "benchmark-ai-judge".to_string(),
                    status: "succeeded".to_string(),
                    subject_id: "br_1234567890abcdef".to_string(),
                    summary: "benchmark complete".to_string(),
                    leased_by_session: None,
                    lease_expires_at: None,
                    attempts: 1,
                    last_error: None,
                    log_path: "/tmp/.praxis/jobs/job_1234567890abcdef.log".to_string(),
                    created_at: "2026-01-01T00:00:00Z".to_string(),
                    updated_at: "2026-01-01T00:00:00Z".to_string(),
                }],
            },
            create: CreateSnapshot {
                drafts: vec![DraftSummary {
                    id: "draft_skill_demo".to_string(),
                    name: "demo-draft".to_string(),
                    artifact_kind: "skill".to_string(),
                    version_id: "drv_demo".to_string(),
                    preset: "skill".to_string(),
                    lineage: DraftLineage {
                        origin_kind: "create".to_string(),
                        source_id: None,
                        parent_version_id: None,
                        parent_name: None,
                        augmentation_prompt: None,
                        promotion_path: None,
                        promoted_at: None,
                    },
                    created_at: "2026-01-01T00:00:00Z".to_string(),
                    updated_at: "2026-01-01T00:00:00Z".to_string(),
                }],
            },
            warnings: Vec::new(),
        };

        let value = serde_json::to_value(snapshot).expect("serialize workspace snapshot");
        let manifest = value
            .get("manifest")
            .and_then(Value::as_object)
            .expect("manifest");
        let settings = manifest
            .get("settings")
            .and_then(Value::as_object)
            .expect("settings");
        let installs = manifest
            .get("installs")
            .and_then(Value::as_array)
            .expect("installs");
        let selection = installs[0]
            .get("selection")
            .and_then(Value::as_object)
            .expect("selection");
        let lock = value.get("lock").and_then(Value::as_object).expect("lock");
        let library = value
            .get("library")
            .and_then(Value::as_object)
            .expect("library");
        let evaluation = value
            .get("evaluation")
            .and_then(Value::as_object)
            .expect("evaluation");
        let jobs = value.get("jobs").and_then(Value::as_object).expect("jobs");
        let create = value
            .get("create")
            .and_then(Value::as_object)
            .expect("create");

        assert!(settings.contains_key("target_profile"));
        assert!(settings.contains_key("write_codex_agent_alias"));
        assert!(selection.contains_key("agent_file_templates"));
        assert!(lock.contains_key("generated_at"));
        assert!(library.contains_key("db_path"));
        assert!(library.contains_key("artifact_root"));
        assert!(library
            .get("stats")
            .and_then(Value::as_object)
            .expect("library stats")
            .contains_key("artifacts"));
        assert!(evaluation.get("suites").is_some());
        assert!(evaluation.get("recent_runs").is_some());
        assert!(jobs.get("recent_jobs").is_some());
        assert!(create.get("drafts").is_some());
    }

    #[test]
    fn install_plan_and_agent_file_snapshot_serialize_current_contract_keys() {
        let plan = InstallPlan {
            source_id: "github:owner/repo@default#root".to_string(),
            label: "repo".to_string(),
            resolved_reference: None,
            source_hash: "hash".to_string(),
            targets: vec![Agent::Codex],
            selection: InstallSelection {
                all: false,
                decks: Vec::new(),
                skills: Vec::new(),
                exclude_skills: Vec::new(),
                agent_file_templates: vec!["codex-project-root".to_string()],
            },
            target_paths: TargetPaths {
                codex_skills: "/tmp/.agents/skills".to_string(),
                claude_skills: "/tmp/.claude/skills".to_string(),
                codex_agents: "/tmp/AGENTS.md".to_string(),
                codex_override: "/tmp/AGENTS.override.md".to_string(),
                codex_agent_alias: "/tmp/AGENT.md".to_string(),
                claude_root: "/tmp/CLAUDE.md".to_string(),
                claude_dot: "/tmp/.claude/CLAUDE.md".to_string(),
            },
            skills: Vec::new(),
            bundles: Vec::new(),
            agent_file_actions: vec![PlannedAgentFileAction {
                template_id: "codex-project-root".to_string(),
                title: "Codex Root".to_string(),
                slot: AgentFileSlot::CodexProjectRoot,
                source_relative_path: "AGENTS.md".to_string(),
                target_path: "/tmp/AGENTS.md".to_string(),
            }],
            warnings: Vec::new(),
            notes: Vec::new(),
            conflicts: Vec::new(),
            summary: PlanSummary {
                total_skills: 0,
                total_agent_file_actions: 1,
                total_bundles: 0,
                codex_skills: 0,
                claude_skills: 0,
                codex_bundles: 0,
                claude_bundles: 0,
            },
        };
        let snapshot = AgentFileSnapshot {
            slots: vec![AgentFileState {
                slot: AgentFileSlot::CodexProjectRoot,
                target_path: "/tmp/AGENTS.md".to_string(),
                exists: true,
                user_content: "user".to_string(),
                managed_blocks: vec![ManagedAgentFileBlock {
                    source_id: "github:owner/repo@default#root".to_string(),
                    template_id: "codex-project-root".to_string(),
                    slot: AgentFileSlot::CodexProjectRoot,
                    content_hash: "hash".to_string(),
                }],
                effective_content: "effective".to_string(),
            }],
        };

        let plan_value = serde_json::to_value(plan).expect("serialize install plan");
        let snapshot_value = serde_json::to_value(snapshot).expect("serialize agent file snapshot");

        assert!(plan_value.get("agent_file_actions").is_some());
        assert!(plan_value
            .get("summary")
            .and_then(Value::as_object)
            .expect("summary")
            .contains_key("total_agent_file_actions"));
        assert!(snapshot_value.get("slots").is_some());
        assert!(snapshot_value["slots"][0].get("slot").is_some());
    }

    #[test]
    fn benchmark_and_draft_preview_serialize_current_contract_keys() {
        let run = BenchmarkRunSummary {
            id: "br_1234567890abcdef".to_string(),
            suite_id: "runtime-conformance".to_string(),
            candidate_source_id: "github:owner/repo@default#root".to_string(),
            baseline_source_id: Some("github:owner/repo@old#root".to_string()),
            job_id: Some("job_1234567890abcdef".to_string()),
            mode: "human-review".to_string(),
            status: "awaiting_human".to_string(),
            recommendation: "manual_review".to_string(),
            score: 0.0,
            summary: "Awaiting human review.".to_string(),
            reviewer_note: None,
            review_decision: None,
            evidence_path: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            finished_at: String::new(),
        };
        let preview = DraftPreview {
            draft: DraftSummary {
                id: "draft_skill_demo".to_string(),
                name: "demo-draft".to_string(),
                artifact_kind: "skill".to_string(),
                version_id: "drv_demo".to_string(),
                preset: "fork".to_string(),
                lineage: DraftLineage {
                    origin_kind: "fork".to_string(),
                    source_id: Some("github:owner/repo@default#root".to_string()),
                    parent_version_id: Some("sv_demo".to_string()),
                    parent_name: Some("demo-skill".to_string()),
                    augmentation_prompt: None,
                    promotion_path: None,
                    promoted_at: None,
                },
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
            },
            files: vec![praxis_core::model::PreviewTreeEntry {
                path: "SKILL.md".to_string(),
                entry_kind: "file".to_string(),
            }],
            documents: vec![DraftDocument {
                path: "SKILL.md".to_string(),
                content: "# Demo".to_string(),
                editable: true,
            }],
            promotion_target: "/tmp/.agents/skills/demo-draft".to_string(),
            review: PromotionReviewSummary {
                changed_files: 1,
                latest_recommendation: Some("promote".to_string()),
                latest_run_status: Some("succeeded".to_string()),
                latest_run_summary: Some("Looks good.".to_string()),
                pending_job_count: 0,
            },
        };

        let run_value = serde_json::to_value(run).expect("serialize benchmark run");
        let preview_value = serde_json::to_value(preview).expect("serialize draft preview");

        assert!(run_value.get("mode").is_some());
        assert!(run_value.get("job_id").is_some());
        assert!(run_value.get("reviewer_note").is_some());
        assert!(run_value.get("review_decision").is_some());
        assert!(run_value.get("evidence_path").is_some());
        assert!(preview_value.get("documents").is_some());
        assert!(preview_value.get("review").is_some());
        assert!(preview_value["documents"][0].get("content").is_some());
        assert!(preview_value["documents"][0].get("editable").is_some());
    }
}
