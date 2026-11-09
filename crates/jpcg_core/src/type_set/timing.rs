// ============================================================================
// timing —— 技能时间轴量化（#92 时序引擎）
//
// 游戏内所有技能时长（读条 / 引导 / 公共 CD / 调息）以「帧」为单位量化：
//   - 16 帧 = 1 秒（1 帧 = 0.0625 秒）
//   - 加速率(%) = 加速等级 / 加速系数  （加速系数 = LevelConstant::jiasu_xishu；
//     50 级 = 101.079，即 101.079 加速等级 ≈ 1% 加速率）
//   - 加速后帧数 = floor(基础帧数 / (1 + 加速率))，向下取整
//     → 极低的加速等级也可能直接降 1 帧（这是游戏内「加速档位」的成因）
//   - 加速率对「受加速影响」的调息时间同样按帧量化
//
// 实测对照（50 级，基础 1.5 秒 = 24 帧，加速系数 101.079）：
//   加速等级 0    → 加速率  0.00% → 24 帧 → 1.50 秒
//   加速等级 10   → 加速率  0.10% → 23 帧 → 1.44 秒
//   加速等级 445  → 加速率  4.40% → 22 帧 → 1.38 秒
//   加速等级 928  → 加速率  9.18% → 21 帧 → 1.31 秒
//   加速等级 1452 → 加速率 14.37% → 20 帧 → 1.25 秒
//   加速等级 2024 → 加速率 20.02% → 19 帧 → 1.19 秒
//   （见本模块 tests::jiasu_table_matches_measured_50_level）
// ============================================================================

/// 游戏时间基准：每秒帧数
pub const FRAMES_PER_SECOND: u32 = 16;
/// 单帧秒数（1/16 秒）
pub const SECS_PER_FRAME: f32 = 0.0625;
/// 加速率上限（公共 CD 计算用）：大部分门派 GCD 加速封顶 25%
/// （1.5 秒 GCD → 最低 19 帧 = 1.1875 秒 ≈ 1.19；1.0 秒 GCD → 最低 12 帧 = 0.75 秒）
/// 部分技能/奇穴的加速可突破该上限，由 buff 定义上的 `jiasu_ignore_cap` 标记。
pub const HASTE_CAP: f32 = 0.25;

