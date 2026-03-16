export type Agent = "codex" | "claude" | "gemini";
export type Scope = "repo" | "user";
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
export type GuideKind = "codex-agents" | "codex-override" | "claude-root" | "claude-dot";
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

export interface GuideAsset {
  id: string;
  kind: GuideKind;
  title: string;
  description: string;
  relative_path: string;
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
  recommended_agent_file_templates?: string[];
  recommended_guides: string[];
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
  agent_file_templates?: AgentFileTemplate[];
  guides: GuideAsset[];
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

export interface AppliedGuide {
  asset_id: string;
  kind: GuideKind;
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
  guides: AppliedGuide[];
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
    guides: string[];
  };
}

export interface WorkspaceSettings {
  default_agents: Agent[];
  write_agent_md_alias: boolean;
  claude_project_guide_location: "root" | "dot-claude";
}

export interface WorkspaceSnapshot {
  manifest: {
    version: number;
    settings: WorkspaceSettings;
    installs: SourceInstall[];
  };
  lock: {
    version: number;
    installs: AppliedInstall[];
  };
  targets: {
    codex_skills: string;
    claude_skills: string;
    gemini_skills?: string;
    codex_agents: string;
    codex_override: string;
    codex_agent_alias: string;
    claude_root: string;
    claude_dot: string;
    gemini_project_root?: string;
  };
  warnings: string[];
}

export interface ManagedGuideBlock {
  source_id: string;
  asset_id: string;
  kind: GuideKind;
  content_hash: string;
}

export interface ManagedAgentFileBlock {
  source_id: string;
  template_id: string;
  slot: AgentFileSlot;
  content_hash: string;
}

export interface GuideState {
  kind: GuideKind;
  target_path: string;
  exists: boolean;
  user_content: string;
  managed_blocks: ManagedGuideBlock[];
  effective_content: string;
}

export interface GuidanceSnapshot {
  slots?: AgentFileState[];
  guides: GuideState[];
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

export interface PlannedGuide {
  asset_id: string;
  title: string;
  kind: GuideKind;
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
  total_guides: number;
  total_bundles: number;
  codex_skills: number;
  claude_skills: number;
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
    guides: string[];
  };
  target_paths: WorkspaceSnapshot["targets"];
  skills: PlannedSkill[];
  bundles: PlannedBundle[];
  guides: PlannedGuide[];
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
  guides: string[];
  targets: Agent[];
}

export interface RemovePayload {
  scope: Scope;
  root?: string | null;
  source: string;
  decks: string[];
  skills: string[];
  guides: string[];
  remove_all: boolean;
}

export interface GuidanceWritePayload {
  scope: Scope;
  root?: string | null;
  kind: GuideKind;
  content: string;
}
