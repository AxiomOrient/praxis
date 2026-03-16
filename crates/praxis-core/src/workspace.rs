use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{
    AgentFileSlot, Scope, TargetPaths, WorkspaceLock, WorkspaceManifest, WorkspaceSettings,
};

#[derive(Debug, Clone)]
pub struct WorkspacePaths {
    pub scope: Scope,
    pub repo_root: Option<PathBuf>,
    pub state_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub lock_path: PathBuf,
    // Skill roots
    pub codex_skills_dir: PathBuf,
    pub claude_skills_dir: PathBuf,
    pub gemini_skills_dir: PathBuf,
    // Codex agent file paths
    pub codex_user_agents_path: PathBuf,
    pub codex_user_override_path: PathBuf,
    pub codex_project_agents_path: PathBuf,
    pub codex_project_override_path: PathBuf,
    pub codex_agent_alias_path: PathBuf,
    // Claude agent file paths
    pub claude_user_root_path: PathBuf,
    pub claude_project_root_path: PathBuf,
    pub claude_project_dot_path: PathBuf,
    // Gemini agent file paths
    pub gemini_user_root_path: PathBuf,
    pub gemini_project_root_path: PathBuf,
}

pub fn resolve_workspace_paths(scope: Scope, root: Option<String>) -> Result<WorkspacePaths> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("home directory not found"))?;
    let codex_home = std::env::var("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".codex"));

    match scope {
        Scope::Repo => {
            let repo_root = match root {
                Some(value) => PathBuf::from(value),
                None => std::env::current_dir().context("failed to get current directory")?,
            };
            let state_dir = repo_root.join(".praxis");

            Ok(WorkspacePaths {
                scope: Scope::Repo,
                repo_root: Some(repo_root.clone()),
                manifest_path: state_dir.join("manifest.toml"),
                lock_path: state_dir.join("lock.json"),
                cache_dir: state_dir.join("cache"),
                state_dir,
                codex_skills_dir: repo_root.join(".agents").join("skills"),
                claude_skills_dir: repo_root.join(".claude").join("skills"),
                gemini_skills_dir: repo_root.join(".gemini").join("skills"),
                codex_user_agents_path: codex_home.join("AGENTS.md"),
                codex_user_override_path: codex_home.join("AGENTS.override.md"),
                codex_project_agents_path: repo_root.join("AGENTS.md"),
                codex_project_override_path: repo_root.join("AGENTS.override.md"),
                codex_agent_alias_path: repo_root.join("AGENT.md"),
                claude_user_root_path: home.join(".claude").join("CLAUDE.md"),
                claude_project_root_path: repo_root.join("CLAUDE.md"),
                claude_project_dot_path: repo_root.join(".claude").join("CLAUDE.md"),
                gemini_user_root_path: home.join(".gemini").join("GEMINI.md"),
                gemini_project_root_path: repo_root.join("GEMINI.md"),
            })
        }
        Scope::User => {
            let state_dir = home
                .join("Library")
                .join("Application Support")
                .join("Praxis");

            Ok(WorkspacePaths {
                scope: Scope::User,
                repo_root: None,
                manifest_path: state_dir.join("manifest.toml"),
                lock_path: state_dir.join("lock.json"),
                cache_dir: state_dir.join("cache"),
                state_dir,
                codex_skills_dir: home.join(".agents").join("skills"),
                claude_skills_dir: home.join(".claude").join("skills"),
                gemini_skills_dir: home.join(".gemini").join("skills"),
                codex_user_agents_path: codex_home.join("AGENTS.md"),
                codex_user_override_path: codex_home.join("AGENTS.override.md"),
                // User scope: project slots fall back to user slot paths
                codex_project_agents_path: codex_home.join("AGENTS.md"),
                codex_project_override_path: codex_home.join("AGENTS.override.md"),
                codex_agent_alias_path: codex_home.join("AGENT.md"),
                claude_user_root_path: home.join(".claude").join("CLAUDE.md"),
                claude_project_root_path: home.join(".claude").join("CLAUDE.md"),
                claude_project_dot_path: home.join(".claude").join("CLAUDE.md"),
                gemini_user_root_path: home.join(".gemini").join("GEMINI.md"),
                gemini_project_root_path: home.join(".gemini").join("GEMINI.md"),
            })
        }
    }
}

pub fn ensure_workspace(paths: &WorkspacePaths) -> Result<()> {
    fs::create_dir_all(&paths.state_dir)?;
    fs::create_dir_all(&paths.cache_dir)?;
    fs::create_dir_all(&paths.codex_skills_dir)?;
    fs::create_dir_all(&paths.claude_skills_dir)?;

    for path in [
        &paths.codex_project_agents_path,
        &paths.codex_project_override_path,
        &paths.codex_agent_alias_path,
        &paths.claude_project_root_path,
        &paths.claude_project_dot_path,
        &paths.gemini_project_root_path,
    ] {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
    }

    if !paths.manifest_path.exists() {
        save_manifest(&paths.manifest_path, &WorkspaceManifest::default())?;
    }
    if !paths.lock_path.exists() {
        let mut lock = WorkspaceLock::default();
        lock.generated_at = Utc::now().to_rfc3339();
        save_lock(&paths.lock_path, &lock)?;
    }

    Ok(())
}

