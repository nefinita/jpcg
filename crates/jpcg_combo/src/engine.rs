// ============================================================================
// engine — 连招伤害编排与击杀率计算
//
// 双通道模型：
//   期望通道 —— 每步 g/h/q/dot 期望、累计期望/方差（保持迁移前 kill_prob 的
//               DTO 语义：cumulative_mean_wan 为预期总伤害，追加真伤只加期望）。
//   蒙特卡洛 —— samples 条随机路径：会心按技能判定，dot 技能逐跳独立判定，
//               追加真伤按路径实时血量结算（已损失 × 系数），
//               击杀率 = 击杀路径占比；确定性无质连招等价于精确解。
//
// hp 语义（core 追加真伤公式的调用方）：
//   - hostile.max_hp > 0        → 以 max_hp 为总血量，current_hp（合法时）为初始血量
//   - 否则 target_hp > 0        → 满血模型（总血量 = 初始 = target_hp，迁移前语义）
//   - 都未提供（0）             → 无真伤、无血量推进（迁移前）：击杀率恒 1
// ============================================================================

use jpcg_core::engine::atkcal::JpcgConfig;
use jpcg_core::type_set::buff::BuffConfig;
use jpcg_core::type_set::coefficient::CoefficientConfig;
use jpcg_core::type_set::hostilepile::HostilepileConfig;
use jpcg_core::type_set::player::PlayerConfig;
use jpcg_core::type_set::skilltype::Skilltype;
use jpcg_core::type_set::xinfa::XinfaConfig;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::DEFAULT_SAMPLES;

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
    /// 追加真伤期望（已损失生命值 × 系数，无视防御；期望通道本步值）
    pub lost_hp_zhenshi_damage: f64,
}

pub struct ComboResult {
    pub steps: Vec<ComboStepResult>,
    pub total_expected_damage: f64,
    pub total_expected_damage_wan: f64,
    pub final_kill_prob: f64,
    pub kill_prob_curve: Vec<(usize, f64)>,
}

/// 蒙特卡洛通道配置（host 层默认 DEFAULT_SAMPLES + 随机种子）
#[derive(Debug, Clone)]
pub struct ComboConfig {
    pub samples: u32,
    /// 固定种子（确定性可复现测试用）；None = OS 熵源
    pub seed: Option<u64>,
}

impl Default for ComboConfig {
    fn default() -> Self {
        ComboConfig {
            samples: DEFAULT_SAMPLES,
            seed: None,
        }
    }
}

/// 单步采样模型（预构建，热路径避免重复逐技能配置）
struct StepModel {
    g_damage: u32,
    h_damage: u32,
    q_damage: u32,
    crit_rate: f32,
    /// 无质（确定性伤害）
    has_critical_strike: bool,
    /// 追加真伤系数
    lost_coeff: f64,
    /// dot 每跳 g/h（独立会心判定）
    dot_jumps_g: Vec<u32>,
    dot_jumps_h: Vec<u32>,
}

/// 血量模型（当前 / 总）
fn hp_preset(hostile: &HostilepileConfig) -> Option<(f64, f64)> {
    if hostile.max_hp > 0 {
        let max = hostile.max_hp as f64;
        let cur = if hostile.current_hp > 0 && hostile.current_hp as f64 <= max {
            hostile.current_hp as f64
        } else {
            max
        };
        Some((cur, max))
    } else if hostile.target_hp > 0 {
        let t = hostile.target_hp as f64;
        Some((t, t))
    } else {
        None
    }
}

