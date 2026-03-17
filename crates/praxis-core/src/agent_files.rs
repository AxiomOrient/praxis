//! Agent file composition engine.
//!
//! Manages the composition of managed instruction blocks into agent runtime
//! files (AGENTS.md, CLAUDE.md, GEMINI.md, etc.), preserving user-authored
//! content while deterministically ordering managed blocks.
//!
//! Block marker format (v2):
//!   <!-- praxis:begin slot=<slot-id> source=<source-id> template=<template-id> hash=<hash> -->
//!   ...content...
//!   <!-- praxis:end -->
//!
//! Backward-compatible with the v1 marker format:
//!   <!-- praxis:begin source=<source-id> asset=<asset-id> hash=<hash> -->

use anyhow::{Context, Result};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static BEGIN_V2_RE: OnceLock<Regex> = OnceLock::new();
static BEGIN_V1_RE: OnceLock<Regex> = OnceLock::new();
const MARKER: &str = "praxis";
const END_MARKER: &str = "<!-- praxis:end -->";
const ALL_AGENT_FILE_SLOTS: [AgentFileSlot; 9] = [
    AgentFileSlot::CodexProjectRoot,
    AgentFileSlot::CodexProjectOverride,
    AgentFileSlot::CodexUserRoot,
    AgentFileSlot::CodexUserOverride,
    AgentFileSlot::ClaudeProjectRoot,
    AgentFileSlot::ClaudeProjectDot,
    AgentFileSlot::ClaudeUserRoot,
    AgentFileSlot::GeminiProjectRoot,
    AgentFileSlot::GeminiUserRoot,
];

use crate::model::{
    AgentFileSlot, AgentFileSnapshot, AgentFileState, AgentFileWriteRequest, ManagedAgentFileBlock,
    WorkspaceManifest,
};
use crate::workspace::{
    agent_file_slot_path, ensure_workspace, load_manifest, resolve_workspace_paths, WorkspacePaths,
};

/// A desired managed block to be composed into an agent file slot.
#[derive(Debug, Clone)]
pub struct DesiredAgentFileBlock {
    pub source_id: String,
    pub source_hash: String,
    pub resolved_reference: Option<String>,
    pub template_id: String,
    pub slot: AgentFileSlot,
    pub priority: u32,
    pub content_hash: String,
    pub content: String,
    pub target_path: PathBuf,
}

/// Read the current state of all canonical agent file slots.
pub fn read_agent_file_state(
    scope: crate::model::Scope,
    root: Option<String>,
) -> Result<AgentFileSnapshot> {
    let paths = resolve_workspace_paths(scope, root)?;
    ensure_workspace(&paths)?;
    let _manifest = load_manifest(&paths.manifest_path)?;

    let mut slots = Vec::new();
    for slot in ALL_AGENT_FILE_SLOTS {
        slots.push(read_one_slot(&paths, slot)?);
    }

    Ok(AgentFileSnapshot { slots })
}

/// Write user-authored content to an agent file slot, preserving managed blocks.
pub fn write_agent_file_user_content(req: AgentFileWriteRequest) -> Result<AgentFileSnapshot> {
    let paths = resolve_workspace_paths(req.scope, req.root)?;
    ensure_workspace(&paths)?;
    let target = agent_file_slot_path(&paths, &req.slot);
    let current = match fs::read_to_string(&target) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.into()),
    };
    let managed_blocks = parse_managed_blocks(&current);
    let desired = render_agent_file(&req.content, &managed_blocks);
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

    read_agent_file_state(
        paths.scope,
        paths.repo_root.map(|p| p.to_string_lossy().to_string()),
    )
}

