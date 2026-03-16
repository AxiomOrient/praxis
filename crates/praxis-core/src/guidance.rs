use anyhow::{Context, Result};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

static STRIP_RE: OnceLock<Regex> = OnceLock::new();
static PARSE_RE: OnceLock<Regex> = OnceLock::new();
const GUIDE_MARKER: &str = "praxis";

use crate::model::{
    GuidanceSnapshot, GuidanceWriteRequest, GuideKind, GuideState, ManagedGuideBlock,
    WorkspaceManifest,
};
use crate::workspace::{
    ensure_workspace, guide_target_path, load_manifest, resolve_workspace_paths, WorkspacePaths,
};

#[derive(Debug, Clone)]
pub struct DesiredGuideBlock {
    pub source_id: String,
    pub source_hash: String,
    pub resolved_reference: Option<String>,
    pub asset_id: String,
    pub kind: GuideKind,
    pub content_hash: String,
    pub content: String,
    pub target_path: PathBuf,
}

pub fn read_guidance_state(
    scope: crate::model::Scope,
    root: Option<String>,
) -> Result<GuidanceSnapshot> {
    let paths = resolve_workspace_paths(scope, root)?;
    ensure_workspace(&paths)?;
    let manifest = load_manifest(&paths.manifest_path)?;

    let kinds = [
        GuideKind::CodexAgents,
        GuideKind::CodexOverride,
        GuideKind::ClaudeRoot,
        GuideKind::ClaudeDot,
    ];

    let mut guides = Vec::new();
    for kind in kinds {
        guides.push(read_one_guide(&paths, &manifest, kind)?);
    }

    Ok(GuidanceSnapshot { guides })
}

pub fn write_guidance_user_content(req: GuidanceWriteRequest) -> Result<GuidanceSnapshot> {
    let paths = resolve_workspace_paths(req.scope, req.root)?;
    ensure_workspace(&paths)?;
    let manifest = load_manifest(&paths.manifest_path)?;
    let target = guide_target_path(&paths, &manifest.settings, &req.kind);
    let current = match fs::read_to_string(&target) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.into()),
    };
    let managed_blocks = parse_managed_blocks(&current);
    let desired = render_guide(&req.content, &managed_blocks);
    if desired.trim().is_empty() {
        if let Err(e) = fs::remove_file(&target) {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(e.into());
            }
        }
    } else {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, desired)?;
    }

    read_guidance_state(
        paths.scope,
        paths.repo_root.map(|p| p.to_string_lossy().to_string()),
    )
}

pub fn apply_guides(
    paths: &WorkspacePaths,
    manifest: &WorkspaceManifest,
    desired: &[DesiredGuideBlock],
) -> Result<()> {
    let kinds = [
        GuideKind::CodexAgents,
        GuideKind::CodexOverride,
        GuideKind::ClaudeRoot,
        GuideKind::ClaudeDot,
    ];
    let mut processed_targets: BTreeSet<PathBuf> = BTreeSet::new();

    for kind in kinds {
        let target = guide_target_path(paths, &manifest.settings, &kind);
        if !processed_targets.insert(target.clone()) {
            continue;
        }
        let current = match fs::read_to_string(&target) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e.into()),
        };
        let user_content = strip_managed_blocks(&current);
        let block_sources: Vec<String> = desired
            .iter()
            .filter(|block| block.kind == kind)
            .map(|block| {
                format!(
                    "<!-- {}:begin source={} asset={} hash={} -->\n{}\n<!-- {}:end -->",
                    GUIDE_MARKER,
                    block.source_id,
                    block.asset_id,
                    block.content_hash,
                    block.content.trim(),
                    GUIDE_MARKER
                )
            })
            .collect();

        let rendered = render_guide_from_strings(&user_content, &block_sources);

        if rendered.trim().is_empty() {
            if let Err(e) = fs::remove_file(&target) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(e.into());
                }
            }
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&target, &rendered)?;
        }

        if kind == GuideKind::CodexAgents && manifest.settings.write_agent_md_alias {
            if rendered.trim().is_empty() {
                if let Err(e) = fs::remove_file(&paths.codex_agent_alias_path) {
                    if e.kind() != std::io::ErrorKind::NotFound {
                        return Err(e.into());
                    }
                }
            } else {
                fs::write(&paths.codex_agent_alias_path, &rendered)?;
            }
        }
    }

    if !manifest.settings.write_agent_md_alias {
        if let Err(e) = fs::remove_file(&paths.codex_agent_alias_path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(e.into());
            }
        }
    }

    Ok(())
}

pub fn render_guide(user_content: &str, blocks: &[ManagedGuideBlock]) -> String {
    let rendered_blocks: Vec<String> = blocks
        .iter()
        .map(|block| format!(
            "<!-- {}:begin source={} asset={} hash={} -->\n<!-- managed content unavailable in editor-only flow -->\n<!-- {}:end -->",
            GUIDE_MARKER, block.source_id, block.asset_id, block.content_hash, GUIDE_MARKER
        ))
        .collect();

    render_guide_from_strings(user_content, &rendered_blocks)
}

