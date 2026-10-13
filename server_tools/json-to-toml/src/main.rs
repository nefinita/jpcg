// ============================================================================
// json-to-toml — IcyTide/Generator 技能 JSON → JPCG data/shuxing TOML 转换器
//
// 数据来源: https://github.com/IcyTide/Generator（assets/json/skills.json、
// dots.json/belongs.json），伤害均为公式字符串。
//
// 转换规则：
// - 全量输出：每个技能实体的所有 lv 形态全部保留（等级成长/层数/点数/距离/目标数
//   等均为独立可调形态，如段氏「引窍」0~100 点任脉 101 个形态），不再压缩为最大 lv。
// - 同名多 lv（同 sid+sub 多形态）后缀区分：优先用 comment（如「0点任脉」）生成
//   `名·comment`；无 comment 或 comment 与名字相同则退回 `名(lv{N})`。
// - 同名多实体（不同 sid/sub，如「宫」「剑·角」）全部输出，以 skill_id/sub_id 区分。
// - base_damage1/2 ← source_attribute.{lunar|solar|neutral|physical}_damage_base(+rand)
// - atk_xishu ← skill_attribute.{magical|physical|surplus}_attack_power_cof，
//   recipe_XXX_1 引用按 0（未点奇穴/秘籍）求值，int() 按截断求值。
// - has_critical_strike（JPCG 语义 = 无质，伤害固定为期望 Q）：
//     · 数据源 critical_strike/critical_power 任一为 "0"/"O" 占位 → 无质
//     · overrides.pools[].wuzhi 名单命中（数据层漏标时的兜底）→ 无质
//   （注意：该字段与「可暴击」字面语义相反，见 type_set/skilltype.rs 注释）
// - zhenshishanghai ← skill_attribute.custom_damage_base 存在（真实伤害标签）。
// - 追加真伤（lost_hp_zhenshishanghai）：overrides.pools[].lost_hp_zhenshi 名单命中的
//   技能按「已损失生命值×系数」追加真伤，每个原始形态展开为 破绽0..N 层 形态
//   （如刀宗「怒锋倾涛·单持·破绽2层」），系数 = per_layer × 层数。
// - dot：dots.json 的 skills[源技能id][lv] 子条目自带全部伤害参数：
//   base ← damages[0].source_attribute（dot 无 rand，base1=base2）；
//   cof ← skill_attribute.magical_attack_power_cof（原样输出数据源值）；
//   dot_up ← 公式中的 `X ** (tick - N)` 因子 → X - 1（每跳等比递增）；
//   interval 为帧（16帧=1秒）→ dot_interval 秒；dot_duration = max_tick × interval。
//   同名双条目（two_dot_forms 池，如莫问商/角 DOT 48帧/6跳 与 32帧/9跳）
//   小 interval 条目标记 jihuoqixue = "疏曲"。
// - 无 base/cof/真伤标签的纯控制技能跳过并列出警告。
// ============================================================================

use clap::Parser;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "json-to-toml",
    version,
    about = "IcyTide/Generator 技能 JSON → JPCG data/shuxing TOML"
)]
struct Args {
    /// skills.json 路径
    #[arg(long, default_value = "skills.json")]
    skills: PathBuf,
    /// dots.json 路径
    #[arg(long, default_value = "dots.json")]
    dots: PathBuf,
    /// overrides.json 路径
    #[arg(long, default_value = "overrides.json")]
    overrides: PathBuf,
    /// 输出目录
    #[arg(long, default_value = "../../data/shuxing")]
    out: PathBuf,
    /// 要转换的池 id（可多次），如 --xinfa 10447 --xinfa 10786
    #[arg(long)]
    xinfa: Vec<String>,
    /// 游戏等级
    #[arg(long, default_value_t = 130)]
    level: u32,
    /// 赛季号
    #[arg(long, default_value_t = 3)]
    season: u32,
    /// 数据日期 YYYYMMDD
    #[arg(long, default_value_t = 20260814)]
    modified: u32,
}

