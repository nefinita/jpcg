// ============================================================================
// paths — 路径定位
// 负责定位数据根、职业数据、数值集目录与连招预设目录。
//
// 正式布局：data/<profession>/{shuxing.toml, combat.toml, gear/}；values/ 全职业共享。
// 旧布局 data/{shuxing,combat,gear}/ 仅保留读取兼容，新保存与更新应写正式布局。
// 数据根读取：JPCG_DATA_DIR → bundle 用户目录（更新落盘处）→ bundle Resources → exe 同目录 data。
// 写入：JPCG_DATA_DIR → bundle 用户目录 → exe 同目录 data；saved_config.toml 仍位于工作目录。
// ============================================================================

use std::path::{Path, PathBuf};

/// 目录是否像一个数据根（含旧 shuxing/ 或共享 values/）。任意已存在目录也接受，
/// 以便只含新职业目录或即将写入数据的目录作为 JPCG_DATA_DIR。
fn has_data(dir: &Path) -> bool {
    dir.join("shuxing").is_dir() || dir.join("values").is_dir()
}

/// macOS .app bundle 的用户数据根（`~/Library/Application Support/com.qinthirteen.jpcg/data`）。
/// 与连招预设目录同源；bundle 资源只读，故更新落到用户数据目录。
fn bundle_user_data(home: &Path) -> PathBuf {
    home.join("Library")
        .join("Application Support")
        .join("com.qinthirteen.jpcg")
        .join("data")
}

/// 读取数据根候选（纯函数，便于测试）；顺序即优先级。
/// bundle（exe 位于 `Contents/MacOS`）：用户目录（更新落盘处）优先，其次随包只读资源；
/// 最后兜底 exe 同目录 data（开发模式）。
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

/// 读取数据根的候选目录（不含 `JPCG_DATA_DIR`，由调用方优先处理）。
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

/// 将 `JPCG_DATA_DIR` 的值归一为数据根（兼容指向根、旧 shuxing、或任意已存在目录）。
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

/// 当前进程的数据根。设置环境变量时只读该根，未设置时按用户目录→随包资源→exe 合并。
fn data_roots() -> Vec<PathBuf> {
    if let Some(dir) = std::env::var_os("JPCG_DATA_DIR") {
        return env_data_root(&PathBuf::from(dir)).into_iter().collect();
    }
    read_root_candidates()
}

/// 纯函数版可写根解析（便于测试）：
/// `JPCG_DATA_DIR` → bundle 用户目录（exe 位于 `Contents/MacOS`）→ `exe_dir/data`。
fn writable_root(exe_dir: &Path, env_dir: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    if let Some(p) = env_dir {
        return env_data_root(p);
    }
    if exe_dir.ends_with("MacOS") {
        return Some(bundle_user_data(home?));
    }
    Some(exe_dir.join("data"))
}

/// 可写数据根（更新落盘目标）。读取侧见各职业文件与 [`values_dir`]。
pub fn data_root_writable() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;
    let env_dir = std::env::var("JPCG_DATA_DIR").ok().map(PathBuf::from);
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    writable_root(exe_dir, env_dir.as_deref(), home.as_deref())
}

/// 心法标识只能作为单个目录名使用，不能携带路径、扩展名或空白。
pub(crate) fn validate_profession(profession: &str) -> Result<(), String> {
    if profession.is_empty()
        || profession.len() > 128
        || !profession
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
    {
        return Err("心法标识仅允许字母、数字、下划线和连字符，且长度为 1..128".into());
    }
    Ok(())
}

/// 每个数据根内均为新布局优先、旧布局其次；根之间仍保持用户目录→随包资源顺序。
fn profession_files_from_roots(
    roots: &[PathBuf],
    profession: &str,
    legacy_dir: &str,
    canonical_name: &str,
) -> Result<Vec<PathBuf>, String> {
    validate_profession(profession)?;
    Ok(roots
        .iter()
        .flat_map(|root| {
            [
                root.join(profession).join(canonical_name),
                root.join(legacy_dir).join(format!("{profession}.toml")),
            ]
        })
        .collect())
}

/// 某职业技能文件的严格读取候选；调用方取首个存在者，损坏时不得回退。
pub fn shuxing_file_paths(profession: &str) -> Result<Vec<PathBuf>, String> {
    profession_files_from_roots(&data_roots(), profession, "shuxing", "shuxing.toml")
}

/// 某职业战斗 overlay 的严格读取候选；文件全部不存在时由调用方返回空 overlay。
pub fn combat_file_paths(profession: &str) -> Result<Vec<PathBuf>, String> {
    profession_files_from_roots(&data_roots(), profession, "combat", "combat.toml")
}

/// 技能编辑的唯一写入位置。即使旧布局仍可读取，新保存也只写职业目录。
pub fn shuxing_save_path(root: &Path, profession: &str) -> Result<PathBuf, String> {
    validate_profession(profession)?;
    Ok(root.join(profession).join("shuxing.toml"))
}

/// 扫描正式布局的职业技能文件；返回值保留数据根优先级，由调用方按职业去重。
fn profession_files_in_roots(roots: &[PathBuf]) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    for root in roots {
        let Ok(entries) = std::fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let Some(profession) = dir.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if profession.starts_with('_') || validate_profession(profession).is_err() {
                continue;
            }
            let path = dir.join("shuxing.toml");
            if path.is_file() {
                out.push((profession.to_string(), path));
            }
        }
    }
    out
}

