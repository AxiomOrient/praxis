use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::agent_files::{apply_agent_files, hash_text, DesiredAgentFileBlock};
use crate::create::{
    create_skill_draft as persist_create_skill_draft, fork_skill_draft as persist_fork_skill_draft,
    preview_draft as persist_preview_draft, promote_draft as persist_promote_draft,
    read_create_snapshot, update_draft_file as persist_update_draft_file,
};
use crate::evaluation::{
    read_evaluation_snapshot, run_benchmark as persist_benchmark_run,
    submit_human_review as persist_submit_human_review,
};
use crate::library::{read_library_store_snapshot, sync_catalog_to_library};
use crate::model::{
    Agent, AgentFileTemplate, AppliedAgentFileAction, AppliedBundle, AppliedInstall, AppliedSkill,
    BenchmarkRunRequest, BenchmarkRunSummary, CreateDraftRequest, DraftPreview,
    DraftPreviewRequest, DraftUpdateRequest, DoctorCheck, DoctorReport, ForkDraftRequest,
    HumanReviewRequest, InstallPlan, InstallRequest, PlanSummary, PlannedAgentFileAction,
    PlannedBundle, PlannedSkill, PromoteDraftRequest, RemoveRequest, Scope, SkillInfo,
    SourceCatalog, SourceInstall, SourceRef, TargetProfile, WorkspaceLock, WorkspaceManifest,
    WorkspaceSnapshot,
};
use crate::parser::{canonical_source_id, parse_source_input};
use crate::source::{hash_directory, scan_source, should_skip_dir, source_cache_key};
use crate::workspace::{
    agent_file_slot_path, ensure_workspace, load_lock, load_manifest, resolve_workspace_paths,
    save_lock, save_manifest, target_paths, WorkspacePaths,
};

#[derive(Debug, Clone)]
struct DesiredSkill {
    source_id: String,
    source_hash: String,
    resolved_reference: Option<String>,
    agent: Agent,
    name: String,
    display_name: Option<String>,
    category: Option<String>,
    source_relative_path: String,
    source_path: PathBuf,
    target_path: PathBuf,
    content_hash: String,
}

#[derive(Debug, Clone)]
struct DesiredBundle {
    source_id: String,
    source_hash: String,
    resolved_reference: Option<String>,
    agent: Agent,
    id: String,
    description: String,
    source_relative_path: String,
    source_path: PathBuf,
    target_path: PathBuf,
    content_hash: String,
}

#[derive(Debug, Clone)]
struct InstallArtifacts {
    catalog: SourceCatalog,
    desired_skills: Vec<DesiredSkill>,
    desired_bundles: Vec<DesiredBundle>,
    desired_agent_file_blocks: Vec<DesiredAgentFileBlock>,
    warnings: Vec<String>,
    notes: Vec<String>,
}

pub fn init_workspace(scope: Scope, root: Option<String>) -> Result<WorkspaceSnapshot> {
    let paths = resolve_workspace_paths(scope, root)?;
    ensure_workspace(&paths)?;
    list_workspace(
        paths.scope.clone(),
        paths.repo_root.map(|p| p.to_string_lossy().to_string()),
    )
}

pub fn list_workspace(scope: Scope, root: Option<String>) -> Result<WorkspaceSnapshot> {
    let paths = resolve_workspace_paths(scope, root)?;
    ensure_workspace(&paths)?;
    let manifest = load_manifest(&paths.manifest_path)?;
    let lock = load_lock(&paths.lock_path)?;
    Ok(WorkspaceSnapshot {
        targets: target_paths(&paths, &manifest.settings),
        library: read_library_store_snapshot(&paths)?,
        evaluation: read_evaluation_snapshot(&paths)?,
        create: read_create_snapshot(&paths)?,
        manifest,
        lock,
        warnings: Vec::new(),
    })
}

pub fn inspect_source_input(
    scope: Scope,
    root: Option<String>,
    source_input: &str,
) -> Result<SourceCatalog> {
    let paths = resolve_workspace_paths(scope, root)?;
    ensure_workspace(&paths)?;
    let source = parse_source_input(source_input)?;
    let catalog = scan_source(&source, &paths.cache_dir)?;
    sync_catalog_to_library(&paths, &catalog, "manual")?;
    Ok(catalog)
}

pub fn plan_install(req: InstallRequest) -> Result<InstallPlan> {
    let paths = resolve_workspace_paths(req.scope.clone(), req.root.clone())?;
    ensure_workspace(&paths)?;

    let manifest = load_manifest(&paths.manifest_path)?;
    let old_lock = load_lock(&paths.lock_path)?;
    let source = parse_source_input(&req.source)?;
    let install = build_install_record(&manifest, source, &req)?;

    let artifacts = collect_install_artifacts(&paths, &manifest, &install)?;

    let mut conflicts = collect_skill_collisions(&artifacts.desired_skills);
    conflicts.extend(collect_unmanaged_conflicts(
        &artifacts.desired_skills,
        &artifacts.desired_bundles,
        &old_lock,
    ));
    conflicts.sort();
    conflicts.dedup();

    Ok(build_install_plan(
        &artifacts.catalog,
        &install,
        &paths,
        &manifest,
        artifacts.desired_skills,
        artifacts.desired_bundles,
        artifacts.desired_agent_file_blocks,
        artifacts.warnings,
        artifacts.notes,
        conflicts,
    ))
}

pub fn install_source(req: InstallRequest) -> Result<WorkspaceSnapshot> {
    let paths = resolve_workspace_paths(req.scope.clone(), req.root.clone())?;
    ensure_workspace(&paths)?;

    let mut manifest = load_manifest(&paths.manifest_path)?;
    let source = parse_source_input(&req.source)?;
    let next_install = build_install_record(&manifest, source, &req)?;

    upsert_install(&mut manifest, next_install);
    save_manifest(&paths.manifest_path, &manifest)?;
    sync_workspace(req.scope, req.root)
}

