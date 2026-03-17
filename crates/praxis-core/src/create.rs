use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

use crate::library::ensure_library_store;
use crate::model::{CreateSnapshot, DraftDocument, DraftPreview, DraftSummary, PreviewTreeEntry, SourceCatalog};
use crate::workspace::WorkspacePaths;

pub fn ensure_create_store(paths: &WorkspacePaths) -> Result<()> {
    ensure_library_store(paths)?;
    fs::create_dir_all(&paths.library_drafts_dir)
        .with_context(|| format!("failed to create {}", paths.library_drafts_dir.display()))?;
    let conn = open_connection(paths)?;
    init_schema(&conn)?;
    Ok(())
}

pub fn read_create_snapshot(paths: &WorkspacePaths) -> Result<CreateSnapshot> {
    ensure_create_store(paths)?;
    let conn = open_connection(paths)?;
    let mut stmt = conn
        .prepare(
            "SELECT draft_id, name, artifact_kind, version_id, preset, created_at, updated_at
             FROM drafts
             ORDER BY updated_at DESC",
        )
        .context("prepare drafts query")?;
    let drafts = stmt
        .query_map([], |row| {
            Ok(DraftSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                artifact_kind: row.get(2)?,
                version_id: row.get(3)?,
                preset: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .context("query drafts")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("collect drafts")?;

    Ok(CreateSnapshot { drafts })
}

pub fn create_skill_draft(
    paths: &WorkspacePaths,
    name: &str,
    description: &str,
    preset: &str,
) -> Result<DraftPreview> {
    ensure_create_store(paths)?;

    let sanitized_name = sanitize_name(name);
    if sanitized_name.is_empty() {
        return Err(anyhow!("draft name must contain at least one ASCII letter or number"));
    }
    let now = Utc::now().to_rfc3339();
    let draft_id = format!("draft_skill_{}", short_hash(&format!("{sanitized_name}:{now}")));
    let version_id = format!("drv_{}", short_hash(&format!("{draft_id}:{now}")));
    let draft_dir = paths.library_drafts_dir.join(&draft_id).join(&version_id);
    fs::create_dir_all(&draft_dir)
        .with_context(|| format!("failed to create {}", draft_dir.display()))?;

    let skill_body = format!(
        "# {title}\n\n{description}\n\n## Purpose\n\nDescribe what this skill should do.\n\n## Inputs\n\n- Define the expected inputs.\n\n## Outputs\n\n- Define the expected outputs.\n",
        title = name.trim()
    );
    fs::write(draft_dir.join("SKILL.md"), skill_body)
        .with_context(|| format!("failed to write {}", draft_dir.join("SKILL.md").display()))?;
    fs::write(
        draft_dir.join("draft.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "name": sanitized_name,
            "display_name": name.trim(),
            "description": description,
            "preset": preset,
        }))
        .context("serialize draft metadata")?,
    )
    .with_context(|| format!("failed to write {}", draft_dir.join("draft.json").display()))?;

    let conn = open_connection(paths)?;
    conn.execute(
        "INSERT INTO drafts (
            draft_id,
            name,
            artifact_kind,
            version_id,
            preset,
            description,
            draft_path,
            created_at,
            updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![
            &draft_id,
            &sanitized_name,
            "skill",
            &version_id,
            preset,
            description,
            draft_dir.to_string_lossy().to_string(),
            &now,
        ],
    )
    .context("insert draft record")?;

    preview_draft(paths, &draft_id)
}

pub fn preview_draft(paths: &WorkspacePaths, draft_id: &str) -> Result<DraftPreview> {
    ensure_create_store(paths)?;
    let (draft, draft_path, _description) = load_draft_record(paths, draft_id)?;
    let draft_path = PathBuf::from(&draft_path);
    let files = collect_preview_entries(&draft_path, &draft_path)?;
    let documents = collect_documents(&draft_path, &draft_path)?;
    let promotion_target = promotion_target_path(paths, &draft.name, None)?;

    Ok(DraftPreview {
        draft,
        files,
        documents,
        promotion_target,
    })
}

pub fn promote_draft(
    paths: &WorkspacePaths,
    draft_id: &str,
    destination_root: Option<&str>,
) -> Result<DraftPreview> {
    ensure_create_store(paths)?;
    let conn = open_connection(paths)?;
    let (draft, draft_path, _description) = load_draft_record(paths, draft_id)?;

    let source_dir = PathBuf::from(&draft_path);
    let promotion_target = PathBuf::from(promotion_target_path(paths, &draft.name, destination_root)?);
    if promotion_target.exists() {
        return Err(anyhow!(
            "promotion target already exists: {}",
            promotion_target.display()
        ));
    }

    copy_dir(&source_dir, &promotion_target)?;
    let updated_at = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE drafts SET updated_at = ?2 WHERE draft_id = ?1",
        params![draft_id, &updated_at],
    )
    .context("update draft promotion timestamp")?;

    Ok(DraftPreview {
        files: collect_preview_entries(&promotion_target, &promotion_target)?,
        documents: collect_documents(&promotion_target, &promotion_target)?,
        draft: DraftSummary {
            updated_at,
            ..draft
        },
        promotion_target: promotion_target.to_string_lossy().to_string(),
    })
}

