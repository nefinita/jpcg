//! 整页截图（秘籍选择、奇穴选择）的切卡处理。
//!
//! 整页 OCR 效果很差（游戏字体 + 半透明底），但把每张卡单独裁出来再 OCR，
//! 「伤害提高4%。」这类效果文案几乎全对。这里用系统 `sips` 做裁剪，
//! 不引入额外图像依赖（本工具本来就只在 macOS 上跑）。

use std::path::{Path, PathBuf};
use std::process::Command;

/// 卡片网格区域的默认占比（x0, y0, x1, y1），对应无界「秘籍选择」页面。
pub const DEFAULT_AREA: [f32; 4] = [0.020, 0.130, 0.994, 0.815];

pub struct Grid {
    pub cols: u32,
    pub rows: u32,
    pub area: [f32; 4],
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            cols: 4,
            rows: 2,
            area: DEFAULT_AREA,
        }
    }
}

/// 按比例裁一块区域到临时文件（x0, y0, x1, y1 都是 0~1 的相对坐标）。
pub fn crop_region(img: &Path, area: [f32; 4], tag: &str) -> Result<PathBuf, String> {
    let (w, h) = image_size(img)?;
    let x0 = (area[0] * w as f32) as u32;
    let y0 = (area[1] * h as f32) as u32;
    let cw = (((area[2] - area[0]) * w as f32) as u32).max(40);
    let ch = (((area[3] - area[1]) * h as f32) as u32).max(40);
    let stem = img
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "img".into());
    let out = std::env::temp_dir().join(format!("skill-shots-{}-{tag}.png", stem));
    let status = Command::new("/usr/bin/sips")
        .args(["-c", &ch.to_string(), &cw.to_string()])
        .args(["--cropOffset", &y0.to_string(), &x0.to_string()])
        .arg(img)
        .arg("--out")
        .arg(&out)
        .status()
        .map_err(|e| format!("sips 启动失败：{e}"))?;
    if !status.success() {
        return Err("sips 裁剪失败".into());
    }
    Ok(out)
}
/// 把图片等比放大到目标宽度（只放大不缩小），给 tesseract 更清晰的像素。
/// 游戏 UI 里的 tooltip 通常只有 300~900px 宽，直接 OCR 会把「变徵」认成「变征」。
pub fn upscale(img: &Path, target_width: u32) -> Result<PathBuf, String> {
    let (w, _h) = image_size(img)?;
    if w >= target_width {
        return Ok(img.to_path_buf());
    }
    let stem = img
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "img".into());
    let out = std::env::temp_dir().join(format!("skill-shots-{}x.png", stem));
    let status = Command::new("/usr/bin/sips")
        .args(["-Z", &target_width.to_string()])
        .arg(img)
        .arg("--out")
        .arg(&out)
        .status()
        .map_err(|e| format!("sips 启动失败：{e}"))?;
    if !status.success() {
        return Err("sips 放大失败".into());
    }
    Ok(out)
}

/// 读取图片尺寸。
pub fn image_size(img: &Path) -> Result<(u32, u32), String> {
    let out = Command::new("/usr/bin/sips")
        .args(["-g", "pixelWidth", "-g", "pixelHeight"])
        .arg(img)
        .output()
        .map_err(|e| format!("sips 启动失败：{e}"))?;
    if !out.status.success() {
        return Err(format!(
            "sips 读取尺寸失败：{}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut w = None;
    let mut h = None;
    for line in text.lines() {
        if let Some(v) = line.trim().strip_prefix("pixelWidth:") {
            w = v.trim().parse::<u32>().ok();
        }
        if let Some(v) = line.trim().strip_prefix("pixelHeight:") {
            h = v.trim().parse::<u32>().ok();
        }
    }
    match (w, h) {
        (Some(w), Some(h)) => Ok((w, h)),
        _ => Err("sips 输出里没有 pixelWidth/pixelHeight".into()),
    }
}

/// 按网格把整页截图裁成若干卡片 PNG，写入 `tmp_dir`，返回文件路径。
pub fn crop_cells(img: &Path, grid: &Grid, tmp_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let (w, h) = image_size(img)?;
    let (w, h) = (w as f32, h as f32);
    let [ax0, ay0, ax1, ay1] = grid.area;
    let (x0, y0, x1, y1) = (ax0 * w, ay0 * h, ax1 * w, ay1 * h);
    let cw = (x1 - x0) / grid.cols as f32;
    let ch = (y1 - y0) / grid.rows as f32;

    std::fs::create_dir_all(tmp_dir).map_err(|e| format!("创建临时目录失败：{e}"))?;
    let stem = img
        .file_stem()
        .map(|s| s.to_string_lossy().replace([' ', '/'], "_"))
        .unwrap_or_else(|| "page".into());

    let mut out = Vec::new();
    for row in 0..grid.rows {
        for col in 0..grid.cols {
            let cx = (x0 + cw * col as f32).round() as i32;
            let cy = (y0 + ch * row as f32).round() as i32;
            let cw_px = cw.round() as i32;
            let ch_px = ch.round() as i32;
            let path = tmp_dir.join(format!("{stem}_r{row}c{col}.png"));
            let status = Command::new("/usr/bin/sips")
                .args(["-c", &ch_px.to_string(), &cw_px.to_string()])
                .args(["--cropOffset", &cy.to_string(), &cx.to_string()])
                .arg(img)
                .arg("--out")
                .arg(&path)
                .output()
                .map_err(|e| format!("sips 裁剪失败：{e}"))?;
            if !status.status.success() {
                return Err(format!(
                    "sips 裁剪 {} 失败：{}",
                    img.display(),
                    String::from_utf8_lossy(&status.stderr).trim()
                ));
            }
            out.push(path);
        }
    }
    Ok(out)
}
