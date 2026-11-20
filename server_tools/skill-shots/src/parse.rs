//! 从归一化后的 OCR 文本里抽取技能 tooltip / 属性面板的字段。
//!
//! 设计原则：**宁可留空也不要猜**。每个字段都是 `Option`，抽不到就 `None`，
//! 并在 `warnings` 里说明；原始文本始终保留，方便人工复核。

use regex::Regex;
use serde::{Deserialize, Serialize};

/// 伤害属性（内功五行 + 外功）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Element {
    Outer,
    Yang,
    Hunyuan,
    Yin,
    Du,
    /// 只写了「内功」没写五行
    Inner,
    /// 无情/无属性等其它
    Other,
}

impl Element {
    /// 从文案里识别属性；"混元性内功" 要优先于 "内功" 匹配。
    pub fn detect(text: &str) -> Option<Element> {
        const TABLE: &[(&str, Element)] = &[
            ("混元性内功", Element::Hunyuan),
            ("阳性内功", Element::Yang),
            ("阴性内功", Element::Yin),
            ("毒性内功", Element::Du),
            ("混元内功", Element::Hunyuan),
            ("外功", Element::Outer),
            ("无情", Element::Other),
            ("内功", Element::Inner),
        ];
        for (needle, el) in TABLE {
            if text.contains(needle) {
                return Some(*el);
            }
        }
        None
    }
}

/// 技能 tooltip 中抽取到的结构化字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tooltip {
    /// 技能名（优先取文件名；OCR 文本只作辅助）
    pub name: Option<String>,
    /// 是否有「无调息」字样（即不占公共调息）
    pub no_cooldown: bool,
    pub element: Option<Element>,
    /// 基础伤害区间（tooltip 左侧「80-90」）
    pub base_min: Option<u32>,
    pub base_max: Option<u32>,
    /// 攻击力贡献（tooltip 括号里「+22018」）
    pub attack_bonus: Option<u32>,
    /// 释放时间（秒）
    pub cast_time: Option<f32>,
    /// 括号里的加速修正（如 -0.38 秒），负值表示已加速
    pub cast_haste_delta: Option<f32>,
    /// 调息/充能时间（秒）
    pub cooldown: Option<f32>,
    /// 充能层数（「充能：3 层」）
    pub charges: Option<u32>,
    /// 距离（尺）
    pub range: Option<u32>,
    /// 内力消耗描述
    pub cost: Option<String>,
    /// 秘籍/奇穴里的数值型修正，例如 ("伤害提高", 4.0, "%")
    pub modifiers: Vec<Modifier>,
    /// 文案里被引用的技能/招式名（「宫」「笑傲光阴」），奇穴与秘籍用来定位作用对象
    pub targets: Vec<String>,
    /// 原始归一化文本，永远保留
    pub raw: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modifier {
    pub subject: String,
    pub verb: String,
    pub value: f32,
    pub unit: String,
}

/// 属性面板（攻击力页面）里抽取到的最终攻击力。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttackPanel {
    pub outer: Option<u32>,
    pub yang: Option<u32>,
    pub hunyuan: Option<u32>,
    pub yin: Option<u32>,
    pub du: Option<u32>,
    /// 武器伤害区间（外功技能会用到）
    pub weapon_damage_min: Option<u32>,
    pub weapon_damage_max: Option<u32>,
    pub raw: String,
    pub warnings: Vec<String>,
}

impl AttackPanel {
    pub fn attack_for(&self, element: Element) -> Option<u32> {
        match element {
            Element::Outer => self.outer,
            Element::Yang => self.yang,
            Element::Hunyuan => self.hunyuan,
            Element::Yin => self.yin,
            Element::Du => self.du,
            // 没写五行的「内功」取最大值兜底
            Element::Inner => {
                let mut vals: Vec<u32> = [self.yang, self.hunyuan, self.yin, self.du]
                    .into_iter()
                    .flatten()
                    .collect();
                vals.sort_unstable();
                vals.pop()
            }
            Element::Other => None,
        }
    }
}

fn re(p: &str) -> Regex {
    Regex::new(p).expect("内置正则必须能编译")
}

