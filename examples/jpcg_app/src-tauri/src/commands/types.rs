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

// ============ 自动求导 DTO ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDerivativeDTO {
    pub skill_name: String,
    pub derivative: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivativeEntryDTO {
    pub attr_name: String,
    pub attr_id: String,
    pub current_value: f32,
    pub total_derivative: f32,
    pub per_skill: Vec<SkillDerivativeDTO>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritVsPofangDTO {
    pub better: String,
    pub huixin_total: f32,
    pub pofang_total: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAttrDTO {
    pub attr_name: String,
    pub attr_id: String,
    pub total_derivative: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeRecommendationDTO {
    pub crit_vs_pofang: CritVsPofangDTO,
    pub top3: Vec<TopAttrDTO>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivativesOutputDTO {
    pub derivatives: Vec<DerivativeEntryDTO>,
    pub recommendation: OptimizeRecommendationDTO,
}

impl From<jpcg_core::cal::derivatives::SkillDerivative> for SkillDerivativeDTO {
    fn from(d: jpcg_core::cal::derivatives::SkillDerivative) -> Self {
        SkillDerivativeDTO {
            skill_name: d.skill_name,
            derivative: d.derivative,
        }
    }
}

impl From<jpcg_core::cal::derivatives::DerivativeEntry> for DerivativeEntryDTO {
    fn from(d: jpcg_core::cal::derivatives::DerivativeEntry) -> Self {
        DerivativeEntryDTO {
            attr_name: d.attr_name,
            attr_id: d.attr_id,
            current_value: d.current_value,
            total_derivative: d.total_derivative,
            per_skill: d.per_skill.into_iter().map(SkillDerivativeDTO::from).collect(),
        }
    }
}

impl From<jpcg_core::cal::derivatives::CritVsPofang> for CritVsPofangDTO {
    fn from(c: jpcg_core::cal::derivatives::CritVsPofang) -> Self {
        CritVsPofangDTO {
            better: c.better,
            huixin_total: c.huixin_total,
            pofang_total: c.pofang_total,
        }
    }
}

impl From<jpcg_core::cal::derivatives::TopAttr> for TopAttrDTO {
    fn from(t: jpcg_core::cal::derivatives::TopAttr) -> Self {
        TopAttrDTO {
            attr_name: t.attr_name,
            attr_id: t.attr_id,
            total_derivative: t.total_derivative,
        }
    }
}

impl From<jpcg_core::cal::derivatives::OptimizeRecommendation> for OptimizeRecommendationDTO {
    fn from(r: jpcg_core::cal::derivatives::OptimizeRecommendation) -> Self {
        OptimizeRecommendationDTO {
            crit_vs_pofang: CritVsPofangDTO::from(r.crit_vs_pofang),
            top3: r.top3.into_iter().map(TopAttrDTO::from).collect(),
        }
    }
}

impl From<jpcg_core::cal::derivatives::DerivativesOutput> for DerivativesOutputDTO {
    fn from(o: jpcg_core::cal::derivatives::DerivativesOutput) -> Self {
        DerivativesOutputDTO {
            derivatives: o.derivatives.into_iter().map(DerivativeEntryDTO::from).collect(),
            recommendation: OptimizeRecommendationDTO::from(o.recommendation),
        }
    }
}

// ============ 技能编辑器 DTO ============

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SkillEditorItemDTO {
    pub skill_name: String,
    pub skill_id: u32,
    pub sub_id: u32,
    pub group: u8,
    pub weapon_request: u8,
    pub design_effect: u8,
    pub kind_type: u8,
    pub cast_mode: u8,
    pub guaranteed_hit: bool,
    pub has_critical_strike: bool,
    pub effect_type: u8,
    pub jihuoqixue: String,
    pub base_damage1: u32,
    pub base_damage2: u32,
    pub atk_xishu: f32,
    pub watk_xishu: u32,
    pub hit_up: u32,
    pub huixin_up: u32,
    pub huixiao_up: u32,
    pub wushifangyu: u32,
    pub wushihuajin: u32,
    pub wushijianshang: u32,
    pub zhenshishanghai: u32,
    pub dot_flag: u8,
    pub dot_num: u8,
    pub dot_up: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct VersionInfoDTO {
    pub level: u32,
    pub season: u32,
    pub modified: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SkillEditorDataDTO {
    pub xinfa: XinfaConfigDTO,
    pub version: Option<VersionInfoDTO>,
    pub skills: Vec<SkillEditorItemDTO>,
}

impl From<SkillEditorItemDTO> for jpcg_core::type_set::skilltype::Skilltype {
    fn from(dto: SkillEditorItemDTO) -> Self {
        jpcg_core::type_set::skilltype::Skilltype {
            skill_name: dto.skill_name,
            skill_id: dto.skill_id,
            sub_id: dto.sub_id,
            group: dto.group,
            weapon_request: dto.weapon_request,
            design_effect: dto.design_effect,
            kind_type: dto.kind_type,
            cast_mode: dto.cast_mode,
            guaranteed_hit: dto.guaranteed_hit,
            has_critical_strike: dto.has_critical_strike,
            effect_type: dto.effect_type,
            jihuoqixue: dto.jihuoqixue,
            base_damage1: dto.base_damage1,
            base_damage2: dto.base_damage2,
            atk_xishu: dto.atk_xishu,
            watk_xishu: dto.watk_xishu,
            hit_up: dto.hit_up,
            huixin_up: dto.huixin_up,
            huixiao_up: dto.huixiao_up,
            wushifangyu: dto.wushifangyu,
            wushihuajin: dto.wushihuajin,
            wushijianshang: dto.wushijianshang,
            zhenshishanghai: dto.zhenshishanghai,
            dot_flag: dto.dot_flag,
            dot_num: dto.dot_num,
            dot_up: dto.dot_up,
        }
    }
}

impl From<jpcg_core::type_set::skilltype::Skilltype> for SkillEditorItemDTO {
    fn from(core: jpcg_core::type_set::skilltype::Skilltype) -> Self {
        SkillEditorItemDTO {
            skill_name: core.skill_name,
            skill_id: core.skill_id,
            sub_id: core.sub_id,
            group: core.group,
            weapon_request: core.weapon_request,
            design_effect: core.design_effect,
            kind_type: core.kind_type,
            cast_mode: core.cast_mode,
            guaranteed_hit: core.guaranteed_hit,
            has_critical_strike: core.has_critical_strike,
            effect_type: core.effect_type,
            jihuoqixue: core.jihuoqixue,
            base_damage1: core.base_damage1,
            base_damage2: core.base_damage2,
            atk_xishu: core.atk_xishu,
            watk_xishu: core.watk_xishu,
            hit_up: core.hit_up,
            huixin_up: core.huixin_up,
            huixiao_up: core.huixiao_up,
            wushifangyu: core.wushifangyu,
            wushihuajin: core.wushihuajin,
            wushijianshang: core.wushijianshang,
            zhenshishanghai: core.zhenshishanghai,
            dot_flag: core.dot_flag,
            dot_num: core.dot_num,
            dot_up: core.dot_up,
        }
    }
}

impl From<VersionInfoDTO> for jpcg_core::type_set::xinfa::VersionInfo {
    fn from(dto: VersionInfoDTO) -> Self {
        jpcg_core::type_set::xinfa::VersionInfo {
            level: dto.level,
            season: dto.season,
            modified: dto.modified,
        }
    }
}

impl From<jpcg_core::type_set::xinfa::VersionInfo> for VersionInfoDTO {
    fn from(core: jpcg_core::type_set::xinfa::VersionInfo) -> Self {
        VersionInfoDTO {
            level: core.level,
            season: core.season,
            modified: core.modified,
        }
    }
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
