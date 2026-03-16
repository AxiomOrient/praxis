#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use praxis_core::{
    doctor_workspace, inspect_source_input, install_source, list_workspace, plan_install,
    read_agent_file_state, remove_from_source, sync_workspace, update_workspace,
    write_agent_file_user_content, AgentFileWriteRequest, InstallRequest, RemoveRequest, Scope,
};

fn parse_scope(scope: &str) -> Result<Scope, String> {
    match scope {
        "repo" => Ok(Scope::Repo),
        "user" => Ok(Scope::User),
        _ => Err(format!("invalid scope: {scope}")),
    }
}

#[tauri::command]
fn workspace(scope: &str, root: Option<String>) -> Result<snapshot::WorkspaceSnapshot, String> {
    let scope = parse_scope(scope)?;
    list_workspace(scope, root).map_err(|e| e.to_string())
}

#[tauri::command]
fn inspect(
    scope: &str,
    root: Option<String>,
    source: String,
) -> Result<snapshot::SourceCatalog, String> {
    let scope = parse_scope(scope)?;
    inspect_source_input(scope, root, &source).map_err(|e| e.to_string())
}

#[tauri::command]
fn plan(payload: InstallRequest) -> Result<snapshot::InstallPlan, String> {
    plan_install(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn install(payload: InstallRequest) -> Result<snapshot::WorkspaceSnapshot, String> {
    install_source(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_install(payload: RemoveRequest) -> Result<snapshot::WorkspaceSnapshot, String> {
    remove_from_source(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn sync(scope: &str, root: Option<String>) -> Result<snapshot::WorkspaceSnapshot, String> {
    let scope = parse_scope(scope)?;
    sync_workspace(scope, root).map_err(|e| e.to_string())
}

#[tauri::command]
fn update(scope: &str, root: Option<String>) -> Result<snapshot::WorkspaceSnapshot, String> {
    let scope = parse_scope(scope)?;
    update_workspace(scope, root).map_err(|e| e.to_string())
}

#[tauri::command]
fn doctor(scope: &str, root: Option<String>) -> Result<snapshot::DoctorReport, String> {
    let scope = parse_scope(scope)?;
    doctor_workspace(scope, root).map_err(|e| e.to_string())
}

#[tauri::command]
fn guidance(scope: &str, root: Option<String>) -> Result<snapshot::AgentFileSnapshot, String> {
    let scope = parse_scope(scope)?;
    read_agent_file_state(scope, root).map_err(|e| e.to_string())
}

#[tauri::command]
fn guidance_write(payload: AgentFileWriteRequest) -> Result<snapshot::AgentFileSnapshot, String> {
    write_agent_file_user_content(payload).map_err(|e| e.to_string())
}

mod snapshot {
    pub use praxis_core::model::{
        AgentFileSnapshot, DoctorReport, InstallPlan, SourceCatalog, WorkspaceSnapshot,
    };
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            workspace,
            inspect,
            plan,
            install,
            remove_install,
            sync,
            update,
            doctor,
            guidance,
            guidance_write,
        ])
        .run(tauri::generate_context!())
        .expect("error while running praxis desktop");
}