/// 解析技能 tooltip。`name_hint` 一般来自文件名（不含扩展名）。
pub fn parse_tooltip(text: &str, name_hint: Option<&str>) -> Tooltip {
    let mut t = Tooltip {
        name: name_hint.map(str::to_string),
        raw: text.to_string(),
        ..Default::default()
    };

    t.no_cooldown = text.contains("无调息") || text.contains("不占调息");

    // 伤害：造成80-90点(+22018)阴性内功伤害 / 造成123点(+456)外功伤害
    let dmg = re(r"造成(\d+)-(\d+)点\(\+(\d+)\)");
    let dmg_single = re(r"造成(\d+)点\(\+(\d+)\)");
    if let Some(c) = dmg.captures(text) {
        t.base_min = c[1].parse().ok();
        t.base_max = c[2].parse().ok();
        t.attack_bonus = c[3].parse().ok();
    } else if let Some(c) = dmg_single.captures(text) {
        t.base_min = c[1].parse().ok();
        t.base_max = t.base_min;
        t.attack_bonus = c[2].parse().ok();
    } else if let Some(c) = re(r"(\d+)-(\d+)点\(\+(\d+)\)").captures(text) {
        // 绝招/奇穴的「主动效果：850-875点(+92291)」不带「造成」前缀
        t.base_min = c[1].parse().ok();
        t.base_max = c[2].parse().ok();
        t.attack_bonus = c[3].parse().ok();
    } else {
        t.warnings.push("未找到「造成X-Y点(+Z)」伤害结构".into());
    }

    // 属性：在伤害数字之后的 24 个字符窗口里找，避免把「内功防御」之类误判
    if let Some(pos) = text.find("点(+") {
        let tail: String = text[pos..].chars().take(60).collect();
        t.element = Element::detect(&tail);
    }
    if t.element.is_none() {
        t.element = Element::detect(text);
    }

    // 释放：1.75秒(-0.38秒)；OCR 常把「释放」认错，退化只认「X秒(-Y秒)」
    let cast = re(r"释放:(\d+(?:\.\d+)?)秒(?:\((-?\d+(?:\.\d+)?)秒\))?");
    if let Some(c) = cast.captures(text) {
        t.cast_time = c[1].parse().ok();
        t.cast_haste_delta = c.get(2).and_then(|m| m.as_str().parse().ok());
    } else if let Some(c) = re(r"(\d+(?:\.\d+)?)秒\((-?\d+(?:\.\d+)?)秒\)").captures(text) {
        t.cast_time = c[1].parse().ok();
        t.cast_haste_delta = c[2].parse().ok();
    }

    // 调息/充能
    let cd = re(r"(?:调息|充能)[:：]?(\d+(?:\.\d+)?)秒");
    if let Some(c) = cd.captures(text) {
        t.cooldown = c[1].parse().ok();
    }
    let charges = re(r"充能:(\d+)层");
    if let Some(c) = charges.captures(text) {
        t.charges = c[1].parse().ok();
    }

    // 距离
    let range = re(r"距离[^0-9]{0,3}(\d+)尺");
    if let Some(c) = range.captures(text) {
        t.range = c[1].parse().ok();
    }

    // 内力消耗
    if text.contains("不消耗内力") {
        t.cost = Some("不消耗内力".into());
    } else if let Some(c) = re(r"消耗(\d+)点内力").captures(text) {
        t.cost = Some(format!("{}点内力", &c[1]));
    }

    t.modifiers = parse_modifiers(text);
    t.targets = parse_targets(text);
    t
}

/// 合并多 psm 的 OCR 结果时同一段话常被识别两遍（细节略有差异），
/// 按句子去重：保留第一次出现的、去掉后面重复/被包含的句子。
fn dedupe_sentences(text: &str) -> String {
    let mut seen: Vec<String> = Vec::new();
    let mut out: Vec<String> = Vec::new();
    for raw in text.split(['。', '\n', '；', ';']) {
        let t = raw.trim().trim_matches(|c: char| c.is_whitespace());
        if t.chars().count() < 4 {
            continue;
        }
        let key: String = t
            .chars()
            .filter(|c| ('\u{4e00}'..='\u{9fa5}').contains(c) || c.is_ascii_digit())
            .collect();
        if key.chars().count() < 4 {
            continue;
        }
        // OCR 两次识别的细节会差一两个字（「曲风」vs「曲凡」），用 2-gram 相似度判断
        if seen.iter().any(|s| similar(s, &key)) {
            continue;
        }
        seen.push(key);
        out.push(t.to_string());
    }
    let mut joined = out.join("。");
    if !joined.ends_with('。') && !joined.is_empty() {
        joined.push('。');
    }
    joined
}

