// ============================================================================
// paths — 路径定位
// 负责定位数据文件目录与连招预设目录。
// 数据文件位于 {exe_dir}/data/shuxing/{心法名}.toml，
// 保存文件位于工作目录下的 saved_config.toml。
// ============================================================================

use std::path::{Path, PathBuf};

pub fn data_dir() -> Option<PathBuf> {
    // 环境变量覆盖（CLI/Python/跨进程场景：current_exe 不可用时指向任意数据目录）
    if let Ok(dir) = std::env::var("JPCG_DATA_DIR") {
        let p = PathBuf::from(dir);
        return if p.join("shuxing").is_dir() {
            Some(p.join("shuxing"))
        } else if p.is_dir() {
            Some(p)
        } else {
            None
        };
    }

    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;

    // 开发模式: exe_dir/data/shuxing
    let dev = exe_dir.join("data").join("shuxing");
    if dev.is_dir() {
        return Some(dev);
    }

    // macOS .app bundle: exe 在 Contents/MacOS/，资源在 Contents/Resources/
    if exe_dir.ends_with("MacOS")
        && let Some(contents) = exe_dir.parent()
    {
        let bundle = contents.join("Resources").join("data").join("shuxing");
        if bundle.is_dir() {
            return Some(bundle);
        }
    }

    None
}

/// 数值集目录（data/values）：存放 index.toml 与各值集快照。
/// 解析顺序与 [`data_dir`] 一致，且兼容 `JPCG_DATA_DIR` 直接指向 `data/shuxing` 的既有用法。
pub fn values_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("JPCG_DATA_DIR") {
        return values_dir_from_env(&PathBuf::from(dir));
    }

    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;

    let dev = exe_dir.join("data").join("values");
    if dev.is_dir() {
        return Some(dev);
    }

    if exe_dir.ends_with("MacOS")
        && let Some(contents) = exe_dir.parent()
    {
        let bundle = contents.join("Resources").join("data").join("values");
        if bundle.is_dir() {
            return Some(bundle);
        }
    }

    None
}

/// 由 `JPCG_DATA_DIR` 的值解析 values 目录。
/// 兼容两种既有写法：
///   - 指向数据根（含 shuxing/、values/）→ `<root>/values`
///   - 直接指向 `<root>/shuxing`       → `<root>/values`（取同级）
pub(crate) fn values_dir_from_env(p: &Path) -> Option<PathBuf> {
    let direct = p.join("values");
    if direct.is_dir() {
        return Some(direct);
    }
    if p.file_name().and_then(|n| n.to_str()) == Some("shuxing")
        && let Some(parent) = p.parent()
    {
        let sibling = parent.join("values");
        if sibling.is_dir() {
            return Some(sibling);
        }
    }
    None
}

/// 连招预设目录路径（开发模式: exe_dir/data/combo；.app bundle: 用户数据目录/combo）
pub fn combo_presets_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;

    let dir = if exe_dir.ends_with("MacOS") {
        // macOS .app bundle — 用用户数据目录（可写）
        let home = std::env::var("HOME").ok()?;
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("com.qinthirteen.jpcg")
            .join("combo")
    } else {
        // 开发模式: exe_dir/data/combo
        exe_dir.join("data").join("combo")
    };
    Some(dir)
}
