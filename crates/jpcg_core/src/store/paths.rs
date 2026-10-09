// ============================================================================
// paths — 路径定位
// 负责定位数据根、数值集目录与连招预设目录。
//
// 数据根（含 shuxing/、values/）是**读写唯一真源**：
//   - 读取：JPCG_DATA_DIR → bundle 用户目录（更新落盘处）→ bundle Resources → exe 同目录 data
//   - 写入（更新落盘，见 data_root_writable）：JPCG_DATA_DIR → bundle 用户目录 → exe 同目录 data
// 安装版（.app）资源只读，故更新落到用户数据目录；开发版落到 exe 同目录 data。
// 保存文件 saved_config.toml 仍位于工作目录。
// ============================================================================

use std::path::{Path, PathBuf};

/// 目录是否像一个数据根（含 shuxing/ 或 values/）
fn has_data(dir: &Path) -> bool {
    dir.join("shuxing").is_dir() || dir.join("values").is_dir()
}

/// macOS .app bundle 的用户数据根（`~/Library/Application Support/com.qinthirteen.jpcg/data`）。
/// 与连招预设目录同源；bundle 资源只读，故更新落到此处。
fn bundle_user_data(home: &Path) -> PathBuf {
    home.join("Library")
        .join("Application Support")
        .join("com.qinthirteen.jpcg")
        .join("data")
}

/// 读取数据根候选（纯函数，便于测试）；顺序即优先级。
/// bundle（exe 位于 `Contents/MacOS`）：用户目录（更新落盘处）优先，其次随包只读资源；
/// 最后兑底 exe 同目录 data（开发模式）。
fn read_roots(exe_dir: &Path, home: Option<&Path>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if exe_dir.ends_with("MacOS") {
        if let Some(home) = home {
            out.push(bundle_user_data(home));
        }
        if let Some(contents) = exe_dir.parent() {
            out.push(contents.join("Resources").join("data"));
        }
    }
    out.push(exe_dir.join("data"));
    out
}

/// 读取数据根的候选目录（不含 `JPCG_DATA_DIR`，由调用方优先处理）
fn read_root_candidates() -> Vec<PathBuf> {
    let Ok(exe) = std::env::current_exe() else {
        return Vec::new();
    };
    let Some(exe_dir) = exe.parent() else {
        return Vec::new();
    };
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    read_roots(exe_dir, home.as_deref())
}

/// 将 `JPCG_DATA_DIR` 的值归一为数据根（兼容指向根、指向 shuxing、或任意已存在目录）
fn env_data_root(p: &Path) -> Option<PathBuf> {
    if has_data(p) {
        return Some(p.to_path_buf());
    }
    if p.file_name().and_then(|n| n.to_str()) == Some("shuxing") {
        return p.parent().map(Path::to_path_buf);
    }
    if p.is_dir() {
        return Some(p.to_path_buf());
    }
    None
}

/// 纯函数版可写根解析（便于测试）：
/// `JPCG_DATA_DIR` → bundle 用户目录（exe 位于 `Contents/MacOS`）→ `exe_dir/data`
fn writable_root(exe_dir: &Path, env_dir: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    if let Some(p) = env_dir {
        return env_data_root(p);
    }
    if exe_dir.ends_with("MacOS") {
        return Some(bundle_user_data(home?));
    }
    Some(exe_dir.join("data"))
}

/// 可写数据根（更新落盘目标）。读取侧见 [`data_dir`] / [`values_dir`]。
pub fn data_root_writable() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;
    let env_dir = std::env::var("JPCG_DATA_DIR").ok().map(PathBuf::from);
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    writable_root(exe_dir, env_dir.as_deref(), home.as_deref())
}

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
    read_root_candidates()
        .into_iter()
        .map(|r| r.join("shuxing"))
        .find(|d| d.is_dir())
}

/// 数值集目录（data/values）：存放 index.toml 与各值集快照。
/// 解析顺序与 [`data_dir`] 一致，且兼容 `JPCG_DATA_DIR` 直接指向 `data/shuxing` 的既有用法。
pub fn values_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("JPCG_DATA_DIR") {
        return values_dir_from_env(&PathBuf::from(dir));
    }
    read_root_candidates()
        .into_iter()
        .map(|r| r.join("values"))
        .find(|d| d.is_dir())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("jpcg-paths-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn writable_root_dev_is_exe_data() {
        let exe = tmp("dev");
        assert_eq!(writable_root(&exe, None, None), Some(exe.join("data")));
        let _ = std::fs::remove_dir_all(&exe);
    }

    #[test]
    fn writable_root_bundle_is_user_dir() {
        let base = tmp("bundle");
        let exe = base.join("JPCG.app/Contents/MacOS");
        std::fs::create_dir_all(&exe).unwrap();
        let home = tmp("home");
        assert_eq!(
            writable_root(&exe, None, Some(home.as_path())),
            Some(
                home.join("Library")
                    .join("Application Support")
                    .join("com.qinthirteen.jpcg")
                    .join("data")
            )
        );
        let _ = std::fs::remove_dir_all(&base);
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn writable_root_env_overrides() {
        let root = tmp("envroot");
        std::fs::create_dir_all(root.join("values")).unwrap();
        let exe = tmp("env-exe");
        assert_eq!(writable_root(&exe, Some(&root), None), Some(root.clone()));
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&exe);
    }

    #[test]
    fn env_data_root_accepts_shuxing_and_root() {
        let root = tmp("envshuxing");
        let shuxing = root.join("shuxing");
        std::fs::create_dir_all(&shuxing).unwrap();
        // 指向 <root>/shuxing → 归一为 root
        assert_eq!(env_data_root(&shuxing), Some(root.clone()));
        // 指向含 shuxing/ 的 root → 即 root
        assert_eq!(env_data_root(&root), Some(root.clone()));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn bundle_reader_prefers_writable_user_dir_then_resources() {
        let base = tmp("read");
        let exe = base.join("JPCG.app/Contents/MacOS");
        std::fs::create_dir_all(&exe).unwrap();
        let home = tmp("read-home");
        let roots = read_roots(&exe, Some(home.as_path()));
        let expected_user = bundle_user_data(&home);
        assert_eq!(
            roots.first().map(PathBuf::as_path),
            Some(expected_user.as_path())
        );
        assert_eq!(
            roots.get(1).map(PathBuf::as_path),
            Some(base.join("JPCG.app/Contents/Resources/data").as_path())
        );
        // 可写根（更新落盘处）必是读取首选 → 写读一致
        assert_eq!(
            writable_root(&exe, None, Some(home.as_path())),
            roots.first().cloned()
        );
        let _ = std::fs::remove_dir_all(&base);
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn dev_reader_and_writer_share_exe_data() {
        let exe = tmp("dev-share");
        let roots = read_roots(&exe, None);
        assert_eq!(roots, vec![exe.join("data")]);
        assert_eq!(writable_root(&exe, None, None), roots.first().cloned());
        let _ = std::fs::remove_dir_all(&exe);
    }
}
