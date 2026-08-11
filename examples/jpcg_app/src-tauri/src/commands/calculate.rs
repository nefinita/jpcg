use crate::commands::types::*;

#[tauri::command]
pub async fn calculate_damage(req: CalculateRequest) -> Result<Vec<SkillResultDTO>, String> {
    calculate_impl(req)
}

#[cfg(feature = "static")]
fn calculate_impl(req: CalculateRequest) -> Result<Vec<SkillResultDTO>, String> {
    jpcg_core::host::calc::calculate(req)
}

#[cfg(feature = "dynamic")]
fn calculate_impl(req: CalculateRequest) -> Result<Vec<SkillResultDTO>, String> {
    crate::commands::ffi_bridge::call("calculate", &req)
}
