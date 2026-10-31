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
