use crate::commands::types::*;

#[tauri::command]
pub async fn calculate_damage(req: CalculateRequest) -> Result<Vec<SkillResultDTO>, String> {
    let player = req.player.into_core();
    let hostile = req.hostile.into_core();
    let xinfa = req.xinfa_config.into_core();
    let buff = jpcg_core::type_set::buff::BuffConfig {
        base_atk_pct: req.buff.base_atk_pct,
        huixin_pct: req.buff.huixin_pct,
        huixiao_pct: req.buff.huixiao_pct,
        pofang_pct: req.buff.pofang_pct,
        wushi_fangyu_pct: req.buff.wushi_fangyu_pct,
        shanghai_pct: req.buff.shanghai_pct,
        mode_is_point: req.buff.mode_is_point,
    };
    let coeff = jpcg_core::type_set::coefficient::CoefficientConfig {
        pofang_xishu: req.coefficient.pofang_xishu,
        huixin_xishu: req.coefficient.huixin_xishu,
        huixiao_xishu: req.coefficient.huixiao_xishu,
        huajin_xishu: req.coefficient.huajin_xishu,
        fangyu_xishu: req.coefficient.fangyu_xishu,
        pvp_global_jianshang: req.coefficient.pvp_global_jianshang,
    };

    let results = jpcg_core::calculate::start_with_config(player, hostile, xinfa, &buff, &coeff)
        .map_err(|e| e.to_string())?;

    Ok(results.into_iter().map(SkillResultDTO::from).collect())
}
