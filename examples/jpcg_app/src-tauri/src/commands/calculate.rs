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

#[tauri::command]
pub async fn panel_stats(req: CalculateRequest) -> Result<PanelStatsDTO, String> {
    #[cfg(feature = "static")]
    {
        Ok(jpcg_core::host::calc::panel_stats(req))
    }
    #[cfg(feature = "dynamic")]
    {
        crate::commands::ffi_bridge::call("panel_stats", &req)
    }
}
