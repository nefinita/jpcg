use jpcg_gear::{Catalog, Composition, Loadout};
use serde::Serialize;
use serde::de::DeserializeOwned;

#[cfg(feature = "static")]
fn call<T: Serialize, R: DeserializeOwned>(method: &str, input: &T) -> Result<R, String> {
    let input = serde_json::to_string(input).map_err(|e| e.to_string())?;
    serde_json::from_value(jpcg_gear::host::call(method, &input)?).map_err(|e| e.to_string())
}
#[cfg(feature = "dynamic")]
fn call<T: Serialize, R: DeserializeOwned>(method: &str, input: &T) -> Result<R, String> {
    super::gear_bridge::call(method, input)
}

#[tauri::command]
pub async fn gear_catalog() -> Result<Catalog, String> {
    tauri::async_runtime::spawn_blocking(|| call("catalog", &serde_json::json!({})))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn compose_gear(loadout: Loadout) -> Result<Composition, String> {
    tauri::async_runtime::spawn_blocking(move || call("compose", &loadout))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn export_gear(loadout: Loadout) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || call("export", &loadout))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn import_gear(toml: String) -> Result<Composition, String> {
    tauri::async_runtime::spawn_blocking(move || call("import", &serde_json::json!({"toml":toml})))
        .await
        .map_err(|e| e.to_string())?
}
#[cfg(feature = "static")]
#[tauri::command]
pub async fn optimize_gear(
    app: tauri::AppHandle,
    request: jpcg_gear::OptimizeRequest,
) -> Result<jpcg_gear::OptimizeResult, String> {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use tauri::Emitter;
    let progress = Arc::new(jpcg_gear::OptimizeProgress::new());
    let monitor = {
        let progress = Arc::clone(&progress);
        let app = app.clone();
        std::thread::spawn(move || {
            let mut last = u64::MAX;
            while !progress.done.load(Ordering::Relaxed) {
                let pct = progress.percent();
                if pct != last {
                    last = pct;
                    let _ = app.emit("gear-optimize-progress", pct);
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            let _ = app.emit("gear-optimize-progress", 100u64);
        })
    };
    let p = Arc::clone(&progress);
    let result = tauri::async_runtime::spawn_blocking(move || {
        let catalog = jpcg_gear::host::catalog()?;
        jpcg_gear::optimize_parallel(&catalog, &request, &p)
    })
    .await
    .map_err(|e| e.to_string());
    progress.done.store(true, Ordering::Relaxed);
    let _ = monitor.join();
    result?
}

#[cfg(feature = "dynamic")]
#[tauri::command]
pub async fn optimize_gear(
    request: jpcg_gear::OptimizeRequest,
) -> Result<jpcg_gear::OptimizeResult, String> {
    tauri::async_runtime::spawn_blocking(move || call("optimize", &request))
        .await
        .map_err(|e| e.to_string())?
}