/// Apply desired managed blocks into all affected agent file slots.
///
/// Blocks are sorted deterministically per spec §11.4:
///   slot ordinal → priority ascending → source_id lexicographic → template_id lexicographic
pub fn apply_agent_files(
    paths: &WorkspacePaths,
    manifest: &WorkspaceManifest,
    desired: &[DesiredAgentFileBlock],
) -> Result<()> {
    // Track processed target paths to avoid double-writing aliased paths.
    let mut processed_targets: BTreeSet<PathBuf> = BTreeSet::new();
    let desired_by_slot = desired_blocks_by_slot(desired);

    for slot in &ALL_AGENT_FILE_SLOTS {
        let target = agent_file_slot_path(paths, slot);
        if !processed_targets.insert(target.clone()) {
            continue;
        }
        let slot_blocks = desired_by_slot.get(slot).cloned().unwrap_or_default();
        if slot_blocks.is_empty() && !target.exists() {
            continue;
        }

        let rendered = render_slot_file(&target, slot, &slot_blocks)?;
        write_rendered_slot(&target, &rendered)?;

        // Write AGENT.md alias when enabled and this is the Codex project root slot.
        if *slot == AgentFileSlot::CodexProjectRoot && manifest.settings.write_codex_agent_alias {
            write_rendered_slot(&paths.codex_agent_alias_path, &rendered)?;
        }
    }

    // If alias disabled, prune any existing alias file.
    if !manifest.settings.write_codex_agent_alias {
        if let Err(e) = fs::remove_file(&paths.codex_agent_alias_path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(e.into());
            }
        }
    }

    Ok(())
}

/// Render the final agent file content from user content and managed blocks (editor-only flow).
pub fn render_agent_file(user_content: &str, blocks: &[ManagedAgentFileBlock]) -> String {
    let rendered_blocks: Vec<String> = blocks
        .iter()
        .map(|block| {
            format!(
                "<!-- {}:begin slot={} source={} template={} hash={} -->\n<!-- managed content unavailable in editor-only flow -->\n<!-- {}:end -->",
                MARKER,
                block.slot.as_str(),
                block.source_id,
                block.template_id,
                block.content_hash,
                MARKER
            )
        })
        .collect();

    render_agent_file_from_strings(user_content, &rendered_blocks)
}

pub fn render_agent_file_from_strings(user_content: &str, blocks: &[String]) -> String {
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
    let lines = split_lines(content);
    let spans = parse_managed_block_spans(&lines);
    if spans.is_empty() {
        return content.trim().to_string();
    }

    let mut out = String::new();
    let mut cursor = 0usize;
    for span in spans {
        for line in &lines[cursor..span.start_line] {
            out.push_str(line);
        }
        cursor = span.end_line_exclusive;
    }
    for line in &lines[cursor..] {
        out.push_str(line);
    }

    out.trim().to_string()
}

pub fn parse_managed_blocks(content: &str) -> Vec<ManagedAgentFileBlock> {
    let lines = split_lines(content);
    parse_managed_block_spans(&lines)
        .into_iter()
        .map(|span| span.block)
        .collect()
}

fn parse_slot_id(s: &str) -> AgentFileSlot {
    match s {
        "codex-user-root" => AgentFileSlot::CodexUserRoot,
        "codex-user-override" => AgentFileSlot::CodexUserOverride,
        "codex-project-root" => AgentFileSlot::CodexProjectRoot,
        "codex-project-override" => AgentFileSlot::CodexProjectOverride,
        "claude-user-root" => AgentFileSlot::ClaudeUserRoot,
        "claude-project-root" => AgentFileSlot::ClaudeProjectRoot,
        "claude-project-dot" => AgentFileSlot::ClaudeProjectDot,
        "gemini-user-root" => AgentFileSlot::GeminiUserRoot,
        "gemini-project-root" => AgentFileSlot::GeminiProjectRoot,
        _ => AgentFileSlot::CodexProjectRoot,
    }
}

#[derive(Debug, Clone)]
struct ManagedBlockSpan {
    block: ManagedAgentFileBlock,
    start_line: usize,
    end_line_exclusive: usize,
}

