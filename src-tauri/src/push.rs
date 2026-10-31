//! UnifiedPush on Android: a thin bridge to the Kotlin `PushPlugin` (registration, distributor
//! choice). On iOS there is no push service (no paid Apple developer account), so only the
//! keep-alive part exists: the Swift `ios-keepalive` plugin keeps the app running in the background
//! and the web client raises local notifications itself. Other platforms report that push is
//! unsupported, so the web client can hide the UI.

use serde_json::{json, Value};
use tauri::AppHandle;

#[cfg(target_os = "android")]
use tauri::{
    plugin::{Builder, PluginHandle, TauriPlugin},
    Manager, Wry,
};

#[cfg(target_os = "android")]
pub struct Push(PluginHandle<Wry>);

#[cfg(target_os = "android")]
pub fn init() -> TauriPlugin<Wry> {
    Builder::new("push")
        .setup(|app, api| {
            let handle = api.register_android_plugin("chat.mudrix.mobile", "PushPlugin")?;
            app.manage(Push(handle));
            Ok(())
        })
        .build()
}

#[cfg(target_os = "android")]
fn call(app: &AppHandle, command: &str, payload: Value) -> Result<Value, String> {
    app.state::<Push>()
        .0
        .run_mobile_plugin::<Value>(command, payload)
        .map_err(|e| e.to_string())
}

/// iOS: the persisted keep-alive state plus whether local notifications are allowed. There is no
/// push endpoint or distributor, so those fields are empty.
#[cfg(target_os = "ios")]
fn ios_status(app: &AppHandle) -> Result<Value, String> {
    use tauri::Manager;
    let native = app
        .state::<tauri_plugin_ios_keepalive::KeepAlive<tauri::Wry>>()
        .status()?;
    let flag = |key: &str| native.get(key).and_then(Value::as_bool).unwrap_or(false);
    Ok(json!({
        "supported": true,
        "distributors": [],
        "distributor": null,
        "endpoint": null,
        "error": null,
        "notificationsAllowed": flag("notificationsAllowed"),
        "keepAlive": flag("keepAlive"),
        "keepAliveStrategy": tauri_plugin_ios_keepalive::normalize_strategy(
            native.get("keepAliveStrategy").and_then(Value::as_str),
        ),
    }))
}

#[tauri::command]
pub async fn push_status(app: AppHandle) -> Result<Value, String> {
    #[cfg(target_os = "android")]
    {
        call(&app, "status", json!({}))
    }
    #[cfg(target_os = "ios")]
    {
        ios_status(&app)
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let _ = app;
        Ok(json!({ "supported": false }))
    }
}

#[tauri::command]
pub async fn push_register(app: AppHandle, distributor: Option<String>) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        call(&app, "register", json!({ "distributor": distributor })).map(|_| ())
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, distributor);
        Err("push is not supported on this platform".into())
    }
}

#[tauri::command]
pub async fn push_unregister(app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        call(&app, "unregister", json!({})).map(|_| ())
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Ok(())
    }
}

/// Android: starts or stops the foreground service (`strategy` is ignored). iOS: starts or stops
/// the background keep-alive; `strategy` is `"audio"` (default) or `"location"`.
#[tauri::command]
pub async fn push_keepalive(
    app: AppHandle,
    enabled: bool,
    strategy: Option<String>,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let _ = strategy;
        call(&app, "setKeepAlive", json!({ "enabled": enabled })).map(|_| ())
    }
    #[cfg(target_os = "ios")]
    {
        use tauri::Manager;
        app.state::<tauri_plugin_ios_keepalive::KeepAlive<tauri::Wry>>()
            .set_keepalive(
                enabled,
                Some(tauri_plugin_ios_keepalive::normalize_strategy(strategy.as_deref())),
            )
            .map(|_| ())
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let _ = (app, enabled, strategy);
        Err("not supported on this platform".into())
    }
}
