//! 击杀率蒙特卡洛内核：f32 HP + 计数器哈希。
//!
//! - 同一 `seed` 下，所有候选/所有序列共用同一组随机流（common random numbers），
//!   保证不同配装之间的比较公平且可复现。
//! - 标量实现是唯一参考；x86 AVX-512 实现逐位同序（parity 单测保证）。
//! - 返回值是「每个事件的击杀数」，由调用方按事件→动作 step 的归属累加成曲线。

/// 单个伤害事件的掷点参数。
///
/// `fixed`（无质）事件由调用方折成 `g == h`、`rate == 1.0` 的普通事件。
#[derive(Clone, Copy, Debug)]
pub struct McEvent {
    /// 未会心伤害
    pub g: u32,
    /// 会心伤害
    pub h: u32,
    /// 会心概率 0..=1
    pub rate: f32,
    /// 追加真伤系数（0 = 无）
    pub lost: f32,
}

#[inline]
fn mix32(x: u32) -> u32 {
    let mut v = x;
    v ^= v >> 16;
    v = v.wrapping_mul(0x7feb_352d);
    v ^= v >> 15;
    v = v.wrapping_mul(0x846c_a68b);
    v ^= v >> 16;
    v
}

#[inline]
fn seed32(seed: u64) -> u32 {
    (seed as u32) ^ ((seed >> 32) as u32)
}

/// 标量参考实现。
pub fn kill_counts_scalar(events: &[McEvent], hp0: f32, samples: u32, seed: u32) -> Vec<u64> {
    let mut out = vec![0u64; events.len()];
    for s in 0..samples {
        let mut hp = hp0;
        let base = seed ^ s.wrapping_mul(0x85EB_CA6B);
        for (e, ev) in events.iter().enumerate() {
            let r = mix32(base ^ (e as u32).wrapping_mul(0xC2B2_AE35));
            let u = (r >> 8) as f32 * (1.0 / 16_777_216.0);
            let dmg = if u < ev.rate { ev.h } else { ev.g };
            hp -= dmg as f32;
            if hp < 0.0 {
                hp = 0.0;
            }
            if ev.lost > 0.0 && hp > 0.0 {
                hp -= (hp0 - hp) * ev.lost;
                if hp < 0.0 {
                    hp = 0.0;
                }
            }
            if hp <= 0.0 {
                out[e] += 1;
                break;
            }
        }
    }
    out
}

/// AVX-512 实现（16 样本一条向量；运行时检测后调用）。
///
/// # Safety
/// 调用前必须确认 `avx512_available()` 为 true，满足 f/dq/vl/bw 全部子集。
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512dq,avx512vl,avx512bw")]
pub unsafe fn kill_counts_avx512(
    events: &[McEvent],
    hp0: f32,
    samples: u32,
    seed: u32,
) -> Vec<u64> {
    use std::arch::x86_64::*;
    let mut out = vec![0u64; events.len()];
    let nb = events.len();
    let full = (samples / 16) * 16;
    let hp0v = _mm512_set1_ps(hp0);
    let one = _mm512_set1_ps(1.0 / 16_777_216.0);
    let zero = _mm512_setzero_ps();
    let gv: Vec<__m512> = events.iter().map(|e| _mm512_set1_ps(e.g as f32)).collect();
    let hv: Vec<__m512> = events.iter().map(|e| _mm512_set1_ps(e.h as f32)).collect();
    let rv: Vec<__m512> = events.iter().map(|e| _mm512_set1_ps(e.rate)).collect();
    let lv: Vec<__m512> = events.iter().map(|e| _mm512_set1_ps(e.lost)).collect();
    let mut s: u32 = 0;
    while s < full {
        let idx = _mm512_setr_epi32(
            s as i32,
            (s + 1) as i32,
            (s + 2) as i32,
            (s + 3) as i32,
            (s + 4) as i32,
            (s + 5) as i32,
            (s + 6) as i32,
            (s + 7) as i32,
            (s + 8) as i32,
            (s + 9) as i32,
            (s + 10) as i32,
            (s + 11) as i32,
            (s + 12) as i32,
            (s + 13) as i32,
            (s + 14) as i32,
            (s + 15) as i32,
        );
        let mut hp = hp0v;
        let mut killed: u16 = 0;
        let base = _mm512_xor_si512(
            _mm512_set1_epi32(seed as i32),
            _mm512_mullo_epi32(idx, _mm512_set1_epi32(0x85EB_CA6B_u32 as i32)),
        );
        for e in 0..nb {
            let mut r = _mm512_xor_si512(
                base,
                _mm512_set1_epi32((e as u32).wrapping_mul(0xC2B2_AE35) as i32),
            );
            r = _mm512_xor_si512(r, _mm512_srli_epi32(r, 16));
            r = _mm512_mullo_epi32(r, _mm512_set1_epi32(0x7feb_352d_u32 as i32));
            r = _mm512_xor_si512(r, _mm512_srli_epi32(r, 15));
            r = _mm512_mullo_epi32(r, _mm512_set1_epi32(0x846c_a68b_u32 as i32));
            r = _mm512_xor_si512(r, _mm512_srli_epi32(r, 16));
            let u = _mm512_mul_ps(_mm512_cvtepi32_ps(_mm512_srli_epi32(r, 8)), one);
            let crit = _mm512_cmp_ps_mask(u, rv[e], _CMP_LT_OQ);
            let dmg = _mm512_mask_blend_ps(crit, gv[e], hv[e]);
            hp = _mm512_max_ps(_mm512_sub_ps(hp, dmg), zero);
            let alive = _mm512_cmp_ps_mask(hp, zero, _CMP_GT_OQ);
            let lost_mask = _mm512_cmp_ps_mask(lv[e], zero, _CMP_GT_OQ);
            let extra_mask = alive & lost_mask;
            if extra_mask != 0 {
                let extra = _mm512_mul_ps(_mm512_sub_ps(hp0v, hp), lv[e]);
                hp = _mm512_mask_max_ps(hp, extra_mask, _mm512_sub_ps(hp, extra), zero);
            }
            let dead = _mm512_cmp_ps_mask(hp, zero, _CMP_LE_OQ) & !killed;
            if dead != 0 {
                out[e] += dead.count_ones() as u64;
                killed |= dead;
                if killed == 0xFFFF {
                    break;
                }
            }
        }
        s += 16;
    }
    // 余数（samples 非 16 倍数时）走标量，保证与参考实现一致。
    if full < samples {
        // 尾部样本的 base 与标量公式里 s 保持一致。
        for t in full..samples {
            let mut hp = hp0;
            let base = seed ^ t.wrapping_mul(0x85EB_CA6B);
            for (e, ev) in events.iter().enumerate() {
                let r = mix32(base ^ (e as u32).wrapping_mul(0xC2B2_AE35));
                let u = (r >> 8) as f32 * (1.0 / 16_777_216.0);
                let dmg = if u < ev.rate { ev.h } else { ev.g };
                hp -= dmg as f32;
                if hp < 0.0 {
                    hp = 0.0;
                }
                if ev.lost > 0.0 && hp > 0.0 {
                    hp -= (hp0 - hp) * ev.lost;
                    if hp < 0.0 {
                        hp = 0.0;
                    }
                }
                if hp <= 0.0 {
                    out[e] += 1;
                    break;
                }
            }
        }
    }
    out
}