#[derive(Deserialize, Default)]
struct PoolOverride {
    /// 输出文件名（默认 <池id>.toml）
    file: Option<String>,
    xinfa_name: String,
    xinfa_nom: String,
    atk_up: Option<f32>,
    pofang_up: Option<f32>,
    huixin_up: Option<f32>,
    /// 无质技能名单（name 精确匹配，按原始名，不含 lv 后缀）
    wuzhi: Vec<String>,
    /// 追加真伤名单：{name, per_layer, max_layers}，按原始名匹配
    /// 命中技能展开为 破绽0..=max_layers 层形态，每层追加真伤 = 已损失生命值 × per_layer × 层
    #[serde(default)]
    lost_hp_zhenshi: Vec<LostHpZhenshi>,
    /// 显示名重命名：原始 name → 显示名（如数据源「骤风」→「骤风令」belongs 名）
    #[serde(default)]
    rename: BTreeMap<String, String>,
}

#[derive(Deserialize, Clone)]
struct LostHpZhenshi {
    name: String,
    per_layer: f32,
    max_layers: u32,
}

#[derive(Deserialize, Default)]
struct Overrides {
    pools: BTreeMap<String, PoolOverride>,
}

struct SkillOut {
    name: String,
    sid: u32,
    sub: u32,
    base1: u32,
    base2: u32,
    atk_xishu: Option<f32>,
    wuzhi: bool,
    zhenshi: bool,
    /// 追加真伤系数（已损失生命值 × 系数，0=无）
    lost_hp: f32,
    dot: Option<DotOut>,
}

struct DotOut {
    interval_sec: f32,
    duration_sec: f32,
    dot_up: Option<f32>,
}

// ============================================================================
// 公式求值（recipe 已替换为 0，tick 已替换为 1）：
// 支持数字、括号、+ - * / **、int(...)；** 右结合，优先级高于 * /
// int() 语义 = 向零截断（与游戏 int() 一致）
// ============================================================================

fn eval_expr(s: &str) -> Result<f64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("空表达式".into());
    }
    let mut p = 0usize;
    let v = parse_expr(s, &mut p)?;
    skip_ws(s, &mut p);
    if p < s.len() {
        return Err(format!("尾部未解析: {}", &s[p..]));
    }
    Ok(v)
}

fn skip_ws(s: &str, p: &mut usize) {
    let b = s.as_bytes();
    while *p < b.len() && b[*p].is_ascii_whitespace() {
        *p += 1;
    }
}

fn parse_expr(s: &str, p: &mut usize) -> Result<f64, String> {
    let mut v = parse_term(s, p)?;
    loop {
        skip_ws(s, p);
        match s.as_bytes().get(*p) {
            Some(b'+') => {
                *p += 1;
                v += parse_term(s, p)?;
            }
            Some(b'-') => {
                *p += 1;
                v -= parse_term(s, p)?;
            }
            _ => break,
        }
    }
    Ok(v)
}

fn parse_term(s: &str, p: &mut usize) -> Result<f64, String> {
    let mut v = parse_power(s, p)?;
    loop {
        skip_ws(s, p);
        match s.as_bytes().get(*p) {
            Some(b'*') if !s[*p..].starts_with("**") => {
                *p += 1;
                v *= parse_power(s, p)?;
            }
            Some(b'/') => {
                *p += 1;
                let d = parse_power(s, p)?;
                if d == 0.0 {
                    return Err("除以零".into());
                }
                v /= d;
            }
            _ => break,
        }
    }
    Ok(v)
}

/// 幂运算：`a ** b`，右结合（1.12 ** (tick - 1)）
fn parse_power(s: &str, p: &mut usize) -> Result<f64, String> {
    let base = parse_factor(s, p)?;
    skip_ws(s, p);
    if s[*p..].starts_with("**") {
        *p += 2;
        let exp = parse_power(s, p)?;
        Ok(base.powf(exp))
    } else {
        Ok(base)
    }
}

