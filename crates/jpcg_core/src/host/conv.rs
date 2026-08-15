// ============================================================================
// host::conv — DTO ↔ core 类型转换
// 原 Tauri 壳 commands/types.rs 中的 From 实现迁入 core，
// 使 JSON 契约类型（jpcg_api）与 core 领域类型双向转换有单一来源。
// ============================================================================

use jpcg_api::{
    ComboPresetDTO, ComboStepDTO, CritVsPofangDTO, DerivativeEntryDTO, DerivativesOutputDTO,
    OptimizeRecommendationDTO, SkillDerivativeDTO, SkillEditorDataDTO, SkillEditorItemDTO,
    SkillPoolItemDTO, SkillResultDTO, TopAttrDTO, VersionInfoDTO, XinfaSummaryDTO,
};

use crate::engine;
use crate::store::TomlConfig;
use crate::type_set::combo::{ComboPreset, ComboStep, StepOverride};
use crate::type_set::skilltype::Skilltype;
use crate::type_set::xinfa::{VersionInfo, XinfaSummary};

// ============ 求导结果 ============

impl From<engine::derivatives::SkillDerivative> for SkillDerivativeDTO {
    fn from(d: engine::derivatives::SkillDerivative) -> Self {
        SkillDerivativeDTO {
            skill_name: d.skill_name,
            derivative: d.derivative,
        }
    }
}

impl From<engine::derivatives::DerivativeEntry> for DerivativeEntryDTO {
    fn from(d: engine::derivatives::DerivativeEntry) -> Self {
        DerivativeEntryDTO {
            attr_name: d.attr_name,
            attr_id: d.attr_id,
            current_value: d.current_value,
            total_derivative: d.total_derivative,
            per_skill: d
                .per_skill
                .into_iter()
                .map(SkillDerivativeDTO::from)
                .collect(),
        }
    }
}

impl From<engine::derivatives::CritVsPofang> for CritVsPofangDTO {
    fn from(c: engine::derivatives::CritVsPofang) -> Self {
        CritVsPofangDTO {
            better: c.better,
            huixin_total: c.huixin_total,
            pofang_total: c.pofang_total,
        }
    }
}

impl From<engine::derivatives::TopAttr> for TopAttrDTO {
    fn from(t: engine::derivatives::TopAttr) -> Self {
        TopAttrDTO {
            attr_name: t.attr_name,
            attr_id: t.attr_id,
            total_derivative: t.total_derivative,
        }
    }
}

impl From<engine::derivatives::OptimizeRecommendation> for OptimizeRecommendationDTO {
    fn from(r: engine::derivatives::OptimizeRecommendation) -> Self {
        OptimizeRecommendationDTO {
            crit_vs_pofang: CritVsPofangDTO::from(r.crit_vs_pofang),
            top3: r.top3.into_iter().map(TopAttrDTO::from).collect(),
        }
    }
}

impl From<engine::derivatives::DerivativesOutput> for DerivativesOutputDTO {
    fn from(o: engine::derivatives::DerivativesOutput) -> Self {
        DerivativesOutputDTO {
            derivatives: o
                .derivatives
                .into_iter()
                .map(DerivativeEntryDTO::from)
                .collect(),
            recommendation: OptimizeRecommendationDTO::from(o.recommendation),
        }
    }
}

// ============ 技能编辑器 ============

impl From<SkillEditorItemDTO> for Skilltype {
    fn from(dto: SkillEditorItemDTO) -> Self {
        Skilltype {
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
            lost_hp_zhenshishanghai: dto.lost_hp_zhenshishanghai,
            dot_flag: dto.dot_flag,
            dot_interval: dto.dot_interval,
            dot_duration: dto.dot_duration,
            dot_up: dto.dot_up,
        }
    }
}

impl From<Skilltype> for SkillEditorItemDTO {
    fn from(core: Skilltype) -> Self {
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
            lost_hp_zhenshishanghai: core.lost_hp_zhenshishanghai,
            dot_flag: core.dot_flag,
            dot_interval: core.dot_interval,
            dot_duration: core.dot_duration,
            dot_up: core.dot_up,
        }
    }
}

