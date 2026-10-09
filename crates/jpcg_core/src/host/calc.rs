// ============================================================================
// host::calc — 伤害计算与自动求导入口
// ============================================================================

use jpcg_api::{CalculateRequest, DerivativesOutputDTO, SkillResultDTO};

use crate::engine;
use crate::host::conv;
use crate::type_set::{
    buff::BuffConfig, coefficient::CoefficientConfig, hostilepile::HostilepileConfig,
    player::PlayerConfig, xinfa::XinfaConfig,
};

/// 将 DTO 计算请求转换为 core 领域类型（委托 [`host::conv`] 单一来源）
pub(crate) fn into_core(
    req: CalculateRequest,
) -> (
    PlayerConfig,
    HostilepileConfig,
    XinfaConfig,
    BuffConfig,
    CoefficientConfig,
) {
    conv::into_core(
        req.player,
        req.hostile,
        req.xinfa_config,
        req.buff,
        req.coefficient,
    )
}

/// 伤害计算（单技能表，不含连招）
pub fn calculate(req: CalculateRequest) -> Result<Vec<SkillResultDTO>, String> {
    let (player, hostile, xinfa, buff, coeff) = into_core(req);
    let results = engine::start_calculation_with_config(player, hostile, xinfa, &buff, &coeff)
        .map_err(|e| e.to_string())?;
    Ok(results.into_iter().map(SkillResultDTO::from).collect())
}

/// 自动求导（6 属性对全部技能）
pub fn compute_derivatives(req: CalculateRequest) -> Result<DerivativesOutputDTO, String> {
    let (player, hostile, xinfa, buff, coeff) = into_core(req);
    let toml_config = crate::store::load_config(&xinfa.profession);
    let output = engine::derivatives::compute_derivatives(
        &player,
        &hostile,
        &buff,
        &coeff,
        &xinfa,
        &toml_config.skill,
    );
    Ok(DerivativesOutputDTO::from(output))
}
