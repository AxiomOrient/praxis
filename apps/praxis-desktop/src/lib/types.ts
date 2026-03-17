export type Agent = "codex" | "claude" | "gemini";
export type Scope = "repo" | "user";
export type TargetProfile =
  | "codex-open-standard"
  | "claude-native"
  | "gemini-native"
  | "codex-gemini-shared-open-standard"
  | "multi-runtime-default";
export type AgentFileSlot =
  | "codex-user-root"
  | "codex-user-override"
  | "codex-project-root"
  | "codex-project-override"
  | "claude-user-root"
  | "claude-project-root"
  | "claude-project-dot"
  | "gemini-user-root"
  | "gemini-project-root";
export type AgentFileTemplateOrigin = "declared" | "discovered" | "recipe" | "draft";
export type SourceRef =
  | {
      kind: "github";
      owner: string;
      repo: string;
      reference?: string | null;
      subdir?: string | null;
    }
  | {
      kind: "local";
      path: string;
    };

export interface DeckInfo {
  id: string;
  name: string;
  description: string;
  skills: string[];
  synthesized: boolean;
}

export interface SkillInfo {
  name: string;
  description: string;
  relative_path: string;
  root_component: string;
  display_name?: string | null;
  category?: string | null;
  tags: string[];
}

export interface AgentFileTemplate {
  id: string;
  title: string;
  description: string;
  relative_path: string;
  slots: AgentFileSlot[];
  priority: number;
  origin: AgentFileTemplateOrigin;
}

export interface RecipeBundle {
  id: string;
  relative_path: string;
  target_name: string;
  agents: Agent[];
  description: string;
}

export interface RecipeHint {
  key: string;
  label: string;
  description: string;
  bundles: RecipeBundle[];
  notes: string[];
  recommended_agent_file_templates: string[];
}

export interface SourceCatalog {
  source_id: string;
  label: string;
  source: SourceRef;
  checkout_root: string;
  resolved_reference?: string | null;
  source_hash: string;
  decks: DeckInfo[];
  skills: SkillInfo[];
  agent_file_templates: AgentFileTemplate[];
  recipe?: RecipeHint | null;
  warnings: string[];
  notes: string[];
}

export interface AppliedSkill {
  name: string;
  agent: Agent;
  source_relative_path: string;
  target_path: string;
  content_hash: string;
}

export interface AppliedBundle {
  id: string;
  agent: Agent;
  source_relative_path: string;
  target_path: string;
  content_hash: string;
}

export interface AppliedAgentFileAction {
  template_id: string;
  slot: AgentFileSlot;
  target_path: string;
  content_hash: string;
}

export interface AppliedInstall {
  source_id: string;
  source_hash: string;
  resolved_reference?: string | null;
  skills: AppliedSkill[];
  bundles: AppliedBundle[];
  agent_file_actions: AppliedAgentFileAction[];
}

export interface SourceInstall {
  id: string;
  source: SourceRef;
  targets: Agent[];
  selection: {
    all: boolean;
    decks: string[];
    skills: string[];
    exclude_skills: string[];
    agent_file_templates: string[];
  };
}

export interface WorkspaceSettings {
  target_profile: TargetProfile;
  write_codex_agent_alias: boolean;
}

export interface WorkspaceSnapshot {
  manifest: {
    version: number;
    settings: WorkspaceSettings;
    installs: SourceInstall[];
  };
  lock: {
    version: number;
    generated_at: string;
    installs: AppliedInstall[];
  };
  targets: {
    codex_skills: string;
    claude_skills: string;
    gemini_skills: string;
    codex_agents: string;
    codex_override: string;
    codex_agent_alias: string;
    claude_root: string;
    claude_dot: string;
    gemini_project_root: string;
  };
  library: {
    db_path: string;
    artifact_root: string;
    stats: {
      sources: number;
      snapshots: number;
      artifacts: number;
    };
  };
  evaluation: {
    suites: BenchmarkSuiteSummary[];
    recent_runs: BenchmarkRunSummary[];
  };
  create: CreateSnapshot;
  warnings: string[];
}

export interface BenchmarkSuiteSummary {
  id: string;
  title: string;
  description: string;
  case_count: number;
  suite_kind: string;
}

