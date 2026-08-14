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
            // 注：预设加载经 ComboPresetDTO 还原完整技能属性，此处 DTO 直转即可
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

#[cfg(test)]
mod tests {
    use super::*;
    use jpcg_api::{ComboStepResultDTO, SkillPoolItemDTO};

    fn pool_item(name: &str, lost_hp: f32) -> SkillPoolItemDTO {
        SkillPoolItemDTO {
            skill_name: name.into(),
            skill_id: 32614,
            sub_id: 32616,
            base_damage1: 35,
            base_damage2: 40,
            atk_xishu: 5.15625,
            watk_xishu: 0,
            hit_up: 0,
            huixin_up: 0,
            huixiao_up: 0,
            wushifangyu: 0,
            wushihuajin: 0,
            dot_flag: 0,
            has_critical_strike: true,
            lost_hp_zhenshishanghai: lost_hp,
        }
    }

    fn dto(
        player: &PlayerConfigDTO,
        hostile: &HostileConfigDTO,
        xinfa: &XinfaConfigDTO,
    ) -> (PlayerConfigDTO, HostileConfigDTO, XinfaConfigDTO) {
        (player.clone(), hostile.clone(), xinfa.clone())
    }

    fn player() -> PlayerConfigDTO {
        PlayerConfigDTO {
            jcsx: "gengu".into(),
            jichu_shuxing: 21371,
            jichu_gongji: 64329,
            huixin_dengji: 61877,
            huixin_xiaoguo: 2925,
            pofang_dengji: 109160,
            wuqi_shanghai: 0,
        }
    }

    fn hostile() -> HostileConfigDTO {
        HostileConfigDTO {
            waigong_fangyu: 15176,
            neigong_fangyu: 21388,
            yujin_dengji: 5047,
            huajin_dengji: 59402,
            jianshang_bili: 0,
            target_hp: 2_000_000,
        }
    }

    fn xinfa() -> XinfaConfigDTO {
        XinfaConfigDTO {
            profession: "mowen".into(),
            xinfa_name: "莫问".into(),
            xinfa_nom: "gengu".into(),
            atk_up: 1.96,
            pofang_up: 2.0,
            huixin_up: 0.0,
        }
    }

    fn buff_coeff() -> (BuffConfigDTO, CoefficientConfigDTO) {
        (
            BuffConfigDTO::default(),
            // DTO Default 全 0 → 除法溢出；填 core 层默认换算系数
            CoefficientConfigDTO {
                pofang_xishu: 225957.6,
                huixin_xishu: 197703.0,
                huixiao_xishu: 72844.2,
                huajin_xishu: 30115.8,
                fangyu_xishu: 126007.2,
                pvp_global_jianshang: 0.9,
            },
        )
    }

    /// 预设往返：DTO → ComboStep(快照) → 预设 TOML → DTO，技能全属性（含追加真伤）不丢失
    #[test]
    fn preset_roundtrip_keeps_full_skill() {
        let step_dto = ComboStepDTO {
            skill: pool_item("怒锋倾涛·单持·破绽3层", 0.18),
            overrides: None,
        };
        let core: ComboStep = ComboStep::from(step_dto.clone());
        assert!(core.skill_snapshot.is_some(), "保存时应写入技能快照");

        let toml_str = toml::to_string_pretty(&ComboPreset {
            name: "test".into(),
            steps: vec![core],
        })
        .expect("序列化");
        let loaded: ComboPreset = toml::from_str(&toml_str).expect("反序列化");
        let back_dto = ComboPresetDTO::from(loaded);
        let skill = &back_dto.steps[0].skill;
        assert_eq!(skill.skill_name, "怒锋倾涛·单持·破绽3层");
        assert_eq!(skill.base_damage1, 35);
        assert_eq!(skill.atk_xishu, 5.15625);
        assert_eq!(
            skill.lost_hp_zhenshishanghai, 0.18,
            "追加真伤应随预设往返保留"
        );
    }

    /// 加载预设后计算：追加真伤按已损失生命值生效（快照丢失则本步追加为 0）
    #[test]
    fn loaded_preset_applies_lost_hp_zhenshi() {
        let step_dto = ComboStepDTO {
            skill: pool_item("怒锋倾涛·单持·破绽3层", 0.18),
            overrides: None,
        };
        let core: ComboStep = ComboStep::from(step_dto);
        let rounds: ComboPresetDTO = ComboPresetDTO::from(ComboPreset {
            name: "test".into(),
            steps: vec![core],
        });
        let (b, c) = buff_coeff();
        let (p, h, x) = dto(&player(), &hostile(), &xinfa());
        let result: ComboResultDTO = calculate_combo(rounds.steps, p, h, x, b, c).expect("计算");
        let s0: &ComboStepResultDTO = &result.steps[0];
        assert!(
            s0.lost_hp_zhenshi_damage > 0.0,
            "加载预设后首击追加真伤应 > 0: got {}",
            s0.lost_hp_zhenshi_damage
        );
        assert!(
            s0.cumulative_mean_wan > s0.q_damage as f64 / 10000.0,
            "累计期望应含追加真伤"
        );
    }

    /// 旧存档（无快照）兼容：回退 DTO 重建，不 panic
    #[test]
    fn old_preset_without_snapshot_parses() {
        let legacy_toml = r#"
name = "旧存档"
[[steps]]
skill_id = 32614
sub_id = 32616
skill_name = "怒锋倾涛·单持·破绽0层"
"#;
        let loaded: ComboPreset = toml::from_str(legacy_toml).expect("旧存档可解析");
        assert!(loaded.steps[0].skill_snapshot.is_none());
        let back = ComboPresetDTO::from(loaded);
        assert_eq!(back.steps[0].skill.skill_name, "怒锋倾涛·单持·破绽0层");
    }
}