pub fn remove_from_source(req: RemoveRequest) -> Result<WorkspaceSnapshot> {
    let paths = resolve_workspace_paths(req.scope.clone(), req.root.clone())?;
    ensure_workspace(&paths)?;

    let mut manifest = load_manifest(&paths.manifest_path)?;
    let source = parse_source_input(&req.source).unwrap_or(SourceRef::Local {
        path: req.source.clone(),
    });
    let source_id = canonical_source_id(&source);

    let index = manifest
        .installs
        .iter()
        .position(|install| install.id == source_id || install.id == req.source)
        .ok_or_else(|| anyhow!("install record not found for '{}'", req.source))?;

    if req.remove_all
        || (req.decks.is_empty() && req.skills.is_empty() && req.agent_file_templates.is_empty())
    {
        manifest.installs.remove(index);
    } else {
        let install = manifest
            .installs
            .get_mut(index)
            .ok_or_else(|| anyhow!("install record vanished"))?;

        install.selection.all = false;
        install.selection.decks = subtract(&install.selection.decks, &req.decks);
        install.selection.skills = subtract(&install.selection.skills, &req.skills);
        install.selection.agent_file_templates = subtract(
            &install.selection.agent_file_templates,
            &req.agent_file_templates,
        );

        let empty = !install.selection.all
            && install.selection.decks.is_empty()
            && install.selection.skills.is_empty()
            && install.selection.agent_file_templates.is_empty();

        if empty {
            manifest.installs.remove(index);
        }
    }

    save_manifest(&paths.manifest_path, &manifest)?;
    sync_workspace(req.scope, req.root)
}

pub fn sync_workspace(scope: Scope, root: Option<String>) -> Result<WorkspaceSnapshot> {
    let paths = resolve_workspace_paths(scope.clone(), root.clone())?;
    ensure_workspace(&paths)?;

    let manifest = load_manifest(&paths.manifest_path)?;
    let old_lock = load_lock(&paths.lock_path)?;

    let mut desired_skills = Vec::new();
    let mut desired_bundles = Vec::new();
    let mut desired_agent_file_blocks: Vec<DesiredAgentFileBlock> = Vec::new();
    let mut warnings = Vec::new();

    for install in &manifest.installs {
        let mut artifacts = collect_install_artifacts(&paths, &manifest, install)
            .with_context(|| format!("failed to scan source '{}'", install.id))?;

        sync_catalog_to_library(&paths, &artifacts.catalog, "compat-install").with_context(
            || format!("failed to sync library metadata for '{}'", install.id),
        )?;

        desired_skills.append(&mut artifacts.desired_skills);
        desired_bundles.append(&mut artifacts.desired_bundles);
        desired_agent_file_blocks.append(&mut artifacts.desired_agent_file_blocks);
        warnings.extend(artifacts.warnings);
        warnings.extend(artifacts.notes);
    }

    ensure_no_conflicts(&desired_skills, &desired_bundles, &old_lock)?;

    apply_bundles(&desired_bundles, &old_lock)?;
    apply_skills(&desired_skills, &old_lock)?;
    apply_agent_files(&paths, &manifest, &desired_agent_file_blocks)?;
    prune_cache(&paths, &manifest)?;

    let new_lock = build_lock(
        &manifest,
        &desired_skills,
        &desired_bundles,
        &desired_agent_file_blocks,
    );
    save_lock(&paths.lock_path, &new_lock)?;

    Ok(WorkspaceSnapshot {
        targets: target_paths(&paths, &manifest.settings),
        library: read_library_store_snapshot(&paths)?,
        evaluation: read_evaluation_snapshot(&paths)?,
        create: read_create_snapshot(&paths)?,
        manifest,
        lock: new_lock,
        warnings,
    })
}

pub fn update_workspace(scope: Scope, root: Option<String>) -> Result<WorkspaceSnapshot> {
    sync_workspace(scope, root)
}

pub fn benchmark_source(req: BenchmarkRunRequest) -> Result<BenchmarkRunSummary> {
    let paths = resolve_workspace_paths(req.scope.clone(), req.root.clone())?;
    ensure_workspace(&paths)?;

    let manifest = load_manifest(&paths.manifest_path)?;
    let source = parse_source_input(&req.source)?;
    let catalog = scan_source(&source, &paths.cache_dir)?;
    let baseline_source_id = manifest.installs.first().map(|install| install.id.as_str());
    let mode = req.mode.as_deref().unwrap_or("deterministic");

    persist_benchmark_run(&paths, &req.suite_id, &catalog, baseline_source_id, mode)
}

pub fn submit_human_review(req: HumanReviewRequest) -> Result<BenchmarkRunSummary> {
    let paths = resolve_workspace_paths(req.scope, req.root)?;
    ensure_workspace(&paths)?;
    persist_submit_human_review(&paths, &req.run_id, &req.decision, &req.note)
}

pub fn create_draft(req: CreateDraftRequest) -> Result<DraftPreview> {
    let paths = resolve_workspace_paths(req.scope, req.root)?;
    ensure_workspace(&paths)?;
    persist_create_skill_draft(&paths, &req.name, &req.description, &req.preset)
}

pub fn preview_draft(req: DraftPreviewRequest) -> Result<DraftPreview> {
    let paths = resolve_workspace_paths(req.scope, req.root)?;
    ensure_workspace(&paths)?;
    persist_preview_draft(&paths, &req.draft_id)
}

pub fn promote_draft(req: PromoteDraftRequest) -> Result<DraftPreview> {
    let paths = resolve_workspace_paths(req.scope, req.root)?;
    ensure_workspace(&paths)?;
    persist_promote_draft(&paths, &req.draft_id, req.destination_root.as_deref())
}

pub fn fork_draft(req: ForkDraftRequest) -> Result<DraftPreview> {
    let paths = resolve_workspace_paths(req.scope.clone(), req.root.clone())?;
    ensure_workspace(&paths)?;

    let source = parse_source_input(&req.source)?;
    let catalog = scan_source(&source, &paths.cache_dir)?;
    persist_fork_skill_draft(
        &paths,
        &catalog,
        &req.skill_name,
        req.draft_name.as_deref(),
        req.description.as_deref(),
    )
}