export interface BenchmarkRunSummary {
  id: string;
  suite_id: string;
  candidate_source_id: string;
  baseline_source_id?: string | null;
  mode: string;
  status: string;
  recommendation: string;
  score: number;
  summary: string;
  reviewer_note?: string | null;
  review_decision?: string | null;
  created_at: string;
  finished_at: string;
}

export interface DraftSummary {
  id: string;
  name: string;
  artifact_kind: string;
  version_id: string;
  preset: string;
  created_at: string;
  updated_at: string;
}

export interface PreviewTreeEntry {
  path: string;
  entry_kind: string;
}

export interface DraftPreview {
  draft: DraftSummary;
  files: PreviewTreeEntry[];
  documents: DraftDocument[];
  promotion_target: string;
}

export interface CreateSnapshot {
  drafts: DraftSummary[];
}

export interface DraftDocument {
  path: string;
  content: string;
  editable: boolean;
}

export interface ManagedAgentFileBlock {
  source_id: string;
  template_id: string;
  slot: AgentFileSlot;
  content_hash: string;
}

export interface AgentFileState {
  slot: AgentFileSlot;
  target_path: string;
  exists: boolean;
  user_content: string;
  managed_blocks: ManagedAgentFileBlock[];
  effective_content: string;
}

export interface AgentFileSnapshot {
  slots: AgentFileState[];
}

export interface DoctorCheck {
  level: string;
  code: string;
  message: string;
}

export interface DoctorReport {
  ok: boolean;
  checks: DoctorCheck[];
}

export interface PlannedSkill {
  name: string;
  display_name?: string | null;
  category?: string | null;
  agent: Agent;
  source_relative_path: string;
  target_path: string;
}

export interface PlannedBundle {
  id: string;
  description: string;
  agent: Agent;
  source_relative_path: string;
  target_path: string;
}

export interface PlannedAgentFileAction {
  template_id: string;
  title: string;
  slot: AgentFileSlot;
  source_relative_path: string;
  target_path: string;
}

export interface PlanSummary {
  total_skills: number;
  total_agent_file_actions: number;
  total_bundles: number;
  codex_skills: number;
  claude_skills: number;
  gemini_skills: number;
  codex_bundles: number;
  claude_bundles: number;
}

export interface InstallPlan {
  source_id: string;
  label: string;
  resolved_reference?: string | null;
  source_hash: string;
  targets: Agent[];
  selection: {
    all: boolean;
    decks: string[];
    skills: string[];
    exclude_skills: string[];
    agent_file_templates: string[];
  };
  target_paths: WorkspaceSnapshot["targets"];
  skills: PlannedSkill[];
  bundles: PlannedBundle[];
  agent_file_actions: PlannedAgentFileAction[];
  warnings: string[];
  notes: string[];
  conflicts: string[];
  summary: PlanSummary;
}

export interface InstallPayload {
  scope: Scope;
  root?: string | null;
  source: string;
  all: boolean;
  decks: string[];
  skills: string[];
  exclude_skills: string[];
  agent_file_templates: string[];
  targets: Agent[];
}

export interface RemovePayload {
  scope: Scope;
  root?: string | null;
  source: string;
  decks: string[];
  skills: string[];
  agent_file_templates: string[];
  remove_all: boolean;
}

export interface AgentFileWritePayload {
  scope: Scope;
  root?: string | null;
  slot: AgentFileSlot;
  content: string;
}

export interface BenchmarkRunPayload {
  scope: Scope;
  root?: string | null;
  suite_id: string;
  source: string;
  mode?: string | null;
}

export interface HumanReviewPayload {
  scope: Scope;
  root?: string | null;
  run_id: string;
  decision: string;
  note: string;
}

export interface CreateDraftPayload {
  scope: Scope;
  root?: string | null;
  name: string;
  description: string;
  preset: string;
}

export interface DraftPreviewPayload {
  scope: Scope;
  root?: string | null;
  draft_id: string;
}

export interface PromoteDraftPayload {
  scope: Scope;
  root?: string | null;
  draft_id: string;
  destination_root?: string | null;
}

export interface ForkDraftPayload {
  scope: Scope;
  root?: string | null;
  source: string;
  skill_name: string;
  draft_name?: string | null;
  description?: string | null;
}

export interface DraftUpdatePayload {
  scope: Scope;
  root?: string | null;
  draft_id: string;
  relative_path: string;
  content: string;
}
