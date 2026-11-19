//! Per-install device id — beside app settings, never inside a synced journal.

use debrief_journal::load_or_create_device_id;
use tauri::{AppHandle, Manager, Runtime};

/// Load or create this install's device id under the app data directory.
pub fn install_device_id<R: Runtime>(app: &AppHandle<R>) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("device_id");
    load_or_create_device_id(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn journal_install_device_id(app: AppHandle) -> Result<String, String> {
    install_device_id(&app)
}