fn parse_factor(s: &str, p: &mut usize) -> Result<f64, String> {
    skip_ws(s, p);
    let b = s.as_bytes();
    if *p >= b.len() {
        return Err("表达式意外结束".into());
    }
    if b[*p] == b'(' {
        *p += 1;
        let v = parse_expr(s, p)?;
        skip_ws(s, p);
        if s.as_bytes().get(*p) != Some(&b')') {
            return Err("缺少右括号".into());
        }
        *p += 1;
        return Ok(v);
    }
    if s[*p..].starts_with("int(") {
        *p += 4;
        let v = parse_expr(s, p)?;
        skip_ws(s, p);
        if s.as_bytes().get(*p) != Some(&b')') {
            return Err("int() 缺少右括号".into());
        }
        *p += 1;
        return Ok(v.trunc());
    }
    parse_num(s, p)
}

fn parse_num(s: &str, p: &mut usize) -> Result<f64, String> {
    let b = s.as_bytes();
    let start = *p;
    if *p < b.len() && b[*p] == b'-' {
        *p += 1;
    }
    while *p < b.len() && (b[*p].is_ascii_digit() || b[*p] == b'.') {
        *p += 1;
    }
    if *p < b.len() && (b[*p] == b'e' || b[*p] == b'E') {
        let exp_mark = *p;
        *p += 1;
        if *p < b.len() && (b[*p] == b'+' || b[*p] == b'-') {
            *p += 1;
        }
        let dstart = *p;
        while *p < b.len() && b[*p].is_ascii_digit() {
            *p += 1;
        }
        if dstart == *p {
            *p = exp_mark;
        }
    }
    if *p == start {
        return Err(format!("无法解析数字: {}", &s[start..]));
    }
    s[start..*p].parse::<f64>().map_err(|e| e.to_string())
}

/// cof/interval 公式求值：tick→1（首跳，`1.12 ** (tick-1)` → 1）、
/// recipe_XXX(_1) → 0（未点奇穴/秘籍）后整体求值
fn eval_cof(raw: &str) -> Option<f64> {
    let re = Regex::new(r"recipe_\d+(_\d+)?").unwrap();
    let expr = re.replace_all(raw.trim(), "0").to_string();
    let expr = expr.replace("tick", "1");
    eval_expr(&expr).ok()
}

/// 从 cof 公式提取 dot 递增：`X ** (tick - N)` → X - 1（等比系数）；无则 None
fn extract_dot_up(cof_raw: &str) -> Option<f32> {
    let re = Regex::new(r"(\d+(?:\.\d+)?) \*\* \(tick - \d+\)").unwrap();
    let cap = re.captures(cof_raw)?;
    let x: f64 = cap[1].parse().ok()?;
    Some((x - 1.0) as f32)
}

// ============================================================================
// 基础伤害提取
// ============================================================================

const BASE_KEYS: [&str; 4] = [
    "lunar_damage_base",
    "solar_damage_base",
    "neutral_damage_base",
    "physical_damage_base",
];
const RAND_KEYS: [&str; 4] = [
    "lunar_damage_rand",
    "solar_damage_rand",
    "neutral_damage_rand",
    "physical_damage_rand",
];

fn src_num(v: &Value) -> Option<f64> {
    match v {
        Value::String(s) => s.trim().parse::<f64>().ok(),
        Value::Number(n) => n.as_f64(),
        _ => None,
    }
}

/// 从 source_attribute 提取 (base, base+rand)；无则 (0, 0)
fn extract_base(src: &Value) -> (u32, u32) {
    if let Some(obj) = src.as_object() {
        let base = BASE_KEYS.iter().find_map(|k| obj.get(*k).and_then(src_num));
        if let Some(b) = base {
            let rand = RAND_KEYS
                .iter()
                .find_map(|k| obj.get(*k).and_then(src_num))
                .unwrap_or(0.0);
            return (b as u32, (b + rand) as u32);
        }
    }
    (0, 0)
}

// ============================================================================
// 三层结构工具
// ============================================================================

/// lv 层数字键（过滤 str 叶子等非数字键）
fn parse_keys(lvls: &Value) -> Vec<u32> {
    let mut ks: Vec<u32> = lvls
        .as_object()
        .map(|o| o.keys().filter_map(|k| k.parse::<u32>().ok()).collect())
        .unwrap_or_default();
    ks.sort_unstable();
    ks
}