pub fn update_draft(req: DraftUpdateRequest) -> Result<DraftPreview> {
    let paths = resolve_workspace_paths(req.scope, req.root)?;
    ensure_workspace(&paths)?;
    persist_update_draft_file(&paths, &req.draft_id, &req.relative_path, &req.content)
}

pub fn doctor_workspace(scope: Scope, root: Option<String>) -> Result<DoctorReport> {
    let paths = resolve_workspace_paths(scope, root)?;
    ensure_workspace(&paths)?;

    let manifest = load_manifest(&paths.manifest_path)?;
    let lock = load_lock(&paths.lock_path)?;

    let mut checks = Vec::new();
    let mut planned_skills = Vec::new();
    let mut planned_bundles = Vec::new();

    if let Err(err) = validate_runtime_file_targets(&manifest.settings.target_profile, &[]) {
        checks.push(DoctorCheck {
            level: "error".to_string(),
            code: "invalid-target-profile".to_string(),
            message: err.to_string(),
        });
    }

    let mut ids = BTreeSet::new();
    for install in &manifest.installs {
        if let Err(err) = validate_runtime_file_targets(&manifest.settings.target_profile, &install.targets)
        {
            checks.push(DoctorCheck {
                level: "error".to_string(),
                code: "invalid-runtime-target".to_string(),
                message: format!("install '{}': {}", install.id, err),
            });
        }

        if !ids.insert(install.id.clone()) {
            checks.push(DoctorCheck {
                level: "error".to_string(),
                code: "duplicate-source-install".to_string(),
                message: format!("manifest contains duplicate source id '{}'", install.id),
            });
        }

        let empty = !install.selection.all
            && install.selection.decks.is_empty()
            && install.selection.skills.is_empty()
            && install.selection.agent_file_templates.is_empty();
        if empty {
            checks.push(DoctorCheck {
                level: "warning".to_string(),
                code: "empty-source-selection".to_string(),
                message: format!("source '{}' has an empty selection", install.id),
            });
        }

        let catalog = match scan_source(&install.source, &paths.cache_dir) {
            Ok(catalog) => catalog,
            Err(err) => {
                checks.push(DoctorCheck {
                    level: "error".to_string(),
                    code: "source-scan-failed".to_string(),
                    message: format!("failed to scan source '{}': {err}", install.id),
                });
                continue;
            }
        };

        let mut artifacts =
            match collect_install_artifacts_from_catalog(&paths, &manifest, install, &catalog) {
                Ok(parts) => parts,
                Err(err) => {
                    checks.push(DoctorCheck {
                        level: "error".to_string(),
                        code: "install-plan-failed".to_string(),
                        message: format!("source '{}': {err}", install.id),
                    });
                    continue;
                }
            };

        for warning in artifacts.warnings.drain(..) {
            checks.push(DoctorCheck {
                level: "warning".to_string(),
                code: "source-warning".to_string(),
                message: format!("source '{}': {warning}", install.id),
            });
        }

        planned_skills.append(&mut artifacts.desired_skills);
        planned_bundles.append(&mut artifacts.desired_bundles);
    }

    for conflict in collect_skill_collisions(&planned_skills) {
        checks.push(DoctorCheck {
            level: "error".to_string(),
            code: "skill-collision".to_string(),
            message: conflict,
        });
    }

    for conflict in collect_unmanaged_conflicts(&planned_skills, &planned_bundles, &lock) {
        checks.push(DoctorCheck {
            level: "error".to_string(),
            code: "unmanaged-conflict".to_string(),
            message: conflict,
        });
    }

    for applied in &lock.installs {
        for skill in &applied.skills {
            let path = PathBuf::from(&skill.target_path);
            if !path.exists() {
                checks.push(DoctorCheck {
                    level: "warning".to_string(),
                    code: "missing-managed-skill".to_string(),
                    message: format!("managed skill path missing: {}", path.display()),
                });
            }
        }
        for bundle in &applied.bundles {
            let path = PathBuf::from(&bundle.target_path);
            if !path.exists() {
                checks.push(DoctorCheck {
                    level: "warning".to_string(),
                    code: "missing-managed-bundle".to_string(),
                    message: format!("managed bundle path missing: {}", path.display()),
                });
            }
        }
        for action in &applied.agent_file_actions {
            let path = PathBuf::from(&action.target_path);
            if !path.exists() {
                checks.push(DoctorCheck {
                    level: "warning".to_string(),
                    code: "missing-managed-agent-file".to_string(),
                    message: format!("managed agent file path missing: {}", path.display()),
                });
            }
        }
    }

    if manifest.settings.write_codex_agent_alias {
        let agents = &paths.codex_project_agents_path;
        let alias = &paths.codex_agent_alias_path;
        if agents.exists() && alias.exists() {
            let a = fs::read_to_string(agents).unwrap_or_default();
            let b = fs::read_to_string(alias).unwrap_or_default();
            if a != b {
                checks.push(DoctorCheck {
                    level: "warning".to_string(),
                    code: "agent-alias-diverged".to_string(),
                    message: "AGENT.md alias diverged from AGENTS.md".to_string(),
                });
            }
        }
    }

    if checks.is_empty() {
        checks.push(DoctorCheck {
            level: "info".to_string(),
            code: "ok".to_string(),
            message: "no manifest/lock problems detected".to_string(),
        });
    }

    let ok = !checks.iter().any(|c| c.level == "error");
    Ok(DoctorReport { ok, checks })
}

fn build_install_record(
    manifest: &WorkspaceManifest,
    source: SourceRef,
    req: &InstallRequest,
) -> Result<SourceInstall> {
    let source_id = canonical_source_id(&source);
    let normalized_targets = normalize_targets(req.targets.clone(), manifest)?;
    let selection_is_empty = req.decks.is_empty()
        && req.skills.is_empty()
        && req.exclude_skills.is_empty()
        && req.agent_file_templates.is_empty();
    let implied_all = req.all || selection_is_empty;

    Ok(SourceInstall {
        id: source_id,
        source,
        targets: normalized_targets,
        selection: crate::model::InstallSelection {
            all: implied_all,
            decks: sort_dedup(req.decks.clone()),
            skills: sort_dedup(req.skills.clone()),
            exclude_skills: sort_dedup(req.exclude_skills.clone()),
            agent_file_templates: sort_dedup(req.agent_file_templates.clone()),
        },
    })
}

