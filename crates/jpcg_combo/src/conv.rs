// ============================================================================
// conv — 连招结果 DTO 转换
//
// 技能 DTO ↔ core 的转换统一在 `jpcg_core::host::conv`（单一来源）；
// `ComboStep` / `ComboPreset` 的 From 亦在彼处（孤儿规则）。
// 此处仅存本 crate 自身结果的转换。
// ============================================================================

use jpcg_api::{ComboResultDTO, ComboStepResultDTO};

use crate::engine::{ComboResult, ComboStepResult};

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
