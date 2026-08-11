use crate::commands::types::SkillPoolItemDTO;

#[tauri::command]
pub fn load_skill_pool(profession: String) -> Vec<SkillPoolItemDTO> {
    load_skill_pool_impl(profession)
}

#[cfg(feature = "static")]
fn load_skill_pool_impl(profession: String) -> Vec<SkillPoolItemDTO> {
    jpcg_core::host::skill::load_skill_pool(profession)
}

#[cfg(feature = "dynamic")]
fn load_skill_pool_impl(profession: String) -> Vec<SkillPoolItemDTO> {
    let req = serde_json::json!({ "profession": profession });
    crate::commands::ffi_bridge::call("load_skill_pool", &req).unwrap_or_default()
}
