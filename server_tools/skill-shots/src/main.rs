//! skill-shots：把人工截的游戏技能/奇穴/秘籍/属性面板截图批量转成结构化数据。
//!
//! 目录约定（由玩家维护，改图替换即可重跑）：
//!
//! ```text
//! <root>/
//!   莫问/
//!     攻击力.png            # 可选：属性面板（最终XX攻击）
//!     技能/
//!       宫/
//!         宫.png            # 技能本身 tooltip
//!         秘籍/
//!           秘籍A.png        # 可选：秘籍/秘籍组合 tooltip
//!       商/...
//!     奇穴/
//!       1/宫-奇穴.png        # 每个位置一张（该位置奇穴说明）
//!       2/...
//! ```
//!
//! 输出：`<out>/<职业>.json` + `<out>/summary.json`，字段见 parse.rs。

mod ocr;
mod pages;
mod parse;

use clap::Parser;
use parse::{AttackPanel, Element, Modifier, Tooltip};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

mod names;

#[derive(Parser)]
#[command(
    author,
    about = "把技能/奇穴/秘籍截图批量 OCR 成结构化 JSON",
    disable_version_flag = true
)]
struct Args {
    /// 截图根目录（下面每个子目录是一个职业）
    #[arg(default_value = "data/skill-shots")]
    root: PathBuf,

    /// 输出目录，默认 <root>/out
    #[arg(long)]
    out: Option<PathBuf>,

    /// tesseract 可执行文件
    #[arg(long, default_value = "tesseract")]
    tesseract: String,

    /// OCR 语言
    #[arg(long, default_value = "chi_sim+eng")]
    lang: String,

    /// tesseract 页面分割模式（多个结果拼接，提高字段命中率）
    #[arg(long, value_delimiter = ',', default_value = "4,11")]
    psms: Vec<u8>,

    /// 倍率基准：当前版本内外功统一为 160
    #[arg(long, default_value_t = 160.0)]
    base: f32,

    /// 只处理某个职业
    #[arg(long)]
    only: Option<String>,

    /// 整页截图（秘籍/奇穴）的列数
    #[arg(long, default_value_t = 4)]
    page_cols: u32,

    /// 整页截图（秘籍/奇穴）的行数
    #[arg(long, default_value_t = 2)]
    page_rows: u32,

    /// 卡片区域占比 x0,y0,x1,y1
    #[arg(long, default_value = "0.02,0.13,0.994,0.815")]
    page_area: String,

    /// 关闭整页切卡（直接整页 OCR）
    #[arg(long)]
    no_grid: bool,

    /// 名词白名单目录（读取 data/shuxing/*.toml 的 skill_name，用于 OCR 纠错）
    #[arg(long, default_value = "data/shuxing")]
    names: PathBuf,
}

/// 一次 OCR 纠错记录（from → to），保留在输出里供人工复核。
#[derive(Debug, Serialize)]
struct Correction {
    from: String,
    to: String,
}

/// 单张截图的识别结果。
#[derive(Debug, Serialize)]
struct Shot {
    file: String,
    tooltip: Tooltip,
    /// OCR 名词纠错记录（空数组表示没改）
    corrections: Vec<Correction>,
    /// 反推倍率 = 攻击力贡献 / 最终攻击力 × 基准
    ratio: Option<f32>,
    /// 取整后的倍率
    ratio_int: Option<u32>,
    /// 与取整值的残差，>0.02 说明要么攻击力读错、要么该技能不是线性倍率
    ratio_residual: Option<f32>,
}

#[derive(Debug, Serialize)]
struct SkillEntry {
    name: String,
    shots: Vec<Shot>,
    miji: Vec<PageShot>,
}

/// 整页截图（秘籍/奇穴）的处理结果：切成若干卡片后逐张识别。
#[derive(Debug, Serialize)]
struct PageShot {
    file: String,
    cards: Vec<Card>,
}