pub fn load_manifest(path: &Path) -> Result<WorkspaceManifest> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok(toml::from_str(&raw)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(WorkspaceManifest::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn save_manifest(path: &Path, manifest: &WorkspaceManifest) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = toml::to_string_pretty(manifest)?;
    fs::write(path, raw)?;
    Ok(())
}

pub fn load_lock(path: &Path) -> Result<WorkspaceLock> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok(serde_json::from_str(&raw)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(WorkspaceLock::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn save_lock(path: &Path, lock: &WorkspaceLock) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(lock)?;
    fs::write(path, raw)?;
    Ok(())
}

/// Resolve the concrete filesystem path for an agent file slot.
pub fn agent_file_slot_path(paths: &WorkspacePaths, slot: &AgentFileSlot) -> PathBuf {
    match slot {
        AgentFileSlot::CodexUserRoot => paths.codex_user_agents_path.clone(),
        AgentFileSlot::CodexUserOverride => paths.codex_user_override_path.clone(),
        AgentFileSlot::CodexProjectRoot => paths.codex_project_agents_path.clone(),
        AgentFileSlot::CodexProjectOverride => paths.codex_project_override_path.clone(),
        AgentFileSlot::ClaudeUserRoot => paths.claude_user_root_path.clone(),
        AgentFileSlot::ClaudeProjectRoot => paths.claude_project_root_path.clone(),
        AgentFileSlot::ClaudeProjectDot => paths.claude_project_dot_path.clone(),
        AgentFileSlot::GeminiUserRoot => paths.gemini_user_root_path.clone(),
        AgentFileSlot::GeminiProjectRoot => paths.gemini_project_root_path.clone(),
    }
}

pub fn target_paths(paths: &WorkspacePaths, _settings: &WorkspaceSettings) -> TargetPaths {
    TargetPaths {
        codex_skills: paths.codex_skills_dir.to_string_lossy().to_string(),
        claude_skills: paths.claude_skills_dir.to_string_lossy().to_string(),
        gemini_skills: paths.gemini_skills_dir.to_string_lossy().to_string(),
        codex_agents: paths
            .codex_project_agents_path
            .to_string_lossy()
            .to_string(),
        codex_override: paths
            .codex_project_override_path
            .to_string_lossy()
            .to_string(),
        codex_agent_alias: paths.codex_agent_alias_path.to_string_lossy().to_string(),
        claude_root: paths.claude_project_root_path.to_string_lossy().to_string(),
        claude_dot: paths.claude_project_dot_path.to_string_lossy().to_string(),
        gemini_project_root: paths.gemini_project_root_path.to_string_lossy().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_paths() -> WorkspacePaths {
        let repo_root = PathBuf::from("/tmp/praxis");
        let home = PathBuf::from("/tmp/home");
        WorkspacePaths {
            scope: Scope::Repo,
            repo_root: Some(repo_root.clone()),
            state_dir: repo_root.join(".praxis"),
            cache_dir: repo_root.join(".praxis").join("cache"),
            manifest_path: repo_root.join(".praxis").join("manifest.toml"),
            lock_path: repo_root.join(".praxis").join("lock.json"),
            codex_skills_dir: repo_root.join(".agents").join("skills"),
            claude_skills_dir: repo_root.join(".claude").join("skills"),
            gemini_skills_dir: repo_root.join(".gemini").join("skills"),
            codex_user_agents_path: home.join(".codex").join("AGENTS.md"),
            codex_user_override_path: home.join(".codex").join("AGENTS.override.md"),
            codex_project_agents_path: repo_root.join("AGENTS.md"),
            codex_project_override_path: repo_root.join("AGENTS.override.md"),
            codex_agent_alias_path: repo_root.join("AGENT.md"),
            claude_user_root_path: home.join(".claude").join("CLAUDE.md"),
            claude_project_root_path: repo_root.join("CLAUDE.md"),
            claude_project_dot_path: repo_root.join(".claude").join("CLAUDE.md"),
            gemini_user_root_path: home.join(".gemini").join("GEMINI.md"),
            gemini_project_root_path: repo_root.join("GEMINI.md"),
        }
    }

    #[test]
    fn agent_file_slot_path_resolves_gemini_slots() {
        let paths = sample_paths();

        assert_eq!(
            agent_file_slot_path(&paths, &AgentFileSlot::GeminiUserRoot),
            PathBuf::from("/tmp/home/.gemini/GEMINI.md")
        );
        assert_eq!(
            agent_file_slot_path(&paths, &AgentFileSlot::GeminiProjectRoot),
            PathBuf::from("/tmp/praxis/GEMINI.md")
        );
    }
}
