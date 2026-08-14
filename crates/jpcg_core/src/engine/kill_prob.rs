use crate::engine::atkcal::JpcgConfig;
use crate::type_set::buff::BuffConfig;
use crate::type_set::coefficient::CoefficientConfig;
use crate::type_set::hostilepile::HostilepileConfig;
use crate::type_set::player::PlayerConfig;
use crate::type_set::skilltype::Skilltype;
use crate::type_set::xinfa::XinfaConfig;

#[derive(Debug, Clone)]
pub struct ComboStepResult {
    pub skill_name: String,
    pub g_damage: u32,
    pub h_damage: u32,
    pub q_damage: u32,
    pub crit_rate: f32,
    pub cumulative_mean: f64,
    pub cumulative_std: f64,
    pub kill_prob: f64,
    pub dot_jumps: Vec<u32>,
    /// 无质（伤害固定 = 期望 Q，含会心加权）
    pub has_critical_strike: bool,
    /// 真实伤害（数据源 custom_damage_base 标签，无视防御减免）
    pub zhenshishanghai: u32,
    /// 追加真伤（已损失生命值 × 系数，无视防御，确定性只加期望）
    pub lost_hp_zhenshi_damage: f64,
}

pub struct ComboResult {
    pub steps: Vec<ComboStepResult>,
    pub total_expected_damage: f64,
    pub total_expected_damage_wan: f64,
    pub final_kill_prob: f64,
    pub kill_prob_curve: Vec<(usize, f64)>,
}

pub fn calculate_combo(
    skills: &[Skilltype],
    player: &PlayerConfig,
    hostilepile: &HostilepileConfig,
    xinfa: &XinfaConfig,
    buff: &BuffConfig,
    coeff: &CoefficientConfig,
) -> ComboResult {
    let mut steps = Vec::new();
    let mut cum_mean = 0.0f64;
    let mut cum_var = 0.0f64;
    let target_hp = hostilepile.target_hp as f64;

    for skill in skills {
        let calc = JpcgConfig::new_with_config(player, hostilepile, skill, xinfa, buff, coeff);
        let damage = calc.q_cal();
        let crit_rate =
            calc.guo_huixin() + skill.huixin_up as f32 / 100.0 + buff.huixin_pct / 100.0;

        let g = damage.g_damage as f64;
        let h = damage.h_damage as f64;
        let q = damage.q_damage as f64;
        let p = crit_rate as f64;
        let mean = q;
        // 无质技能伤害固定（恒为期望 Q，含会心加权），无随机浮动 → 方差为 0
        let variance = if skill.has_critical_strike {
            0.0
        } else {
            (h - g).powi(2) * p * (1.0 - p)
        };

        // 追加真伤（已损失生命值 × 系数，无视防御，确定性只加期望不加方差）：
        // 按本步结算后目标的期望已损失血量一次性结算，不迭代
        let lost_hp_zhenshi_damage = if skill.lost_hp_zhenshishanghai > 0.0 && target_hp > 0.0 {
            let lost = (target_hp - (cum_mean + mean)).max(0.0);
            lost * skill.lost_hp_zhenshishanghai as f64
        } else {
            0.0
        };

        cum_mean += mean + lost_hp_zhenshi_damage;
        cum_var += variance;
        let cum_std = cum_var.sqrt();

        let kill_prob = if target_hp <= 0.0 {
            1.0
        } else if cum_std > 0.0 {
            let z = (cum_mean - target_hp) / cum_std;
            1.0 - normal_cdf(-z)
        } else if cum_mean >= target_hp {
            // 确定性伤害（会心率 0 / 全无质连招）：期望 ≥ 血量即必杀
            1.0
        } else {
            0.0
        };

        steps.push(ComboStepResult {
            skill_name: skill.skill_name.clone(),
            g_damage: damage.g_damage,
            h_damage: damage.h_damage,
            q_damage: damage.q_damage,
            crit_rate,
            cumulative_mean: cum_mean,
            cumulative_std: cum_std,
            kill_prob,
            dot_jumps: damage.dot_jumps,
            has_critical_strike: skill.has_critical_strike,
            zhenshishanghai: skill.zhenshishanghai,
            lost_hp_zhenshi_damage,
        });
    }

    let final_kill_prob = steps.last().map(|s| s.kill_prob).unwrap_or(0.0);
    let total_expected_damage = cum_mean;
    let total_expected_damage_wan = cum_mean / 10000.0;
    let kill_prob_curve: Vec<(usize, f64)> = steps
        .iter()
        .enumerate()
        .map(|(i, s)| (i + 1, (s.kill_prob * 100.0 * 100.0).round() / 100.0))
        .collect();

    ComboResult {
        steps,
        total_expected_damage,
        total_expected_damage_wan,
        final_kill_prob,
        kill_prob_curve,
    }
}

fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf_approx(x / std::f64::consts::SQRT_2))
}

fn erf_approx(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t + a3) * t + a2) * t + a1) * t) * (-x * x).exp();
    sign * y
}