/// 必须覆盖 target_feature 声明的完整指令子集。
pub fn avx512_available() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        std::is_x86_feature_detected!("avx512f")
            && std::is_x86_feature_detected!("avx512dq")
            && std::is_x86_feature_detected!("avx512vl")
            && std::is_x86_feature_detected!("avx512bw")
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

/// 击杀计数入口：x86 上优先 AVX-512，否则标量。
pub fn kill_counts(events: &[McEvent], hp0: f64, samples: u32, seed: u64) -> Vec<u64> {
    let hp0 = hp0 as f32;
    let seed = seed32(seed);
    if events.is_empty() || samples == 0 {
        return vec![0u64; events.len()];
    }
    #[cfg(target_arch = "x86_64")]
    {
        if avx512_available() {
            // SAFETY: 已确认 target_feature 声明的全部 AVX-512 子集可用。
            return unsafe { kill_counts_avx512(events, hp0, samples, seed) };
        }
    }
    kill_counts_scalar(events, hp0, samples, seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn events() -> Vec<McEvent> {
        (0..8)
            .map(|i| McEvent {
                g: 500 + i * 100,
                h: 1500 + i * 120,
                rate: 0.2 + i as f32 * 0.05,
                lost: if i % 3 == 0 { 0.02 } else { 0.0 },
            })
            .collect()
    }

    #[test]
    fn scalar_is_deterministic() {
        let a = kill_counts_scalar(&events(), 9000.0, 2000, 12345);
        let b = kill_counts_scalar(&events(), 9000.0, 2000, 12345);
        assert_eq!(a, b);
        assert!(a.iter().sum::<u64>() > 0);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn avx512_matches_scalar_scalar_only_when_16_aligned() {
        if !avx512_available() {
            return;
        }
        let ev = events();
        let scalar = kill_counts_scalar(&ev, 9000.0, 16 * 64, 12345);
        // SAFETY: 已确认 avx512f 可用
        let vec = unsafe { kill_counts_avx512(&ev, 9000.0, 16 * 64, 12345) };
        assert_eq!(scalar, vec);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn avx512_handles_tail() {
        if !avx512_available() {
            return;
        }
        let ev = events();
        let scalar = kill_counts_scalar(&ev, 9000.0, 16 * 5 + 7, 777);
        // SAFETY: 已确认 avx512f 可用
        let vec = unsafe { kill_counts_avx512(&ev, 9000.0, 16 * 5 + 7, 777) };
        assert_eq!(scalar, vec);
    }
}