pub fn update_draft_file(
    paths: &WorkspacePaths,
    draft_id: &str,
    relative_path: &str,
    content: &str,
) -> Result<DraftPreview> {
    ensure_create_store(paths)?;
    let (_draft, draft_path, _description) = load_draft_record(paths, draft_id)?;
    let relative = normalize_relative_path(relative_path)?;
    let target = PathBuf::from(&draft_path).join(&relative);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&target, content).with_context(|| format!("failed to write {}", target.display()))?;
    touch_draft(paths, draft_id)?;

    preview_draft(paths, draft_id)
}

pub fn fork_skill_draft(
    paths: &WorkspacePaths,
    catalog: &SourceCatalog,
    skill_name: &str,
    draft_name: Option<&str>,
    description: Option<&str>,
) -> Result<DraftPreview> {
    ensure_create_store(paths)?;

    let skill = catalog
        .skills
        .iter()
        .find(|skill| skill.name == skill_name)
        .ok_or_else(|| anyhow!("skill '{}' not found in source {}", skill_name, catalog.source_id))?;
    let name = draft_name.unwrap_or(skill.display_name.as_deref().unwrap_or(&skill.name));
    let description = description.unwrap_or(&skill.description);
    let sanitized_name = sanitize_name(name);
    if sanitized_name.is_empty() {
        return Err(anyhow!("forked draft name must contain at least one ASCII letter or number"));
    }

    let now = Utc::now().to_rfc3339();
    let draft_id = format!("draft_skill_{}", short_hash(&format!("fork:{sanitized_name}:{now}")));
    let version_id = format!(
        "drv_{}",
        short_hash(&format!("{}:{}:{}", draft_id, catalog.source_hash, skill.name))
    );
    let draft_dir = paths.library_drafts_dir.join(&draft_id).join(&version_id);
    let source_dir = PathBuf::from(&catalog.checkout_root).join(&skill.relative_path);
    copy_dir(&source_dir, &draft_dir)?;

    let metadata = serde_json::json!({
        "name": sanitized_name,
        "display_name": name.trim(),
        "description": description,
        "preset": "fork",
        "source_id": catalog.source_id,
        "forked_skill_name": skill.name,
    });
    fs::write(
        draft_dir.join("draft.json"),
        serde_json::to_string_pretty(&metadata).context("serialize fork draft metadata")?,
    )
    .with_context(|| format!("failed to write {}", draft_dir.join("draft.json").display()))?;

    let conn = open_connection(paths)?;
    conn.execute(
        "INSERT INTO drafts (
            draft_id,
            name,
            artifact_kind,
            version_id,
            preset,
            description,
            draft_path,
            created_at,
            updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![
            &draft_id,
            &sanitized_name,
            "skill",
            &version_id,
            "fork",
            description,
            draft_dir.to_string_lossy().to_string(),
            &now,
        ],
    )
    .context("insert fork draft record")?;

    preview_draft(paths, &draft_id)
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
        CREATE TABLE IF NOT EXISTS drafts (
            draft_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            artifact_kind TEXT NOT NULL,
            version_id TEXT NOT NULL,
            preset TEXT NOT NULL,
            description TEXT NOT NULL,
            draft_path TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_drafts_updated_at
            ON drafts(updated_at DESC);
        ",
    )
    .context("initialize create schema")?;
    Ok(())
}

