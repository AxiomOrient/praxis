use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{LibraryStats, LibraryStoreSnapshot, SourceCatalog, SourceRef};
use crate::source::hash_directory;
use crate::workspace::WorkspacePaths;

pub fn ensure_library_store(paths: &WorkspacePaths) -> Result<()> {
    let conn = open_connection(paths)?;
    init_schema(&conn)?;
    Ok(())
}

pub fn read_library_store_snapshot(paths: &WorkspacePaths) -> Result<LibraryStoreSnapshot> {
    ensure_library_store(paths)?;
    let conn = open_connection(paths)?;

    let sources = count_rows(&conn, "sources")?;
    let snapshots = count_rows(&conn, "source_snapshots")?;
    let artifacts = count_rows(&conn, "artifacts")?;

    Ok(LibraryStoreSnapshot {
        db_path: paths.library_db_path.to_string_lossy().to_string(),
        artifact_root: paths.library_dir.to_string_lossy().to_string(),
        stats: LibraryStats {
            sources,
            snapshots,
            artifacts,
        },
    })
}

pub fn sync_catalog_to_library(
    paths: &WorkspacePaths,
    catalog: &SourceCatalog,
    import_mode: &str,
) -> Result<()> {
    ensure_library_store(paths)?;

    let snapshot_id = source_snapshot_id(catalog);
    let import_manifest_path = paths.library_imports_dir.join(format!("{snapshot_id}.json"));
    fs::write(
        &import_manifest_path,
        serde_json::to_string_pretty(catalog).context("serialize source catalog")?,
    )
    .with_context(|| {
        format!(
            "failed to write import manifest {}",
            import_manifest_path.display()
        )
    })?;

    let mut conn = open_connection(paths)?;
    let tx = conn.transaction().context("start library transaction")?;
    let now = Utc::now().to_rfc3339();

    tx.execute(
        "INSERT INTO sources (
            source_id,
            canonical_locator,
            source_kind,
            display_name,
            created_at,
            updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?5)
        ON CONFLICT(source_id) DO UPDATE SET
            canonical_locator = excluded.canonical_locator,
            source_kind = excluded.source_kind,
            display_name = excluded.display_name,
            updated_at = excluded.updated_at",
        params![
            &catalog.source_id,
            canonical_locator(&catalog.source),
            source_kind(&catalog.source),
            &catalog.label,
            &now,
        ],
    )
    .context("upsert source record")?;

    tx.execute(
        "INSERT INTO source_snapshots (
            snapshot_id,
            source_id,
            resolved_reference,
            source_hash,
            scan_status,
            warnings_json,
            notes_json,
            import_mode,
            fetched_at,
            checkout_root,
            import_manifest_path
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ON CONFLICT(snapshot_id) DO UPDATE SET
            resolved_reference = excluded.resolved_reference,
            source_hash = excluded.source_hash,
            scan_status = excluded.scan_status,
            warnings_json = excluded.warnings_json,
            notes_json = excluded.notes_json,
            import_mode = excluded.import_mode,
            fetched_at = excluded.fetched_at,
            checkout_root = excluded.checkout_root,
            import_manifest_path = excluded.import_manifest_path",
        params![
            &snapshot_id,
            &catalog.source_id,
            catalog.resolved_reference.as_deref(),
            &catalog.source_hash,
            scan_status(catalog),
            serde_json::to_string(&catalog.warnings).context("serialize warnings")?,
            serde_json::to_string(&catalog.notes).context("serialize notes")?,
            import_mode,
            &now,
            &catalog.checkout_root,
            import_manifest_path.to_string_lossy().to_string(),
        ],
    )
    .context("upsert source snapshot record")?;

    for skill in &catalog.skills {
        let source_path = PathBuf::from(&catalog.checkout_root).join(&skill.relative_path);
        let version_id = prefixed_hash("sv", &hash_directory(&source_path)?);
        let stored_path = paths
            .library_skills_dir
            .join(sanitize_component(&skill.name))
            .join(&version_id);
        copy_tree(&source_path, &stored_path)?;

        let metadata = json!({
            "display_name": skill.display_name,
            "category": skill.category,
            "tags": skill.tags,
            "root_component": skill.root_component,
        });
        upsert_artifact(
            &tx,
            &snapshot_id,
            "skill",
            &skill.name,
            &version_id,
            &skill.relative_path,
            skill.display_name.as_deref().unwrap_or(&skill.name),
            &skill.description,
            &stored_path,
            &metadata,
            &now,
        )?;
    }

    for deck in &catalog.decks {
        let deck_json = serde_json::to_string_pretty(deck).context("serialize deck")?;
        let version_id = prefixed_hash("dv", &hash_text(&deck_json));
        let deck_dir = paths
            .library_decks_dir
            .join(sanitize_component(&deck.id))
            .join(&version_id);
        fs::create_dir_all(&deck_dir)
            .with_context(|| format!("failed to create {}", deck_dir.display()))?;
        let stored_path = deck_dir.join("deck.json");
        fs::write(&stored_path, deck_json)
            .with_context(|| format!("failed to write {}", stored_path.display()))?;

        let metadata = json!({
            "skills": deck.skills,
            "synthesized": deck.synthesized,
        });
        upsert_artifact(
            &tx,
            &snapshot_id,
            "deck",
            &deck.id,
            &version_id,
            &deck.id,
            &deck.name,
            &deck.description,
            &stored_path,
            &metadata,
            &now,
        )?;
    }

    for template in &catalog.agent_file_templates {
        let source_path = PathBuf::from(&catalog.checkout_root).join(&template.relative_path);
        let content = fs::read_to_string(&source_path)
            .with_context(|| format!("failed to read {}", source_path.display()))?;
        let version_id = prefixed_hash("gv", &hash_text(&content));
        let template_dir = paths
            .library_agent_files_dir
            .join(sanitize_component(&template.id))
            .join(&version_id);
        fs::create_dir_all(&template_dir)
            .with_context(|| format!("failed to create {}", template_dir.display()))?;
        let stored_path = template_dir.join(file_name_or_default(&source_path, "AGENTS.md"));
        fs::write(&stored_path, content)
            .with_context(|| format!("failed to write {}", stored_path.display()))?;

        let metadata = json!({
            "slots": template.slots,
            "priority": template.priority,
            "origin": template.origin,
        });
        upsert_artifact(
            &tx,
            &snapshot_id,
            "agent-file-template",
            &template.id,
            &version_id,
            &template.relative_path,
            &template.title,
            &template.description,
            &stored_path,
            &metadata,
            &now,
        )?;
    }

    if let Some(recipe) = &catalog.recipe {
        for bundle in &recipe.bundles {
            let source_path = PathBuf::from(&catalog.checkout_root).join(&bundle.relative_path);
            let version_id = if source_path.is_dir() {
                prefixed_hash("bv", &hash_directory(&source_path)?)
            } else {
                let bytes = fs::read(&source_path)
                    .with_context(|| format!("failed to read {}", source_path.display()))?;
                prefixed_hash("bv", &hash_bytes(&bytes))
            };
            let stored_path = paths
                .library_bundles_dir
                .join(sanitize_component(&bundle.id))
                .join(&version_id);
            copy_tree(&source_path, &stored_path)?;

            let metadata = json!({
                "agents": bundle.agents,
                "target_name": bundle.target_name,
            });
            upsert_artifact(
                &tx,
                &snapshot_id,
                "bundle",
                &bundle.id,
                &version_id,
                &bundle.relative_path,
                &bundle.id,
                &bundle.description,
                &stored_path,
                &metadata,
                &now,
            )?;
        }
    }

    tx.commit().context("commit library transaction")?;
    Ok(())
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
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS sources (
            source_id TEXT PRIMARY KEY,
            canonical_locator TEXT NOT NULL,
            source_kind TEXT NOT NULL,
            display_name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS source_snapshots (
            snapshot_id TEXT PRIMARY KEY,
            source_id TEXT NOT NULL REFERENCES sources(source_id) ON DELETE CASCADE,
            resolved_reference TEXT,
            source_hash TEXT NOT NULL,
            scan_status TEXT NOT NULL,
            warnings_json TEXT NOT NULL,
            notes_json TEXT NOT NULL,
            import_mode TEXT NOT NULL,
            fetched_at TEXT NOT NULL,
            checkout_root TEXT NOT NULL,
            import_manifest_path TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS artifacts (
            artifact_key TEXT PRIMARY KEY,
            source_snapshot_id TEXT NOT NULL REFERENCES source_snapshots(snapshot_id) ON DELETE CASCADE,
            artifact_kind TEXT NOT NULL,
            artifact_id TEXT NOT NULL,
            version_id TEXT NOT NULL,
            relative_path TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            content_path TEXT NOT NULL,
            metadata_json TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_source_snapshots_source_id
            ON source_snapshots(source_id);
        CREATE INDEX IF NOT EXISTS idx_artifacts_snapshot
            ON artifacts(source_snapshot_id);
        CREATE INDEX IF NOT EXISTS idx_artifacts_kind_id
            ON artifacts(artifact_kind, artifact_id);
        ",
    )
    .context("initialize library schema")?;
    Ok(())
}

fn count_rows(conn: &Connection, table: &str) -> Result<usize> {
    let mut stmt = conn
        .prepare(&format!("SELECT COUNT(*) FROM {table}"))
        .with_context(|| format!("prepare count query for {table}"))?;
    let count = stmt
        .query_row([], |row| row.get::<_, i64>(0))
        .with_context(|| format!("count rows for {table}"))?;
    Ok(count as usize)
}

fn upsert_artifact(
    tx: &rusqlite::Transaction<'_>,
    snapshot_id: &str,
    artifact_kind: &str,
    artifact_id: &str,
    version_id: &str,
    relative_path: &str,
    title: &str,
    description: &str,
    content_path: &Path,
    metadata_json: &serde_json::Value,
    created_at: &str,
) -> Result<()> {
    let artifact_key = format!("{artifact_kind}:{artifact_id}:{version_id}");
    tx.execute(
        "INSERT INTO artifacts (
            artifact_key,
            source_snapshot_id,
            artifact_kind,
            artifact_id,
            version_id,
            relative_path,
            title,
            description,
            content_path,
            metadata_json,
            created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ON CONFLICT(artifact_key) DO UPDATE SET
            source_snapshot_id = excluded.source_snapshot_id,
            relative_path = excluded.relative_path,
            title = excluded.title,
            description = excluded.description,
            content_path = excluded.content_path,
            metadata_json = excluded.metadata_json,
            created_at = excluded.created_at",
        params![
            artifact_key,
            snapshot_id,
            artifact_kind,
            artifact_id,
            version_id,
            relative_path,
            title,
            description,
            content_path.to_string_lossy().to_string(),
            serde_json::to_string(metadata_json).context("serialize artifact metadata")?,
            created_at,
        ],
    )
    .with_context(|| format!("upsert {artifact_kind} artifact '{artifact_id}'"))?;
    Ok(())
}

fn canonical_locator(source: &SourceRef) -> String {
    match source {
        SourceRef::Github {
            owner,
            repo,
            reference,
            subdir,
        } => format!(
            "github:{owner}/{repo}@{}#{}",
            reference.as_deref().unwrap_or("default"),
            subdir.as_deref().unwrap_or("root")
        ),
        SourceRef::Local { path } => format!("local:{path}"),
    }
}

fn source_kind(source: &SourceRef) -> &'static str {
    match source {
        SourceRef::Github { .. } => "github",
        SourceRef::Local { .. } => "local",
    }
}

