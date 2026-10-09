// ============================================================================
// host::conv — DTO ↔ core 类型转换
// 原 Tauri 壳 commands/types.rs 中的 From 实现迁入 core，
// 使 JSON 契约类型（jpcg_api）与 core 领域类型双向转换有单一来源。
// ============================================================================

use jpcg_api::{
    BuffConfigDTO, CoefficientConfigDTO, ComboPresetDTO, ComboStepDTO, CritVsPofangDTO,
    DerivativeEntryDTO, DerivativesOutputDTO, HostileConfigDTO, OptimizeRecommendationDTO,
    PlayerConfigDTO, SkillDerivativeDTO, SkillEditorDataDTO, SkillEditorItemDTO, SkillPoolItemDTO,
    SkillResultDTO, TopAttrDTO, VersionInfoDTO, XinfaConfigDTO, XinfaSummaryDTO,
};

use crate::engine;
use crate::store::TomlConfig;
use crate::type_set::buff::BuffConfig;
use crate::type_set::coefficient::CoefficientConfig;
use crate::type_set::combo::{ComboPreset, ComboStep, StepOverride};
use crate::type_set::hostilepile::HostilepileConfig;
use crate::type_set::player::PlayerConfig;
use crate::type_set::skilltype::Skilltype;
use crate::type_set::xinfa::{VersionInfo, XinfaConfig, XinfaSummary};

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
            skill_snapshot: Some(skill_pool_item_to_skilltype(&dto.skill)),
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
pub fn skilltype_to_pool_item(s: &Skilltype) -> SkillPoolItemDTO {
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

// ============ 配置 DTO ↔ core（单一来源：单技能/求导/连招/保存/加载共用） ============

/// `PlayerConfigDTO` → `PlayerConfig`
pub fn player_from_dto(dto: PlayerConfigDTO) -> PlayerConfig {
    PlayerConfig::new(
        dto.jcsx,
        dto.jichu_shuxing,
        dto.jichu_gongji,
        dto.huixin_dengji,
        dto.huixin_xiaoguo,
        dto.pofang_dengji,
        dto.wuqi_shanghai,
    )
}

/// `HostileConfigDTO` → `HostilepileConfig`
pub fn hostile_from_dto(dto: HostileConfigDTO) -> HostilepileConfig {
    HostilepileConfig {
        waigong_fangyu: dto.waigong_fangyu,
        neigong_fangyu: dto.neigong_fangyu,
        yujin_dengji: dto.yujin_dengji,
        huajin_dengji: dto.huajin_dengji,
        jianshang_bili: dto.jianshang_bili,
        target_hp: dto.target_hp,
        max_hp: dto.max_hp,
        current_hp: dto.current_hp,
    }
}

/// `XinfaConfigDTO` → `XinfaConfig`
pub fn xinfa_from_dto(dto: XinfaConfigDTO) -> XinfaConfig {
    XinfaConfig::new(
        dto.profession,
        dto.xinfa_name,
        dto.xinfa_nom,
        dto.atk_up,
        dto.pofang_up,
        dto.huixin_up,
    )
}

/// `BuffConfigDTO` → `BuffConfig`
pub fn buff_from_dto(dto: BuffConfigDTO) -> BuffConfig {
    BuffConfig {
        base_atk_pct: dto.base_atk_pct,
        huixin_pct: dto.huixin_pct,
        huixiao_pct: dto.huixiao_pct,
        pofang_pct: dto.pofang_pct,
        wushi_fangyu_pct: dto.wushi_fangyu_pct,
        shanghai_pct: dto.shanghai_pct,
        mode_is_point: dto.mode_is_point,
    }
}

/// `CoefficientConfigDTO` → `CoefficientConfig`（0/缺失分母回退真源默认）
pub fn coefficient_from_dto(dto: &CoefficientConfigDTO) -> CoefficientConfig {
    CoefficientConfig::from(dto)
}

/// 计算请求五件套 → core 领域类型（单技能/求导/连招共用）
pub fn into_core(
    player: PlayerConfigDTO,
    hostile: HostileConfigDTO,
    xinfa: XinfaConfigDTO,
    buff: BuffConfigDTO,
    coefficient: CoefficientConfigDTO,
) -> (
    PlayerConfig,
    HostilepileConfig,
    XinfaConfig,
    BuffConfig,
    CoefficientConfig,
) {
    (
        player_from_dto(player),
        hostile_from_dto(hostile),
        xinfa_from_dto(xinfa),
        buff_from_dto(buff),
        coefficient_from_dto(&coefficient),
    )
}

/// 技能池条目 → core 领域技能（**完整**字段：含 dot / 无视防御 / 真伤）
pub fn skill_pool_item_to_skilltype(s: &SkillPoolItemDTO) -> Skilltype {
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
        wushijianshang: s.wushijianshang,
        zhenshishanghai: s.zhenshishanghai,
        has_critical_strike: s.has_critical_strike,
        lost_hp_zhenshishanghai: s.lost_hp_zhenshishanghai,
        dot_flag: s.dot_flag,
        dot_interval: s.dot_interval,
        dot_duration: s.dot_duration,
        dot_up: s.dot_up,
        ..Default::default()
    }
}