/// 加速等级 → 加速率（小数，非百分比）。
/// `jiasu_xishu <= 0`（值集未逆向该系数）时返回 0，即不启用加速换算。
pub fn jiasu_rate(jiasu_dengji: u32, jiasu_xishu: f32) -> f32 {
    if jiasu_xishu.is_finite() && jiasu_xishu > 0.0 {
        (jiasu_dengji as f32 / jiasu_xishu / 100.0).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// 秒 → 帧（四舍五入到最近整数帧；非正数返回 0）
pub fn secs_to_frames(secs: f32) -> u32 {
    if secs.is_finite() && secs > 0.0 {
        (secs * FRAMES_PER_SECOND as f32).round() as u32
    } else {
        0
    }
}

/// 帧 → 秒
pub fn frames_to_secs(frames: u32) -> f32 {
    frames as f32 * SECS_PER_FRAME
}

/// 按加速率量化帧数：floor(基础帧数 / (1 + 加速率))
/// 基础帧数为 0（瞬时技能）时返回 0。
pub fn quantize_frames(secs: f32, haste_rate: f32) -> u32 {
    let base = secs_to_frames(secs);
    if base == 0 {
        return 0;
    }
    let rate = if haste_rate.is_finite() && haste_rate > 0.0 {
        haste_rate
    } else {
        0.0
    };
    let frames = ((base as f32) / (1.0 + rate)).floor() as u32;
    frames.max(1) // 有基础时长的技能至少保留 1 帧
}

/// 按加速率量化时长（秒）
pub fn quantize_secs(secs: f32, haste_rate: f32) -> f32 {
    frames_to_secs(quantize_frames(secs, haste_rate))
}

/// 公共 CD 量化：加速率先夹到 [`HASTE_CAP`]（GCD 封顶 25%），再按帧量化。
/// 1.5 秒 GCD：0% → 1.50，25% → 1.1875（≈1.19）；1.0 秒 GCD：25% → 0.75。
pub fn quantize_gcd(gcd_secs: f32, haste_rate: f32) -> f32 {
    let base = if gcd_secs.is_finite() && gcd_secs > 0.0 {
        gcd_secs
    } else {
        return 0.0;
    };
    let rate = if haste_rate.is_finite() {
        haste_rate.clamp(0.0, HASTE_CAP)
    } else {
        0.0
    };
    quantize_secs(base, rate)
}

/// 合并加速来源（#92 口径）：先汇总「受 25% 上限」的来源并截断，再叠加可突破上限的来源。
///
/// `final_rate = min(capped_rate, 0.25) + uncapped_rate`
pub fn combine_jiasu(capped_rate: f32, uncapped_rate: f32) -> f32 {
    let capped = if capped_rate.is_finite() {
        capped_rate.clamp(0.0, HASTE_CAP)
    } else {
        0.0
    };
    let uncapped = if uncapped_rate.is_finite() {
        uncapped_rate.max(0.0)
    } else {
        0.0
    };
    capped + uncapped
}

/// Dot 逐跳时间点（相对施加时刻的秒偏移，长度 = 跳数）。
///
/// 剑三 Dot 为「总跳数固定，总时长随加速缩短」：时长按帧量化后，逐跳在上均匀分布
/// （第 i 跳于 `总时长 × i / 跳数` 处结算；跳数不随加速变化）。
pub fn dot_tick_times(ticks: u32, duration_secs: f32, haste_rate: f32) -> Vec<f32> {
    if ticks == 0 {
        return Vec::new();
    }
    let total = quantize_secs(duration_secs, haste_rate);
    (1..=ticks)
        .map(|i| total * i as f32 / ticks as f32)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 50 级实测表：加速系数 101.079，基础读条 1.5 秒（24 帧）
    /// (加速等级, 期望读条秒数, 期望加速率百分比)
    const TABLE: [(u32, f32, f32); 6] = [
        (0, 1.50, 0.00),
        (10, 1.44, 0.10),
        (445, 1.38, 4.40),
        (928, 1.31, 9.18),
        (1452, 1.25, 14.37),
        (2024, 1.19, 20.02),
    ];

    fn round2(v: f32) -> f32 {
        (v * 100.0).round() / 100.0
    }

    #[test]
    fn jiasu_table_matches_measured_50_level() {
        const JIASU_XISHU: f32 = 101.079;
        let base = 1.5_f32;
        for (dengji, expect_secs, expect_pct) in TABLE {
            let rate = jiasu_rate(dengji, JIASU_XISHU);
            assert_eq!(
                round2(rate * 100.0),
                expect_pct,
                "加速等级 {dengji} 的加速率应为 {expect_pct}%"
            );
            assert_eq!(
                round2(quantize_secs(base, rate)),
                expect_secs,
                "加速等级 {dengji} 的读条时间应为 {expect_secs}s"
            );
        }
    }

    /// 第二张实测表：基础时长 18 秒（288 帧）、9 跳（每跳 2 秒），50 级加速系数 101.079。
    /// 记录的是逐帧掉档的「阈值点」：加速等级不足该档位时读条时间不变（阶梯式）。
    const TABLE_18S: [(u32, f32); 59] = [
        (0, 18.00),
        (10, 17.94),
        (40, 17.88),
        (79, 17.81),
        (109, 17.75),
        (149, 17.69),
        (188, 17.62),
        (218, 17.56),
        (257, 17.50),
        (297, 17.44),
        (336, 17.38),
        (366, 17.31),
        (405, 17.25),
        (445, 17.19),
        (484, 17.12),
        (524, 17.06),
        (563, 17.00),
        (603, 16.94),
        (642, 16.88),
        (682, 16.81),
        (721, 16.75),
        (761, 16.69),
        (800, 16.62),
        (840, 16.56),
        (879, 16.50),
        (928, 16.44),
        (968, 16.38),
        (1007, 16.31),
        (1047, 16.25),
        (1096, 16.19),
        (1136, 16.12),
        (1185, 16.06),
        (1225, 16.00),
        (1274, 15.94),
        (1313, 15.88),
        (1363, 15.81),
        (1402, 15.75),
        (1452, 15.69),
        (1491, 15.62),
        (1540, 15.56),
        (1590, 15.50),
        (1639, 15.44),
        (1679, 15.38),
        (1728, 15.31),
        (1777, 15.25),
        (1827, 15.19),
        (1876, 15.12),
        (1925, 15.06),
        (1975, 15.00),
        (2024, 14.94),
        (2073, 14.88),
        (2133, 14.81),
        (2182, 14.75),
        (2231, 14.69),
        (2281, 14.62),
        (2340, 14.56),
        (2389, 14.50),
        (2449, 14.44),
        (2498, 14.38),
    ];

    #[test]
    fn jiasu_table_matches_measured_18s_9ticks() {
        const JIASU_XISHU: f32 = 101.079;
        let base = 18.0_f32;
        for (dengji, expect_secs) in TABLE_18S {
            let rate = jiasu_rate(dengji, JIASU_XISHU);
            // 实测表按 2 位小数「截断」记录，统一折算回帧数比较（避免半帧舍入差）
            let expect_frames = (expect_secs * FRAMES_PER_SECOND as f32).round() as u32;
            assert_eq!(
                quantize_frames(base, rate),
                expect_frames,
                "加速等级 {dengji} 的 18 秒技能应为 {expect_secs}s（{expect_frames} 帧）"
            );
        }
    }

    #[test]
    fn zero_coefficient_disables_haste() {
        assert_eq!(jiasu_rate(2024, 0.0), 0.0);
        assert_eq!(quantize_secs(1.5, jiasu_rate(2024, 0.0)), 1.5);
    }

    #[test]
    fn instant_skills_stay_instant() {
        assert_eq!(quantize_frames(0.0, 0.2), 0);
        assert_eq!(quantize_secs(0.0, 0.2), 0.0);
    }

    #[test]
    fn gcd_capped_at_25_percent_haste() {
        // 普通门派 1.5 秒 GCD：25% 封顶 → 19 帧 = 1.1875 秒（≈1.19）
        let at_cap = quantize_gcd(1.5, 0.25);
        assert_eq!(at_cap, 1.1875);
        // 超过 25% 的加速率对 GCD 无效（封顶）
        assert_eq!(quantize_gcd(1.5, 0.60), at_cap);
        // 特殊门派 1.0 秒 GCD：25% 封顶 → 12 帧 = 0.75 秒
        assert_eq!(quantize_gcd(1.0, 0.25), 0.75);
        assert_eq!(quantize_gcd(1.0, 0.2315), 0.75);
    }

    /// 特殊门派 1.0 秒 GCD 实测表（50 级，加速系数 101.079）
    #[test]
    fn gcd_1s_table_matches_measured() {
        const JIASU_XISHU: f32 = 101.079;
        let table = [
            (0u32, 1.00f32),
            (10, 0.94),
            (682, 0.88),
            (1452, 0.81),
            (2340, 0.75),
        ];
        for (dengji, expect) in table {
            let rate = jiasu_rate(dengji, JIASU_XISHU);
            assert_eq!(round2(quantize_gcd(1.0, rate)), expect, "加速等级 {dengji}");
        }
    }

    #[test]
    fn dot_tick_times_shrink_with_haste_but_keep_count() {
        // 18 秒 9 跳：0% → 每跳 2 秒；9.18% → 总时长 263 帧 = 16.4375 秒，9 跳均分
        let no_haste = dot_tick_times(9, 18.0, 0.0);
        assert_eq!(no_haste.len(), 9);
        assert!((no_haste[8] - 18.0).abs() < 1e-6);
        let hasted = dot_tick_times(9, 18.0, jiasu_rate(928, 101.079));
        assert_eq!(hasted.len(), 9);
        assert!((hasted[8] - 16.4375).abs() < 1e-6);
        assert!((hasted[0] - 16.4375 / 9.0).abs() < 1e-6);
    }
}
