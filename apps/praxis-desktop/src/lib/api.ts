import { invoke } from "@tauri-apps/api/core";
import type {
  AgentFileSnapshot,
  AgentFileWritePayload,
  BenchmarkRunPayload,
  BenchmarkRunSummary,
  CreateDraftPayload,
  DraftUpdatePayload,
  DoctorReport,
  DraftPreview,
  DraftPreviewPayload,
  ForkDraftPayload,
  HumanReviewPayload,
  InstallPayload,
  InstallPlan,
  PromoteDraftPayload,
  RemovePayload,
  Scope,
  SourceCatalog,
  WorkspaceSnapshot,
} from "./types";

export async function workspace(scope: Scope, root?: string | null): Promise<WorkspaceSnapshot> {
  return invoke("workspace", { scope, root: root ?? null });
}

export async function inspect(scope: Scope, source: string, root?: string | null): Promise<SourceCatalog> {
  return invoke("inspect", { scope, root: root ?? null, source });
}

export async function plan(payload: InstallPayload): Promise<InstallPlan> {
  return invoke("plan", { payload });
}

export async function install(payload: InstallPayload): Promise<WorkspaceSnapshot> {
  return invoke("install", { payload });
}

export async function remove(payload: RemovePayload): Promise<WorkspaceSnapshot> {
  return invoke("remove_install", { payload });
}

export async function sync(scope: Scope, root?: string | null): Promise<WorkspaceSnapshot> {
  return invoke("sync", { scope, root: root ?? null });
}

export async function update(scope: Scope, root?: string | null): Promise<WorkspaceSnapshot> {
  return invoke("update", { scope, root: root ?? null });
}

export async function doctor(scope: Scope, root?: string | null): Promise<DoctorReport> {
  return invoke("doctor", { scope, root: root ?? null });
}

export async function guidance(scope: Scope, root?: string | null): Promise<AgentFileSnapshot> {
  return invoke("guidance", { scope, root: root ?? null });
}

export async function guidanceWrite(payload: AgentFileWritePayload): Promise<AgentFileSnapshot> {
  return invoke("guidance_write", { payload });
}

export async function benchmarkRun(payload: BenchmarkRunPayload): Promise<BenchmarkRunSummary> {
  return invoke("benchmark_run", { payload });
}

export async function submitHumanReview(payload: HumanReviewPayload): Promise<BenchmarkRunSummary> {
  return invoke("submit_human_review", { payload });
}

export async function createSkillDraft(payload: CreateDraftPayload): Promise<DraftPreview> {
  return invoke("create_skill_draft", { payload });
}

export async function previewCreateDraft(payload: DraftPreviewPayload): Promise<DraftPreview> {
  return invoke("preview_create_draft", { payload });
}

export async function promoteCreateDraft(payload: PromoteDraftPayload): Promise<DraftPreview> {
  return invoke("promote_create_draft", { payload });
}

export async function forkCreateDraft(payload: ForkDraftPayload): Promise<DraftPreview> {
  return invoke("fork_create_draft", { payload });
}

export async function updateCreateDraft(payload: DraftUpdatePayload): Promise<DraftPreview> {
  return invoke("update_create_draft", { payload });
}