/// 2-gram 相似度：用于识别「同一句话的两次 OCR」。
fn similar(a: &str, b: &str) -> bool {
    let grams = |t: &str| -> Vec<String> {
        let cs: Vec<char> = t.chars().collect();
        cs.windows(2).map(|w| w.iter().collect()).collect()
    };
    let ga = grams(a);
    let gb = grams(b);
    if ga.is_empty() || gb.is_empty() {
        return a.contains(b) || b.contains(a);
    }
    let inter = ga.iter().filter(|g| gb.contains(g)).count();
    let union = ga.len() + gb.len() - inter;
    inter * 100 / union.max(1) >= 60
}

/// 奇穴 tooltip：名称 + 主动/被动 + 效果说明。
#[derive(Debug, Default, Serialize)]
pub struct QixueTooltip {
    /// 奇穴名（优先取文件名，识别不到时用正文首行）
    pub name: Option<String>,
    /// 主动招式 / 被动招式
    pub kind: Option<String>,
    /// 效果说明（去掉标题、按钮等噪声后的正文）
    pub text: String,
    /// 文案里引用的技能名
    pub targets: Vec<String>,
    /// 文案里出现的数字（伤害、秒数、层数……），供人工核对
    pub numbers: Vec<f64>,
    /// 绝招类奇穴本身就是可施放技能，额外按技能 tooltip 解析一遍
    pub active: Option<Tooltip>,
    /// OCR 名词纠错记录（由 main 的名词白名单填充，from → to）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub corrections: Vec<(String, String)>,
    pub raw: String,
}

/// 解析奇穴 tooltip 的 OCR 文本。
pub fn parse_qixue_tooltip(text: &str, name_hint: Option<&str>) -> QixueTooltip {
    let mut out = QixueTooltip {
        name: name_hint.map(|s| s.to_string()),
        raw: text.to_string(),
        ..QixueTooltip::default()
    };

    // 面板上方会混进页签、配置名、其它心法入口等噪声；真正的文案从
    // 「被动招式 / 主动招式 / 绝招」这些标记开始，到「装备 / 无界 / beta」结束。
    // OCR 常把「被动招式」认成「被动招却」「被动招=」，所以只匹配到「被动招 / 主动招」。
    let marker = Regex::new(r"(被动招[式却=]?|主动招[式却=]?|绝招)").ok();
    let mut body_text = text.to_string();
    if let Some(re) = &marker
        && let Some(m) = re.find(text)
    {
        let word = m.as_str();
        out.kind = Some(
            if word.starts_with("被动") {
                "被动招式"
            } else if word.starts_with("主动") {
                "主动招式"
            } else {
                "绝招"
            }
            .to_string(),
        );
        body_text = text[m.end()..].to_string();
    }
    if out.kind.is_none()
        && body_text.contains("调息")
        && (body_text.contains("距离") || body_text.contains("武器"))
        && (body_text.contains("伤害") || body_text.contains("效果"))
    {
        // 绝招标签（面板右上角）常被裁掉或漏识别，但正文一定带调息/距离/武器
        out.kind = Some("绝招".to_string());
    }

    // 标题（奇穴名）与标记经常在同一行，去掉开头的杂乱字符与名字本身
    body_text = body_text
        .trim_start_matches(|c: char| c.is_ascii() || c.is_whitespace())
        .to_string();

    // 结尾的「装备 300」「无界武学」「beta版本」等按钮/水印
    if let Some(pos) = ["装备", "无界", "beta", "非最终"]
        .iter()
        .filter_map(|k| body_text.find(k))
        .min()
    {
        body_text.truncate(pos);
    }

    let mut body: Vec<String> = Vec::new();
    for line in body_text.lines() {
        let l = line.trim();
        if l.chars().count() < 2 {
            continue;
        }
        // 整行拉丁字母占比过高的是 OCR 噪声
        let latin = l.chars().filter(|c| c.is_ascii_alphabetic()).count();
        if latin * 3 > l.chars().count() {
            continue;
        }
        let cleaned: String = l
            .chars()
            .filter(|c| !c.is_ascii() || c.is_ascii_digit() || c.is_ascii_punctuation())
            .collect();
        let cleaned = cleaned.trim_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace());
        if cleaned.chars().count() >= 2 {
            body.push(cleaned.to_string());
        }
    }
    out.text = dedupe_sentences(&body.join(""));
    if let Some(name) = &out.name
        && let Some(pos) = out.text.find(name.as_str())
        && pos == 0
    {
        out.text = out.text[name.len()..].trim_start().to_string();
    }
    out.targets = parse_targets(&out.text);
    let num = Regex::new(r"\d+(?:\.\d+)?").ok();
    if let Some(re) = num {
        for c in re.find_iter(&out.text) {
            if let Ok(v) = c.as_str().parse::<f64>() {
                out.numbers.push(v);
            }
        }
    }
    if out.kind.as_deref() == Some("绝招") {
        out.active = Some(parse_tooltip(&out.text, name_hint));
    }
    out
}