/// 正式布局职业技能文件（用于门派列表；单职业加载请用 [`shuxing_file_paths`]）。
pub fn profession_files() -> Vec<(String, PathBuf)> {
    profession_files_in_roots(&data_roots())
}

/// 旧 shuxing 目录候选，仅供旧布局职业列表合并读取。
/// 新职业加载不依赖此函数；保留多目录合并，避免保存单个心法遮蔽其他心法。
pub fn data_dirs() -> Vec<PathBuf> {
    data_roots()
        .into_iter()
        .map(|root| root.join("shuxing"))
        .filter(|dir| dir.is_dir())
        .collect()
}

/// 数值集目录（data/values）：存放 index.toml 与各值集快照，全职业共享。
pub fn values_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("JPCG_DATA_DIR") {
        return values_dir_from_env(&PathBuf::from(dir));
    }
    read_root_candidates()
        .into_iter()
        .map(|r| r.join("values"))
        .find(|d| d.is_dir())
}

fn gear_dir_from_roots(roots: &[PathBuf], profession: &str) -> Result<PathBuf, String> {
    validate_profession(profession)?;
    for root in roots {
        let canonical = root.join(profession).join("gear");
        if canonical.is_dir() {
            return Ok(canonical);
        }
        let legacy = root.join("gear");
        if legacy.is_dir() {
            return Ok(legacy);
        }
    }
    Err(format!(
        "找不到职业 {profession} 的配装数据（应为 data/{profession}/gear），请安装对应数据或设置 JPCG_DATA_DIR"
    ))
}

/// 当前职业的配装目录。旧 `data/gear` 仅作兼容；正式数据按职业隔离。
pub fn gear_dir(profession: &str) -> Result<PathBuf, String> {
    gear_dir_from_roots(&data_roots(), profession)
}

/// 由 `JPCG_DATA_DIR` 的值解析 values 目录。
/// 兼容两种既有写法：
///   - 指向数据根（含职业目录、旧 shuxing/、values/）→ `<root>/values`
///   - 直接指向 `<root>/shuxing` → `<root>/values`（取同级）。
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

/// 连招预设目录路径（开发模式: exe_dir/data/combo；.app bundle: 用户数据目录/combo）。
pub fn combo_presets_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;

    let dir = if exe_dir.ends_with("MacOS") {
        let home = std::env::var("HOME").ok()?;
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("com.qinthirteen.jpcg")
            .join("combo")
    } else {
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
        assert_eq!(env_data_root(&shuxing), Some(root.clone()));
        assert_eq!(env_data_root(&root), Some(root.clone()));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn profession_paths_prefer_new_layout_inside_each_root() {
        let user = tmp("path-user");
        let bundle = tmp("path-bundle");
        let roots = [user.clone(), bundle.clone()];
        assert_eq!(
            profession_files_from_roots(&roots, "mowen", "shuxing", "shuxing.toml").unwrap(),
            vec![
                user.join("mowen/shuxing.toml"),
                user.join("shuxing/mowen.toml"),
                bundle.join("mowen/shuxing.toml"),
                bundle.join("shuxing/mowen.toml"),
            ]
        );
        assert!(
            profession_files_from_roots(&roots, "../mowen", "shuxing", "shuxing.toml").is_err()
        );
        let _ = std::fs::remove_dir_all(&user);
        let _ = std::fs::remove_dir_all(&bundle);
    }

    #[test]
    fn canonical_professions_scan_and_gear_selection_are_isolated() {
        let root = tmp("canonical");
        for profession in ["mowen", "badao"] {
            let dir = root.join(profession);
            std::fs::create_dir_all(dir.join("gear")).unwrap();
            std::fs::write(dir.join("shuxing.toml"), "[xinfa]\n").unwrap();
        }
        std::fs::create_dir_all(root.join("values")).unwrap();
        std::fs::create_dir_all(root.join("_template")).unwrap();
        std::fs::write(root.join("_template/shuxing.toml"), "[xinfa]\n").unwrap();
        let mut found = profession_files_in_roots(&[root.clone()]);
        found.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            found.into_iter().map(|(id, _)| id).collect::<Vec<_>>(),
            vec!["badao", "mowen"]
        );
        assert_eq!(
            gear_dir_from_roots(&[root.clone()], "badao").unwrap(),
            root.join("badao/gear")
        );
        assert!(gear_dir_from_roots(&[root.clone()], "missing").is_err());
        assert!(gear_dir_from_roots(&[root.clone()], "../mowen").is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn legacy_gear_is_only_a_fallback() {
        let root = tmp("legacy-gear");
        std::fs::create_dir_all(root.join("gear")).unwrap();
        assert_eq!(
            gear_dir_from_roots(&[root.clone()], "mowen").unwrap(),
            root.join("gear")
        );
        std::fs::create_dir_all(root.join("mowen/gear")).unwrap();
        assert_eq!(
            gear_dir_from_roots(&[root.clone()], "mowen").unwrap(),
            root.join("mowen/gear")
        );
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
