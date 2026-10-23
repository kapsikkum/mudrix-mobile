//! UnifiedPush on Android: a thin bridge to the Kotlin `PushPlugin` (registration, distributor
//! choice). Other platforms report that push is unsupported, so the web client can hide the UI.

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

#[tauri::command]
pub async fn push_status(app: AppHandle) -> Result<Value, String> {
    #[cfg(target_os = "android")]
    {
        call(&app, "status", json!({}))
    }
    #[cfg(not(target_os = "android"))]
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

#[tauri::command]
pub async fn push_keepalive(app: AppHandle, enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        call(&app, "setKeepAlive", json!({ "enabled": enabled })).map(|_| ())
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, enabled);
        Err("not supported on this platform".into())
    }
}
