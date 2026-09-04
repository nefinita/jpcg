// ============================================================================
// level_constant —— 等级常数（编译期固化）
//
// 数据源：preset/level_constant.toml（include_str! 内嵌，改文件即自动重编）。
// 解析：const fn 逐行白名单解析，任何未知 key / 缺字段 / 坏值都会在
//       编译期 panic（const 求值 = 编译错误），防"策划改字段忘同步解析器"。
// 设计：运行时零依赖、零 I/O；level 只作快照记录（LEVEL），不进入结构。
//       f32 十进制解析用"整数尾数 ÷ 10^k"保证与 Rust 字面量 bit 完全一致
//       （本数据量级尾数 < 2^24，整数部分 f32 精确，除法按 IEEE 正确舍入）。
// 注意：const fn 不允许切片/range 下标（Index 未 stable const），
//       故全程以 (bytes, start, end) 偏移 + 单元素下标实现扫描。
// ============================================================================

/// 等级常数结构（纯数据镜像，无 serde，无 level 字段）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelConstant {
    /// 破防等级 → 破防减伤（×1024 制分母）
    pub pofang_xishu: f32,
    /// 会心等级 → 会心率（×1024 制分母）
    pub huixin_xishu: f32,
    /// 会效等级 → 会心伤害（×1024 制分母）
    pub huixiao_xishu: f32,
    /// 御劲 → 目标会心率减免（×1024 制分母）
    pub yujin_xishu: f32,
    /// 御劲 → 目标会心伤害减免（×1024 制分母）
    pub yuhui_xishu: f32,
    /// 化劲等级 → 伤害减免（×1024 制分母）
    pub huajin_xishu: f32,
    /// 防御等级 → 伤害减免（×1024 制分母）
    pub fangyu_xishu: f32,
    /// PVP 全局减伤（乘法比例，非 1024 制）
    pub pvp_global_jianshang: f32,
}

impl LevelConstant {
    /// 全零占位（解析器填充用）
    const fn zero() -> Self {
        Self {
            pofang_xishu: 0.0,
            huixin_xishu: 0.0,
            huixiao_xishu: 0.0,
            yujin_xishu: 0.0,
            yuhui_xishu: 0.0,
            huajin_xishu: 0.0,
            fangyu_xishu: 0.0,
            pvp_global_jianshang: 0.0,
        }
    }
}

/// 快照文本（改 preset 即触发重编）
const SNAPSHOT_TEXT: &str = include_str!("../preset/level_constant.toml");

/// 快照对应等级（仅记录/预计算用）
pub const LEVEL: u32 = parse_snapshot(SNAPSHOT_TEXT).0;

/// 当前快照的等级常数（编译期解析固化）
pub const CURRENT: LevelConstant = parse_snapshot(SNAPSHOT_TEXT).1;

