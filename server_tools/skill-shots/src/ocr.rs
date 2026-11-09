//! 调用系统 tesseract 做文字识别，并做归一化处理。
//!
//! 归一化目标：让 parse.rs 的正则只面对「去空白、半角标点、半角数字」的文本。
//! 游戏 UI 截图里常见全角数字（８４３８）、全角括号（（）、各种破折号（–—－），
//! OCR 有时也会把这些识别成半角/全角混合，因此统一在这里折叠。

use std::path::Path;
use std::process::Command;

/// 全窗截图里 tooltip 面板的大致区域（相对坐标，x0,y0,x1,y1）。
/// 游戏里 tooltip 跟着鼠标走，位置会左右浮动，所以取几块候选区域逐块试。
pub const TOOLTIP_AREAS: [[f32; 4]; 4] = [
    [0.50, 0.10, 0.90, 0.75],
    [0.30, 0.08, 0.72, 0.75],
    [0.52, 0.18, 0.86, 0.70],
    [0.0, 0.0, 1.0, 1.0],
];

/// 在整窗截图里找 tooltip 面板并识别：切几块候选区域，分别放大后 OCR，
/// 取「汉字最多 + 命中关键词最多」的那一块。
pub fn recognize_auto(
    image: &Path,
    tesseract: &str,
    lang: &str,
    psms: &[u8],
) -> Result<String, OcrError> {
    let (w, _h) = crate::pages::image_size(image).unwrap_or((0, 0));
    // 小图（已经是 tooltip 面板）直接识别，不用找区域；过小的图先放大再 OCR
    if w == 0 || w <= 1500 {
        let prepared = if w > 0 && w < 900 {
            crate::pages::upscale(image, 900).unwrap_or_else(|_| image.to_path_buf())
        } else {
            image.to_path_buf()
        };
        return recognize(&prepared, tesseract, lang, psms);
    }

    let mut best: Option<(i32, String)> = None;
    for (i, area) in TOOLTIP_AREAS.iter().enumerate() {
        let Ok(crop) = crate::pages::crop_region(image, *area, &format!("a{i}")) else {
            continue;
        };
        let Ok(text) = recognize(&crop, tesseract, lang, psms) else {
            continue;
        };
        let score = score_text(&text);
        if std::env::var("SKILL_SHOTS_DEBUG").is_ok() {
            eprintln!(
                "[debug] area {i} score={score} text={:?}",
                text.chars().take(120).collect::<String>()
            );
        }
        if best.as_ref().is_none_or(|(s, _)| score > *s) {
            best = Some((score, text));
        }
    }
    match best {
        Some((_, text)) => Ok(text),
        None => Err(OcrError::Spawn("所有候选区域都识别失败".into())),
    }
}

/// 给 OCR 结果打分：汉字数量 + 关键词命中，用来在候选区域里挑最像 tooltip 的那块。
fn score_text(text: &str) -> i32 {
    let cjk = text
        .chars()
        .filter(|c| ('\u{4e00}'..='\u{9fa5}').contains(c))
        .count() as i32;
    let keywords = [
        "造成", "伤害", "提高", "降低", "消耗", "释放", "调息", "距离", "被动", "主动", "效果",
        "奇穴", "曲风", "内力",
    ];
    let hits = keywords.iter().filter(|k| text.contains(**k)).count() as i32;
    // 整窗 OCR 的典型特征是混进大量拉丁字母（图标名、快捷键、beta 水印），
    // 给字母计数做惩罚，避免「汉字最多但全是噪声」的整窗结果胜出。
    let latin = text.chars().filter(|c| c.is_ascii_alphabetic()).count() as i32;
    let long_penalty = if text.chars().count() > 700 { 40 } else { 0 };
    cjk + hits * 20 - latin * 2 - long_penalty
}

/// OCR 失败或 tesseract 不可用时返回的错误。
#[derive(Debug)]
pub enum OcrError {
    /// tesseract 进程启动失败（多半是没安装）
    Spawn(String),
    /// tesseract 返回非零状态
    Status(i32, String),
}

