use crate::commands::types::*;

#[tauri::command]
pub fn calculate_combo_cmd(
    steps: Vec<ComboStepDTO>,
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
    buff: BuffConfigDTO,
    coefficient: CoefficientConfigDTO,
) -> Result<ComboResultDTO, String> {
    calculate_combo_impl(steps, player, hostile, xinfa, buff, coefficient)
}

#[cfg(feature = "static")]
fn calculate_combo_impl(
    steps: Vec<ComboStepDTO>,
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
    buff: BuffConfigDTO,
    coefficient: CoefficientConfigDTO,
) -> Result<ComboResultDTO, String> {
    use jpcg_combo::engine::ComboConfig;
    jpcg_combo::host::calculate_combo(
        steps,
        player,
        hostile,
        xinfa,
        buff,
        coefficient,
        ComboConfig::default(),
    )
}

#[cfg(feature = "dynamic")]
fn calculate_combo_impl(
    steps: Vec<ComboStepDTO>,
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
    buff: BuffConfigDTO,
    coefficient: CoefficientConfigDTO,
) -> Result<ComboResultDTO, String> {
    let req = serde_json::json!({
        "steps": steps, "player": player, "hostile": hostile,
        "xinfa": xinfa, "buff": buff, "coefficient": coefficient
    });
    crate::commands::ffi_bridge::call_combo("calculate_combo", &req)
}

#[tauri::command]
pub fn save_combo_preset(name: String, steps: Vec<ComboStepDTO>) -> Result<(), String> {
    save_combo_preset_impl(name, steps)
}

#[cfg(feature = "static")]
fn save_combo_preset_impl(name: String, steps: Vec<ComboStepDTO>) -> Result<(), String> {
    jpcg_combo::host::save_combo_preset(name, steps)
}

#[cfg(feature = "dynamic")]
fn save_combo_preset_impl(name: String, steps: Vec<ComboStepDTO>) -> Result<(), String> {
    let req = serde_json::json!({ "name": name, "steps": steps });
    crate::commands::ffi_bridge::call_combo::<_, serde_json::Value>("save_combo_preset", &req)
        .map(|_| ())
}

#[tauri::command]
pub fn list_combo_presets() -> Vec<String> {
    list_combo_presets_impl()
}

#[cfg(feature = "static")]
fn list_combo_presets_impl() -> Vec<String> {
    jpcg_combo::host::list_combo_presets()
}

#[cfg(feature = "dynamic")]
fn list_combo_presets_impl() -> Vec<String> {
    crate::commands::ffi_bridge::call_combo_no_args("list_combo_presets").unwrap_or_default()
}

#[tauri::command]
pub fn load_combo_preset(name: String) -> Result<ComboPresetDTO, String> {
    load_combo_preset_impl(name)
}

#[cfg(feature = "static")]
fn load_combo_preset_impl(name: String) -> Result<ComboPresetDTO, String> {
    jpcg_combo::host::load_combo_preset(name)
}

#[cfg(feature = "dynamic")]
fn load_combo_preset_impl(name: String) -> Result<ComboPresetDTO, String> {
    let req = serde_json::json!({ "name": name });
    crate::commands::ffi_bridge::call_combo("load_combo_preset", &req)
}

#[tauri::command]
pub fn delete_combo_preset(name: String) -> Result<(), String> {
    delete_combo_preset_impl(name)
}

#[cfg(feature = "static")]
fn delete_combo_preset_impl(name: String) -> Result<(), String> {
    jpcg_combo::host::delete_combo_preset(name)
}

#[cfg(feature = "dynamic")]
fn delete_combo_preset_impl(name: String) -> Result<(), String> {
    let req = serde_json::json!({ "name": name });
    crate::commands::ffi_bridge::call_combo::<_, serde_json::Value>("delete_combo_preset", &req)
        .map(|_| ())
}

#[tauri::command]
pub fn export_config_cmd() -> Result<String, String> {
    export_config_impl()
}

#[cfg(feature = "static")]
fn export_config_impl() -> Result<String, String> {
    jpcg_combo::host::export_config()
}

#[cfg(feature = "dynamic")]
fn export_config_impl() -> Result<String, String> {
    crate::commands::ffi_bridge::call_combo_no_args("export_config")
}

#[tauri::command]
pub fn import_config_cmd(toml_str: String) -> Result<(), String> {
    import_config_impl(toml_str)
}

#[cfg(feature = "static")]
fn import_config_impl(toml_str: String) -> Result<(), String> {
    jpcg_combo::host::import_config(toml_str)
}

#[cfg(feature = "dynamic")]
fn import_config_impl(toml_str: String) -> Result<(), String> {
    let req = serde_json::json!({ "toml_str": toml_str });
    crate::commands::ffi_bridge::call_combo::<_, serde_json::Value>("import_config", &req)
        .map(|_| ())
}