fn upsert_install(manifest: &mut WorkspaceManifest, next_install: SourceInstall) {
    if let Some(pos) = manifest
        .installs
        .iter()
        .position(|i| i.id == next_install.id)
    {
        manifest.installs[pos] = next_install;
    } else {
        manifest.installs.push(next_install);
    }
    manifest.installs.sort_by(|a, b| a.id.cmp(&b.id));
}

fn collect_install_artifacts(
    paths: &WorkspacePaths,
    manifest: &WorkspaceManifest,
    install: &SourceInstall,
) -> Result<InstallArtifacts> {
    let catalog = scan_source(&install.source, &paths.cache_dir)?;
    collect_install_artifacts_from_catalog(paths, manifest, install, &catalog)
}

fn collect_install_artifacts_from_catalog(
    paths: &WorkspacePaths,
    manifest: &WorkspaceManifest,
    install: &SourceInstall,
    catalog: &SourceCatalog,
) -> Result<InstallArtifacts> {
    let selected_skills = resolve_selection(&catalog, install)?;
    let selected_agent_file_templates = resolve_agent_file_template_selection(catalog, install)?;

    // TargetProfile is currently a manifest-level declaration only. If profile-
    // aware slot filtering is introduced, apply it to selected_agent_file_templates
    // before desired blocks are expanded.
    let _target_profile = &manifest.settings.target_profile;

    // Pre-compute content hashes once — same source dir is reused across agents.
    let all_source_paths: BTreeSet<PathBuf> = catalog
        .recipe
        .as_ref()
        .into_iter()
        .flat_map(|r| {
            r.bundles
                .iter()
                .map(|b| PathBuf::from(&catalog.checkout_root).join(&b.relative_path))
        })
        .chain(
            selected_skills
                .iter()
                .filter_map(|name| find_skill(catalog, name))
                .map(|skill| PathBuf::from(&catalog.checkout_root).join(&skill.relative_path)),
        )
        .collect();

    let path_hashes: BTreeMap<PathBuf, String> = all_source_paths
        .into_iter()
        .map(|path| hash_directory(&path).map(|h| (path, h)))
        .collect::<Result<_>>()?;

    let mut desired_skills = Vec::new();
    let mut desired_bundles = Vec::new();
    let mut desired_agent_file_blocks: Vec<DesiredAgentFileBlock> = Vec::new();

    for agent in &install.targets {
        let target_root = agent_skill_root(paths, agent);
        fs::create_dir_all(&target_root)?;

        if let Some(recipe) = &catalog.recipe {
            for bundle in &recipe.bundles {
                if !bundle.agents.iter().any(|a| a == agent) {
                    continue;
                }
                let source_path = PathBuf::from(&catalog.checkout_root).join(&bundle.relative_path);
                let target_path = target_root.join(&bundle.target_name);
                let content_hash = path_hashes[&source_path].clone();
                desired_bundles.push(DesiredBundle {
                    source_id: install.id.clone(),
                    source_hash: catalog.source_hash.clone(),
                    resolved_reference: catalog.resolved_reference.clone(),
                    agent: agent.clone(),
                    id: bundle.id.clone(),
                    description: bundle.description.clone(),
                    source_relative_path: bundle.relative_path.clone(),
                    source_path,
                    target_path,
                    content_hash,
                });
            }
        }

        for skill_name in &selected_skills {
            let skill = find_skill(&catalog, skill_name).ok_or_else(|| {
                anyhow!(
                    "selected skill '{}' not found in source {}",
                    skill_name,
                    install.id
                )
            })?;
            let source_path = PathBuf::from(&catalog.checkout_root).join(&skill.relative_path);
            let target_path = target_root.join(&skill.name);
            let content_hash = path_hashes[&source_path].clone();
            desired_skills.push(DesiredSkill {
                source_id: install.id.clone(),
                source_hash: catalog.source_hash.clone(),
                resolved_reference: catalog.resolved_reference.clone(),
                agent: agent.clone(),
                name: skill.name.clone(),
                display_name: skill.display_name.clone(),
                category: skill.category.clone(),
                source_relative_path: skill.relative_path.clone(),
                source_path,
                target_path,
                content_hash,
            });
        }
    }

    for template in selected_agent_file_templates {
        let source_path = PathBuf::from(&catalog.checkout_root).join(&template.relative_path);
        let content = fs::read_to_string(&source_path).with_context(|| {
            format!(
                "failed to read agent file template {}",
                source_path.display()
            )
        })?;
        let content_hash = hash_text(&content);
        for slot in &template.slots {
            let target_path = agent_file_slot_path(paths, slot);
            desired_agent_file_blocks.push(DesiredAgentFileBlock {
                source_id: install.id.clone(),
                source_hash: catalog.source_hash.clone(),
                resolved_reference: catalog.resolved_reference.clone(),
                template_id: template.id.clone(),
                slot: slot.clone(),
                priority: template.priority,
                target_path,
                content_hash: content_hash.clone(),
                content: content.clone(),
            });
        }
    }

    let warnings = catalog.warnings.clone();
    let mut notes = catalog.notes.clone();
    if let Some(recipe) = &catalog.recipe {
        notes.extend(recipe.notes.clone());
    }

    Ok(InstallArtifacts {
        catalog: catalog.clone(),
        desired_skills,
        desired_bundles,
        desired_agent_file_blocks,
        warnings,
        notes,
    })
}

