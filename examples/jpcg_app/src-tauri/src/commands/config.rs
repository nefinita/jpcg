use crate::commands::types::*;

#[tauri::command]
pub fn save_config_cmd(
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
    buff: BuffConfigDTO,
    coefficient: CoefficientConfigDTO,
    value_set: Option<String>,
) -> Result<(), String> {
    save_config_impl(player, hostile, xinfa, buff, coefficient, value_set)
}

#[cfg(feature = "static")]
fn save_config_impl(
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
    buff: BuffConfigDTO,
    coefficient: CoefficientConfigDTO,
    value_set: Option<String>,
) -> Result<(), String> {
    jpcg_core::host::config::save_config(player, hostile, xinfa, buff, coefficient, value_set);
    Ok(())
}

#[cfg(feature = "dynamic")]
fn save_config_impl(
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
    buff: BuffConfigDTO,
    coefficient: CoefficientConfigDTO,
    value_set: Option<String>,
) -> Result<(), String> {
    let req = serde_json::json!({
        "player": player,
        "hostile": hostile,
        "xinfa": xinfa,
        "buff": buff,
        "coefficient": coefficient,
        "value_set": value_set,
    });
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

/// 列出可用数值集（正式服/体验服等；来源 data/values/index.toml，缺失时为内置兜底）
#[tauri::command]
pub fn list_value_sets_cmd() -> Result<Vec<ValueSetDTO>, String> {
    list_value_sets_impl()
}

#[cfg(feature = "static")]
fn list_value_sets_impl() -> Result<Vec<ValueSetDTO>, String> {
    Ok(jpcg_core::host::values::list_value_sets())
}

#[cfg(feature = "dynamic")]
fn list_value_sets_impl() -> Result<Vec<ValueSetDTO>, String> {
    crate::commands::ffi_bridge::call_no_args("list_value_sets")
}

/// 解析指定值集**实际使用**的信息（含回退：如请求 live-130 但快照不可用时返回 builtin）
#[tauri::command]
pub fn resolve_value_set_cmd(value_set: Option<String>) -> Result<ValueSetDTO, String> {
    resolve_value_set_impl(value_set)
}

#[cfg(feature = "static")]
fn resolve_value_set_impl(value_set: Option<String>) -> Result<ValueSetDTO, String> {
    Ok(jpcg_core::host::values::resolve_value_set(
        value_set.as_deref(),
    ))
}

#[cfg(feature = "dynamic")]
fn resolve_value_set_impl(value_set: Option<String>) -> Result<ValueSetDTO, String> {
    let req = serde_json::json!({ "value_set": value_set });
    crate::commands::ffi_bridge::call("resolve_value_set", &req)
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