/// 抽出文案里用「」或《》括起来的技能名。
pub fn parse_targets(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for pat in [
        r"「([^」]{1,12})」",
        r"《([^》]{1,12})》",
        r"“([^”]{1,12})”",
    ] {
        if let Ok(re) = Regex::new(pat) {
            for c in re.captures_iter(text) {
                let name = c[1].to_string();
                if !out.contains(&name) {
                    out.push(name);
                }
            }
        }
    }
    out
}

/// 抽出「伤害提高4%」「运功时间减少0.125秒」这类数值型修饰。
pub fn parse_modifiers(text: &str) -> Vec<Modifier> {
    let mut out = Vec::new();
    let modifier =
        re(r"([\u{4e00}-\u{9fa5}]{2,10}?)(提高|提升|增加|降低|减少)(\d+(?:\.\d+)?)(%|点|秒|层|尺)");
    for c in modifier.captures_iter(text) {
        out.push(Modifier {
            subject: c[1].to_string(),
            verb: c[2].to_string(),
            value: c[3].parse().unwrap_or_default(),
            unit: c[4].to_string(),
        });
    }
    out
}

/// 秘籍选择页的一张秘籍卡。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MijiEntry {
    /// 《音·宫》里的「音·宫」
    pub group: String,
    /// 秘籍名，如「经脉图残页」，抽不到时为空
    pub paper: String,
    /// 卡片上的效果原文
    pub effect: String,
    pub modifiers: Vec<Modifier>,
    /// 是否标着「已装备」
    pub equipped: bool,
}

/// 解析「秘籍选择」整页截图：一页会同时列出该技能所有秘籍，一网打尽。
///
/// 文本形如：
/// `秘籍选择(4/4)《音·宫》经脉图残页运功时间减少0.125秒。已装备《音·宫》参悟断篇伤害提高4%。`
pub fn parse_miji_page(text: &str) -> Vec<MijiEntry> {
    let mut out = Vec::new();
    let paper_re =
        re(r"^([\u{4e00}-\u{9fa5}·]{1,10}?(?:残页|断篇|绝章|秘章|真传|初篇|中篇|末篇|全篇))");
    let mut search = text;
    while let Some(open) = search.find('《') {
        let after_open = &search[open + '《'.len_utf8()..];
        let Some(close) = after_open.find('》') else {
            break;
        };
        let group = after_open[..close].to_string();
        let rest = &after_open[close + '》'.len_utf8()..];
        // 卡片到下一个《 为止（不同 psm 拼接可能带换行）
        let chunk_end = rest.find('《').unwrap_or(rest.len());
        let chunk = &rest[..chunk_end];
        let equipped = chunk.contains("已装备");
        let chunk = chunk.replace("已装备", "");

        let (paper, body) = match paper_re.captures(&chunk) {
            Some(c) => {
                let paper = c[1].to_string();
                (paper.clone(), chunk[paper.len()..].to_string())
            }
            None => (String::new(), chunk.clone()),
        };
        let effect: String = body
            .split(['。', ';', '；'])
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        if effect.is_empty() && paper.is_empty() {
            search = rest;
            continue;
        }
        out.push(MijiEntry {
            group,
            paper,
            modifiers: parse_modifiers(&effect),
            effect,
            equipped,
        });
        search = rest;
    }
    out
}

