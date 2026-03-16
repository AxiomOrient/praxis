import { invoke } from "@tauri-apps/api/core";
import type {
  DoctorReport,
  GuidanceSnapshot,
  GuidanceWritePayload,
  InstallPayload,
  InstallPlan,
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

export async function guidance(scope: Scope, root?: string | null): Promise<GuidanceSnapshot> {
  return invoke("guidance", { scope, root: root ?? null });
}

export async function guidanceWrite(payload: GuidanceWritePayload): Promise<GuidanceSnapshot> {
  return invoke("guidance_write", { payload });
}