fn scan_status(catalog: &SourceCatalog) -> &'static str {
    if catalog.warnings.is_empty() {
        "ok"
    } else {
        "warning"
    }
}

fn source_snapshot_id(catalog: &SourceCatalog) -> String {
    let seed = format!(
        "{}:{}:{}",
        catalog.source_id,
        catalog.resolved_reference.as_deref().unwrap_or("unresolved"),
        catalog.source_hash
    );
    prefixed_hash("ss", &hash_text(&seed))
}

fn prefixed_hash(prefix: &str, hex_seed: &str) -> String {
    format!("{prefix}_{}", &hex_seed[..16.min(hex_seed.len())])
}

fn sanitize_component(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "artifact".to_string()
    } else {
        out
    }
}

fn file_name_or_default(path: &Path, default_name: &str) -> String {
    path.file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| default_name.to_string())
}

fn copy_tree(source: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        if target.is_dir() {
            fs::remove_dir_all(target)
                .with_context(|| format!("failed to replace {}", target.display()))?;
        } else {
            fs::remove_file(target)
                .with_context(|| format!("failed to replace {}", target.display()))?;
        }
    }

    if source.is_dir() {
        copy_dir(source, target)
    } else {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::copy(source, target).with_context(|| {
            format!("failed to copy {} to {}", source.display(), target.display())
        })?;
        Ok(())
    }
}