/// `PlayerConfig` → `PlayerConfigDTO`（`zuizhong_gongji` 不属 DTO，外部计算填入）
pub fn player_to_dto(c: PlayerConfig) -> PlayerConfigDTO {
    PlayerConfigDTO {
        jcsx: c.jcsx,
        jichu_shuxing: c.jichu_shuxing,
        jichu_gongji: c.jichu_gongji,
        huixin_dengji: c.huixin_dengji,
        huixin_xiaoguo: c.huixin_xiaoguo,
        pofang_dengji: c.pofang_dengji,
        wuqi_shanghai: c.wuqi_shanghai,
    }
}

/// `HostilepileConfig` → `HostileConfigDTO`
pub fn hostile_to_dto(c: HostilepileConfig) -> HostileConfigDTO {
    HostileConfigDTO {
        waigong_fangyu: c.waigong_fangyu,
        neigong_fangyu: c.neigong_fangyu,
        yujin_dengji: c.yujin_dengji,
        huajin_dengji: c.huajin_dengji,
        jianshang_bili: c.jianshang_bili,
        target_hp: c.target_hp,
        max_hp: c.max_hp,
        current_hp: c.current_hp,
    }
}

/// `XinfaConfig` → `XinfaConfigDTO`
pub fn xinfa_to_dto(c: XinfaConfig) -> XinfaConfigDTO {
    XinfaConfigDTO {
        profession: c.profession,
        xinfa_name: c.xinfa_name,
        xinfa_nom: c.xinfa_nom,
        atk_up: c.atk_up,
        pofang_up: c.pofang_up,
        huixin_up: c.huixin_up,
    }
}

/// `BuffConfig` → `BuffConfigDTO`
pub fn buff_to_dto(c: BuffConfig) -> BuffConfigDTO {
    BuffConfigDTO {
        base_atk_pct: c.base_atk_pct,
        huixin_pct: c.huixin_pct,
        huixiao_pct: c.huixiao_pct,
        pofang_pct: c.pofang_pct,
        wushi_fangyu_pct: c.wushi_fangyu_pct,
        shanghai_pct: c.shanghai_pct,
        mode_is_point: c.mode_is_point,
    }
}

