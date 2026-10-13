use crate::commands::types::*;

#[tauri::command]
pub fn load_skill_data(profession: String) -> Result<SkillEditorDataDTO, String> {
    load_skill_data_impl(profession)
}

#[cfg(feature = "static")]
fn load_skill_data_impl(profession: String) -> Result<SkillEditorDataDTO, String> {
    jpcg_core::host::skill::load_skill_data(profession)
}

#[cfg(feature = "dynamic")]
fn load_skill_data_impl(profession: String) -> Result<SkillEditorDataDTO, String> {
    let req = serde_json::json!({ "profession": profession });
    crate::commands::ffi_bridge::call("load_skill_data", &req)
}

#[tauri::command]
pub fn save_skill_data(profession: String, data: SkillEditorDataDTO) -> Result<(), String> {
    save_skill_data_impl(profession, data)
}

#[cfg(feature = "static")]
fn save_skill_data_impl(profession: String, data: SkillEditorDataDTO) -> Result<(), String> {
    jpcg_core::host::skill::save_skill_data(profession, data)
}

#[cfg(feature = "dynamic")]
fn save_skill_data_impl(profession: String, data: SkillEditorDataDTO) -> Result<(), String> {
    let req = serde_json::json!({ "profession": profession, "data": data });
    crate::commands::ffi_bridge::call::<_, serde_json::Value>("save_skill_data", &req).map(|_| ())
}