impl From<VersionInfoDTO> for VersionInfo {
    fn from(dto: VersionInfoDTO) -> Self {
        VersionInfo {
            level: dto.level,
            season: dto.season,
            modified: dto.modified,
        }
    }
}

impl From<VersionInfo> for VersionInfoDTO {
    fn from(core: VersionInfo) -> Self {
        VersionInfoDTO {
            level: core.level,
            season: core.season,
            modified: core.modified,
        }
    }
}

// ============ 技能编辑器数据（TomlConfig 组装） ============

/// TomlConfig → SkillEditorDataDTO
pub fn toml_to_editor_data(toml_cfg: &TomlConfig) -> SkillEditorDataDTO {
    SkillEditorDataDTO {
        xinfa: jpcg_api::XinfaConfigDTO {
            profession: toml_cfg.xinfa.profession.clone(),
            xinfa_name: toml_cfg.xinfa.xinfa_name.clone(),
            xinfa_nom: toml_cfg.xinfa.xinfa_nom.clone(),
            atk_up: toml_cfg.xinfa.atk_up,
            pofang_up: toml_cfg.xinfa.pofang_up,
            huixin_up: toml_cfg.xinfa.huixin_up,
        },
        version: toml_cfg.version.clone().map(VersionInfoDTO::from),
        skills: toml_cfg
            .skill
            .iter()
            .cloned()
            .map(SkillEditorItemDTO::from)
            .collect(),
    }
}

// ============ 计算/连招结果 ============

impl From<engine::CalculateResult> for SkillResultDTO {
    fn from(core: engine::CalculateResult) -> Self {
        SkillResultDTO {
            skill_name: core.skill_name,
            y: core.y,
            b: core.b,
            i: core.i,
            n: core.n,
            h: core.h,
            q: core.q,
            dot_jumps: core.dot_jumps,
            has_critical_strike: core.has_critical_strike,
            zhenshishanghai: core.zhenshishanghai,
            lost_hp_zhenshishanghai: core.lost_hp_zhenshishanghai,
        }
    }
}

impl From<ComboStepDTO> for ComboStep {
    fn from(dto: ComboStepDTO) -> Self {
        ComboStep {
            skill_id: dto.skill.skill_id,
            sub_id: dto.skill.sub_id,
            skill_name: dto.skill.skill_name.clone(),
            skill_snapshot: Some(super::calc::skill_dto_to_skilltype(&dto.skill)),
            overrides: dto.overrides.map(|o| StepOverride {
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

/// Skilltype → 技能池条目（预设加载时用快照还原完整属性）
fn skilltype_to_pool_item(s: &Skilltype) -> SkillPoolItemDTO {
    SkillPoolItemDTO {
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
        dot_interval: s.dot_interval,
        dot_duration: s.dot_duration,
        dot_up: s.dot_up,
        wushijianshang: s.wushijianshang,
        zhenshishanghai: s.zhenshishanghai,
        has_critical_strike: s.has_critical_strike,
        lost_hp_zhenshishanghai: s.lost_hp_zhenshishanghai,
    }
}

impl From<ComboPreset> for ComboPresetDTO {
    fn from(core: ComboPreset) -> Self {
        ComboPresetDTO {
            name: core.name,
            steps: core
                .steps
                .into_iter()
                .map(|s| {
                    let mut skill = match &s.skill_snapshot {
                        Some(snap) => skilltype_to_pool_item(snap),
                        // 旧存档：仅有名称与 ID，属性缺失（保留原行为）
                        None => SkillPoolItemDTO {
                            skill_name: s.skill_name,
                            skill_id: s.skill_id,
                            sub_id: s.sub_id,
                            ..Default::default()
                        },
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
                        overrides: s.overrides.map(|o| jpcg_api::StepOverrideDTO {
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
                })
                .collect(),
        }
    }
}

// ============ 门派列表 ============

impl From<XinfaSummary> for XinfaSummaryDTO {
    fn from(core: XinfaSummary) -> Self {
        XinfaSummaryDTO {
            value: core.value,
            label: core.label,
            nom: core.nom,
            version_label: core.version_label,
        }
    }
}
