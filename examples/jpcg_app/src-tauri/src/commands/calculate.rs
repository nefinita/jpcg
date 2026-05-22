use crate::commands::types::*;

#[tauri::command]
pub async fn calculate_damage(req: CalculateRequest) -> Result<Vec<SkillResultDTO>, String> {
    let player = req.player.into_core();
    let hostile = req.hostile.into_core();
    let xinfa = req.xinfa_config.into_core();

    let results = jpcg_core::calculate::start(player, hostile, xinfa);

    Ok(results.into_iter().flatten().map(SkillResultDTO::from).collect())
}