/// 一张卡片的识别结果。
#[derive(Debug, Serialize)]
struct Card {
    cell: String,
    /// 卡片标题（《音·宫》参悟断篇 之类），抽不到就 None
    title: Option<String>,
    /// 效果原文
    effect: Option<String>,
    modifiers: Vec<Modifier>,
    raw: String,
}

#[derive(Debug, Serialize)]
struct QixueShot {
    file: String,
    tooltip: parse::QixueTooltip,
}

/// 混选奇穴：9 个候选里选 3 个。游戏按混池顺序自动排进 7~9 号槽位，这里只记池子元数据。
#[derive(Debug, Serialize)]
struct PoolInfo {
    size: usize,
    pick: usize,
    slot_from: u32,
    slot_to: u32,
}

#[derive(Debug, Serialize)]
struct QixueSlot {
    slot: String,
    /// 「第一重」→ 1 …「第六重」→ 6；混选奇穴为 None（见 pool）
    slot_index: Option<u32>,
    /// 混选奇穴才有：9 选 3 → 7~9 号槽位
    pool: Option<PoolInfo>,
    shots: Vec<QixueShot>,
    /// 绝招类奇穴 = 新增的可施放技能（进 #92 时带 `jihuoqixue = 奇穴名`）
    skills: Vec<QixueActiveSkill>,
    cards: Vec<PageShot>,
}

/// 奇穴激活后新增的主动技能（如「不愧君」）。
#[derive(Debug, Serialize)]
struct QixueActiveSkill {
    /// 技能名（= 奇穴名，游戏里就是这么命名的）
    name: String,
    /// 激活前置：这个技能要求的前置奇穴
    jihuoqixue: String,
    /// 技能 tooltip 解析结果（伤害区间、攻击力贡献、调息、距离、属性）
    active: Tooltip,
    ratio: Option<f32>,
    ratio_int: Option<u32>,
    ratio_residual: Option<f32>,
    corrections: Vec<Correction>,
}