fn build_install_plan(
    catalog: &SourceCatalog,
    install: &SourceInstall,
    paths: &WorkspacePaths,
    manifest: &WorkspaceManifest,
    desired_skills: Vec<DesiredSkill>,
    desired_bundles: Vec<DesiredBundle>,
    desired_agent_file_blocks: Vec<DesiredAgentFileBlock>,
    mut warnings: Vec<String>,
    mut notes: Vec<String>,
    conflicts: Vec<String>,
) -> InstallPlan {
    warnings.sort();
    warnings.dedup();

    notes.sort();
    notes.dedup();

    let mut skills = desired_skills
        .into_iter()
        .map(|skill| PlannedSkill {
            name: skill.name,
            display_name: skill.display_name,
            category: skill.category,
            agent: skill.agent,
            source_relative_path: skill.source_relative_path,
            target_path: skill.target_path.to_string_lossy().to_string(),
        })
        .collect::<Vec<_>>();
    skills.sort_by(|a, b| a.target_path.cmp(&b.target_path));

    let mut bundles = desired_bundles
        .into_iter()
        .map(|bundle| PlannedBundle {
            id: bundle.id,
            description: bundle.description,
            agent: bundle.agent,
            source_relative_path: bundle.source_relative_path,
            target_path: bundle.target_path.to_string_lossy().to_string(),
        })
        .collect::<Vec<_>>();
    bundles.sort_by(|a, b| a.target_path.cmp(&b.target_path));

    let mut agent_file_actions = desired_agent_file_blocks
        .into_iter()
        .map(|block| {
            let template = find_agent_file_template(catalog, &block.template_id);
            PlannedAgentFileAction {
                template_id: block.template_id.clone(),
                title: template
                    .map(|t| t.title.clone())
                    .unwrap_or_else(|| block.template_id.clone()),
                slot: block.slot,
                source_relative_path: template
                    .map(|t| t.relative_path.clone())
                    .unwrap_or_default(),
                target_path: block.target_path.to_string_lossy().to_string(),
            }
        })
        .collect::<Vec<_>>();
    agent_file_actions.sort_by(|a, b| a.target_path.cmp(&b.target_path));

    let summary = PlanSummary {
        total_skills: skills.len(),
        total_agent_file_actions: agent_file_actions.len(),
        total_bundles: bundles.len(),
        codex_skills: skills
            .iter()
            .filter(|skill| skill.agent == Agent::Codex)
            .count(),
        claude_skills: skills
            .iter()
            .filter(|skill| skill.agent == Agent::Claude)
            .count(),
        gemini_skills: skills
            .iter()
            .filter(|skill| skill.agent == Agent::Gemini)
            .count(),
        codex_bundles: bundles
            .iter()
            .filter(|bundle| bundle.agent == Agent::Codex)
            .count(),
        claude_bundles: bundles
            .iter()
            .filter(|bundle| bundle.agent == Agent::Claude)
            .count(),
    };

    InstallPlan {
        source_id: install.id.clone(),
        label: catalog.label.clone(),
        resolved_reference: catalog.resolved_reference.clone(),
        source_hash: catalog.source_hash.clone(),
        targets: install.targets.clone(),
        selection: install.selection.clone(),
        target_paths: target_paths(paths, &manifest.settings),
        skills,
        bundles,
        agent_file_actions,
        warnings,
        notes,
        conflicts,
        summary,
    }
}

fn normalize_targets(targets: Vec<Agent>, manifest: &WorkspaceManifest) -> Result<Vec<Agent>> {
    let normalized = if targets.is_empty() {
        sort_dedup_agents(manifest.settings.target_profile.default_targets())
    } else {
        sort_dedup_agents(targets)
    };

    validate_runtime_file_targets(&manifest.settings.target_profile, &normalized)?;
    Ok(normalized)
}

fn validate_runtime_file_targets(profile: &TargetProfile, targets: &[Agent]) -> Result<()> {
    if profile.references_gemini_runtime_targets() {
        bail!(
            "target profile '{}' references Gemini runtime-file targets, but Gemini remains an integration target until promoted",
            profile.as_str()
        );
    }

    if targets.iter().any(|target| *target == Agent::Gemini) {
        bail!(
            "Gemini runtime-file targets are not supported yet; Gemini remains an integration target until promoted"
        );
    }

    Ok(())
}

fn resolve_selection(catalog: &SourceCatalog, install: &SourceInstall) -> Result<Vec<String>> {
    let mut names = BTreeSet::new();

    if install.selection.all {
        for skill in &catalog.skills {
            names.insert(skill.name.clone());
        }
    }

    for deck_id in &install.selection.decks {
        let deck = catalog
            .decks
            .iter()
            .find(|deck| deck.id == *deck_id)
            .ok_or_else(|| anyhow!("deck '{}' not found in source {}", deck_id, install.id))?;
        for name in &deck.skills {
            names.insert(name.clone());
        }
    }

    for skill in &install.selection.skills {
        names.insert(skill.clone());
    }

    for exclude in &install.selection.exclude_skills {
        names.remove(exclude);
    }

    for name in &names {
        if find_skill(catalog, name).is_none() {
            bail!(
                "selected skill '{}' not found in source {}",
                name,
                install.id
            );
        }
    }

    Ok(names.into_iter().collect())
}

fn resolve_agent_file_template_selection<'a>(
    catalog: &'a SourceCatalog,
    install: &SourceInstall,
) -> Result<Vec<&'a AgentFileTemplate>> {
    let mut ids = BTreeSet::new();

    if install.selection.all {
        for template in &catalog.agent_file_templates {
            ids.insert(template.id.clone());
        }
    }

    for template_id in &install.selection.agent_file_templates {
        ids.insert(template_id.clone());
    }

    ids.into_iter()
        .map(|id| {
            find_agent_file_template(catalog, &id).ok_or_else(|| {
                anyhow!(
                    "selected agent file template '{}' not found in source {}",
                    id,
                    install.id
                )
            })
        })
        .collect()
}

fn find_skill<'a>(catalog: &'a SourceCatalog, name: &str) -> Option<&'a SkillInfo> {
    catalog.skills.iter().find(|skill| skill.name == name)
}

fn find_agent_file_template<'a>(
    catalog: &'a SourceCatalog,
    id: &str,
) -> Option<&'a AgentFileTemplate> {
    catalog.agent_file_templates.iter().find(|t| t.id == id)
}

fn agent_skill_root(paths: &WorkspacePaths, agent: &Agent) -> PathBuf {
    match agent {
        Agent::Codex => paths.codex_skills_dir.clone(),
        Agent::Claude => paths.claude_skills_dir.clone(),
        Agent::Gemini => paths.gemini_skills_dir.clone(),
    }
}

