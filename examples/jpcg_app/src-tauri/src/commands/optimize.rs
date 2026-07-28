use crate::commands::types::*;

#[tauri::command]
pub async fn compute_derivatives(req: CalculateRequest) -> Result<DerivativesOutputDTO, String> {
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

    let toml_config = jpcg_core::skill_editor::load_skills(&xinfa.profession);
    let skills = toml_config.skill;

    let output = jpcg_core::derivatives::compute_derivatives(
        &player, &hostile, &buff, &coeff, &xinfa, &skills,
    );

    Ok(DerivativesOutputDTO::from(output))
}