pub fn render_guide_from_strings(user_content: &str, blocks: &[String]) -> String {
    let mut parts = Vec::new();

    let user = user_content.trim();
    if !user.is_empty() {
        parts.push(user.to_string());
    }

    for block in blocks {
        parts.push(block.trim().to_string());
    }

    parts.join("\n\n")
}

pub fn strip_managed_blocks(content: &str) -> String {
    let re = STRIP_RE.get_or_init(|| {
        Regex::new(r"(?s)<!-- praxis:begin .*?-->.*?<!-- praxis:end -->\n?")
            .expect("regex should compile")
    });
    re.replace_all(content, "").trim().to_string()
}

pub fn parse_managed_blocks(content: &str) -> Vec<ManagedGuideBlock> {
    let re = PARSE_RE.get_or_init(|| {
        Regex::new(
            r#"<!-- praxis:begin source=(?P<source>[^ ]+) asset=(?P<asset>[^ ]+) hash=(?P<hash>[^ ]+) -->"#,
        )
        .expect("regex should compile")
    });

    let mut out = Vec::new();
    for caps in re.captures_iter(content) {
        let asset = caps.name("asset").map(|m| m.as_str()).unwrap_or_default();
        let kind = match asset {
            "codex" | "codex-agents" => GuideKind::CodexAgents,
            "codex-override" => GuideKind::CodexOverride,
            "claude-root" | "claude" => GuideKind::ClaudeRoot,
            "claude-dot" => GuideKind::ClaudeDot,
            _ => GuideKind::CodexAgents,
        };
        out.push(ManagedGuideBlock {
            source_id: caps
                .name("source")
                .map(|m| m.as_str().to_string())
                .unwrap_or_default(),
            asset_id: asset.to_string(),
            kind,
            content_hash: caps
                .name("hash")
                .map(|m| m.as_str().to_string())
                .unwrap_or_default(),
        });
    }
    out
}

fn read_one_guide(
    paths: &WorkspacePaths,
    manifest: &WorkspaceManifest,
    kind: GuideKind,
) -> Result<GuideState> {
    let path = guide_target_path(paths, &manifest.settings, &kind);
    let (exists, effective_content) = match fs::read_to_string(&path) {
        Ok(s) => (true, s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (false, String::new()),
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display())),
    };
    let user_content = strip_managed_blocks(&effective_content);
    let managed_blocks = parse_managed_blocks(&effective_content);

    Ok(GuideState {
        kind,
        target_path: path.to_string_lossy().to_string(),
        exists,
        user_content,
        managed_blocks,
        effective_content,
    })
}

pub fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ClaudeGuideLocation, WorkspaceManifest, WorkspaceSettings};
    use crate::workspace::WorkspacePaths;
    use tempfile::tempdir;

    #[test]
    fn apply_guides_preserves_claude_root_when_target_path_is_shared() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let state_dir = root.join(".praxis");
        let paths = WorkspacePaths {
            scope: crate::model::Scope::Repo,
            repo_root: Some(root.to_path_buf()),
            state_dir: state_dir.clone(),
            cache_dir: state_dir.join("cache"),
            manifest_path: state_dir.join("manifest.toml"),
            lock_path: state_dir.join("lock.json"),
            codex_skills_dir: root.join(".agents").join("skills"),
            claude_skills_dir: root.join(".claude").join("skills"),
            codex_agents_path: root.join("AGENTS.md"),
            codex_override_path: root.join("AGENTS.override.md"),
            codex_agent_alias_path: root.join("AGENT.md"),
            claude_root_path: root.join("CLAUDE.md"),
            claude_dot_path: root.join(".claude").join("CLAUDE.md"),
        };
        let manifest = WorkspaceManifest {
            version: 1,
            settings: WorkspaceSettings {
                default_agents: vec![],
                write_agent_md_alias: false,
                claude_project_guide_location: ClaudeGuideLocation::Root,
            },
            installs: vec![],
        };

        apply_guides(
            &paths,
            &manifest,
            &[DesiredGuideBlock {
                source_id: "local:/tmp/source".to_string(),
                source_hash: "abc".to_string(),
                resolved_reference: None,
                asset_id: "claude-root".to_string(),
                kind: GuideKind::ClaudeRoot,
                content_hash: "hash".to_string(),
                content: "root guide".to_string(),
                target_path: root.join("CLAUDE.md"),
            }],
        )
        .expect("apply guides");

        assert_eq!(
            fs::read_to_string(root.join("CLAUDE.md")).expect("claude guide"),
            "<!-- praxis:begin source=local:/tmp/source asset=claude-root hash=hash -->\nroot guide\n<!-- praxis:end -->"
        );
    }
}