fn collect_skill_collisions(skills: &[DesiredSkill]) -> Vec<String> {
    let mut ownership: BTreeMap<(String, String), String> = BTreeMap::new();
    let mut conflicts = Vec::new();

    for skill in skills {
        let key = (skill.agent.as_str().to_string(), skill.name.clone());
        if let Some(existing) = ownership.insert(key.clone(), skill.source_id.clone()) {
            if existing != skill.source_id {
                conflicts.push(format!(
                    "skill collision for agent '{}' skill '{}': owned by '{}' and '{}'",
                    key.0, key.1, existing, skill.source_id
                ));
            }
        }
    }

    conflicts.sort();
    conflicts.dedup();
    conflicts
}

fn collect_unmanaged_conflicts(
    skills: &[DesiredSkill],
    bundles: &[DesiredBundle],
    old_lock: &WorkspaceLock,
) -> Vec<String> {
    let managed_targets: BTreeSet<String> = old_lock
        .installs
        .iter()
        .flat_map(|install| {
            install
                .skills
                .iter()
                .map(|s| s.target_path.clone())
                .chain(install.bundles.iter().map(|b| b.target_path.clone()))
        })
        .collect();

    let mut conflicts = Vec::new();

    for skill in skills {
        let target = skill.target_path.to_string_lossy().to_string();
        if skill.target_path.exists() && !managed_targets.contains(&target) {
            conflicts.push(format!(
                "unmanaged target already exists: {}",
                skill.target_path.display()
            ));
        }
    }

    for bundle in bundles {
        let target = bundle.target_path.to_string_lossy().to_string();
        if bundle.target_path.exists() && !managed_targets.contains(&target) {
            conflicts.push(format!(
                "unmanaged target already exists: {}",
                bundle.target_path.display()
            ));
        }
    }

    conflicts.sort();
    conflicts.dedup();
    conflicts
}

fn ensure_no_conflicts(
    skills: &[DesiredSkill],
    bundles: &[DesiredBundle],
    old_lock: &WorkspaceLock,
) -> Result<()> {
    let mut conflicts = collect_skill_collisions(skills);
    conflicts.extend(collect_unmanaged_conflicts(skills, bundles, old_lock));
    if let Some(first) = conflicts.first() {
        bail!(first.clone());
    }
    Ok(())
}

fn apply_bundles(bundles: &[DesiredBundle], old_lock: &WorkspaceLock) -> Result<()> {
    let desired_targets: BTreeSet<String> = bundles
        .iter()
        .map(|b| b.target_path.to_string_lossy().to_string())
        .collect();

    remove_stale_managed_dirs(
        old_lock.installs.iter().flat_map(|install| {
            install
                .bundles
                .iter()
                .map(|bundle| bundle.target_path.as_str())
        }),
        &desired_targets,
    )?;
    replace_managed_dirs(
        bundles,
        |bundle| &bundle.source_path,
        |bundle| &bundle.target_path,
    )?;

    Ok(())
}

fn apply_skills(skills: &[DesiredSkill], old_lock: &WorkspaceLock) -> Result<()> {
    let desired_targets: BTreeSet<String> = skills
        .iter()
        .map(|s| s.target_path.to_string_lossy().to_string())
        .collect();

    remove_stale_managed_dirs(
        old_lock.installs.iter().flat_map(|install| {
            install
                .skills
                .iter()
                .map(|skill| skill.target_path.as_str())
        }),
        &desired_targets,
    )?;
    replace_managed_dirs(
        skills,
        |skill| &skill.source_path,
        |skill| &skill.target_path,
    )?;

    Ok(())
}

fn remove_stale_managed_dirs<'a>(
    managed_targets: impl Iterator<Item = &'a str>,
    desired_targets: &BTreeSet<String>,
) -> Result<()> {
    for target in managed_targets {
        if !desired_targets.contains(target) {
            let path = PathBuf::from(target);
            if path.exists() {
                fs::remove_dir_all(&path).with_context(|| {
                    format!("failed to remove stale managed dir {}", path.display())
                })?;
            }
        }
    }

    Ok(())
}

fn replace_managed_dirs<T>(
    desired_entries: &[T],
    source_path: impl Fn(&T) -> &Path,
    target_path: impl Fn(&T) -> &Path,
) -> Result<()> {
    for desired in desired_entries {
        let target = target_path(desired);
        if target.exists() {
            fs::remove_dir_all(target)
                .with_context(|| format!("failed to replace {}", target.display()))?;
        }
        copy_dir(source_path(desired), target)?;
    }

    Ok(())
}

fn build_lock(
    manifest: &WorkspaceManifest,
    desired_skills: &[DesiredSkill],
    desired_bundles: &[DesiredBundle],
    desired_agent_file_blocks: &[DesiredAgentFileBlock],
) -> WorkspaceLock {
    let mut by_source: BTreeMap<String, AppliedInstall> = BTreeMap::new();

    for desired in desired_skills {
        with_applied_install(
            &mut by_source,
            &desired.source_id,
            &desired.source_hash,
            desired.resolved_reference.clone(),
            |install| {
                install.skills.push(AppliedSkill {
                    name: desired.name.clone(),
                    agent: desired.agent.clone(),
                    source_relative_path: desired.source_relative_path.clone(),
                    target_path: desired.target_path.to_string_lossy().to_string(),
                    content_hash: desired.content_hash.clone(),
                });
            },
        );
    }

    for desired in desired_bundles {
        with_applied_install(
            &mut by_source,
            &desired.source_id,
            &desired.source_hash,
            desired.resolved_reference.clone(),
            |install| {
                install.bundles.push(AppliedBundle {
                    id: desired.id.clone(),
                    agent: desired.agent.clone(),
                    source_relative_path: desired.source_relative_path.clone(),
                    target_path: desired.target_path.to_string_lossy().to_string(),
                    content_hash: desired.content_hash.clone(),
                });
            },
        );
    }

    for desired in desired_agent_file_blocks {
        with_applied_install(
            &mut by_source,
            &desired.source_id,
            &desired.source_hash,
            desired.resolved_reference.clone(),
            |install| {
                install.agent_file_actions.push(AppliedAgentFileAction {
                    template_id: desired.template_id.clone(),
                    slot: desired.slot.clone(),
                    target_path: desired.target_path.to_string_lossy().to_string(),
                    content_hash: desired.content_hash.clone(),
                });
            },
        );
    }

    let mut installs: Vec<AppliedInstall> = manifest
        .installs
        .iter()
        .filter_map(|record| by_source.remove(&record.id))
        .collect();

    sort_applied_installs(&mut installs);

    WorkspaceLock {
        version: 1,
        generated_at: Utc::now().to_rfc3339(),
        installs,
    }
}