fn desired_blocks_by_slot<'a>(
    desired: &'a [DesiredAgentFileBlock],
) -> BTreeMap<AgentFileSlot, Vec<&'a DesiredAgentFileBlock>> {
    let mut out: BTreeMap<AgentFileSlot, Vec<&DesiredAgentFileBlock>> = BTreeMap::new();
    for block in desired {
        out.entry(block.slot.clone()).or_default().push(block);
    }
    for blocks in out.values_mut() {
        blocks.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.source_id.cmp(&b.source_id))
                .then_with(|| a.template_id.cmp(&b.template_id))
        });
    }
    out
}

fn render_slot_file(
    target: &Path,
    slot: &AgentFileSlot,
    desired_blocks: &[&DesiredAgentFileBlock],
) -> Result<String> {
    let current = read_optional_text(target)?;
    let user_content = strip_managed_blocks(&current);
    let rendered_blocks = desired_blocks
        .iter()
        .map(|block| render_managed_block(slot, block))
        .collect::<Vec<_>>();
    Ok(render_agent_file_from_strings(
        &user_content,
        &rendered_blocks,
    ))
}

fn render_managed_block(slot: &AgentFileSlot, block: &DesiredAgentFileBlock) -> String {
    format!(
        "<!-- {}:begin slot={} source={} template={} hash={} -->\n{}\n<!-- {}:end -->",
        MARKER,
        slot.as_str(),
        block.source_id,
        block.template_id,
        block.content_hash,
        block.content.trim(),
        MARKER
    )
}

fn write_rendered_slot(path: &Path, rendered: &str) -> Result<()> {
    if rendered.trim().is_empty() {
        remove_file_if_exists(path)
    } else {
        write_text_if_changed(path, rendered)
    }
}

fn write_text_if_changed(path: &Path, content: &str) -> Result<()> {
    if read_optional_text(path)? == content {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

fn remove_file_if_exists(path: &Path) -> Result<()> {
    if let Err(e) = fs::remove_file(path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(e.into());
        }
    }
    Ok(())
}

fn read_optional_text(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e.into()),
    }
}

fn split_lines(content: &str) -> Vec<&str> {
    if content.is_empty() {
        return Vec::new();
    }
    content.split_inclusive('\n').collect()
}

fn parse_managed_block_spans(lines: &[&str]) -> Vec<ManagedBlockSpan> {
    let mut spans = Vec::new();
    let mut line_index = 0usize;

    while line_index < lines.len() {
        let Some(block) = parse_begin_marker(lines[line_index].trim()) else {
            line_index += 1;
            continue;
        };

        let mut end_index = line_index + 1;
        while end_index < lines.len() && lines[end_index].trim() != END_MARKER {
            end_index += 1;
        }

        if end_index >= lines.len() {
            break;
        }

        spans.push(ManagedBlockSpan {
            block,
            start_line: line_index,
            end_line_exclusive: end_index + 1,
        });
        line_index = end_index + 1;
    }

    spans
}

fn parse_begin_marker(line: &str) -> Option<ManagedAgentFileBlock> {
    if let Some(caps) = begin_v2_re().captures(line) {
        return Some(ManagedAgentFileBlock {
            source_id: caps.name("source")?.as_str().to_string(),
            template_id: caps.name("template")?.as_str().to_string(),
            slot: parse_slot_id(caps.name("slot")?.as_str()),
            content_hash: caps.name("hash")?.as_str().to_string(),
        });
    }

    let caps = begin_v1_re().captures(line)?;
    let asset = caps.name("asset")?.as_str();
    Some(ManagedAgentFileBlock {
        source_id: caps.name("source")?.as_str().to_string(),
        template_id: asset.to_string(),
        slot: legacy_asset_slot(asset),
        content_hash: caps.name("hash")?.as_str().to_string(),
    })
}

