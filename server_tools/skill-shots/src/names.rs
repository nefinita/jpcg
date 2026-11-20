//! 名词白名单 + 近似纠错。
//!
//! OCR 对游戏专有名词的误读很稳定：变徵→变惩、曲风→曲凡、青鸾剑→青弯剑。
//! 这里做两层纠错，每次替换都记录在输出的 `corrections` 里，便于人工核对：
//!
//! 1. 引号里的技能名（“变惩”）：整段与白名单比对，编辑距离 ≤1 才替换；
//! 2. 高频错词表：只收录实测出现过的固定错法（曲凡→曲风……）。
//!
//! 不做「任意 2 字滑窗模糊替换」——中文两字词太多，那样会把正文改得面目全非。

use std::path::Path;

/// 领域词表：技能名之外的固定术语（曲风、调息、阴性内功……）。
const TERMS: &[&str] = &[
    "曲风",
    "调息",
    "运功",
    "会心",
    "会效",
    "破防",
    "御劲",
    "加速",
    "内力",
    "伤害",
    "治疗",
    "奇穴",
    "秘籍",
    "招式",
    "套路",
    "心法",
    "外功",
    "内功",
    "减伤",
    "护盾",
    "气血",
    "真气",
    "武器",
    "距离",
    "释放",
    "持续",
    "目标",
    "敌对",
    "侠士",
    "层数",
    "命中",
    "闪避",
    "化解",
    "无双",
    "体质",
    "根骨",
    "身法",
    "力道",
    "元气",
    "混元性内功",
    "阴性内功",
    "阳性内功",
    "毒性内功",
    "持续伤害",
    "治疗效果",
    "调息时间",
    "移动速度",
];

/// 实测出现过的高频错词（OCR 稳定认错，直接映射回正确写法）。
const PAIRS: &[(&str, &str)] = &[
    ("变惩", "变徵"),
    ("变征", "变徵"),
    ("曲凡", "曲风"),
    ("招却", "招式"),
    ("青弯", "青鸾"),
    ("攻力", "攻击"),
    ("秘宽", "秘籍"),
    ("奇宂", "奇穴"),
    ("御幼", "御劲"),
    ("破访", "破防"),
];

const OPEN_QUOTES: [char; 4] = ['“', '"', '「', '《'];
const CLOSE_QUOTES: [char; 4] = ['”', '"', '」', '》'];

#[derive(Debug, Default)]
pub struct Whitelist {
    words: Vec<String>,
}

impl Whitelist {
    /// 从 `data/shuxing/*.toml` 读取技能名；目录不存在时只保留内置词表。
    pub fn load(dir: Option<&Path>) -> Self {
        let mut words: Vec<String> = TERMS.iter().map(|s| s.to_string()).collect();
        if let Some(dir) = dir
            && dir.is_dir()
            && let Ok(entries) = std::fs::read_dir(dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "toml") {
                    let Ok(text) = std::fs::read_to_string(&path) else {
                        continue;
                    };
                    for line in text.lines() {
                        let line = line.trim();
                        if let Some(rest) = line.strip_prefix("skill_name")
                            && let Some(v) = rest.split('"').nth(1)
                        {
                            words.push(v.to_string());
                        }
                    }
                }
            }
        }
        words.retain(|w| w.chars().count() >= 2);
        words.sort();
        words.dedup();
        Self { words }
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// 纠错：先套高频错词表，再按白名单纠正引号里的技能名。
    /// 返回 (纠错后文本, 替换列表)。
    pub fn correct(&self, text: &str) -> (String, Vec<(String, String)>) {
        let mut fixes: Vec<(String, String)> = Vec::new();
        // 1. 实测错词（整词替换，优先级最高，避免被近似匹配抢走）
        let mut text = text.to_string();
        for (from, to) in PAIRS {
            if text.contains(from) {
                text = text.replace(from, to);
                let from = from.to_string();
                let to = to.to_string();
                if !fixes.iter().any(|(f, _)| f == &from) {
                    fixes.push((from, to));
                }
            }
        }

        // 2. 引号里的技能名按白名单近似匹配
        let chars: Vec<char> = text.chars().collect();
        let mut out = String::with_capacity(text.len());
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if OPEN_QUOTES.contains(&c) {
                let close = CLOSE_QUOTES[OPEN_QUOTES.iter().position(|q| *q == c).unwrap_or(0)];
                if let Some(end) = (i + 1..chars.len()).find(|&j| chars[j] == close) {
                    let inner: String = chars[i + 1..end].iter().collect();
                    let fixed = self.correct_name(&inner);
                    if fixed != inner {
                        if !fixes.iter().any(|(f, _)| f == &inner) {
                            fixes.push((inner.clone(), fixed.clone()));
                        }
                        out.push(c);
                        out.push_str(&fixed);
                        out.push(chars[end]);
                        i = end + 1;
                        continue;
                    }
                }
            }
            out.push(c);
            i += 1;
        }
        (out, fixes)
    }

    /// 引号内的名字：命中白名单返回原样，否则找编辑距离 ≤1 的白名单词。
    fn correct_name(&self, name: &str) -> String {
        let name = name.trim();
        let n = name.chars().count();
        if !(2..=8).contains(&n) || !name.chars().all(is_cjk) {
            return name.to_string();
        }
        if self.words.iter().any(|w| w == name) {
            return name.to_string();
        }
        let mut best: Option<(usize, &String)> = None;
        for cand in &self.words {
            if cand.chars().count() != n {
                continue;
            }
            let d = edit_distance(name, cand, 2);
            if d <= 1 && best.as_ref().is_none_or(|(bd, _)| d < *bd) {
                best = Some((d, cand));
            }
        }
        best.map_or_else(|| name.to_string(), |(_, s)| s.clone())
    }
}

fn is_cjk(c: char) -> bool {
    ('\u{4e00}'..='\u{9fa5}').contains(&c)
}

/// 编辑距离，超过 `max` 直接返回 `max + 1`（提前剪枝）。
fn edit_distance(a: &str, b: &str, max: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > max {
        return max + 1;
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        if cur.iter().all(|&v| v > max) {
            return max + 1;
        }
        prev = cur;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_whitelist() -> Whitelist {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/shuxing");
        Whitelist::load(Some(&dir))
    }

    #[test]
    fn corrects_quoted_name_and_confusable_words() {
        let w = repo_whitelist();
        let (text, fixes) = w.correct("“变惩”需要消耗3点曲凡方可施展，并造成伤害");
        assert!(text.contains("变徵"), "{text}");
        assert!(text.contains("曲风"), "{text}");
        // 普通正文不能被改：「造成伤害」必须原样保留
        assert!(text.contains("造成伤害"), "{text}");
        assert!(fixes.iter().any(|(f, t)| f == "变惩" && t == "变徵"));
    }

    #[test]
    fn keeps_whitelisted_words_untouched() {
        let w = Whitelist::load(None);
        let (text, fixes) = w.correct("调息20秒，距离20尺，造成阴性内功伤害");
        assert_eq!(text, "调息20秒，距离20尺，造成阴性内功伤害");
        assert!(fixes.is_empty());
    }

    #[test]
    fn does_not_touch_plain_two_char_words() {
        let w = repo_whitelist();
        let (text, _) = w.correct("根据当前曲风数造成阴性内功伤害");
        assert_eq!(text, "根据当前曲风数造成阴性内功伤害");
    }
}
