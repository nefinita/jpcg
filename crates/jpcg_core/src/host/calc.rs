// ============================================================================
// host::calc — 伤害计算与自动求导入口
// ============================================================================

use jpcg_api::{CalculateRequest, DerivativesOutputDTO, SkillResultDTO};

use crate::engine;
use crate::type_set::{
    buff::BuffConfig, coefficient::CoefficientConfig, hostilepile::HostilepileConfig,
    player::PlayerConfig, skilltype::Skilltype, xinfa::XinfaConfig,
};

/// 将 DTO 计算请求转换为 core 领域类型
pub(crate) fn into_core(
    req: CalculateRequest,
) -> (
    PlayerConfig,
    HostilepileConfig,
    XinfaConfig,
    BuffConfig,
    CoefficientConfig,
) {
    let player = PlayerConfig::new(
        req.player.jcsx,
        req.player.jichu_shuxing,
        req.player.jichu_gongji,
        req.player.huixin_dengji,
        req.player.huixin_xiaoguo,
        req.player.pofang_dengji,
        req.player.wuqi_shanghai,
    );
    let hostile = HostilepileConfig {
        waigong_fangyu: req.hostile.waigong_fangyu,
        neigong_fangyu: req.hostile.neigong_fangyu,
        yujin_dengji: req.hostile.yujin_dengji,
        huajin_dengji: req.hostile.huajin_dengji,
        jianshang_bili: req.hostile.jianshang_bili,
        target_hp: req.hostile.target_hp,
        max_hp: req.hostile.max_hp,
        current_hp: req.hostile.current_hp,
    };
    let xinfa = XinfaConfig::new(
        req.xinfa_config.profession,
        req.xinfa_config.xinfa_name,
        req.xinfa_config.xinfa_nom,
        req.xinfa_config.atk_up,
        req.xinfa_config.pofang_up,
        req.xinfa_config.huixin_up,
    );
    let buff = BuffConfig {
        base_atk_pct: req.buff.base_atk_pct,
        huixin_pct: req.buff.huixin_pct,
        huixiao_pct: req.buff.huixiao_pct,
        pofang_pct: req.buff.pofang_pct,
        wushi_fangyu_pct: req.buff.wushi_fangyu_pct,
        shanghai_pct: req.buff.shanghai_pct,
        mode_is_point: req.buff.mode_is_point,
    };
    let coeff = CoefficientConfig {
        pofang_xishu: req.coefficient.pofang_xishu,
        huixin_xishu: req.coefficient.huixin_xishu,
        huixiao_xishu: req.coefficient.huixiao_xishu,
        huajin_xishu: req.coefficient.huajin_xishu,
        fangyu_xishu: req.coefficient.fangyu_xishu,
        pvp_global_jianshang: req.coefficient.pvp_global_jianshang,
    };
    (player, hostile, xinfa, buff, coeff)
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

/// 技能池条目转换（combo 模块复用）
pub(crate) fn skill_dto_to_skilltype(s: &jpcg_api::SkillPoolItemDTO) -> Skilltype {
    Skilltype {
        skill_name: s.skill_name.clone(),
        skill_id: s.skill_id,
        sub_id: s.sub_id,
        base_damage1: s.base_damage1,
        base_damage2: s.base_damage2,
        atk_xishu: s.atk_xishu,
        watk_xishu: s.watk_xishu,
        hit_up: s.hit_up,
        huixin_up: s.huixin_up,
        huixiao_up: s.huixiao_up,
        wushifangyu: s.wushifangyu,
        wushihuajin: s.wushihuajin,
        dot_flag: s.dot_flag,
        has_critical_strike: s.has_critical_strike,
        lost_hp_zhenshishanghai: s.lost_hp_zhenshishanghai,
        ..Default::default()
    }
}
