// ============================================================================
// conv — 连招 DTO ↔ core 领域类型转换
//
// 注意：ComboStep/ComboPreset 的 From 实现留在 jpcg_core::host::conv
// （孤儿规则：接收方类型须与 impl 同 crate）。此处仅存本 crate 自身结果的转换。
// ============================================================================

use jpcg_api::{ComboResultDTO, ComboStepResultDTO};
use jpcg_core::type_set::skilltype::Skilltype;

use crate::engine::{ComboResult, ComboStepResult};

/// 技能池条目 → core 领域技能（连招计算输入；属性缺失用 Default 兜底）
pub fn skill_dto_to_skilltype(s: &jpcg_api::SkillPoolItemDTO) -> Skilltype {
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
        dot_interval: s.dot_interval,
        dot_duration: s.dot_duration,
        dot_up: s.dot_up,
        wushijianshang: s.wushijianshang,
        zhenshishanghai: s.zhenshishanghai,
        has_critical_strike: s.has_critical_strike,
        lost_hp_zhenshishanghai: s.lost_hp_zhenshishanghai,
        ..Default::default()
    }
}

impl From<ComboStepResult> for ComboStepResultDTO {
    fn from(s: ComboStepResult) -> Self {
        ComboStepResultDTO {
            skill_name: s.skill_name,
            g_damage: s.g_damage,
            h_damage: s.h_damage,
            q_damage: s.q_damage,
            crit_rate: s.crit_rate,
            cumulative_mean_wan: s.cumulative_mean / 10000.0,
            kill_prob: s.kill_prob,
            dot_jumps: s.dot_jumps,
            has_critical_strike: s.has_critical_strike,
            zhenshishanghai: s.zhenshishanghai,
            lost_hp_zhenshi_damage: s.lost_hp_zhenshi_damage,
        }
    }
}

impl From<ComboResult> for ComboResultDTO {
    fn from(r: ComboResult) -> Self {
        ComboResultDTO {
            total_expected_damage_wan: r.total_expected_damage_wan,
            final_kill_prob: r.final_kill_prob,
            kill_prob_curve: r.kill_prob_curve,
            steps: r.steps.into_iter().map(ComboStepResultDTO::from).collect(),
        }
    }
}