pub fn calculate_combo(
    skills: &[Skilltype],
    player: &PlayerConfig,
    hostile: &HostilepileConfig,
    xinfa: &XinfaConfig,
    buff: &BuffConfig,
    coeff: &CoefficientConfig,
    config: &ComboConfig,
) -> ComboResult {
    // ---- 期望通道 ----
    let mut steps = Vec::with_capacity(skills.len());
    let mut cum_mean = 0.0f64;
    let mut cum_var = 0.0f64;
    let hp = hp_preset(hostile);
    // 期望通道血量推进（当前 / 总）
    let mut ec_cur = hp.map(|(c, _)| c);
    let ec_max = hp.map(|(_, m)| m);

    let mut models = Vec::with_capacity(skills.len());

    for skill in skills {
        let calc = JpcgConfig::new_with_config(player, hostile, skill, xinfa, buff, coeff);
        let damage = if let (Some(cur), Some(max)) = (ec_cur, ec_max) {
            calc.q_cal_with_hp(Some(cur as u32), Some(max as u32))
        } else {
            calc.q_cal()
        };
        // 会心率（q_cal 语义：buff.huixin_pct 已计入 guo_huixin，勿重复加）
        let crit_rate = calc.guo_huixin() + skill.huixin_up as f32 / 100.0;

        let g = damage.g_damage as f64;
        let h = damage.h_damage as f64;
        let q = damage.q_damage as f64;
        let lost = damage.lost_hp_zhenshi_damage;
        // 血量推进：实际扣血 = 本步期望伤害 + 追加真伤（真伤同样致损）
        if let (Some(cur), Some(_)) = (ec_cur, ec_max) {
            let after = (cur - q).max(0.0);
            ec_cur = Some((after - lost).max(0.0));
        }

        // 方差：无质 0；dot 逐跳独立（各跳方差和）；普通单跳
        let p = crit_rate as f64;
        let variance = if skill.has_critical_strike {
            0.0
        } else if skill.dot_flag > 0 {
            // dot 逐跳独立 → 方差 = 各跳方差之和
            let j = &damage.dot_jumps;
            if j.is_empty() {
                (h - g).powi(2) * p * (1.0 - p)
            } else {
                let base = j[0] as f64;
                j.iter()
                    .map(|&jk| {
                        let r = if base > 0.0 { jk as f64 / base } else { 1.0 };
                        let gk = g * r;
                        let hk = h * r;
                        (hk - gk).powi(2) * p * (1.0 - p)
                    })
                    .sum()
            }
        } else {
            (h - g).powi(2) * p * (1.0 - p)
        };

        // dot 逐跳模型（g/h 按 dot_jumps 等比拆分，与期望通道同源）
        let (dot_jumps_g, dot_jumps_h) = if skill.dot_flag > 0 && !damage.dot_jumps.is_empty() {
            let base = damage.dot_jumps[0] as f64;
            let mut gs = Vec::with_capacity(damage.dot_jumps.len());
            let mut hs = Vec::with_capacity(damage.dot_jumps.len());
            for &jk in &damage.dot_jumps {
                let r = if base > 0.0 { jk as f64 / base } else { 1.0 };
                gs.push((g * r) as u32);
                hs.push((h * r) as u32);
            }
            (gs, hs)
        } else {
            (Vec::new(), Vec::new())
        };

        cum_mean += q + lost;
        cum_var += variance;
        let cum_std = cum_var.sqrt();

        models.push(StepModel {
            g_damage: damage.g_damage,
            h_damage: damage.h_damage,
            q_damage: damage.q_damage,
            crit_rate,
            has_critical_strike: skill.has_critical_strike,
            lost_coeff: skill.lost_hp_zhenshishanghai as f64,
            dot_jumps_g,
            dot_jumps_h,
        });

        steps.push(ComboStepResult {
            skill_name: skill.skill_name.clone(),
            g_damage: damage.g_damage,
            h_damage: damage.h_damage,
            q_damage: damage.q_damage,
            crit_rate,
            cumulative_mean: cum_mean,
            cumulative_std: cum_std,
            kill_prob: 0.0, // MC 通道回填
            dot_jumps: damage.dot_jumps.clone(),
            has_critical_strike: skill.has_critical_strike,
            zhenshishanghai: skill.zhenshishanghai,
            lost_hp_zhenshi_damage: lost,
        });
    }

    // ---- 蒙特卡洛通道（击杀率） ----
    let kill_probs = match hp {
        None => {
            // 无血量语义（迁移前）：击杀率恒 1
            vec![1.0; steps.len()]
        }
        Some((hp0, hp_max)) => {
            let n = config.samples.max(1);
            let mut rng = match config.seed {
                Some(seed) => StdRng::seed_from_u64(seed),
                None => StdRng::from_os_rng(),
            };
            // kills_at[i] = 恰好第 i 步击杀的路径数
            let mut kills_at = vec![0u64; steps.len()];
            for _ in 0..n {
                let mut cur = hp0;
                for (i, m) in models.iter().enumerate() {
                    let dmg = if m.has_critical_strike {
                        m.q_damage as f64
                    } else if !m.dot_jumps_g.is_empty() {
                        // dot 逐跳独立会心
                        let mut s = 0.0;
                        for k in 0..m.dot_jumps_g.len() {
                            s += if rng.random::<f64>() < p_of(m.crit_rate) {
                                m.dot_jumps_h[k] as f64
                            } else {
                                m.dot_jumps_g[k] as f64
                            };
                        }
                        s
                    } else if rng.random::<f64>() < p_of(m.crit_rate) {
                        m.h_damage as f64
                    } else {
                        m.g_damage as f64
                    };
                    let after = (cur - dmg).max(0.0);
                    let lost = if m.lost_coeff > 0.0 {
                        let calc = JpcgConfig::new_with_config(
                            player, hostile, &skills[i], xinfa, buff, coeff,
                        );
                        calc.lost_hp_append(hp_max as u32, after as u32)
                    } else {
                        0.0
                    };
                    cur = (after - lost).max(0.0);
                    if cur <= 0.0 {
                        kills_at[i] += 1;
                        break;
                    }
                }
            }
            // 累计击杀率
            let mut acc = 0u64;
            let mut probs = Vec::with_capacity(steps.len());
            for &k in &kills_at {
                acc += k;
                probs.push(acc as f64 / n as f64);
            }
            probs
        }
    };
    for (i, step) in steps.iter_mut().enumerate() {
        step.kill_prob = kill_probs.get(i).copied().unwrap_or(1.0);
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

fn p_of(crit_rate: f32) -> f64 {
    crit_rate.clamp(0.0, 1.0) as f64
}
#[cfg(test)]
mod tests {
    use super::*;

    fn skill(
        name: &str,
        b1: u32,
        b2: u32,
        atk_xishu: f32,
        hit_up: u32,
        watk_xishu: u32,
        wushifangyu: u32,
    ) -> Skilltype {
        Skilltype {
            skill_name: name.to_string(),
            base_damage1: b1,
            base_damage2: b2,
            atk_xishu,
            hit_up,
            watk_xishu,
            wushifangyu,
            ..Skilltype::default()
        }
    }

    fn player() -> PlayerConfig {
        PlayerConfig {
            jcsx: "gengu".into(),
            jichu_shuxing: 21371,
            jichu_gongji: 64329,
            huixin_dengji: 61877,
            huixin_xiaoguo: 2925,
            pofang_dengji: 109160,
            wuqi_shanghai: 0,
            zuizhong_gongji: 0,
        }
    }

    fn hostile() -> HostilepileConfig {
        HostilepileConfig {
            waigong_fangyu: 15176,
            neigong_fangyu: 21388,
            yujin_dengji: 5047,
            huajin_dengji: 59402,
            jianshang_bili: 0,
            target_hp: 2_000_000,
            max_hp: 0,
            current_hp: 0,
        }
    }

    fn xinfa() -> XinfaConfig {
        XinfaConfig {
            profession: "mowen".into(),
            xinfa_name: "莫问".into(),
            xinfa_nom: "gengu".into(),
            atk_up: 1.96,
            pofang_up: 2.0,
            huixin_up: 0.0,
        }
    }

    fn combo(skills: &[Skilltype], cfg: ComboConfig) -> ComboResult {
        let (p, h, x) = (player(), hostile(), xinfa());
        let (b, c) = (BuffConfig::default(), CoefficientConfig::default());
        calculate_combo(skills, &p, &h, &x, &b, &c, &cfg)
    }

    /// 无质连招：每击固定期望 Q → 方差 0（期望通道）；普通技能方差 > 0
    #[test]
    fn wuzhi_zero_variance() {
        let cfg = ComboConfig {
            samples: 1000,
            seed: Some(42),
        };
        let normal = combo(&[skill("宫", 160, 200, 2.609375, 0, 0, 0)], cfg.clone());
        assert!(
            normal.steps[0].cumulative_std > 0.0,
            "普通技能应存在伤害波动"
        );

        let mut wuzhi = skill("宫", 160, 200, 2.609375, 0, 0, 0);
        wuzhi.has_critical_strike = true;
        let combo = combo(&[wuzhi.clone(), wuzhi], cfg);
        assert_eq!(combo.steps[0].cumulative_mean, 91768.0, "无质期望 = Q");
        assert_eq!(combo.steps[0].cumulative_std, 0.0, "无质方差应为 0");
        assert_eq!(combo.steps[1].cumulative_std, 0.0, "无质累计方差应为 0");
        assert_eq!(combo.steps[1].cumulative_mean, 183536.0, "两次无质期望翻倍");
        // target_hp=2M 满血，两击 183536 ≪ 2M → 击杀率 0（确定性精确）
        assert_eq!(combo.steps[1].kill_prob, 0.0);
    }

    /// 固定种子：蒙特卡洛结果可复现（确定性测试基础）
    #[test]
    fn monte_carlo_deterministic_with_seed() {
        let skills = std::array::from_fn::<_, 5, _>(|_| skill("宫", 160, 200, 2.609375, 0, 0, 0));
        let a = combo(
            &skills,
            ComboConfig {
                samples: 10_000,
                seed: Some(7),
            },
        );
        let b = combo(
            &skills,
            ComboConfig {
                samples: 10_000,
                seed: Some(7),
            },
        );
        assert_eq!(a.kill_prob_curve, b.kill_prob_curve);
        assert_eq!(a.final_kill_prob, b.final_kill_prob);
    }

    /// 蒙特卡洛在确定性子集上的收敛：
    /// 无质连招（零方差）→ 击杀率精确 0/1；含会心波动 → 0 < kill_prob < 1
    #[test]
    fn monte_carlo_kill_prob_behavior() {
        // 无质（高基础伤害，稳定击杀低血目标）
        let mut sk = skill("怒锋倾涛·破绽3层", 3500, 4000, 5.15625, 0, 200, 0);
        sk.has_critical_strike = true;
        let mut h = hostile();
        h.max_hp = 100_000;
        h.current_hp = 100_000;
        let (p, x) = (player(), xinfa());
        let (b, c) = (BuffConfig::default(), CoefficientConfig::default());
        let cfg = ComboConfig {
            samples: 5000,
            seed: Some(1),
        };
        let r = calculate_combo(&[sk.clone(), sk.clone()], &p, &h, &x, &b, &c, &cfg);
        assert_eq!(r.final_kill_prob, 1.0, "无质高伤组合应 100% 击杀");

        // 100k 目标 + 单次命中（期望约 18 万）→ 首步即 100%
        assert_eq!(r.steps[0].kill_prob, 1.0);

        // 会心波动普通技能（低伤害）对 2M 血 → 击杀率必定 < 100%（确定性下为 0）
        let weak = skill("宫", 16, 20, 0.2, 0, 0, 0);
        let r2 = combo(&[weak], cfg);
        assert!(
            r2.final_kill_prob < 1.0,
            "低伤组合击杀率应 < 100%: got {}",
            r2.final_kill_prob
        );
    }

    /// 追加真伤（已损失生命值 × 系数）——语义 A（结算后）：
    /// 满血首击追加 = 主Q × 系数；之后逐步递增（追加也计入损失，斩杀加速）
    #[test]
    fn lost_hp_zhenshi_increases_with_loss() {
        let mut sk = skill("怒锋倾涛·破绽3层", 35, 40, 5.15625, 0, 200, 0);
        sk.lost_hp_zhenshishanghai = 0.18;
        sk.has_critical_strike = true; // 无质主伤害，便于精确断言

        let cfg = ComboConfig {
            samples: 1000,
            seed: Some(3),
        };
        let res = combo(&[sk.clone(), sk.clone(), sk.clone()], cfg);

        let s0 = &res.steps[0];
        let q0 = s0.q_damage as f64;
        // 首击：max=target_hp 满血（default hostile），after = max - q0 → lost = q0 × 0.18
        let expect_0 = q0 * 0.18;
        assert!(
            (s0.lost_hp_zhenshi_damage - expect_0).abs() < 1.0,
            "首击追加 = 已损失(主Q)×0.18: got {} expect {}",
            s0.lost_hp_zhenshi_damage,
            expect_0
        );
        assert!(
            (s0.cumulative_mean - (q0 + expect_0)).abs() < 1.0,
            "首击累计期望应含追加真伤"
        );
        // 追加确定性：方差不受影响（主伤害无质 → 方差 0）
        assert_eq!(s0.cumulative_std, 0.0, "无质主伤害 + 确定性追加 → 方差 0");

        // 第二步：追加按扣血后剩余结算 → 追加递增（斩杀机制），非旧语义几何收敛
        let s1 = &res.steps[1];
        assert!(
            s1.lost_hp_zhenshi_damage > s0.lost_hp_zhenshi_damage,
            "追加应随已损失递增: 首 {} -> 次 {}",
            s0.lost_hp_zhenshi_damage,
            s1.lost_hp_zhenshi_damage
        );

        // 追加为 0 的形态不改变行为（期望通道无真伤）
        let mut sk0 = sk.clone();
        sk0.lost_hp_zhenshishanghai = 0.0;
        let combo0 = combo(
            &[sk0.clone(), sk0.clone()],
            ComboConfig {
                samples: 100,
                seed: Some(4),
            },
        );
        let q0 = combo0.steps[0].q_damage as f64;
        assert_eq!(
            combo0.steps[1].cumulative_mean,
            q0 * 2.0,
            "无真伤 → 期望恰为两次主Q"
        );
    }

    /// dot 每跳独立会心：方差 = 各跳方差之和（期望通道）；MC 采样逐跳判定
    /// 固定随机种子下，dot 连招击杀率在 (0,1) 而非极端区（若按整技能单次判定则方差过小）
    #[test]
    fn dot_per_jump_independent_crit() {
        let mut sk = skill("商（dot）", 160, 200, 2.609375, 0, 0, 0);
        sk.dot_flag = 1;
        sk.dot_interval = 18.0;
        sk.dot_duration = 108.0; // 6 跳
        sk.dot_up = 0.08;

        let cfg = ComboConfig {
            samples: 3000,
            seed: Some(9),
        };
        let r = combo(&[sk.clone()], cfg.clone());
        let step = &r.steps[0];
        assert_eq!(step.dot_jumps.len(), 6, "应产出 6 跳");
        // 方差应为单跳方差 × 6（等比独立），而非单跳方差
        let g0 = step.g_damage as f64;
        let h0 = step.h_damage as f64;
        let p = step.crit_rate as f64;
        let single_var = (h0 - g0).powi(2) * p * (1.0 - p);
        assert!(
            step.cumulative_std.powi(2) > single_var * 5.0,
            "逐跳独立方差应显著大于单跳方差: {} vs {}",
            step.cumulative_std.powi(2),
            single_var
        );
        // MC 击杀率介于 0~1（低血目标大概率未击杀）
        let mut h = hostile();
        h.max_hp = 5_000_000;
        h.current_hp = 5_000_000;
        let weak_cfg = ComboConfig {
            samples: 3000,
            seed: Some(10),
        };
        let (p_, x_) = (player(), xinfa());
        let (b, c) = (BuffConfig::default(), CoefficientConfig::default());
        let r2 = calculate_combo(&[sk.clone()], &p_, &h, &x_, &b, &c, &weak_cfg);
        assert!(
            r2.final_kill_prob < 1.0,
            "单技能远低于 5M 血 → 击杀率应 < 100%: got {}",
            r2.final_kill_prob
        );
    }
}
