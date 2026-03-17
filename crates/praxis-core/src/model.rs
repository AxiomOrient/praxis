use serde::{Deserialize, Serialize};

// ─── Runtime agents ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Agent {
    Codex,
    Claude,
}

impl Agent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }
}

// ─── Scope ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Scope {
    Repo,
    User,
}

// ─── Target profiles ──────────────────────────────────────────────────────────

/// Explicit runtime mapping policy (specs/04-RUNTIME-TARGET-PROFILES.md).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TargetProfile {
    CodexOpenStandard,
    ClaudeNative,
    #[default]
    MultiRuntimeDefault,
}

impl TargetProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CodexOpenStandard => "codex-open-standard",
            Self::ClaudeNative => "claude-native",
            Self::MultiRuntimeDefault => "multi-runtime-default",
        }
    }

    pub fn default_targets(&self) -> Vec<Agent> {
        match self {
            Self::CodexOpenStandard => vec![Agent::Codex],
            Self::ClaudeNative => vec![Agent::Claude],
            Self::MultiRuntimeDefault => vec![Agent::Codex, Agent::Claude],
        }
    }
}

// ─── Agent file slots ─────────────────────────────────────────────────────────

/// Canonical agent file slot identifiers (specs/03-SPEC.md §11.2).
///
/// Each slot maps to a concrete filesystem path determined by workspace scope.
/// Slot IDs are stable and used as managed-block identifiers in rendered files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum AgentFileSlot {
    // Codex
    CodexUserRoot,        // ~/.codex/AGENTS.md
    CodexUserOverride,    // ~/.codex/AGENTS.override.md
    CodexProjectRoot,     // $REPO/AGENTS.md
    CodexProjectOverride, // $REPO/AGENTS.override.md
    // Claude Code
    ClaudeUserRoot,    // ~/.claude/CLAUDE.md
    ClaudeProjectRoot, // $REPO/CLAUDE.md
    ClaudeProjectDot,  // $REPO/.claude/CLAUDE.md
}

impl AgentFileSlot {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CodexUserRoot => "codex-user-root",
            Self::CodexUserOverride => "codex-user-override",
            Self::CodexProjectRoot => "codex-project-root",
            Self::CodexProjectOverride => "codex-project-override",
            Self::ClaudeUserRoot => "claude-user-root",
            Self::ClaudeProjectRoot => "claude-project-root",
            Self::ClaudeProjectDot => "claude-project-dot",
        }
    }

    pub fn is_project_scoped(&self) -> bool {
        matches!(
            self,
            Self::CodexProjectRoot
                | Self::CodexProjectOverride
                | Self::ClaudeProjectRoot
                | Self::ClaudeProjectDot
        )
    }

    pub fn is_user_scoped(&self) -> bool {
        !self.is_project_scoped()
    }
}

// ─── Agent file template ──────────────────────────────────────────────────────

/// Origin of an agent file template (spec §4.1.6).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AgentFileTemplateOrigin {
    Declared,
    Discovered,
    Recipe,
    Draft,
}

/// Reusable agent instruction template discovered in a source (spec §4.1.6).
/// Replaces the old `GuideAsset` type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFileTemplate {
    pub id: String,
    pub title: String,
    pub description: String,
    pub relative_path: String,
    /// Target slots this template contributes content to.
    pub slots: Vec<AgentFileSlot>,
    /// Composition priority; lower values render first. Default 100.
    /// Reserve lower bands for higher-precedence/system-owned templates.
    pub priority: u32,
    pub origin: AgentFileTemplateOrigin,
}

// ─── Source ref ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SourceRef {
    Github {
        owner: String,
        repo: String,
        reference: Option<String>,
        subdir: Option<String>,
    },
    Local {
        path: String,
    },
}

// ─── Workspace settings ───────────────────────────────────────────────────────

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    #[serde(default)]
    pub target_profile: TargetProfile,
    /// Write AGENT.md as a Codex alias. Default true.
    #[serde(default = "default_true", alias = "write_agent_md_alias")]
    pub write_codex_agent_alias: bool,
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            target_profile: TargetProfile::default(),
            write_codex_agent_alias: true,
        }
    }
}