fn load_draft_record(paths: &WorkspacePaths, draft_id: &str) -> Result<(DraftSummary, String, String)> {
    let conn = open_connection(paths)?;
    conn.query_row(
        "SELECT draft_id, name, artifact_kind, version_id, preset, created_at, updated_at, draft_path, description
         FROM drafts
         WHERE draft_id = ?1",
        params![draft_id],
        |row| {
            Ok((
                DraftSummary {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    artifact_kind: row.get(2)?,
                    version_id: row.get(3)?,
                    preset: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                },
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
            ))
        },
    )
    .map_err(|_| anyhow!("draft '{}' not found", draft_id))
}

fn touch_draft(paths: &WorkspacePaths, draft_id: &str) -> Result<()> {
    let conn = open_connection(paths)?;
    let updated_at = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE drafts SET updated_at = ?2 WHERE draft_id = ?1",
        params![draft_id, &updated_at],
    )
    .context("touch draft record")?;
    Ok(())
}

fn collect_preview_entries(root: &Path, dir: &Path) -> Result<Vec<PreviewTreeEntry>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("failed to list {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if file_type.is_dir() {
            entries.push(PreviewTreeEntry {
                path: rel.clone(),
                entry_kind: "dir".to_string(),
            });
            entries.extend(collect_preview_entries(root, &path)?);
        } else if file_type.is_file() {
            entries.push(PreviewTreeEntry {
                path: rel,
                entry_kind: "file".to_string(),
            });
        }
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

fn collect_documents(root: &Path, dir: &Path) -> Result<Vec<DraftDocument>> {
    let mut documents = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("failed to list {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            documents.extend(collect_documents(root, &path)?);
        } else if file_type.is_file() {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if let Ok(content) = fs::read_to_string(&path) {
                documents.push(DraftDocument {
                    path: rel,
                    content,
                    editable: true,
                });
            }
        }
    }
    documents.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(documents)
}

fn promotion_target_path(
    paths: &WorkspacePaths,
    name: &str,
    destination_root: Option<&str>,
) -> Result<String> {
    let base = if let Some(root) = destination_root {
        PathBuf::from(root)
    } else if let Some(repo_root) = &paths.repo_root {
        repo_root.join(".agents").join("skills")
    } else {
        paths.codex_skills_dir.clone()
    };
    Ok(base.join(name).to_string_lossy().to_string())
}

fn copy_dir(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target).with_context(|| format!("failed to create {}", target.display()))?;
    for entry in fs::read_dir(source).with_context(|| format!("failed to list {}", source.display()))? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir(&source_path, &target_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &target_path).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    source_path.display(),
                    target_path.display()
                )
            })?;
        }
    }
    Ok(())
}

fn normalize_relative_path(relative_path: &str) -> Result<String> {
    let path = Path::new(relative_path);
    if path.is_absolute() {
        return Err(anyhow!("draft paths must be relative"));
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(value) => normalized.push(value),
            std::path::Component::CurDir => {}
            _ => return Err(anyhow!("draft paths must not escape the draft root")),
        }
    }
    Ok(normalized.to_string_lossy().replace('\\', "/"))
}

fn sanitize_name(name: &str) -> String {
    let mut out = String::new();
    for ch in name.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if matches!(ch, '-' | '_' | ' ') {
            if !out.ends_with('-') {
                out.push('-');
            }
        }
    }
    out.trim_matches('-').to_string()
}

