// ============================================================================
// host::combo — 连招计算与预设 CRUD 入口
// ============================================================================

use jpcg_api::{
    BuffConfigDTO, CoefficientConfigDTO, ComboPresetDTO, ComboResultDTO, ComboStepDTO,
    HostileConfigDTO, PlayerConfigDTO, XinfaConfigDTO,
};

use crate::engine;
use crate::store;
use crate::type_set::combo::{ComboPreset, ComboStep};
use crate::type_set::{
    buff::BuffConfig, coefficient::CoefficientConfig, hostilepile::HostilepileConfig,
    player::PlayerConfig, xinfa::XinfaConfig,
};

fn into_core(
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
    buff: BuffConfigDTO,
    coeff: CoefficientConfigDTO,
) -> (
    PlayerConfig,
    HostilepileConfig,
    XinfaConfig,
    BuffConfig,
    CoefficientConfig,
) {
    let player = PlayerConfig::new(
        player.jcsx,
        player.jichu_shuxing,
        player.jichu_gongji,
        player.huixin_dengji,
        player.huixin_xiaoguo,
        player.pofang_dengji,
        player.wuqi_shanghai,
    );
    let hostile = HostilepileConfig {
        waigong_fangyu: hostile.waigong_fangyu,
        neigong_fangyu: hostile.neigong_fangyu,
        yujin_dengji: hostile.yujin_dengji,
        huajin_dengji: hostile.huajin_dengji,
        jianshang_bili: hostile.jianshang_bili,
        target_hp: hostile.target_hp,
    };
    let xinfa = XinfaConfig::new(
        xinfa.profession,
        xinfa.xinfa_name,
        xinfa.xinfa_nom,
        xinfa.atk_up,
        xinfa.pofang_up,
        xinfa.huixin_up,
    );
    let buff = BuffConfig {
        base_atk_pct: buff.base_atk_pct,
        huixin_pct: buff.huixin_pct,
        huixiao_pct: buff.huixiao_pct,
        pofang_pct: buff.pofang_pct,
        wushi_fangyu_pct: buff.wushi_fangyu_pct,
        shanghai_pct: buff.shanghai_pct,
        mode_is_point: buff.mode_is_point,
    };
    let coeff = CoefficientConfig {
        pofang_xishu: coeff.pofang_xishu,
        huixin_xishu: coeff.huixin_xishu,
        huixiao_xishu: coeff.huixiao_xishu,
        huajin_xishu: coeff.huajin_xishu,
        fangyu_xishu: coeff.fangyu_xishu,
        pvp_global_jianshang: coeff.pvp_global_jianshang,
    };
    (player, hostile, xinfa, buff, coeff)
}

/// 连招伤害计算
pub fn calculate_combo(
    steps: Vec<ComboStepDTO>,
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
    buff: BuffConfigDTO,
    coefficient: CoefficientConfigDTO,
) -> Result<ComboResultDTO, String> {
    let (player, hostile, xinfa, buff, coeff) =
        into_core(player, hostile, xinfa, buff, coefficient);

    let skilltypes: Vec<_> = steps
        .iter()
        .map(|s| {
            let mut st = super::calc::skill_dto_to_skilltype(&s.skill);
            if let Some(ref o) = s.overrides {
                if let Some(v) = o.base_damage_override {
                    st.base_damage1 = v as u32;
                    st.base_damage2 = v as u32;
                }
                if let Some(v) = o.atk_xishu_override {
                    st.atk_xishu = v;
                }
                if let Some(v) = o.jianshang_bili_override {
                    st.wushijianshang = v as u32;
                }
                if let Some(v) = o.wushihuajin_override {
                    st.wushihuajin = v as u32;
                }
                if let Some(v) = o.extra_crit_pct {
                    st.huixin_up = v as u32;
                }
                if let Some(v) = o.extra_crit_dmg_pct {
                    st.huixiao_up = v as u32;
                }
            }
            st
        })
        .collect();

    let result =
        engine::kill_prob::calculate_combo(&skilltypes, &player, &hostile, &xinfa, &buff, &coeff);
    Ok(ComboResultDTO::from(result))
}

/// 保存连招预设
pub fn save_combo_preset(name: String, steps: Vec<ComboStepDTO>) -> Result<(), String> {
    let core_steps = steps.into_iter().map(ComboStep::from).collect();
    let preset = ComboPreset {
        name,
        steps: core_steps,
    };
    store::save_combo_preset(&preset)
}

/// 列出所有连招预设
pub fn list_combo_presets() -> Vec<String> {
    store::list_combo_presets()
}

/// 加载连招预设
pub fn load_combo_preset(name: String) -> Result<ComboPresetDTO, String> {
    let preset = store::load_combo_preset(&name).ok_or_else(|| "预设不存在".to_string())?;
    Ok(ComboPresetDTO::from(preset))
}

/// 删除连招预设
pub fn delete_combo_preset(name: String) -> Result<(), String> {
    store::delete_combo_preset(&name)
}

/// 导出当前配置为 TOML 字符串
pub fn export_config() -> Result<String, String> {
    store::export_config_toml()
}

/// 导入配置 TOML 字符串
pub fn import_config(toml_str: String) -> Result<(), String> {
    store::import_config_toml(&toml_str)
}