// ─── Install selection ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstallSelection {
    pub all: bool,
    pub decks: Vec<String>,
    pub skills: Vec<String>,
    pub exclude_skills: Vec<String>,
    /// Agent file template IDs from the source catalog.
    #[serde(default, alias = "guides")]
    pub agent_file_templates: Vec<String>,
}

// ─── Source install ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInstall {
    pub id: String,
    pub source: SourceRef,
    pub targets: Vec<Agent>,
    pub selection: InstallSelection,
}

// ─── Workspace manifest ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    pub version: u32,
    pub settings: WorkspaceSettings,
    pub installs: Vec<SourceInstall>,
}

impl Default for WorkspaceManifest {
    fn default() -> Self {
        Self {
            version: 1,
            settings: WorkspaceSettings::default(),
            installs: Vec::new(),
        }
    }
}

// ─── Catalog types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub skills: Vec<String>,
    pub synthesized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub relative_path: String,
    pub root_component: String,
    pub display_name: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeBundle {
    pub id: String,
    pub relative_path: String,
    pub target_name: String,
    pub agents: Vec<Agent>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeHint {
    pub key: String,
    pub label: String,
    pub description: String,
    pub bundles: Vec<RecipeBundle>,
    pub notes: Vec<String>,
    /// Template IDs recommended by this recipe.
    pub recommended_agent_file_templates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCatalog {
    pub source_id: String,
    pub label: String,
    pub source: SourceRef,
    pub checkout_root: String,
    pub resolved_reference: Option<String>,
    pub source_hash: String,
    pub decks: Vec<DeckInfo>,
    pub skills: Vec<SkillInfo>,
    /// Agent file templates discovered in this source.
    pub agent_file_templates: Vec<AgentFileTemplate>,
    pub recipe: Option<RecipeHint>,
    pub warnings: Vec<String>,
    pub notes: Vec<String>,
}

// ─── Applied (lock) types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedSkill {
    pub name: String,
    pub agent: Agent,
    pub source_relative_path: String,
    pub target_path: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedBundle {
    pub id: String,
    pub agent: Agent,
    pub source_relative_path: String,
    pub target_path: String,
    pub content_hash: String,
}

/// A managed agent file block written to a slot during apply.
/// Replaces the old `AppliedGuide` type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedAgentFileAction {
    pub template_id: String,
    pub slot: AgentFileSlot,
    pub target_path: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedInstall {
    pub source_id: String,
    pub source_hash: String,
    pub resolved_reference: Option<String>,
    pub skills: Vec<AppliedSkill>,
    pub bundles: Vec<AppliedBundle>,
    pub agent_file_actions: Vec<AppliedAgentFileAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLock {
    pub version: u32,
    pub generated_at: String,
    pub installs: Vec<AppliedInstall>,
}

impl Default for WorkspaceLock {
    fn default() -> Self {
        Self {
            version: 1,
            generated_at: String::new(),
            installs: Vec::new(),
        }
    }
}

// ─── Target paths ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetPaths {
    pub codex_skills: String,
    pub claude_skills: String,
    pub codex_agents: String,
    pub codex_override: String,
    pub codex_agent_alias: String,
    pub claude_root: String,
    pub claude_dot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryStats {
    pub sources: usize,
    pub snapshots: usize,
    pub artifacts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryStoreSnapshot {
    pub db_path: String,
    pub artifact_root: String,
    pub stats: LibraryStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalExecutorKind {
    #[default]
    Disabled,
    CodexRuntime,
}

impl ExternalExecutorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::CodexRuntime => "codex-runtime",
        }
    }

    pub fn is_enabled(&self) -> bool {
        !matches!(self, Self::Disabled)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExternalExecutorConfig {
    #[serde(default)]
    pub provider: ExternalExecutorKind,
    pub model: Option<String>,
}

impl ExternalExecutorConfig {
    pub fn disabled() -> Self {
        Self::default()
    }

    pub fn is_enabled(&self) -> bool {
        self.provider.is_enabled()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuiteSummary {
    pub id: String,
    pub title: String,
    pub description: String,
    pub case_count: usize,
    pub suite_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRunSummary {
    pub id: String,
    pub suite_id: String,
    pub candidate_source_id: String,
    pub baseline_source_id: Option<String>,
    pub job_id: Option<String>,
    pub mode: String,
    pub status: String,
    pub recommendation: String,
    pub score: f64,
    pub summary: String,
    pub reviewer_note: Option<String>,
    pub review_decision: Option<String>,
    pub evidence_path: Option<String>,
    pub created_at: String,
    pub finished_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSummary {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub subject_id: String,
    pub summary: String,
    pub leased_by_session: Option<String>,
    pub lease_expires_at: Option<String>,
    pub attempts: usize,
    pub last_error: Option<String>,
    pub log_path: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSnapshot {
    pub queued: usize,
    pub running: usize,
    pub failed: usize,
    pub recent_jobs: Vec<JobSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSnapshot {
    pub suites: Vec<BenchmarkSuiteSummary>,
    pub recent_runs: Vec<BenchmarkRunSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftLineage {
    pub origin_kind: String,
    pub source_id: Option<String>,
    pub parent_version_id: Option<String>,
    pub parent_name: Option<String>,
    pub augmentation_prompt: Option<String>,
    pub promotion_path: Option<String>,
    pub promoted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftSummary {
    pub id: String,
    pub name: String,
    pub artifact_kind: String,
    pub version_id: String,
    pub preset: String,
    pub lineage: DraftLineage,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewTreeEntry {
    pub path: String,
    pub entry_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftDocument {
    pub path: String,
    pub content: String,
    pub editable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPreview {
    pub draft: DraftSummary,
    pub files: Vec<PreviewTreeEntry>,
    pub documents: Vec<DraftDocument>,
    pub promotion_target: String,
    pub review: PromotionReviewSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSnapshot {
    pub drafts: Vec<DraftSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionReviewSummary {
    pub changed_files: usize,
    pub latest_recommendation: Option<String>,
    pub latest_run_status: Option<String>,
    pub latest_run_summary: Option<String>,
    pub pending_job_count: usize,
}

// ─── Workspace snapshot ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub manifest: WorkspaceManifest,
    pub lock: WorkspaceLock,
    pub targets: TargetPaths,
    pub library: LibraryStoreSnapshot,
    pub evaluation: EvaluationSnapshot,
    pub jobs: JobSnapshot,
    pub create: CreateSnapshot,
    pub warnings: Vec<String>,
}

// ─── Agent file state ─────────────────────────────────────────────────────────

/// One block contributed by a managed source to an agent file.
/// Replaces the old `ManagedGuideBlock` type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedAgentFileBlock {
    pub source_id: String,
    pub template_id: String,
    pub slot: AgentFileSlot,
    pub content_hash: String,
}

/// Live state of one agent file slot.
/// Replaces the old `GuideState` type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFileState {
    pub slot: AgentFileSlot,
    pub target_path: String,
    pub exists: bool,
    pub user_content: String,
    pub managed_blocks: Vec<ManagedAgentFileBlock>,
    pub effective_content: String,
}

/// Snapshot of all managed agent file slots.
/// Replaces the old `GuidanceSnapshot` type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFileSnapshot {
    pub slots: Vec<AgentFileState>,
}

// ─── Doctor ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub level: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub checks: Vec<DoctorCheck>,
}

// ─── Request types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallRequest {
    pub scope: Scope,
    pub root: Option<String>,
    pub source: String,
    pub all: bool,
    pub decks: Vec<String>,
    pub skills: Vec<String>,
    pub exclude_skills: Vec<String>,
    pub agent_file_templates: Vec<String>,
    pub targets: Vec<Agent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveRequest {
    pub scope: Scope,
    pub root: Option<String>,
    pub source: String,
    pub decks: Vec<String>,
    pub skills: Vec<String>,
    pub agent_file_templates: Vec<String>,
    pub remove_all: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRunRequest {
    pub scope: Scope,
    pub root: Option<String>,
    pub suite_id: String,
    pub source: String,
    pub mode: Option<String>,
    pub executor: Option<ExternalExecutorConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanReviewRequest {
    pub scope: Scope,
    pub root: Option<String>,
    pub run_id: String,
    pub decision: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftAugmentRequest {
    pub scope: Scope,
    pub root: Option<String>,
    pub draft_id: String,
    pub prompt: String,
    pub executor: Option<ExternalExecutorConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobWorkRequest {
    pub scope: Scope,
    pub root: Option<String>,
    pub session_id: Option<String>,
    pub max_jobs: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCancelRequest {
    pub scope: Scope,
    pub root: Option<String>,
    pub job_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRetryRequest {
    pub scope: Scope,
    pub root: Option<String>,
    pub job_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDraftRequest {
    pub scope: Scope,
    pub root: Option<String>,
    pub name: String,
    pub description: String,
    pub preset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPreviewRequest {
    pub scope: Scope,
    pub root: Option<String>,
    pub draft_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoteDraftRequest {
    pub scope: Scope,
    pub root: Option<String>,
    pub draft_id: String,
    pub destination_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkDraftRequest {
    pub scope: Scope,
    pub root: Option<String>,
    pub source: String,
    pub skill_name: String,
    pub draft_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftUpdateRequest {
    pub scope: Scope,
    pub root: Option<String>,
    pub draft_id: String,
    pub relative_path: String,
    pub content: String,
}

/// Write user-authored content to an agent file slot.
/// Replaces the old `GuidanceWriteRequest` type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFileWriteRequest {
    pub scope: Scope,
    pub root: Option<String>,
    pub slot: AgentFileSlot,
    pub content: String,
}

// ─── Plan types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedSkill {
    pub name: String,
    pub display_name: Option<String>,
    pub category: Option<String>,
    pub agent: Agent,
    pub source_relative_path: String,
    pub target_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedBundle {
    pub id: String,
    pub description: String,
    pub agent: Agent,
    pub source_relative_path: String,
    pub target_path: String,
}

/// A planned write action to an agent file slot.
/// Replaces the old `PlannedGuide` type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAgentFileAction {
    pub template_id: String,
    pub title: String,
    pub slot: AgentFileSlot,
    pub source_relative_path: String,
    pub target_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSummary {
    pub total_skills: usize,
    pub total_agent_file_actions: usize,
    pub total_bundles: usize,
    pub codex_skills: usize,
    pub claude_skills: usize,
    pub codex_bundles: usize,
    pub claude_bundles: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallPlan {
    pub source_id: String,
    pub label: String,
    pub resolved_reference: Option<String>,
    pub source_hash: String,
    pub targets: Vec<Agent>,
    pub selection: InstallSelection,
    pub target_paths: TargetPaths,
    pub skills: Vec<PlannedSkill>,
    pub bundles: Vec<PlannedBundle>,
    pub agent_file_actions: Vec<PlannedAgentFileAction>,
    pub warnings: Vec<String>,
    pub notes: Vec<String>,
    pub conflicts: Vec<String>,
    pub summary: PlanSummary,
}

// ─── Library surface types (spec §4.1.14) ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LibraryArtifactKind {
    Skill,
    Deck,
    AgentFileTemplate,
    Draft,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LibraryOrigin {
    Source,
    Imported,
    Draft,
    Recipe,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PresenceState {
    Available,
    Installed,
    Draft,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LibraryStatusFlag {
    Augmented,
    Benchmarked,
    Outdated,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub artifact_kind: LibraryArtifactKind,
    pub artifact_id: String,
    pub source_id: Option<String>,
    pub origin: LibraryOrigin,
    pub presence_state: PresenceState,
    pub status_flags: Vec<LibraryStatusFlag>,
}

#[cfg(test)]
mod tests {
    use super::InstallSelection;
    use serde_json::{json, Value};

    #[test]
    fn install_selection_deserializes_legacy_guides_alias() {
        let selection: InstallSelection = serde_json::from_value(json!({
            "all": false,
            "decks": [],
            "skills": [],
            "exclude_skills": [],
            "guides": ["codex-project-root"]
        }))
        .expect("deserialize install selection");

        assert_eq!(
            selection.agent_file_templates,
            vec!["codex-project-root".to_string()]
        );
    }

    #[test]
    fn install_selection_serializes_canonical_agent_file_templates_key() {
        let selection = InstallSelection {
            all: false,
            decks: Vec::new(),
            skills: Vec::new(),
            exclude_skills: Vec::new(),
            agent_file_templates: vec!["codex-project-root".to_string()],
        };

        let value = serde_json::to_value(selection).expect("serialize install selection");
        let object = value.as_object().expect("selection object");

        assert_eq!(
            object.get("agent_file_templates"),
            Some(&Value::Array(vec![Value::String(
                "codex-project-root".to_string()
            )]))
        );
        assert!(!object.contains_key("guides"));
    }
}