fn short_hash(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hex::encode(hasher.finalize());
    digest[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::{ensure_workspace, WorkspacePaths};
    use tempfile::tempdir;

    fn sample_paths(root: &Path) -> WorkspacePaths {
        let state_dir = root.join(".praxis");
        WorkspacePaths {
            scope: crate::model::Scope::Repo,
            repo_root: Some(root.to_path_buf()),
            state_dir: state_dir.clone(),
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
    fn create_skill_draft_writes_previewable_files() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        fs::create_dir_all(&repo_root).expect("repo root");
        let paths = sample_paths(&repo_root);

        ensure_workspace(&paths).expect("ensure workspace");
        let preview =
            create_skill_draft(&paths, "Demo Draft", "A demo draft", "skill").expect("create draft");

        assert_eq!(preview.draft.name, "demo-draft");
        assert!(preview.files.iter().any(|entry| entry.path == "SKILL.md"));
        assert!(preview.files.iter().any(|entry| entry.path == "draft.json"));
        assert!(preview.documents.iter().any(|entry| entry.path == "SKILL.md"));
    }

    #[test]
    fn promote_draft_writes_repo_contract_path() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        fs::create_dir_all(&repo_root).expect("repo root");
        let paths = sample_paths(&repo_root);

        ensure_workspace(&paths).expect("ensure workspace");
        let preview =
            create_skill_draft(&paths, "Promo Draft", "A promoted draft", "skill").expect("create draft");
        let promoted = promote_draft(&paths, &preview.draft.id, None).expect("promote draft");

        assert!(PathBuf::from(&promoted.promotion_target).join("SKILL.md").is_file());
        assert!(promoted.promotion_target.ends_with("/.agents/skills/promo-draft"));
    }

    #[test]
    fn update_draft_file_persists_new_content() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        fs::create_dir_all(&repo_root).expect("repo root");
        let paths = sample_paths(&repo_root);

        ensure_workspace(&paths).expect("ensure workspace");
        let preview =
            create_skill_draft(&paths, "Editable Draft", "Editable", "skill").expect("create draft");
        let updated = update_draft_file(&paths, &preview.draft.id, "SKILL.md", "# Updated\n").expect("update");

        assert_eq!(
            updated
                .documents
                .iter()
                .find(|entry| entry.path == "SKILL.md")
                .expect("skill document")
                .content,
            "# Updated\n"
        );
    }

    #[test]
    fn fork_skill_draft_copies_source_skill() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        let checkout_root = repo_root.join("fixture");
        fs::create_dir_all(checkout_root.join("demo-skill")).expect("skill dir");
        fs::write(checkout_root.join("demo-skill").join("SKILL.md"), "# Demo\n").expect("skill");
        fs::create_dir_all(&repo_root).expect("repo root");
        let paths = sample_paths(&repo_root);

        ensure_workspace(&paths).expect("ensure workspace");
        let catalog = SourceCatalog {
            source_id: "local:/tmp/demo".to_string(),
            label: "demo".to_string(),
            source: crate::model::SourceRef::Local {
                path: checkout_root.to_string_lossy().to_string(),
            },
            checkout_root: checkout_root.to_string_lossy().to_string(),
            resolved_reference: None,
            source_hash: "hash".to_string(),
            decks: Vec::new(),
            skills: vec![crate::model::SkillInfo {
                name: "demo-skill".to_string(),
                description: "Demo".to_string(),
                relative_path: "demo-skill".to_string(),
                root_component: "SKILL.md".to_string(),
                display_name: Some("Demo Skill".to_string()),
                category: None,
                tags: Vec::new(),
            }],
            agent_file_templates: Vec::new(),
            recipe: None,
            warnings: Vec::new(),
            notes: Vec::new(),
        };

        let preview =
            fork_skill_draft(&paths, &catalog, "demo-skill", Some("Forked Demo"), None).expect("fork");
        assert_eq!(preview.draft.preset, "fork");
        assert!(preview.documents.iter().any(|entry| entry.path == "SKILL.md"));
    }
}