fn begin_v2_re() -> &'static Regex {
    BEGIN_V2_RE.get_or_init(|| {
        Regex::new(
            r#"^<!-- praxis:begin slot=(?P<slot>[^ ]+) source=(?P<source>[^ ]+) template=(?P<template>[^ ]+) hash=(?P<hash>[^ ]+) -->$"#,
        )
        .expect("regex should compile")
    })
}

fn begin_v1_re() -> &'static Regex {
    BEGIN_V1_RE.get_or_init(|| {
        Regex::new(
            r#"^<!-- praxis:begin source=(?P<source>[^ ]+) asset=(?P<asset>[^ ]+) hash=(?P<hash>[^ ]+) -->$"#,
        )
        .expect("regex should compile")
    })
}

fn legacy_asset_slot(asset: &str) -> AgentFileSlot {
    match asset {
        "codex" | "codex-agents" => AgentFileSlot::CodexProjectRoot,
        "codex-override" => AgentFileSlot::CodexProjectOverride,
        "claude-root" | "claude" => AgentFileSlot::ClaudeProjectRoot,
        "claude-dot" => AgentFileSlot::ClaudeProjectDot,
        _ => AgentFileSlot::CodexProjectRoot,
    }
}

fn read_one_slot(paths: &WorkspacePaths, slot: AgentFileSlot) -> Result<AgentFileState> {
    let path = agent_file_slot_path(paths, &slot);
    let effective_content =
        read_optional_text(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let exists = path.exists();
    let user_content = strip_managed_blocks(&effective_content);
    let managed_blocks = parse_managed_blocks(&effective_content);

    Ok(AgentFileState {
        slot,
        target_path: path.to_string_lossy().to_string(),
        exists,
        user_content,
        managed_blocks,
        effective_content,
    })
}

pub(crate) fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Scope, TargetProfile, WorkspaceManifest, WorkspaceSettings};
    use crate::workspace::WorkspacePaths;
    use std::time::Duration;
    use tempfile::tempdir;

    fn test_paths(root: &std::path::Path) -> WorkspacePaths {
        let state_dir = root.join(".praxis");
        WorkspacePaths {
            scope: Scope::Repo,
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
            codex_user_agents_path: root.join("user-codex").join("AGENTS.md"),
            codex_user_override_path: root.join("user-codex").join("AGENTS.override.md"),
            codex_project_agents_path: root.join("AGENTS.md"),
            codex_project_override_path: root.join("AGENTS.override.md"),
            codex_agent_alias_path: root.join("AGENT.md"),
            claude_user_root_path: root.join("user-claude").join("CLAUDE.md"),
            claude_project_root_path: root.join("CLAUDE.md"),
            claude_project_dot_path: root.join(".claude").join("CLAUDE.md"),
            gemini_user_root_path: root.join("user-gemini").join("GEMINI.md"),
            gemini_project_root_path: root.join("GEMINI.md"),
        }
    }

    #[test]
    fn apply_agent_files_writes_claude_project_root() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let paths = test_paths(root);
        let manifest = WorkspaceManifest {
            version: 1,
            settings: WorkspaceSettings {
                target_profile: TargetProfile::default(),
                write_codex_agent_alias: false,
            },
            installs: vec![],
        };

        apply_agent_files(
            &paths,
            &manifest,
            &[DesiredAgentFileBlock {
                source_id: "local:/tmp/source".to_string(),
                source_hash: "abc".to_string(),
                resolved_reference: None,
                template_id: "claude-project-root".to_string(),
                slot: AgentFileSlot::ClaudeProjectRoot,
                priority: 100,
                content_hash: "hash".to_string(),
                content: "root guide".to_string(),
                target_path: root.join("CLAUDE.md"),
            }],
        )
        .expect("apply agent files");

        let written = std::fs::read_to_string(root.join("CLAUDE.md")).expect("claude file");
        assert!(written.contains("slot=claude-project-root"));
        assert!(written.contains("root guide"));
    }

    #[test]
    fn parse_managed_blocks_v1_backward_compat() {
        let content = "<!-- praxis:begin source=local:/tmp/s asset=codex-agents hash=abc123 -->\nsome content\n<!-- praxis:end -->";
        let blocks = parse_managed_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].slot, AgentFileSlot::CodexProjectRoot);
        assert_eq!(blocks[0].template_id, "codex-agents");
    }

    #[test]
    fn parse_managed_blocks_v2_format() {
        let content = "<!-- praxis:begin slot=gemini-project-root source=github:foo/bar template=gemini-root hash=xyz -->\ncontent\n<!-- praxis:end -->";
        let blocks = parse_managed_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].slot, AgentFileSlot::GeminiProjectRoot);
        assert_eq!(blocks[0].template_id, "gemini-root");
    }

    #[test]
    fn parse_managed_blocks_ignores_marker_like_content_inside_block_body() {
        let content = "<!-- praxis:begin slot=codex-project-root source=local:/tmp/s template=codex-root hash=abc -->\n```html\n<meta content=\"-->\">\n```\n<!-- praxis:end -->";
        let blocks = parse_managed_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].slot, AgentFileSlot::CodexProjectRoot);
    }

    #[test]
    fn apply_agent_files_orders_multiple_blocks_for_same_slot() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let paths = test_paths(root);
        let manifest = WorkspaceManifest {
            version: 1,
            settings: WorkspaceSettings {
                target_profile: TargetProfile::default(),
                write_codex_agent_alias: false,
            },
            installs: vec![],
        };

        apply_agent_files(
            &paths,
            &manifest,
            &[
                DesiredAgentFileBlock {
                    source_id: "source-b".to_string(),
                    source_hash: "hash-b".to_string(),
                    resolved_reference: None,
                    template_id: "template-b".to_string(),
                    slot: AgentFileSlot::CodexProjectRoot,
                    priority: 20,
                    content_hash: "content-b".to_string(),
                    content: "second".to_string(),
                    target_path: root.join("AGENTS.md"),
                },
                DesiredAgentFileBlock {
                    source_id: "source-a".to_string(),
                    source_hash: "hash-a".to_string(),
                    resolved_reference: None,
                    template_id: "template-a".to_string(),
                    slot: AgentFileSlot::CodexProjectRoot,
                    priority: 10,
                    content_hash: "content-a".to_string(),
                    content: "first".to_string(),
                    target_path: root.join("AGENTS.md"),
                },
            ],
        )
        .expect("apply agent files");

        let written = std::fs::read_to_string(root.join("AGENTS.md")).expect("agents file");
        assert!(written.find("template-a").unwrap() < written.find("template-b").unwrap());
    }

    #[test]
    fn apply_agent_files_skips_alias_rewrite_when_content_is_unchanged() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let paths = test_paths(root);
        let manifest = WorkspaceManifest {
            version: 1,
            settings: WorkspaceSettings {
                target_profile: TargetProfile::default(),
                write_codex_agent_alias: true,
            },
            installs: vec![],
        };
        let desired = [DesiredAgentFileBlock {
            source_id: "source-a".to_string(),
            source_hash: "hash-a".to_string(),
            resolved_reference: None,
            template_id: "template-a".to_string(),
            slot: AgentFileSlot::CodexProjectRoot,
            priority: 10,
            content_hash: "content-a".to_string(),
            content: "first".to_string(),
            target_path: root.join("AGENTS.md"),
        }];

        apply_agent_files(&paths, &manifest, &desired).expect("first apply");
        let first_modified = std::fs::metadata(root.join("AGENT.md"))
            .expect("alias metadata")
            .modified()
            .expect("alias mtime");

        std::thread::sleep(Duration::from_millis(20));
        apply_agent_files(&paths, &manifest, &desired).expect("second apply");
        let second_modified = std::fs::metadata(root.join("AGENT.md"))
            .expect("alias metadata")
            .modified()
            .expect("alias mtime");

        assert_eq!(first_modified, second_modified);
    }
}