impl std::fmt::Display for OcrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OcrError::Spawn(e) => write!(
                f,
                "无法启动 tesseract：{e}（macOS 可 brew install tesseract tesseract-lang）"
            ),
            OcrError::Status(code, stderr) => write!(f, "tesseract 退出码 {code}：{stderr}"),
        }
    }
}

/// 对单张图片做 OCR，返回**已归一化**的文本。
///
/// 多个 psm 的识别结果会拼接在一起：游戏 tooltip 的排版（图标 + 标题 + 多行说明）
/// 没有一种 psm 能稳定全对——psm 4 更常认出「释放:」，psm 11 更常认出「距离:」，
/// 拼起来再解析能显著提高字段命中率；解析器对重复文本是幂等的（取第一处匹配）。
pub fn recognize(
    image: &Path,
    tesseract: &str,
    lang: &str,
    psms: &[u8],
) -> Result<String, OcrError> {
    // 小图先放大再识别：tooltip 截图的字高通常只有 20~30px，tesseract 在原始尺寸下
    // 很容易把「徵」认成「征」、把百分号吞掉；放大到 1600px 宽后正确率明显提升。
    let upscaled = crate::pages::upscale(image, 1600).unwrap_or_else(|_| image.to_path_buf());
    let image = upscaled.as_path();

    let mut parts = Vec::with_capacity(psms.len());
    for psm in psms {
        let out = Command::new(tesseract)
            .arg(image)
            .arg("stdout")
            .arg("-l")
            .arg(lang)
            .arg("--psm")
            .arg(psm.to_string())
            .output()
            .map_err(|e| OcrError::Spawn(e.to_string()))?;

        if !out.status.success() {
            return Err(OcrError::Status(
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr).trim().to_string(),
            ));
        }
        parts.push(normalize(&String::from_utf8_lossy(&out.stdout)));
    }
    Ok(parts.join("\n"))
}

/// 文本归一化：去所有空白、全角转半角、破折号折叠。
///
/// 规则刻意保守：只处理排版类字符，汉字与英文字母原样保留，避免破坏技能名。
pub fn normalize(text: &str) -> String {
    let mut s = String::with_capacity(text.len());
    for ch in text.chars() {
        let mapped = match ch {
            // 全角数字 / 字母 / 括号 / 冒号 / 百分号 / 加号 / 减号
            '\u{FF10}'..='\u{FF19}' => {
                char::from_u32(ch as u32 - 0xFF10 + '0' as u32).unwrap_or(ch)
            }
            '\u{FF21}'..='\u{FF3A}' => {
                char::from_u32(ch as u32 - 0xFF21 + 'A' as u32).unwrap_or(ch)
            }
            '\u{FF41}'..='\u{FF5A}' => {
                char::from_u32(ch as u32 - 0xFF41 + 'a' as u32).unwrap_or(ch)
            }
            '\u{FF08}' => '(',
            '\u{FF09}' => ')',
            '\u{FF0B}' => '+',
            '\u{FF0D}' => '-',
            '\u{FF1A}' => ':',
            '\u{FF05}' => '%',
            '\u{FF0F}' => '/',
            '\u{FF0C}' => ',',
            '\u{FF0E}' => '.',
            '\u{FF1F}' => '?',
            '\u{FF01}' => '!',
            // 各种破折号/连字符统一成 '-'
            '–' | '—' | '−' | 'ー' => '-',
            // 空白全部丢弃
            c if c.is_whitespace() => continue,
            c => c,
        };
        s.push(mapped);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_fullwidth_and_whitespace() {
        let raw = "武器：琴　释放：１．７５秒\n对目标造成８０－９０点（＋２２０１８）阴性内功伤害";
        assert_eq!(
            normalize(raw),
            "武器:琴释放:1.75秒对目标造成80-90点(+22018)阴性内功伤害"
        );
    }

    #[test]
    fn keeps_chinese_names() {
        assert_eq!(normalize(" 宫 无调息 "), "宫无调息");
    }
}
