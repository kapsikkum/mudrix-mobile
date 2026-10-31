//! iOS only: the counterpart of Android's keep-alive foreground service. There is no push service
//! without a paid Apple developer account, so the app instead keeps itself running in the
//! background (`UIBackgroundModes` `audio` and `location` in `Info.plist`) and the web client's own
//! Matrix sync raises local notifications.
//!
//! The Swift side (`ios/Sources/IosKeepAlivePlugin.swift`) offers two strategies:
//! - `audio` (default): a silent, looping `AVAudioSession` (`.playback`, `.mixWithOthers`), so other
//!   audio is not interrupted. No permission prompt.
//! - `location`: coarse, low-power `CLLocationManager` updates with background updates allowed.
//!   Asks for when-in-use, then always authorization.
//!
//! The enabled flag and the strategy are persisted (UserDefaults) and resumed when the app starts.
//! Enabling keep-alive also asks for local notification permission if it is still undetermined.
//!
//! Commands (called from `push.rs`, not from the web view): `status` and `setKeepAlive`
//! `{ enabled, strategy? }`. On every other platform the plugin registers nothing native.

use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

#[cfg(target_os = "ios")]
use serde_json::Value;
#[cfg(target_os = "ios")]
use tauri::{plugin::PluginHandle, Manager};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_ios_keepalive);

/// Managed state on iOS: the handle to the Swift plugin.
#[cfg(target_os = "ios")]
pub struct KeepAlive<R: Runtime>(PluginHandle<R>);

#[cfg(target_os = "ios")]
impl<R: Runtime> KeepAlive<R> {
    /// `{ notificationsAllowed, keepAlive, keepAliveStrategy }`
    pub fn status(&self) -> Result<Value, String> {
        self.0
            .run_mobile_plugin::<Value>("status", Value::Null)
            .map_err(|e| e.to_string())
    }

    /// Starts or stops keep-alive; `strategy` is `"audio"` (default) or `"location"`.
    pub fn set_keepalive(&self, enabled: bool, strategy: Option<&str>) -> Result<Value, String> {
        self.0
            .run_mobile_plugin::<Value>(
                "setKeepAlive",
                serde_json::json!({ "enabled": enabled, "strategy": strategy }),
            )
            .map_err(|e| e.to_string())
    }
}

/// The plugin. Add it with `.plugin(tauri_plugin_ios_keepalive::init())`.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("ios-keepalive")
        .setup(|_app, api| {
            #[cfg(target_os = "ios")]
            {
                let handle = api.register_ios_plugin(init_plugin_ios_keepalive)?;
                _app.manage(KeepAlive(handle));
            }
            #[cfg(not(target_os = "ios"))]
            let _ = api;
            Ok(())
        })
        .build()
}

/// Strategy names the Swift side understands; anything else falls back to the first.
pub const STRATEGIES: [&str; 2] = ["audio", "location"];

/// Normalizes a requested strategy (missing or unknown means `audio`).
pub fn normalize_strategy(strategy: Option<&str>) -> &'static str {
    match strategy {
        Some("location") => "location",
        _ => STRATEGIES[0],
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_strategy;

    #[test]
    fn unknown_strategies_fall_back_to_audio() {
        assert_eq!(normalize_strategy(None), "audio");
        assert_eq!(normalize_strategy(Some("bogus")), "audio");
        assert_eq!(normalize_strategy(Some("location")), "location");
    }
}
