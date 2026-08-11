// ============================================================================
// paths — 路径定位
// 负责定位数据文件目录与连招预设目录。
// 数据文件位于 {exe_dir}/data/shuxing/{心法名}.toml，
// 保存文件位于工作目录下的 saved_config.toml。
// ============================================================================

use std::path::PathBuf;

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
    if exe_dir.ends_with("MacOS") {
        if let Some(contents) = exe_dir.parent() {
            let bundle = contents.join("Resources").join("data").join("shuxing");
            if bundle.is_dir() {
                return Some(bundle);
            }
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
