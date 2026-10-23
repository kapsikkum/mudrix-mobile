//! Commands the Mudrix web client invokes (see `src/util/desktop.ts` in mudrix-web).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;

/// Deep links that arrive before the web app has started listening are parked here until it
/// calls `take_pending_navigation`.
#[derive(Default)]
pub struct PendingNavigation {
    web_ready: AtomicBool,
    pending: Mutex<Option<String>>,
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
    let state = app.state::<PendingNavigation>();
    if state.web_ready.load(Ordering::SeqCst) {
        let _ = app.emit("desktop://open", url.to_string());
    } else {
        *state.pending.lock().unwrap() = Some(url.to_string());
    }
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
    state.web_ready.store(true, Ordering::SeqCst);
    let pending = state.pending.lock().unwrap().take();
    pending
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
    sound: Option<String>,
) -> Result<(), String> {
    let _ = roomUrl; // opening the room from a tap needs the native notification action hooks
    let builder = app.notification().builder().title(title).body(body);
    #[cfg(target_os = "android")]
    let builder = match message_channel(&app, sound.as_deref()) {
        Some(channel) => builder.channel_id(channel),
        None => builder,
    };
    #[cfg(not(target_os = "android"))]
    let _ = sound;
    builder.show().map_err(|e| e.to_string())
}

/// Android picks a notification's sound from its channel, which cannot change once created. So each
/// sound the user can choose gets its own channel ("msg_<sound>", the sound being a bundled
/// `res/raw` file), and "silent" gets a channel without a sound.
#[cfg(target_os = "android")]
fn message_channel(app: &AppHandle, sound: Option<&str>) -> Option<String> {
    use tauri_plugin_notification::{Channel, Importance};

    let sound = sound?;
    let valid = !sound.is_empty()
        && sound
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !valid {
        return None;
    }
    let channel = if sound == "silent" {
        Channel::builder("msg_silent", "Messages (silent)")
            .importance(Importance::Low)
            .build()
    } else {
        Channel::builder(format!("msg_{sound}"), "Messages")
            .importance(Importance::High)
            .sound(sound)
            .build()
    };
    let id = channel.id().to_string();
    app.notification().create_channel(channel).ok()?;
    Some(id)
}

#[tauri::command]
pub fn open_external(app: AppHandle, url: String) -> Result<(), String> {
    let lower = url.to_ascii_lowercase();
    if !(lower.starts_with("https://")
        || lower.starts_with("http://")
        || lower.starts_with("mailto:"))
    {
        return Err("only http(s) and mailto links can be opened".into());
    }
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

/// Keeps a file name safe to create: no directories, no characters Windows rejects.
fn sanitize_file_name(raw: &str) -> String {
    let name: String = raw
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("")
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let name = name.trim().trim_matches('.').to_string();
    let name = if is_reserved_device_name(&name) {
        format!("_{name}")
    } else {
        name
    };
    // Windows drops trailing spaces and dots, so don't leave any.
    let name = name.trim_end_matches([' ', '.']).to_string();
    if name.is_empty() {
        "download".to_string()
    } else {
        name
    }
}

/// `CON`, `NUL`, `COM1`... are devices on Windows, with or without an extension.
fn is_reserved_device_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or("").trim_end();
    let upper = stem.to_ascii_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((upper.starts_with("COM") || upper.starts_with("LPT"))
            && upper.len() == 4
            && matches!(upper.as_bytes()[3], b'1'..=b'9'))
}

/// Picks `name`, `name (1).ext`, `name (2).ext`... so an existing file is never overwritten.
fn unique_path(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let path = std::path::Path::new(name);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("download");
    let ext = path.extension().and_then(|e| e.to_str());
    for n in 1..1000 {
        let numbered = match ext {
            Some(e) => format!("{stem} ({n}).{e}"),
            None => format!("{stem} ({n})"),
        };
        let candidate = dir.join(numbered);
        if !candidate.exists() {
            return candidate;
        }
    }
    dir.join(name)
}

/// Minimal `%XX` decoder for the file name header (invalid sequences are kept as they are).
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
            if let Ok(v) = u8::from_str_radix(hex, 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Saves an attachment into the app's data directory and returns the path. The name comes in the
/// `x-filename` header (URL-encoded); an existing file is never overwritten.
#[tauri::command]
pub fn save_download(app: AppHandle, request: tauri::ipc::Request<'_>) -> Result<String, String> {
    use std::io::Write;

    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else {
        return Err("expected raw bytes".into());
    };
    let raw_name = request
        .headers()
        .get("x-filename")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("download");
    let name = sanitize_file_name(&percent_decode(raw_name));
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("downloads");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = unique_path(&dir, &name);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
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

#[cfg(test)]
mod download_tests {
    use super::*;

    #[test]
    fn file_names_are_made_safe() {
        assert_eq!(sanitize_file_name(r"..\..\evil.exe"), "evil.exe");
        assert_eq!(sanitize_file_name("a/b:c?.txt"), "b_c_.txt");
        assert_eq!(sanitize_file_name("   "), "download");
        assert_eq!(sanitize_file_name("..."), "download");
        assert_eq!(sanitize_file_name("CON"), "_CON");
        assert_eq!(sanitize_file_name("nul.txt"), "_nul.txt");
        assert_eq!(sanitize_file_name("COM0"), "COM0");
        assert_eq!(sanitize_file_name("report. . "), "report");
    }

    #[test]
    fn header_names_are_percent_decoded() {
        assert_eq!(percent_decode("a%20b.txt"), "a b.txt");
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("%E3%81%82.txt"), "\u{3042}.txt");
    }
}