fn upsert_applied_install<'a>(
    by_source: &'a mut BTreeMap<String, AppliedInstall>,
    source_id: &str,
    source_hash: &str,
    resolved_reference: Option<String>,
) -> &'a mut AppliedInstall {
    by_source
        .entry(source_id.to_string())
        .or_insert_with(|| AppliedInstall {
            source_id: source_id.to_string(),
            source_hash: source_hash.to_string(),
            resolved_reference,
            skills: Vec::new(),
            bundles: Vec::new(),
            agent_file_actions: Vec::new(),
        })
}

fn with_applied_install(
    by_source: &mut BTreeMap<String, AppliedInstall>,
    source_id: &str,
    source_hash: &str,
    resolved_reference: Option<String>,
    mutate: impl FnOnce(&mut AppliedInstall),
) {
    let install = upsert_applied_install(by_source, source_id, source_hash, resolved_reference);
    mutate(install);
}

fn sort_applied_installs(installs: &mut [AppliedInstall]) {
    installs.sort_by(|a, b| a.source_id.cmp(&b.source_id));
    for install in installs {
        install
            .skills
            .sort_by(|a, b| a.target_path.cmp(&b.target_path));
        install
            .bundles
            .sort_by(|a, b| a.target_path.cmp(&b.target_path));
        install
            .agent_file_actions
            .sort_by(|a, b| a.target_path.cmp(&b.target_path));
    }
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src).with_context(|| format!("failed to list {}", src.display()))? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        let ty = entry.file_type()?;
        if ty.is_symlink() {
            bail!("refusing symlink while copying: {}", path.display());
        }
        if ty.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            copy_dir(&path, &target)?;
        } else if ty.is_file() {
            fs::copy(&path, &target).with_context(|| {
                format!("failed to copy {} -> {}", path.display(), target.display())
            })?;
            let metadata = fs::metadata(&path)?;
            fs::set_permissions(&target, metadata.permissions())?;
        }
    }
    Ok(())
}