#[derive(Debug, Serialize)]
struct ProfessionReport {
    profession: String,
    panel: Option<AttackPanel>,
    skills: Vec<SkillEntry>,
    qixue: Vec<QixueSlot>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Summary {
    professions: Vec<ProfessionSummary>,
    total_images: usize,
    total_skills: usize,
    total_miji: usize,
    total_qixue: usize,
    warnings: usize,
}

#[derive(Debug, Serialize)]
struct ProfessionSummary {
    profession: String,
    skills: usize,
    miji: usize,
    qixue: usize,
    warnings: usize,
}

fn main() {
    let args = Args::parse();
    let out_dir = args.out.clone().unwrap_or_else(|| args.root.join("out"));

    let whitelist = names::Whitelist::load(Some(&args.names));
    eprintln!(
        "名词白名单：{} 个词（{}）",
        whitelist.len(),
        args.names.display()
    );

    if let Err(e) = fs::create_dir_all(&out_dir) {
        eprintln!("创建输出目录失败：{e}");
        std::process::exit(1);
    }

    let professions = match list_dirs(&args.root) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("读取截图目录 {} 失败：{e}", args.root.display());
            std::process::exit(1);
        }
    };

    let mut summary = Summary {
        professions: Vec::new(),
        total_images: 0,
        total_skills: 0,
        total_miji: 0,
        total_qixue: 0,
        warnings: 0,
    };

    for prof_dir in professions {
        let name = file_name(&prof_dir);
        if name == "out" || args.only.as_ref().is_some_and(|only| only != &name) {
            continue;
        }

        let mut report = ProfessionReport {
            profession: name.clone(),
            panel: None,
            skills: Vec::new(),
            qixue: Vec::new(),
            warnings: Vec::new(),
        };

        // 职业目录下直接放的图片 = 属性面板
        for img in list_images(&prof_dir) {
            match recognize(&args, &img) {
                Ok(text) => {
                    let panel = parse::parse_panel(&text);
                    if !panel.warnings.is_empty() {
                        report.warnings.push(format!(
                            "{}: {}",
                            img.display(),
                            panel.warnings.join("; ")
                        ));
                    }
                    report.panel = Some(panel);
                    summary.total_images += 1;
                }
                Err(e) => report.warnings.push(format!("{}: {e}", img.display())),
            }
        }

        // 技能/<技能名>/{*.png, 秘籍/*.png}
        for skill_dir in list_dirs(&prof_dir.join("技能")).unwrap_or_default() {
            let skill_name = file_name(&skill_dir);
            let mut entry = SkillEntry {
                name: skill_name.clone(),
                shots: Vec::new(),
                miji: Vec::new(),
            };
            for img in list_images(&skill_dir) {
                match shot(&args, &report.panel, &img, Some(&skill_name), &whitelist) {
                    Ok(s) => {
                        summary.total_images += 1;
                        entry.shots.push(s);
                    }
                    Err(e) => report.warnings.push(format!("{}: {e}", img.display())),
                }
            }
            for img in list_images(&skill_dir.join("秘籍")) {
                match page_shot(&report.panel, &args, &img) {
                    Ok(shot) => {
                        summary.total_images += 1;
                        summary.total_miji += shot.cards.len();
                        entry.miji.push(shot);
                    }
                    Err(e) => report.warnings.push(format!("{}: {e}", img.display())),
                }
            }
            if !entry.shots.is_empty() || !entry.miji.is_empty() {
                summary.total_skills += 1;
                report.skills.push(entry);
            }
        }

        // 奇穴/<位置>/*.png
        for slot_dir in list_dirs(&prof_dir.join("奇穴")).unwrap_or_default() {
            let slot = file_name(&slot_dir);
            let slot_index = parse_slot_index(&slot);
            let pool = is_mixed_pool(&slot).then_some(PoolInfo {
                size: 9,
                pick: 3,
                slot_from: 7,
                slot_to: 9,
            });
            let mut qs = QixueSlot {
                slot: slot.clone(),
                slot_index,
                pool,
                shots: Vec::new(),
                skills: Vec::new(),
                cards: Vec::new(),
            };
            for img in list_images(&slot_dir) {
                // 奇穴截图有两种落法：
                //   1) 单个奇穴的 tooltip（推荐，文案完整）；
                //   2) 该位置可选奇穴的整页列表（退化成切卡，只当清单用）。
                // 先 OCR 一次判断是不是 tooltip：奇穴效果文案一定带「造成/提高/消耗」这类动词。
                match recognize_auto(&args, &img) {
                    Ok(text) => {
                        summary.total_images += 1;
                        let looks_like_tooltip = (text.contains("招式")
                            || text.contains("被动")
                            || text.contains("主动"))
                            && ["造成", "提高", "增加", "减少", "消耗", "附加"]
                                .iter()
                                .any(|k| text.contains(k));
                        if looks_like_tooltip {
                            let name_hint =
                                img.file_stem().map(|s| s.to_string_lossy().to_string());
                            let (text, fixes) = whitelist.correct(&text);
                            let mut tooltip =
                                parse::parse_qixue_tooltip(&text, name_hint.as_deref());
                            tooltip.corrections = fixes;
                            if tooltip.text.is_empty() {
                                report.warnings.push(format!(
                                    "{}: 疑似奇穴 tooltip，但正文为空",
                                    img.display()
                                ));
                            }
                            if tooltip.kind.as_deref() == Some("绝招")
                                && let Some(active) = tooltip.active.clone()
                            {
                                let name = tooltip
                                    .name
                                    .clone()
                                    .or_else(|| name_hint.clone())
                                    .unwrap_or_default();
                                let (ratio, ratio_int, ratio_residual) =
                                    ratio_for(&report.panel, &active, args.base);
                                qs.skills.push(QixueActiveSkill {
                                    jihuoqixue: name.clone(),
                                    name,
                                    active,
                                    ratio,
                                    ratio_int,
                                    ratio_residual,
                                    corrections: tooltip
                                        .corrections
                                        .iter()
                                        .cloned()
                                        .map(|(from, to)| Correction { from, to })
                                        .collect(),
                                });
                            }
                            summary.total_qixue += 1;
                            qs.shots.push(QixueShot {
                                file: img.display().to_string(),
                                tooltip,
                            });
                        } else {
                            match page_shot(&report.panel, &args, &img) {
                                Ok(page) => {
                                    summary.total_qixue += page.cards.len();
                                    qs.cards.push(page);
                                }
                                Err(e) => report.warnings.push(format!("{}: {e}", img.display())),
                            }
                        }
                    }
                    Err(e) => report.warnings.push(format!("{}: {e}", img.display())),
                }
            }
            if !qs.cards.is_empty() || !qs.shots.is_empty() {
                report.qixue.push(qs);
            }
        }

        let warn = report.warnings.len();
        summary.warnings += warn;
        summary.professions.push(ProfessionSummary {
            profession: name.clone(),
            skills: report.skills.len(),
            miji: report
                .skills
                .iter()
                .map(|s| s.miji.iter().map(|m| m.cards.len()).sum::<usize>())
                .sum(),
            qixue: report.qixue.len(),
            warnings: warn,
        });

        let path = out_dir.join(format!("{name}.json"));
        match serde_json::to_string_pretty(&report) {
            Ok(json) => {
                if let Err(e) = fs::write(&path, json) {
                    eprintln!("写 {} 失败：{e}", path.display());
                } else {
                    println!(
                        "{}：技能 {} / 秘籍 {} / 奇穴 {} / 提示 {} → {}",
                        name,
                        report.skills.len(),
                        report
                            .skills
                            .iter()
                            .map(|s| s.miji.iter().map(|m| m.cards.len()).sum::<usize>())
                            .sum::<usize>(),
                        report.qixue.len(),
                        warn,
                        path.display()
                    );
                }
            }
            Err(e) => eprintln!("序列化 {} 失败：{e}", name),
        }
    }

    match serde_json::to_string_pretty(&summary) {
        Ok(json) => {
            let path = out_dir.join("summary.json");
            if let Err(e) = fs::write(&path, json) {
                eprintln!("写 {} 失败：{e}", path.display());
            } else {
                println!(
                    "总计：图片 {} / 技能 {} / 秘籍 {} / 奇穴 {} / 提示 {}",
                    summary.total_images,
                    summary.total_skills,
                    summary.total_miji,
                    summary.total_qixue,
                    summary.warnings
                );
            }
        }
        Err(e) => eprintln!("序列化 summary 失败：{e}"),
    }
}

