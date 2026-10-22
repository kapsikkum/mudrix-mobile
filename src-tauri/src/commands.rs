//! Commands the Mudrix web client invokes (see `src/util/desktop.ts` in mudrix-web).

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;

/// A deep link that arrived before the web app was ready to handle it.
#[derive(Default)]
pub struct PendingNavigation(Mutex<Option<String>>);

impl PendingNavigation {
    pub fn set(&self, target: String) {
        *self.0.lock().unwrap() = Some(target);
    }

    fn take(&self) -> Option<String> {
        self.0.lock().unwrap().take()
    }
}

/// Desktop-only settings. A phone has no tray or autostart, so these are fixed.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DesktopSettings {
    keep_running_in_background: bool,
    launch_at_login: bool,
    start_minimized: bool,
    notifications_muted: bool,
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            keep_running_in_background: true,
            launch_at_login: false,
            start_minimized: false,
            notifications_muted: false,
        }
    }
}

/// A `matrix:` URI or in-app path opened from outside the app.
pub fn open_deep_link(app: &AppHandle, url: &str) {
    let _ = app.emit("desktop://open", url.to_string());
}

#[tauri::command]
pub fn get_desktop_settings() -> DesktopSettings {
    DesktopSettings::default()
}

#[tauri::command]
pub fn set_desktop_settings(_patch: serde_json::Value) -> DesktopSettings {
    DesktopSettings::default()
}

#[tauri::command]
pub fn is_window_visible() -> bool {
    true
}

#[tauri::command]
pub fn take_pending_navigation(state: State<'_, PendingNavigation>) -> Option<String> {
    state.take()
}

/// Launcher badges are not wired up yet; accepted so the web app's calls succeed.
#[tauri::command]
pub fn set_unread_count(_total: u32, _highlight: u32) {}

#[tauri::command]
#[allow(non_snake_case)]
pub fn show_notification(
    app: AppHandle,
    title: String,
    body: String,
    roomUrl: Option<String>,
) -> Result<(), String> {
    let _ = roomUrl; // opening the room from a tap needs the native notification action hooks
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_external(app: AppHandle, url: String) -> Result<(), String> {
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

/// SSO/OIDC: opens the system browser. Returning to the app through a redirect scheme is a
/// follow-up; password login works without it.
#[tauri::command]
pub fn open_auth_in_browser(app: AppHandle, url: String) -> Result<(), String> {
    open_external(app, url)
}

/// Saves an attachment into the app's data directory and returns the path.
#[tauri::command]
pub fn save_download(
    app: AppHandle,
    request: tauri::ipc::Request<'_>,
) -> Result<String, String> {
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else {
        return Err("expected raw bytes".into());
    };
    let name = request
        .headers()
        .get("x-filename")
        .and_then(|v| v.to_str().ok())
        .map(|v| {
            v.replace("%20", " ")
                .chars()
                .filter(|c| !matches!(c, '/' | '\\' | ':' | '\0'))
                .collect::<String>()
        })
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "download".into());
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("downloads");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(name);
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn get_system_accent() -> Option<String> {
    None
}

#[tauri::command]
pub fn set_app_icon(_png: Option<Vec<u8>>) {}

#[tauri::command]
pub fn check_for_updates() -> &'static str {
    "upToDate"
}

#[tauri::command]
pub fn install_update() {}