fn copy_dir(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target).with_context(|| format!("failed to create {}", target.display()))?;
    for entry in
        fs::read_dir(source).with_context(|| format!("failed to list {}", source.display()))?
    {
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

fn hash_text(value: &str) -> String {
    hash_bytes(value.as_bytes())
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AgentFileSlot, AgentFileTemplate, AgentFileTemplateOrigin, DeckInfo, SkillInfo};
    use crate::model::{RecipeHint, RecipeBundle};
    use crate::workspace::ensure_workspace;
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
    fn ensure_library_store_creates_database_and_schema() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        fs::create_dir_all(&repo_root).expect("repo root");
        let paths = sample_paths(&repo_root);

        ensure_workspace(&paths).expect("ensure workspace");
        ensure_library_store(&paths).expect("ensure library");
        let snapshot = read_library_store_snapshot(&paths).expect("read snapshot");

        assert!(paths.library_db_path.is_file());
        assert_eq!(snapshot.stats.sources, 0);
        assert_eq!(snapshot.stats.snapshots, 0);
        assert_eq!(snapshot.stats.artifacts, 0);
    }

    #[test]
    fn sync_catalog_to_library_persists_snapshot_and_artifacts() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path().join("repo");
        let checkout_root = repo_root.join("fixture");
        fs::create_dir_all(checkout_root.join("demo-skill")).expect("skill dir");
        fs::write(
            checkout_root.join("demo-skill").join("SKILL.md"),
            "# Demo\n",
        )
        .expect("write skill");
        fs::write(checkout_root.join("AGENTS.md"), "System instructions\n").expect("write guide");
        fs::create_dir_all(checkout_root.join("bundle")).expect("bundle dir");
        fs::write(checkout_root.join("bundle").join("README.md"), "bundle\n").expect("bundle file");

        let paths = sample_paths(&repo_root);
        ensure_workspace(&paths).expect("ensure workspace");

        let catalog = SourceCatalog {
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
            recipe: Some(RecipeHint {
                key: "demo".to_string(),
                label: "Demo".to_string(),
                description: "Demo recipe".to_string(),
                bundles: vec![RecipeBundle {
                    id: "demo-bundle".to_string(),
                    relative_path: "bundle".to_string(),
                    target_name: "demo-bundle".to_string(),
                    agents: vec![crate::model::Agent::Codex],
                    description: "Demo bundle".to_string(),
                }],
                notes: Vec::new(),
                recommended_agent_file_templates: Vec::new(),
            }),
            warnings: vec!["minor warning".to_string()],
            notes: vec!["recipe note".to_string()],
        };

        sync_catalog_to_library(&paths, &catalog, "manual").expect("sync library");
        let snapshot = read_library_store_snapshot(&paths).expect("read library snapshot");

        assert_eq!(snapshot.stats.sources, 1);
        assert_eq!(snapshot.stats.snapshots, 1);
        assert_eq!(snapshot.stats.artifacts, 4);
        assert!(paths.library_imports_dir.join(format!("{}.json", source_snapshot_id(&catalog))).is_file());
    }
}