/// 整页截图（秘籍/奇穴选择题）→ 切卡 → 逐张 OCR → 卡片结构化。
fn page_shot(_panel: &Option<AttackPanel>, args: &Args, img: &Path) -> Result<PageShot, String> {
    if args.no_grid {
        let text = recognize(args, img)?;
        let entries = parse::parse_miji_page(&text);
        let cards = entries
            .into_iter()
            .map(|e| Card {
                cell: String::new(),
                title: Some(format!("《{}》{}", e.group, e.paper)),
                effect: Some(e.effect),
                modifiers: e.modifiers,
                raw: String::new(),
            })
            .collect();
        return Ok(PageShot {
            file: img.display().to_string(),
            cards,
        });
    }

    let area: Vec<f32> = args
        .page_area
        .split(',')
        .filter_map(|v| v.trim().parse().ok())
        .collect();
    if area.len() != 4 {
        return Err(format!(
            "--page-area 需要 4 个数字，实际：{}",
            args.page_area
        ));
    }
    let grid = pages::Grid {
        cols: args.page_cols,
        rows: args.page_rows,
        area: [area[0], area[1], area[2], area[3]],
    };
    let tmp = std::env::temp_dir().join("skill-shots-cells");
    let cells = pages::crop_cells(img, &grid, &tmp)?;

    let mut cards = Vec::new();
    for cell in cells {
        let text = ocr::recognize(&cell, &args.tesseract, &args.lang, &[4, 6])
            .map_err(|e| e.to_string())?;
        if let Some(card) = parse_card(&text, &cell) {
            cards.push(card);
        }
    }
    Ok(PageShot {
        file: img.display().to_string(),
        cards,
    })
}

