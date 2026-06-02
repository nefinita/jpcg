use crate::commands::types::*;

fn skill_dto_to_skilltype(s: &SkillPoolItemDTO) -> jpcg_core::type_set::skilltype::Skilltype {
    jpcg_core::type_set::skilltype::Skilltype {
        skill_name: s.skill_name.clone(),
        skill_id: s.skill_id,
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
        ..Default::default()
    }
}

#[tauri::command]
pub fn calculate_combo_cmd(
    steps: Vec<ComboStepDTO>,
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
    buff: BuffConfigDTO,
    coefficient: CoefficientConfigDTO,
) -> Result<ComboResultDTO, String> {
    let player_core = player.into_core();
    let hostile_core = hostile.into_core();
    let xinfa_core = xinfa.into_core();
    let buff_core = jpcg_core::type_set::buff::BuffConfig {
        base_atk_pct: buff.base_atk_pct,
        huixin_pct: buff.huixin_pct,
        huixiao_pct: buff.huixiao_pct,
        pofang_pct: buff.pofang_pct,
        wushi_fangyu_pct: buff.wushi_fangyu_pct,
        shanghai_pct: buff.shanghai_pct,
        mode_is_point: buff.mode_is_point,
    };
    let coeff_core = jpcg_core::type_set::coefficient::CoefficientConfig {
        pofang_xishu: coefficient.pofang_xishu,
        huixin_xishu: coefficient.huixin_xishu,
        huixiao_xishu: coefficient.huixiao_xishu,
        huajin_xishu: coefficient.huajin_xishu,
        fangyu_xishu: coefficient.fangyu_xishu,
        pvp_global_jianshang: coefficient.pvp_global_jianshang,
    };

    let skilltypes: Vec<jpcg_core::type_set::skilltype::Skilltype> = steps
        .iter()
        .map(|s| {
            let mut st = skill_dto_to_skilltype(&s.skill);
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

    let result = jpcg_core::calculate::start_combo(
        &skilltypes,
        &player_core,
        &hostile_core,
        &xinfa_core,
        &buff_core,
        &coeff_core,
    );

    Ok(ComboResultDTO {
        total_expected_damage_wan: result.total_expected_damage_wan,
        final_kill_prob: result.final_kill_prob,
        kill_prob_curve: result.kill_prob_curve,
        steps: result.steps.into_iter().map(|s| ComboStepResultDTO {
            skill_name: s.skill_name,
            g_damage: s.g_damage,
            h_damage: s.h_damage,
            q_damage: s.q_damage,
            crit_rate: s.crit_rate,
            cumulative_mean_wan: s.cumulative_mean / 10000.0,
            kill_prob: s.kill_prob,
        }).collect(),
    })
}

#[tauri::command]
pub fn save_combo_preset(name: String, steps: Vec<ComboStepDTO>) -> Result<(), String> {
    let core_steps = steps.into_iter().map(|s| {
        jpcg_core::type_set::combo::ComboStep {
            skill_id: s.skill.skill_id,
            skill_name: s.skill.skill_name,
            overrides: s.overrides.map(|o| jpcg_core::type_set::combo::StepOverride {
                base_damage_override: o.base_damage_override,
                atk_xishu_override: o.atk_xishu_override,
                jianshang_bili_override: o.jianshang_bili_override,
                wushihuajin_override: o.wushihuajin_override,
                extra_atk_pct: o.extra_atk_pct,
                gain_override: o.gain_override,
                extra_crit_pct: o.extra_crit_pct,
                extra_crit_dmg_pct: o.extra_crit_dmg_pct,
            }),
        }
    }).collect();
    let preset = jpcg_core::type_set::combo::ComboPreset {
        name,
        steps: core_steps,
    };
    jpcg_core::combo_io::save_preset(&preset)
}

#[tauri::command]
pub fn list_combo_presets() -> Vec<String> {
    jpcg_core::combo_io::list_presets()
}

#[tauri::command]
pub fn load_combo_preset(name: String) -> Result<ComboPresetDTO, String> {
    let preset = jpcg_core::combo_io::load_preset(&name)
        .ok_or_else(|| "预设不存在".to_string())?;
    let steps = preset.steps.into_iter().map(|s| {
        let mut skill = SkillPoolItemDTO {
            skill_name: s.skill_name.clone(),
            skill_id: s.skill_id,
            ..Default::default()
        };
        if let Some(ref o) = s.overrides {
            if let Some(v) = o.base_damage_override {
                skill.base_damage1 = v as u32;
                skill.base_damage2 = v as u32;
            }
            if let Some(v) = o.atk_xishu_override {
                skill.atk_xishu = v;
            }
        }
        ComboStepDTO {
            skill,
            overrides: s.overrides.map(|o| StepOverrideDTO {
                base_damage_override: o.base_damage_override,
                atk_xishu_override: o.atk_xishu_override,
                jianshang_bili_override: o.jianshang_bili_override,
                wushihuajin_override: o.wushihuajin_override,
                extra_atk_pct: o.extra_atk_pct,
                gain_override: o.gain_override,
                extra_crit_pct: o.extra_crit_pct,
                extra_crit_dmg_pct: o.extra_crit_dmg_pct,
            }),
        }
    }).collect();
    Ok(ComboPresetDTO { name, steps })
}

#[tauri::command]
pub fn delete_combo_preset(name: String) -> Result<(), String> {
    jpcg_core::combo_io::delete_preset(&name)
}

#[tauri::command]
pub fn export_config_cmd() -> Result<String, String> {
    jpcg_core::config_io::export_config()
}

#[tauri::command]
pub fn import_config_cmd(toml_str: String) -> Result<(), String> {
    jpcg_core::config_io::import_config(&toml_str)
}