/// 提取技能实体：返回 (cof, base1, base2, wuzhi_by_data, custom)
/// wuzhi_by_data：数据源 critical_strike/critical_power 任一为 "0"/"O" 占位（无质）。
/// 新数据源（骤风令更新后）critical 平铺在 damages 元素内，node 层无 critical 对象。
fn extract_skill(obj: &Value) -> (Option<f64>, u32, u32, bool, bool) {
    let attr = obj.get("skill_attribute").and_then(|v| v.as_object());
    let cof_raw = attr.and_then(|a| {
        [
            "magical_attack_power_cof",
            "physical_attack_power_cof",
            "surplus_cof",
        ]
        .iter()
        .find_map(|k| a.get(*k).and_then(|v| v.as_str()).filter(|s| !s.is_empty()))
    });
    let cof = cof_raw.and_then(eval_cof);
    let custom = attr.is_some_and(|a| {
        a.get("custom_damage_base")
            .is_some_and(|v| v.as_str().is_some_and(|s| !s.is_empty()))
    });
    // critical 占位检测：优先 damages[0]，兼容旧结构 node.critical
    let is_placeholder = |s: &str| s == "0" || s == "O";
    let wuzhi_by_data = if let Some(d) = obj
        .get("damages")
        .and_then(|d| d.as_array())
        .and_then(|a| a.first())
    {
        d.get("critical_strike")
            .and_then(|v| v.as_str())
            .is_some_and(is_placeholder)
            || d.get("critical_power")
                .and_then(|v| v.as_str())
                .is_some_and(is_placeholder)
    } else {
        let c = obj.get("critical");
        c.and_then(|c| c.get("critical_strike"))
            .and_then(|v| v.as_str())
            .is_some_and(is_placeholder)
            || c.and_then(|c| c.get("critical_power"))
                .and_then(|v| v.as_str())
                .is_some_and(is_placeholder)
    };
    let (mut b1, mut b2) = extract_base(obj.get("source_attribute").unwrap_or(&Value::Null));
    if b1 == 0
        && let Some(d) = obj
            .get("damages")
            .and_then(|d| d.as_array())
            .and_then(|a| a.first())
            .and_then(|d| d.get("source_attribute"))
    {
        (b1, b2) = extract_base(d);
    }
    (cof, b1, b2, wuzhi_by_data, custom)
}

// ============================================================================
// 主流程
// ============================================================================

fn main() {
    let args = Args::parse();
    let overrides: Overrides = fs::read_to_string(&args.overrides)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let skills_root: Value =
        serde_json::from_str(&fs::read_to_string(&args.skills).expect("读取 skills.json")).unwrap();
    let dots_root: Value =
        serde_json::from_str(&fs::read_to_string(&args.dots).expect("读取 dots.json")).unwrap();

    for pool_id in &args.xinfa {
        let pool = match skills_root.get(pool_id) {
            Some(p) => p,
            None => {
                eprintln!("[warn] 池 {pool_id} 不存在，跳过");
                continue;
            }
        };
        let ov = overrides.pools.get(pool_id);
        let mut skipped: Vec<String> = Vec::new();
        let mut skills: Vec<SkillOut> = Vec::new();

        collect_skills(pool, &mut skills, &mut skipped, ov);
        if let Some(dots_pool) = dots_root.get(pool_id) {
            collect_dots(dots_pool, &mut skills, &mut skipped);
        }

        let file_name = ov
            .and_then(|o| o.file.clone())
            .unwrap_or_else(|| format!("{pool_id}.toml"));
        let out = render_toml(&args, ov, &skills);
        let out_path = args.out.join(&file_name);
        match fs::write(&out_path, &out) {
            Ok(()) => println!("[ok] {file_name}: {} 条技能", skills.len()),
            Err(e) => eprintln!("[fail] 写入 {file_name}: {e}"),
        }
        if !skipped.is_empty() {
            println!("  [skip] {} 条跳过（无伤害数据）:", skipped.len());
            for s in &skipped {
                println!("    - {s}");
            }
        }
    }
}