/// 卡片的正文效果清理：
/// OCR 常把卡片标题（《音·宫》参悟断篇）和效果（伤害提高4%）连成一片，
/// 这里从「修饰结构」的起点开始截，句子结束就停，把标题噪声丢掉。
fn clean_effect(text: &str) -> Option<String> {
    // 去掉《…》标题块与卡片上的杂字，只保留「伤害提高4%」这类效果句。
    let group = regex::Regex::new(r"《[^》]*》").ok()?;
    let mut cleaned = group.replace_all(text, " ").to_string();
    // 去掉「经脉图/参悟/秘法/秘诀/真传 + 残页/断篇/绝章」这类秘籍名残留
    let paper =
        regex::Regex::new(r"(?:经脉图|参悟|秘法|秘诀|真传|秘章)?(?:残页|断篇|绝章|秘章|真传)")
            .ok()?;
    cleaned = paper.replace_all(&cleaned, " ").to_string();

    // 已知效果主语优先，避免把 OCR 噪声吃进主语
    let subject = regex::Regex::new(
        r"(?:招式有效距离|运功时间|调息时间|内力消耗|会心效果|会心|伤害|加速|御劲|破防|攻击|疗伤|气血|移动速度)(?:提高|提升|增加|降低|减少)\d",
    )
    .ok()?;
    if let Some(m) = subject.find(&cleaned) {
        let tail = &cleaned[m.start()..];
        let end = tail.find(['。', '\n', ';']).unwrap_or(tail.len());
        let out = tail[..end].trim().to_string();
        if !out.is_empty() {
            return Some(out);
        }
    }

    // 兜底：任意「XX提高N」结构
    let generic =
        regex::Regex::new(r"[\u{4e00}-\u{9fa5}]{2,10}?(?:提高|提升|增加|降低|减少)\d").ok()?;
    if let Some(m) = generic.find(&cleaned) {
        let tail = &cleaned[m.start()..];
        let end = tail.find(['。', '\n', ';']).unwrap_or(tail.len());
        return Some(tail[..end].trim().to_string());
    }
    None
}

/// 从一张卡片的 OCR 文本里抽标题 + 效果 + 数值修正。
fn parse_card(text: &str, cell: &Path) -> Option<Card> {
    let title_re = regex::Regex::new(
        r"《([^》]{1,12})》([\u{4e00}-\u{9fa5}]{2,8}(?:残页|断篇|绝章|秘章|真传))",
    )
    .ok()?;
    let group_re = regex::Regex::new(r"《([^》]{1,12})》").ok()?;

    let title = title_re
        .captures(text)
        .map(|c| format!("《{}》{}", &c[1], &c[2]))
        .or_else(|| group_re.captures(text).map(|c| format!("《{}》", &c[1])));

    // 效果行：从「XX提高N%」这种修饰结构的起点截取到句末，前面的标题噪声直接丢掉
    let effect = clean_effect(text);

    let modifiers = parse::parse_modifiers(effect.as_deref().unwrap_or_default());
    if title.is_none() && effect.is_none() {
        return None;
    }
    Some(Card {
        cell: cell
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        title,
        effect,
        modifiers,
        raw: text.to_string(),
    })
}

fn recognize(args: &Args, img: &Path) -> Result<String, String> {
    ocr::recognize(img, &args.tesseract, &args.lang, &args.psms).map_err(|e| e.to_string())
}