fn prune_cache(paths: &WorkspacePaths, manifest: &WorkspaceManifest) -> Result<()> {
    if !paths.cache_dir.exists() {
        return Ok(());
    }

    let keep: BTreeSet<String> = manifest
        .installs
        .iter()
        .filter_map(|install| match &install.source {
            SourceRef::Github {
                owner,
                repo,
                reference,
                subdir,
            } => Some(source_cache_key(
                owner,
                repo,
                reference.as_deref().unwrap_or("default"),
                subdir.as_deref(),
            )),
            SourceRef::Local { .. } => None,
        })
        .collect();

    for entry in fs::read_dir(&paths.cache_dir)
        .with_context(|| format!("failed to list {}", paths.cache_dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if keep.contains(&name) {
            continue;
        }

        fs::remove_dir_all(entry.path()).with_context(|| {
            format!(
                "failed to remove stale cache directory {}",
                entry.path().display()
            )
        })?;
    }

    Ok(())
}

fn sort_dedup(mut items: Vec<String>) -> Vec<String> {
    items.sort();
    items.dedup();
    items
}

fn sort_dedup_agents(mut items: Vec<Agent>) -> Vec<Agent> {
    items.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    items.dedup_by(|a, b| a == b);
    items
}

fn subtract(existing: &[String], removed: &[String]) -> Vec<String> {
    let removed_set: BTreeSet<&str> = removed.iter().map(String::as_str).collect();
    existing
        .iter()
        .filter(|item| !removed_set.contains(item.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_files::DesiredAgentFileBlock;
    use crate::model::{
        AgentFileSlot, AgentFileTemplate, AgentFileTemplateOrigin, InstallSelection, TargetProfile,
    };
    use tempfile::tempdir;

    #[test]
    fn skill_collisions_are_deduplicated() {
        let skills = vec![
            DesiredSkill {
                source_id: "source-a".to_string(),
                source_hash: String::new(),
                resolved_reference: None,
                agent: Agent::Codex,
                name: "review".to_string(),
                display_name: None,
                category: None,
                source_relative_path: String::new(),
                source_path: PathBuf::new(),
                target_path: PathBuf::from("/tmp/review"),
                content_hash: String::new(),
            },
            DesiredSkill {
                source_id: "source-b".to_string(),
                source_hash: String::new(),
                resolved_reference: None,
                agent: Agent::Codex,
                name: "review".to_string(),
                display_name: None,
                category: None,
                source_relative_path: String::new(),
                source_path: PathBuf::new(),
                target_path: PathBuf::from("/tmp/review"),
                content_hash: String::new(),
            },
            DesiredSkill {
                source_id: "source-b".to_string(),
                source_hash: String::new(),
                resolved_reference: None,
                agent: Agent::Codex,
                name: "review".to_string(),
                display_name: None,
                category: None,
                source_relative_path: String::new(),
                source_path: PathBuf::new(),
                target_path: PathBuf::from("/tmp/review"),
                content_hash: String::new(),
            },
        ];

        let conflicts = collect_skill_collisions(&skills);
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn unmanaged_conflicts_ignore_locked_targets() {
        let dir = tempdir().expect("tempdir");
        let managed_path = dir.path().join("managed-skill");
        let unmanaged_path = dir.path().join("unmanaged-skill");
        fs::create_dir_all(&managed_path).expect("create managed");
        fs::create_dir_all(&unmanaged_path).expect("create unmanaged");

        let skills = vec![
            DesiredSkill {
                source_id: "source-a".to_string(),
                source_hash: String::new(),
                resolved_reference: None,
                agent: Agent::Codex,
                name: "managed".to_string(),
                display_name: None,
                category: None,
                source_relative_path: String::new(),
                source_path: PathBuf::new(),
                target_path: managed_path.clone(),
                content_hash: String::new(),
            },
            DesiredSkill {
                source_id: "source-b".to_string(),
                source_hash: String::new(),
                resolved_reference: None,
                agent: Agent::Codex,
                name: "unmanaged".to_string(),
                display_name: None,
                category: None,
                source_relative_path: String::new(),
                source_path: PathBuf::new(),
                target_path: unmanaged_path.clone(),
                content_hash: String::new(),
            },
        ];

        let lock = WorkspaceLock {
            version: 1,
            generated_at: String::new(),
            installs: vec![AppliedInstall {
                source_id: "source-a".to_string(),
                source_hash: String::new(),
                resolved_reference: None,
                skills: vec![AppliedSkill {
                    name: "managed".to_string(),
                    agent: Agent::Codex,
                    source_relative_path: String::new(),
                    target_path: managed_path.to_string_lossy().to_string(),
                    content_hash: String::new(),
                }],
                bundles: Vec::new(),
                agent_file_actions: Vec::new(),
            }],
        };

        let conflicts = collect_unmanaged_conflicts(&skills, &[], &lock);
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].contains(&unmanaged_path.to_string_lossy().to_string()));
    }

    #[test]
    fn build_lock_preserves_source_metadata_for_agent_file_only_install() {
        let manifest = WorkspaceManifest {
            version: 1,
            settings: Default::default(),
            installs: vec![SourceInstall {
                id: "local:/tmp/agent-file-only".to_string(),
                source: SourceRef::Local {
                    path: "/tmp/agent-file-only".to_string(),
                },
                targets: vec![Agent::Codex],
                selection: Default::default(),
            }],
        };

        let blocks = vec![DesiredAgentFileBlock {
            source_id: "local:/tmp/agent-file-only".to_string(),
            source_hash: "abc123".to_string(),
            resolved_reference: Some("deadbeef".to_string()),
            template_id: "codex-project-root".to_string(),
            slot: AgentFileSlot::CodexProjectRoot,
            priority: 100,
            content_hash: "blockhash".to_string(),
            content: "# Agents".to_string(),
            target_path: PathBuf::from("/tmp/agent-file-only/AGENTS.md"),
        }];

        let lock = build_lock(&manifest, &[], &[], &blocks);
        let install = lock.installs.first().expect("agent-file-only install");

        assert_eq!(install.source_hash, "abc123");
        assert_eq!(install.resolved_reference.as_deref(), Some("deadbeef"));
        assert_eq!(install.agent_file_actions.len(), 1);
    }

    #[test]
    fn resolve_agent_file_template_selection_includes_all_templates_when_requested() {
        let catalog = SourceCatalog {
            source_id: "local:/tmp/source".to_string(),
            label: "source".to_string(),
            source: SourceRef::Local {
                path: "/tmp/source".to_string(),
            },
            checkout_root: "/tmp/source".to_string(),
            resolved_reference: None,
            source_hash: "hash".to_string(),
            decks: Vec::new(),
            skills: Vec::new(),
            agent_file_templates: vec![
                AgentFileTemplate {
                    id: "codex-project-root".to_string(),
                    title: "Codex".to_string(),
                    description: String::new(),
                    relative_path: "AGENTS.md".to_string(),
                    slots: vec![AgentFileSlot::CodexProjectRoot],
                    priority: 100,
                    origin: AgentFileTemplateOrigin::Discovered,
                },
                AgentFileTemplate {
                    id: "claude-project-root".to_string(),
                    title: "Claude".to_string(),
                    description: String::new(),
                    relative_path: "CLAUDE.md".to_string(),
                    slots: vec![AgentFileSlot::ClaudeProjectRoot],
                    priority: 100,
                    origin: AgentFileTemplateOrigin::Discovered,
                },
            ],
            recipe: None,
            warnings: Vec::new(),
            notes: Vec::new(),
        };
        let install = SourceInstall {
            id: "local:/tmp/source".to_string(),
            source: catalog.source.clone(),
            targets: vec![Agent::Codex],
            selection: InstallSelection {
                all: true,
                decks: Vec::new(),
                skills: Vec::new(),
                exclude_skills: Vec::new(),
                agent_file_templates: Vec::new(),
            },
        };

        let selected =
            resolve_agent_file_template_selection(&catalog, &install).expect("template selection");

        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].id, "claude-project-root");
        assert_eq!(selected[1].id, "codex-project-root");
    }

    #[test]
    fn normalize_targets_defaults_from_target_profile() {
        let manifest = WorkspaceManifest {
            version: 1,
            settings: crate::model::WorkspaceSettings {
                target_profile: TargetProfile::ClaudeNative,
                write_codex_agent_alias: true,
            },
            installs: Vec::new(),
        };

        let targets = normalize_targets(Vec::new(), &manifest).expect("normalize targets");
        assert_eq!(targets, vec![Agent::Claude]);
    }

    #[test]
    fn normalize_targets_rejects_explicit_gemini_runtime_target() {
        let manifest = WorkspaceManifest::default();

        let err = normalize_targets(vec![Agent::Gemini], &manifest).expect_err("gemini rejected");
        assert!(
            err.to_string()
                .contains("Gemini runtime-file targets are not supported yet")
        );
    }

    #[test]
    fn normalize_targets_rejects_profile_that_references_gemini_runtime_targets() {
        let manifest = WorkspaceManifest {
            version: 1,
            settings: crate::model::WorkspaceSettings {
                target_profile: TargetProfile::GeminiNative,
                write_codex_agent_alias: true,
            },
            installs: Vec::new(),
        };

        let err = normalize_targets(Vec::new(), &manifest).expect_err("profile rejected");
        assert!(
            err.to_string()
                .contains("references Gemini runtime-file targets")
        );
    }
}