fn collect_skills(
    pool: &Value,
    out: &mut Vec<SkillOut>,
    skipped: &mut Vec<String>,
    ov: Option<&PoolOverride>,
) {
    if let Some(subs_map) = pool.as_object() {
        for (sid, subs) in subs_map {
            // sid 级 name 计数：同 sid 跨 sub 统计（单持/双持等多 sub 同名需加后缀区分）
            let mut name_counts: BTreeMap<String, u32> = BTreeMap::new();
            if let Some(lvls_map) = subs.as_object() {
                for (_, lvls) in lvls_map {
                    if let Some(l) = lvls.as_object() {
                        for (_, v) in l {
                            if let Some(name) = v.get("name").and_then(|v| v.as_str()) {
                                *name_counts.entry(name.to_string()).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
            if let Some(lvls_map) = subs.as_object() {
                for (sub, lvls) in lvls_map {
                    let ks = parse_keys(lvls);
                    if ks.is_empty() {
                        continue;
                    }
                    // 全量输出：保留所有 lv 形态（等级/层数/点数均为独立可调形态）
                    let items: Vec<u32> = ks;
                    for lv in items {
                        let obj = match lvls.get(lv.to_string()) {
                            Some(v) if v.is_object() => v,
                            _ => continue,
                        };
                        let name = obj
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if name.is_empty() {
                            continue;
                        }
                        let display_base = ov
                            .and_then(|o| o.rename.get(&name))
                            .cloned()
                            .unwrap_or_else(|| name.clone());
                        let (cof, b1, b2, wuzhi_by_data, custom) = extract_skill(obj);
                        let sid_u = sid.parse::<u32>().unwrap_or(0);
                        let sub_u = sub.parse::<u32>().unwrap_or(0);
                        if cof.is_none() && b1 == 0 && !custom {
                            skipped.push(format!("{name} (sid={sid_u} sub={sub_u})"));
                            continue;
                        }
                        let wuzhi_list = ov.map(|o| o.wuzhi.as_slice()).unwrap_or(&[]);
                        let wuzhi = wuzhi_by_data || wuzhi_list.iter().any(|w| w == &name);
                        let comment = obj
                            .get("comment")
                            .and_then(|v| v.as_str())
                            .filter(|c| !c.is_empty() && *c != name)
                            .map(|c| c.to_string());
                        let display_name = if name_counts.get(&name).copied().unwrap_or(0) > 1 {
                            match &comment {
                                Some(c) => format!("{display_base}·{c}"),
                                None => format!("{display_base}(lv{lv})"),
                            }
                        } else {
                            display_base.clone()
                        };
                        // 追加真伤形态展开：命中 lost_hp_zhenshi 名单的原始名 →
                        // 破绽0..=max_layers 层 形态（0 层 = 纯主伤害）
                        let lost_cfgs = ov.map(|o| o.lost_hp_zhenshi.clone()).unwrap_or_default();
                        if let Some(cfg) = lost_cfgs.iter().find(|c| c.name == name) {
                            for layer in 0..=cfg.max_layers {
                                let lost = cfg.per_layer * layer as f32;
                                let layer_name = if cfg.max_layers > 0 {
                                    format!("{display_name}·破绽{layer}层")
                                } else {
                                    display_name.clone()
                                };
                                out.push(SkillOut {
                                    name: layer_name,
                                    sid: sid_u,
                                    sub: sub_u,
                                    base1: b1,
                                    base2: b2,
                                    atk_xishu: cof.map(|c| c as f32),
                                    wuzhi,
                                    zhenshi: custom,
                                    lost_hp: lost,
                                    dot: None,
                                });
                            }
                        } else {
                            out.push(SkillOut {
                                name: display_name,
                                sid: sid_u,
                                sub: sub_u,
                                base1: b1,
                                base2: b2,
                                atk_xishu: cof.map(|c| c as f32),
                                wuzhi,
                                zhenshi: custom,
                                lost_hp: 0.0,
                                dot: None,
                            });
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn collect_dots(dots_pool: &Value, out: &mut Vec<SkillOut>, skipped: &mut Vec<String>) {
    // (sid, sub, name, (base1, base2), cof, interval_sec, max_tick, dot_up)
    let mut raw_dots: Vec<(
        u32,
        u32,
        String,
        (u32, u32),
        Option<f32>,
        f32,
        u32,
        Option<f32>,
    )> = Vec::new();

    if let Some(subs_map) = dots_pool.as_object() {
        for (sid, subs) in subs_map {
            if let Some(lvls_map) = subs.as_object() {
                for (sub, lvls) in lvls_map {
                    let ks = parse_keys(lvls);
                    if ks.is_empty() {
                        continue;
                    }
                    // 全量输出：保留所有 lv 形态（与 collect_skills 一致）
                    let items: Vec<u32> = ks;
                    for lv in items {
                        let item = match lvls.get(lv.to_string()) {
                            Some(v) if v.is_object() => v,
                            _ => continue,
                        };
                        let name = item
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if name.is_empty() {
                            continue;
                        }
                        let interval_frames = item
                            .get("interval")
                            .and_then(|v| v.as_str())
                            .and_then(eval_cof)
                            .filter(|f| *f > 0.0);
                        let interval_frames = match interval_frames {
                            Some(f) => f,
                            None => {
                                skipped.push(format!("dot {name}: interval 解析失败"));
                                continue;
                            }
                        };
                        let max_tick = item
                            .get("max_tick")
                            .and_then(src_num)
                            .map(|f| f as u32)
                            .unwrap_or(0);
                        if max_tick == 0 {
                            skipped.push(format!("dot {name}: max_tick 缺失"));
                            continue;
                        }
                        // 伤害参数：dots.json 的 skills[源技能id][lv] 子条目自带全部
                        // 信息（base / cof / dot_up），每 source 一条
                        let mut base = (0u32, 0u32);
                        let mut cof: Option<f32> = None;
                        let mut dot_up: Option<f32> = None;
                        if let Some(srcs) = item.get("skills").and_then(|v| v.as_object()) {
                            for sk_lvls in srcs.values() {
                                let src_lv = parse_keys(sk_lvls).last().copied().unwrap_or(0);
                                let src_obj = match sk_lvls.get(src_lv.to_string()) {
                                    Some(v) if v.is_object() => v,
                                    _ => continue,
                                };
                                // base：damages[0].source_attribute（dot 无 rand）
                                let (b1, b2) = src_obj
                                    .get("damages")
                                    .and_then(|d| d.as_array())
                                    .and_then(|a| a.first())
                                    .and_then(|d| d.get("source_attribute"))
                                    .map(extract_base)
                                    .unwrap_or((0, 0));
                                if b1 > 0 {
                                    base = (b1, b2);
                                }
                                // cof + dot_up：skill_attribute.magical_attack_power_cof
                                let cof_raw = src_obj
                                    .get("skill_attribute")
                                    .and_then(|a| a.get("magical_attack_power_cof"))
                                    .and_then(|v| v.as_str());
                                if let Some(raw) = cof_raw {
                                    if let Some(c) = eval_cof(raw) {
                                        cof = Some(c as f32);
                                    }
                                    dot_up = dot_up.or_else(|| extract_dot_up(raw));
                                }
                                if base.0 > 0 || cof.is_some() {
                                    break;
                                }
                            }
                        }
                        let interval_sec = (interval_frames / 16.0) as f32;
                        raw_dots.push((
                            sid.parse::<u32>().unwrap_or(0),
                            sub.parse::<u32>().unwrap_or(0),
                            name.replace("(DOT)", "（dot）"),
                            base,
                            cof,
                            interval_sec,
                            max_tick,
                            dot_up,
                        ));
                    }
                }
            }
        }
    }

    for (sid, sub, name, base, cof, interval_sec, max_tick, dot_up) in raw_dots {
        let (b1, b2) = base;
        out.push(SkillOut {
            name,
            sid,
            sub,
            base1: b1,
            base2: b2,
            atk_xishu: cof,
            wuzhi: false,
            zhenshi: false,
            lost_hp: 0.0,
            dot: Some(DotOut {
                interval_sec,
                duration_sec: max_tick as f32 * interval_sec,
                dot_up,
            }),
        });
    }
}

fn render_toml(args: &Args, ov: Option<&PoolOverride>, skills: &[SkillOut]) -> String {
    let mut s = String::new();
    s.push_str("[xinfa]\n");
    if let Some(o) = ov {
        s.push_str(&format!("xinfa_name = \"{}\"\n", o.xinfa_name));
        s.push_str(&format!("xinfa_nom = \"{}\"\n", o.xinfa_nom));
        s.push_str(&format!("atk_up = {}\n", fmt_f32(o.atk_up.unwrap_or(0.0))));
        s.push_str(&format!(
            "pofang_up = {}\n",
            fmt_f32(o.pofang_up.unwrap_or(0.0))
        ));
        s.push_str(&format!(
            "huixin_up = {}\n",
            fmt_f32(o.huixin_up.unwrap_or(0.0))
        ));
    } else {
        s.push_str("xinfa_name = \"未知\"\n");
        s.push_str("xinfa_nom = \"\"\n");
        s.push_str("atk_up = 0\n");
        s.push_str("pofang_up = 0\n");
        s.push_str("huixin_up = 0\n");
    }
    s.push('\n');
    s.push_str("[version]\n");
    s.push_str(&format!("level = {}\n", args.level));
    s.push_str(&format!("season = {}\n", args.season));
    s.push_str(&format!("modified = {}\n", args.modified));
    s.push('\n');
    for sk in skills {
        s.push_str("[[skill]]\n");
        s.push_str(&format!("skill_name = \"{}\"           #名字\n", sk.name));
        if sk.sid != 0 {
            s.push_str(&format!(
                "skill_id = {}            #技能ID（数据源池内 id）\n",
                sk.sid
            ));
        }
        if sk.sub != 0 {
            s.push_str(&format!("sub_id = {}              #子ID\n", sk.sub));
        }
        if sk.base1 != 0 || sk.base2 != 0 {
            s.push_str(&format!("base_damage1 = {}          #基本伤害\n", sk.base1));
            s.push_str(&format!(
                "base_damage2 = {}          #基本伤害2\n",
                sk.base2
            ));
        }
        if let Some(atk) = sk.atk_xishu {
            s.push_str(&format!("atk_xishu = {}        #伤害系数\n", fmt_f32(atk)));
        } else {
            s.push_str("atk_xishu = 0        #伤害系数\n");
        }
        if sk.wuzhi {
            s.push_str("has_critical_strike = true   #无质（伤害固定=期望 Q）\n");
        }
        if sk.zhenshi {
            s.push_str("zhenshishanghai = 1          #真实伤害\n");
        }
        if sk.lost_hp > 0.0 {
            s.push_str(&format!(
                "lost_hp_zhenshishanghai = {} #追加真伤（已损失生命值×系数）\n",
                fmt_f32(sk.lost_hp)
            ));
        }
        if let Some(dot) = &sk.dot {
            s.push_str("dot_flag = 1                #dot伤害标签\n");
            s.push_str(&format!(
                "dot_interval = {}          #每跳间隔（秒，16帧=1秒）\n",
                fmt_f32(dot.interval_sec)
            ));
            s.push_str(&format!(
                "dot_duration = {}          #持续时长（秒，总跳数×间隔）\n",
                fmt_f32(dot.duration_sec)
            ));
            if let Some(up) = dot.dot_up {
                s.push_str(&format!(
                    "dot_up = {}                 #每跳递增比例（等比）\n",
                    fmt_f32(up)
                ));
            }
        }
        s.push('\n');
    }
    s
}

/// f32 最短往返表示（16 分位二进制精确，无浮点噪声）
fn fmt_f32(v: f32) -> String {
    format!("{}", v)
}