/// 解析属性面板里的「最终XX攻击」。
pub fn parse_panel(text: &str) -> AttackPanel {
    let mut p = AttackPanel {
        raw: text.to_string(),
        ..Default::default()
    };
    let fin = re(r"最终(外功|阳性内功|混元性内功|阴性内功|毒性内功)攻击(\d+)");
    for c in fin.captures_iter(text) {
        let v: u32 = match c[2].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        match &c[1] {
            "外功" => p.outer = Some(v),
            "阳性内功" => p.yang = Some(v),
            "混元性内功" => p.hunyuan = Some(v),
            "阴性内功" => p.yin = Some(v),
            "毒性内功" => p.du = Some(v),
            _ => {}
        }
    }
    if let Some(c) = re(r"基础武器伤害(\d+)\((\d+)-(\d+)\)").captures(text) {
        p.weapon_damage_min = c[2].parse().ok();
        p.weapon_damage_max = c[3].parse().ok();
    }
    if p.outer.is_none()
        && p.yang.is_none()
        && p.hunyuan.is_none()
        && p.yin.is_none()
        && p.du.is_none()
    {
        p.warnings
            .push("未找到任何「最终XX攻击」字段，确认这是属性面板截图吗？".into());
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    // 来自真实 tesseract 输出（宫 tooltip 截图）的归一化片段
    const GONG: &str = "宫无调息距离:20尺不消耗内力武器:琴释放:1.75秒(-0.38秒)殊影:宫·鸾鸣歌揉弦捻孤韵，喑喑语沉吟。对目标造成80-90点(+22018)阴性内功伤害。伤害提高4%。会心提高4%。攻击技，可在远距离外对目标造成大量伤害。";

    #[test]
    fn parses_gong_tooltip() {
        let t = parse_tooltip(GONG, Some("宫"));
        assert_eq!(t.name.as_deref(), Some("宫"));
        assert!(t.no_cooldown);
        assert_eq!(t.element, Some(Element::Yin));
        assert_eq!(
            (t.base_min, t.base_max, t.attack_bonus),
            (Some(80), Some(90), Some(22018))
        );
        assert_eq!(t.cast_time, Some(1.75));
        assert_eq!(t.cast_haste_delta, Some(-0.38));
        assert_eq!(t.range, Some(20));
        assert_eq!(t.cost.as_deref(), Some("不消耗内力"));
        assert!(
            t.modifiers
                .iter()
                .any(|m| m.subject == "伤害" && m.value == 4.0)
        );
    }

    #[test]
    fn parses_panel_attacks() {
        let raw = "攻击力8438基础武器伤害57(43-71)基础外功攻击3,最终外功攻击3基础阳性内功攻击2457,最终阳性内功攻击2457基础混元性内功攻击2457,最终混元性内功攻击2457基础阴性内功攻击6604,最终阴性内功攻击8438基础毒性内功攻击2457,最终毒性内功攻击2457";
        let p = parse_panel(raw);
        assert_eq!(p.outer, Some(3));
        assert_eq!(p.yang, Some(2457));
        assert_eq!(p.hunyuan, Some(2457));
        assert_eq!(p.yin, Some(8438));
        assert_eq!(p.du, Some(2457));
        assert_eq!(p.weapon_damage_min, Some(43));
        assert_eq!(p.weapon_damage_max, Some(71));
    }

    #[test]
    fn missing_damage_is_reported_not_guessed() {
        let t = parse_tooltip("宫无调息距离:20尺", Some("宫"));
        assert_eq!(t.attack_bonus, None);
        assert!(t.warnings.iter().any(|w| w.contains("未找到")));
    }

    #[test]
    fn qixue_juemoves_are_parsed_as_active_skill() {
        let raw = "不愧君 调息20秒\n距离:20尺 不消耗内力\n武器:琴 瞬间释放\n被动效果:自身音域范围内最多5名敌对侠士目标受到的治疗效果降低45%。\n主动效果:凝聚“青鸾剑”对目标造成850-875点(+92291)[无质]阴性内功伤害。\n装备 300";
        let t = parse_qixue_tooltip(raw, Some("不愧君"));
        assert_eq!(t.kind.as_deref(), Some("绝招"));
        let a = t.active.expect("绝招应能按技能 tooltip 解析");
        assert_eq!(a.base_min, Some(850));
        assert_eq!(a.base_max, Some(875));
        assert_eq!(a.attack_bonus, Some(92291));
        assert_eq!(a.cooldown, Some(20.0));
        assert_eq!(a.range, Some(20));
    }

    #[test]
    fn qixue_tooltip_keeps_effect_text_and_numbers() {
        let raw = "洞天\n被动招式\n“变徵”需要消耗3点曲风方可施展，不再需要运功，改为奏出琴音环绕目标，\n在1.5秒内造成3次(39114点)阴性内功伤害，该效果不受其他奇穴影响。\n装备 300";
        let t = parse_qixue_tooltip(raw, Some("洞天"));
        assert_eq!(t.name.as_deref(), Some("洞天"));
        assert_eq!(t.kind.as_deref(), Some("被动招式"));
        assert!(t.text.contains("1.5秒内造成3次"));
        assert!(t.targets.contains(&"变徵".to_string()));
        assert!(t.numbers.contains(&39114.0));
        assert!(!t.text.contains("装备"));
    }
}