/// `CoefficientConfig` → `CoefficientConfigDTO`
pub fn coefficient_to_dto(c: CoefficientConfig) -> CoefficientConfigDTO {
    CoefficientConfigDTO {
        pofang_xishu: c.pofang_xishu,
        huixin_xishu: c.huixin_xishu,
        huixiao_xishu: c.huixiao_xishu,
        yujin_xishu: c.yujin_xishu,
        yuhui_xishu: c.yuhui_xishu,
        huajin_xishu: c.huajin_xishu,
        fangyu_xishu: c.fangyu_xishu,
        pvp_global_jianshang: c.pvp_global_jianshang,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 结构等价（DTO 未派生 PartialEq，用 JSON 值比较）
    fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
        serde_json::to_value(a).ok() == serde_json::to_value(b).ok()
    }

    #[test]
    fn config_dto_roundtrip_preserves_fields() {
        let player = PlayerConfigDTO {
            jcsx: "gengu".into(),
            jichu_shuxing: 18888,
            jichu_gongji: 4666,
            huixin_dengji: 33000,
            huixin_xiaoguo: 22000,
            pofang_dengji: 25000,
            wuqi_shanghai: 2800,
        };
        assert!(same(
            &player_to_dto(player_from_dto(player.clone())),
            &player
        ));

        let hostile = HostileConfigDTO {
            waigong_fangyu: 21000,
            neigong_fangyu: 21000,
            yujin_dengji: 8500,
            huajin_dengji: 35000,
            jianshang_bili: 35,
            target_hp: 2_000_000,
            max_hp: 2_000_000,
            current_hp: 1_500_000,
        };
        assert!(same(
            &hostile_to_dto(hostile_from_dto(hostile.clone())),
            &hostile
        ));

        let xinfa = XinfaConfigDTO {
            profession: "mowen".into(),
            xinfa_name: "莫问".into(),
            xinfa_nom: "gengu".into(),
            atk_up: 1.96,
            pofang_up: 2.0,
            huixin_up: 0.0,
        };
        assert!(same(&xinfa_to_dto(xinfa_from_dto(xinfa.clone())), &xinfa));

        let buff = BuffConfigDTO {
            base_atk_pct: 10.0,
            huixin_pct: 5.0,
            huixiao_pct: 8.0,
            pofang_pct: 3.0,
            wushi_fangyu_pct: 1.0,
            shanghai_pct: 6.0,
            mode_is_point: true,
        };
        assert!(same(&buff_to_dto(buff_from_dto(buff.clone())), &buff));

        let coeff = CoefficientConfigDTO {
            pofang_xishu: 225957.6,
            huixin_xishu: 197703.0,
            huixiao_xishu: 72844.2,
            yujin_xishu: 197703.0,
            yuhui_xishu: 55123.2,
            huajin_xishu: 30115.8,
            fangyu_xishu: 126007.2,
            pvp_global_jianshang: 0.9,
        };
        assert!(same(
            &coefficient_to_dto(coefficient_from_dto(&coeff)),
            &coeff
        ));
    }

    /// 技能池条目往返：dot / 无视防御 / 真伤字段不丢失（回归 #2）
    #[test]
    fn skill_pool_item_conversion_preserves_dot_and_true_damage() {
        let item = SkillPoolItemDTO {
            skill_name: "宫".into(),
            skill_id: 1,
            sub_id: 2,
            base_damage1: 35,
            base_damage2: 40,
            atk_xishu: 5.0,
            watk_xishu: 1,
            hit_up: 2,
            huixin_up: 3,
            huixiao_up: 4,
            wushifangyu: 512,
            wushihuajin: 256,
            wushijianshang: 7,
            zhenshishanghai: 9,
            lost_hp_zhenshishanghai: 0.18,
            dot_flag: 1,
            dot_interval: 2.0,
            dot_duration: 6.0,
            dot_up: 8.0,
            has_critical_strike: true,
        };
        let st = skill_pool_item_to_skilltype(&item);
        assert_eq!(st.dot_flag, 1);
        assert_eq!(st.dot_interval, 2.0);
        assert_eq!(st.dot_duration, 6.0);
        assert_eq!(st.dot_up, 8.0);
        assert_eq!(st.wushijianshang, 7);
        assert_eq!(st.zhenshishanghai, 9);
        assert_eq!(st.lost_hp_zhenshishanghai, 0.18);
        assert!(st.has_critical_strike);
        // 反向转换亦完整
        assert!(same(&skilltype_to_pool_item(&st), &item));
    }
}
