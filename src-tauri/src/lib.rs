//! Mudrix Mobile: a Tauri v2 shell around the Mudrix web client for iOS and Android.
//!
//! The web client talks to its native shell through the command contract documented in
//! `src/util/desktop.ts` of mudrix-web. This shell implements the subset that makes sense on a
//! phone (notifications, external links, downloads) and answers the desktop-only commands with
//! neutral values, so the same web build runs unchanged.

mod commands;

use tauri::Manager;
use tauri_plugin_deep_link::DeepLinkExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .manage(commands::PendingNavigation::default())
        .setup(|app| {
            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    commands::open_deep_link(&handle, url.as_str());
                }
            });
            if let Ok(Some(urls)) = app.deep_link().get_current() {
                let state = app.state::<commands::PendingNavigation>();
                if let Some(url) = urls.first() {
                    state.set(url.to_string());
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_desktop_settings,
            commands::set_desktop_settings,
            commands::is_window_visible,
            commands::take_pending_navigation,
            commands::set_unread_count,
            commands::show_notification,
            commands::open_external,
            commands::open_auth_in_browser,
            commands::save_download,
            commands::get_system_accent,
            commands::set_app_icon,
            commands::check_for_updates,
            commands::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mudrix");
}