/// 全窗截图：自动找 tooltip 面板区域后识别（工具提示跟着鼠标走，位置不固定）。
fn recognize_auto(args: &Args, img: &Path) -> Result<String, String> {
    ocr::recognize_auto(img, &args.tesseract, &args.lang, &args.psms).map_err(|e| e.to_string())
}

/// 反推倍率 = 攻击力贡献 / 最终攻击力 × 基准（非目标属性或无攻击力贡献时返回 None）。
fn ratio_for(
    panel: &Option<AttackPanel>,
    tooltip: &Tooltip,
    base: f32,
) -> (Option<f32>, Option<u32>, Option<f32>) {
    let Some(bonus) = tooltip.attack_bonus else {
        return (None, None, None);
    };
    let Some(el) = tooltip.element else {
        return (None, None, None);
    };
    let Some(atk) = panel.as_ref().and_then(|p| p.attack_for(el)) else {
        return (None, None, None);
    };
    if atk == 0 || el == Element::Other {
        return (None, None, None);
    }
    let r = bonus as f32 / atk as f32 * base;
    let integer = r.trunc();
    (Some(r), Some(integer as u32), Some((r - integer).abs()))
}

/// 「第一重」→ 1 …「第六重」→ 6；「混选奇穴」返回 None。
fn parse_slot_index(slot: &str) -> Option<u32> {
    let digits = ["一", "二", "三", "四", "五", "六", "七", "八", "九"];
    let rest = slot.strip_prefix("第")?.strip_suffix("重")?;
    digits.iter().position(|d| *d == rest).map(|i| i as u32 + 1)
}

fn is_mixed_pool(slot: &str) -> bool {
    slot.contains("混选")
}

fn shot(
    args: &Args,
    panel: &Option<AttackPanel>,
    img: &Path,
    name_hint: Option<&str>,
    whitelist: &names::Whitelist,
) -> Result<Shot, String> {
    let text = recognize_auto(args, img)?;
    let (text, fixes) = whitelist.correct(&text);
    let tooltip = parse::parse_tooltip(&text, name_hint);
    let corrections: Vec<Correction> = fixes
        .into_iter()
        .map(|(from, to)| Correction { from, to })
        .collect();

    let (ratio, ratio_int, ratio_residual) = ratio_for(panel, &tooltip, args.base);

    Ok(Shot {
        file: img.display().to_string(),
        tooltip,
        corrections,
        ratio,
        ratio_int,
        ratio_residual,
    })
}

fn list_dirs(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut v = Vec::new();
    if !dir.is_dir() {
        return Ok(v);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && !is_hidden(&entry.file_name().to_string_lossy()) {
            v.push(path);
        }
    }
    v.sort();
    Ok(v)
}

fn list_images(dir: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    if !dir.is_dir() {
        return v;
    }
    if let Ok(rd) = fs::read_dir(dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_file() && is_image(&path) {
                v.push(path);
            }
        }
    }
    v.sort();
    v
}

fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            matches!(
                e.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tif" | "tiff"
            )
        })
        .unwrap_or(false)
}

fn is_hidden(name: &str) -> bool {
    name.starts_with('.') || name.starts_with('_')
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod ratio_tests {
    use super::*;

    #[test]
    fn ratio_integer_truncates_without_rounding() {
        let panel = Some(AttackPanel {
            hunyuan: Some(100),
            ..Default::default()
        });
        let tooltip = Tooltip {
            attack_bonus: Some(1),
            element: Some(Element::Hunyuan),
            ..Default::default()
        };
        let (ratio, integer, residual) = ratio_for(&panel, &tooltip, 160.0);
        assert!((ratio.unwrap() - 1.6).abs() < 1e-6);
        assert_eq!(integer, Some(1));
        assert!((residual.unwrap() - 0.6).abs() < 1e-6);
    }
}