/// 解析快照文本 → (level, LevelConstant)
/// 未知 key / 缺字段 / 值非法一律 panic（const 场景即编译错误）
pub const fn parse_snapshot(text: &str) -> (u32, LevelConstant) {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut out = LevelConstant::zero();
    let mut level: u32 = 0;
    let mut seen: u16 = 0; // 9 个 key（含 level）各占一位
    const KEY_MASK: u16 = (1 << 9) - 1;

    let mut i = 0;
    while i < len {
        // 跳过空行
        if bytes[i] == b'\n' {
            i += 1;
            continue;
        }
        // 注释行（#）直接跳到行尾
        if bytes[i] == b'#' {
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // 定位本行 [line_start, line_end)
        let line_start = i;
        while i < len && bytes[i] != b'\n' {
            i += 1;
        }
        let line_end = i;
        if i < len {
            i += 1; // 跨过换行
        }

        // 行内 trim（首尾空白）
        let (ls, le) = trim_range(bytes, line_start, line_end);
        if ls >= le {
            continue;
        }

        // 定位 '=' 分隔
        let mut eq = ls;
        while eq < le && bytes[eq] != b'=' {
            eq += 1;
        }
        if eq <= ls || eq + 1 >= le {
            panic!("level_constant 解析失败：行内缺少 'key = value'");
        }
        let (ks, ke) = trim_range(bytes, ls, eq);
        // value 段：切掉行内 '#' 注释后 trim
        let mut val_end = le;
        let mut c = eq + 1;
        while c < le {
            if bytes[c] == b'#' {
                val_end = c;
                break;
            }
            c += 1;
        }
        let (vs, ve) = trim_range(bytes, eq + 1, val_end);

        // 白名单位标（只允许预置 key，防新增字段忘同步解析器）
        let bit: usize = if key_matches(bytes, ks, ke, b"level") {
            0
        } else if key_matches(bytes, ks, ke, b"pofang_xishu") {
            1
        } else if key_matches(bytes, ks, ke, b"huixin_xishu") {
            2
        } else if key_matches(bytes, ks, ke, b"huixiao_xishu") {
            3
        } else if key_matches(bytes, ks, ke, b"yujin_xishu") {
            4
        } else if key_matches(bytes, ks, ke, b"yuhui_xishu") {
            5
        } else if key_matches(bytes, ks, ke, b"huajin_xishu") {
            6
        } else if key_matches(bytes, ks, ke, b"fangyu_xishu") {
            7
        } else if key_matches(bytes, ks, ke, b"pvp_global_jianshang") {
            8
        } else {
            panic!("level_constant 解析失败：未知 key（请同步白名单解析器）");
        };

        if seen & (1 << bit) != 0 {
            panic!("level_constant 解析失败：key 重复出现");
        }
        seen |= 1 << bit;

        if bit == 0 {
            level = parse_u32(bytes, vs, ve);
        } else {
            let v = parse_f32(bytes, vs, ve);
            match bit {
                1 => out.pofang_xishu = v,
                2 => out.huixin_xishu = v,
                3 => out.huixiao_xishu = v,
                4 => out.yujin_xishu = v,
                5 => out.yuhui_xishu = v,
                6 => out.huajin_xishu = v,
                7 => out.fangyu_xishu = v,
                _ => out.pvp_global_jianshang = v,
            }
        }
    }

    if seen != KEY_MASK {
        panic!("level_constant 解析失败：缺少必需字段");
    }
    (level, out)
}

/// key 段与字节字面量等值判断（[s, e) 内）
const fn key_matches(b: &[u8], s: usize, e: usize, lit: &[u8]) -> bool {
    if e - s != lit.len() {
        return false;
    }
    let mut j = 0;
    while j < lit.len() {
        if b[s + j] != lit[j] {
            return false;
        }
        j += 1;
    }
    true
}

/// 去掉首尾空白（空格 / \t / \r），返回新的 [s, e)
const fn trim_range(b: &[u8], mut s: usize, e: usize) -> (usize, usize) {
    while s < e {
        let c = b[s];
        if c == b' ' || c == b'\t' || c == b'\r' {
            s += 1;
        } else {
            break;
        }
    }
    let mut e2 = e;
    while e2 > s {
        let c = b[e2 - 1];
        if c == b' ' || c == b'\t' || c == b'\r' {
            e2 -= 1;
        } else {
            break;
        }
    }
    (s, e2)
}

/// 解析无符号整数（[s, e) 内仅 ASCII 数字）
const fn parse_u32(b: &[u8], s: usize, e: usize) -> u32 {
    if s >= e {
        panic!("数值为空");
    }
    let mut v: u64 = 0;
    let mut i = s;
    while i < e {
        let c = b[i];
        if !c.is_ascii_digit() {
            panic!("非法整数");
        }
        v = v * 10 + (c - b'0') as u64;
        i += 1;
    }
    if v > u32::MAX as u64 {
        panic!("整数超出 u32 范围");
    }
    v as u32
}

/// 解析十进制 f32（[s, e) 内，允许整数 / `a.b`）
/// 保证与同文本 Rust 字面量舍入一致：整数尾数 < 2^24 时 f32 精确，
/// 再除以精确的 10^k（k ≤ 7），IEEE 除法正确舍入 = 字面量舍入。
const fn parse_f32(b: &[u8], s: usize, e: usize) -> f32 {
    if s >= e {
        panic!("数值为空");
    }
    let mut int_part: u64 = 0;
    let mut i = s;
    let mut int_digits = 0usize;
    while i < e && b[i] != b'.' {
        let c = b[i];
        if !c.is_ascii_digit() {
            panic!("非法小数");
        }
        int_part = int_part * 10 + (c - b'0') as u64;
        int_digits += 1;
        i += 1;
    }
    if i >= e {
        // 纯整数
        if int_digits == 0 {
            panic!("数值为空");
        }
        return int_part as f32;
    }
    // 有小数点：必须紧跟数字
    if i + 1 >= e {
        panic!("小数点后缺少数字");
    }
    let mut frac_part: u64 = 0;
    let mut k: u32 = 0;
    i += 1;
    while i < e {
        let c = b[i];
        if !c.is_ascii_digit() {
            panic!("非法小数");
        }
        frac_part = frac_part * 10 + (c - b'0') as u64;
        k += 1;
        i += 1;
    }
    if int_digits == 0 && frac_part == 0 {
        panic!("数值为空");
    }
    // mantissa = int_part * 10^k + frac_part
    let mut mantissa: u64 = int_part;
    let mut p = 0;
    while p < k {
        mantissa *= 10;
        p += 1;
    }
    mantissa += frac_part;
    if mantissa >= (1 << 24) {
        panic!("小数尾数超出 f32 精确整数范围，请调整表示或改用代码生成");
    }
    let mut divisor: f32 = 1.0;
    p = 0;
    while p < k {
        divisor *= 10.0;
        p += 1;
    }
    mantissa as f32 / divisor
}

#[cfg(test)]
mod tests {
    use super::{CURRENT, LEVEL, LevelConstant, parse_snapshot};

    #[test]
    fn snapshot_matches_rust_literal_bits() {
        // 与 preset/level_constant.toml 当前值逐项 bit 校验（防解析精度漂移）
        let expect = LevelConstant {
            pofang_xishu: 225957.6,
            huixin_xishu: 197703.0,
            huixiao_xishu: 72844.2,
            yujin_xishu: 197703.0,
            yuhui_xishu: 55123.2,
            huajin_xishu: 30115.8,
            fangyu_xishu: 126007.2,
            pvp_global_jianshang: 0.9,
        };
        assert_eq!(LEVEL, 130);
        assert_eq!(CURRENT, expect);
    }

    #[test]
    fn full_text_parse_roundtrip() {
        let (level, val) = parse_snapshot(include_str!("../preset/level_constant.toml"));
        assert_eq!(level, LEVEL);
        assert_eq!(val, CURRENT);
    }

    #[test]
    fn unknown_key_panics() {
        let txt = "level = 130\npofang_xishu = 225957.6\nbogus_key = 1\n";
        let r = std::panic::catch_unwind(|| parse_snapshot(txt));
        assert!(r.is_err(), "未知 key 必须 panic");
    }

    #[test]
    fn missing_field_panics() {
        let txt = "level = 130\npofang_xishu = 225957.6\n";
        let r = std::panic::catch_unwind(|| parse_snapshot(txt));
        assert!(r.is_err(), "缺字段必须 panic");
    }
}
