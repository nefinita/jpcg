use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct XinfaSummaryDTO {
    pub value: String,
    pub label: String,
    pub nom: String,
    pub version_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PlayerConfigDTO {
    pub jcsx: String,
    pub jichu_shuxing: u32,
    pub jichu_gongji: u32,
    pub huixin_dengji: u32,
    pub huixin_xiaoguo: u32,
    pub pofang_dengji: u32,
    pub wuqi_shanghai: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HostileConfigDTO {
    pub waigong_fangyu: u32,
    pub neigong_fangyu: u32,
    pub yujin_dengji: u32,
    pub huajin_dengji: u32,
    pub jianshang_bili: u32,
    pub target_hp: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BuffConfigDTO {
    pub base_atk_pct: f32,
    pub huixin_pct: f32,
    pub huixiao_pct: f32,
    pub pofang_pct: f32,
    pub wushi_fangyu_pct: f32,
    pub shanghai_pct: f32,
    pub mode_is_point: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CoefficientConfigDTO {
    pub pofang_xishu: f32,
    pub huixin_xishu: f32,
    pub huixiao_xishu: f32,
    pub huajin_xishu: f32,
    pub fangyu_xishu: f32,
    pub pvp_global_jianshang: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct XinfaConfigDTO {
    pub profession: String,
    pub xinfa_name: String,
    pub xinfa_nom: String,
    pub atk_up: f32,
    pub pofang_up: f32,
    pub huixin_up: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculateRequest {
    pub player: PlayerConfigDTO,
    pub hostile: HostileConfigDTO,
    pub xinfa_config: XinfaConfigDTO,
    pub buff: BuffConfigDTO,
    pub coefficient: CoefficientConfigDTO,
}

#[derive(Debug, Serialize)]
pub struct SkillResultDTO {
    pub skill_name: String,
    pub y: u32,
    pub b: u32,
    pub i: u32,
    pub n: u32,
    pub h: u32,
    pub q: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SkillPoolItemDTO {
    pub skill_name: String,
    pub skill_id: u32,
    pub base_damage1: u32,
    pub base_damage2: u32,
    pub atk_xishu: f32,
    pub watk_xishu: u32,
    pub hit_up: u32,
    pub huixin_up: u32,
    pub huixiao_up: u32,
    pub wushifangyu: u32,
    pub wushihuajin: u32,
    pub dot_flag: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct StepOverrideDTO {
    pub base_damage_override: Option<f32>,
    pub atk_xishu_override: Option<f32>,
    pub jianshang_bili_override: Option<f32>,
    pub wushihuajin_override: Option<f32>,
    pub extra_atk_pct: Option<f32>,
    pub gain_override: Option<f32>,
    pub extra_crit_pct: Option<f32>,
    pub extra_crit_dmg_pct: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ComboStepDTO {
    pub skill: SkillPoolItemDTO,
    pub overrides: Option<StepOverrideDTO>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ComboPresetDTO {
    pub name: String,
    pub steps: Vec<ComboStepDTO>,
}

#[derive(Debug, Serialize)]
pub struct ComboStepResultDTO {
    pub skill_name: String,
    pub g_damage: u32,
    pub h_damage: u32,
    pub q_damage: u32,
    pub crit_rate: f32,
    pub cumulative_mean_wan: f64,
    pub kill_prob: f64,
}

#[derive(Debug, Serialize)]
pub struct ComboResultDTO {
    pub steps: Vec<ComboStepResultDTO>,
    pub total_expected_damage_wan: f64,
    pub final_kill_prob: f64,
    pub kill_prob_curve: Vec<(usize, f64)>,
}

impl PlayerConfigDTO {
    pub fn into_core(self) -> jpcg_core::type_set::player::PlayerConfig {
        jpcg_core::type_set::player::PlayerConfig::new(
            self.jcsx,
            self.jichu_shuxing,
            self.jichu_gongji,
            self.huixin_dengji,
            self.huixin_xiaoguo,
            self.pofang_dengji,
            self.wuqi_shanghai,
        )
    }
}

impl HostileConfigDTO {
    pub fn into_core(self) -> jpcg_core::type_set::hostilepile::HostilepileConfig {
        jpcg_core::type_set::hostilepile::HostilepileConfig {
            waigong_fangyu: self.waigong_fangyu,
            neigong_fangyu: self.neigong_fangyu,
            yujin_dengji: self.yujin_dengji,
            huajin_dengji: self.huajin_dengji,
            jianshang_bili: self.jianshang_bili,
            target_hp: self.target_hp,
        }
    }
}

impl XinfaConfigDTO {
    pub fn into_core(self) -> jpcg_core::type_set::xinfa::XinfaConfig {
        jpcg_core::type_set::xinfa::XinfaConfig::new(
            self.profession,
            self.xinfa_name,
            self.xinfa_nom,
            self.atk_up,
            self.pofang_up,
            self.huixin_up,
        )
    }
}

impl From<jpcg_core::cal::CalculateResult> for SkillResultDTO {
    fn from(core: jpcg_core::cal::CalculateResult) -> Self {
        Self {
            skill_name: core.skill_name,
            b: core.b,
            i: core.i,
            n: core.n,
            h: core.h,
            q: core.q,
            y: core.y,
        }
    }
}

impl From<ComboStepDTO> for jpcg_core::type_set::combo::ComboStep {
    fn from(dto: ComboStepDTO) -> Self {
        jpcg_core::type_set::combo::ComboStep {
            skill_id: dto.skill.skill_id,
            skill_name: dto.skill.skill_name,
            overrides: dto.overrides.map(|o| jpcg_core::type_set::combo::StepOverride {
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
    }
}

impl From<jpcg_core::type_set::combo::ComboPreset> for ComboPresetDTO {
    fn from(core: jpcg_core::type_set::combo::ComboPreset) -> Self {
        Self {
            name: core.name,
            steps: core.steps.into_iter().map(|s| {
                let mut skill = SkillPoolItemDTO {
                    skill_name: s.skill_name,
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
            }).collect(),
        }
    }
}
