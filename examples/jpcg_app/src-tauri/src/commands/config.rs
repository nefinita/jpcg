use crate::commands::types::*;

#[tauri::command]
pub fn save_config_cmd(
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
) -> Result<(), String> {
    save_config_impl(player, hostile, xinfa)
}

#[cfg(feature = "static")]
fn save_config_impl(
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
) -> Result<(), String> {
    jpcg_core::host::config::save_config(player, hostile, xinfa);
    Ok(())
}

#[cfg(feature = "dynamic")]
fn save_config_impl(
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
) -> Result<(), String> {
    let req = serde_json::json!({ "player": player, "hostile": hostile, "xinfa": xinfa });
    crate::commands::ffi_bridge::call::<_, serde_json::Value>("save_config", &req).map(|_| ())
}

#[tauri::command]
pub fn load_config_cmd() -> Result<ConfigDataDTO, String> {
    load_config_impl()
}

#[cfg(feature = "static")]
fn load_config_impl() -> Result<ConfigDataDTO, String> {
    Ok(jpcg_core::host::config::load_config())
}

#[cfg(feature = "dynamic")]
fn load_config_impl() -> Result<ConfigDataDTO, String> {
    crate::commands::ffi_bridge::call_no_args("load_config")
}

#[tauri::command]
pub fn list_professions_cmd() -> Result<Vec<XinfaSummaryDTO>, String> {
    list_professions_impl()
}

#[cfg(feature = "static")]
fn list_professions_impl() -> Result<Vec<XinfaSummaryDTO>, String> {
    Ok(jpcg_core::host::config::list_professions())
}

#[cfg(feature = "dynamic")]
fn list_professions_impl() -> Result<Vec<XinfaSummaryDTO>, String> {
    crate::commands::ffi_bridge::call_no_args("list_professions")
}

#[tauri::command]
pub fn load_profession_config(profession: String) -> Result<XinfaConfigDTO, String> {
    load_profession_config_impl(profession)
}

#[cfg(feature = "static")]
fn load_profession_config_impl(profession: String) -> Result<XinfaConfigDTO, String> {
    let data = jpcg_core::host::skill::load_skill_data(profession)?;
    Ok(data.xinfa)
}

#[cfg(feature = "dynamic")]
fn load_profession_config_impl(profession: String) -> Result<XinfaConfigDTO, String> {
    let req = serde_json::json!({ "profession": profession });
    let data: jpcg_api::SkillEditorDataDTO =
        crate::commands::ffi_bridge::call("load_skill_data", &req)?;
    Ok(data.xinfa)
}
