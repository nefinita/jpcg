// ============================================================================
// profession — 门派列表扫描
// 扫描 data_dirs()（多目录合并：安装版用户覆盖优先，其后随包资源），
// 同文件名先出现者优先，按前缀分组，每组取赛季版本最新的一份。
// ============================================================================

use std::collections::HashMap;
use std::path::PathBuf;

use crate::type_set::xinfa::XinfaSummary;

use super::toml::TomlConfig;

fn group_key(filename: &str) -> Option<String> {
    let stem = filename.strip_suffix(".toml")?;
    if stem.starts_with('_') {
        return None;
    }
    Some(stem.split('_').next().unwrap_or(stem).to_string())
}

pub fn list_available_professions() -> Vec<XinfaSummary> {
    list_from_dirs(&super::paths::data_dirs())
}

/// 显式目录列表版（便于测试多目录合并）：同文件名先出现者优先，跨目录并集。
pub(crate) fn list_from_dirs(dirs: &[PathBuf]) -> Vec<XinfaSummary> {
    // 先按文件名去重（目录按优先级，先到先得）
    let mut seen: HashMap<String, PathBuf> = HashMap::new();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }
            let Some(fname) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            seen.entry(fname.to_string()).or_insert(path);
        }
    }

    let mut by_group: HashMap<String, Vec<XinfaSummary>> = HashMap::new();
    for (fname, path) in seen {
        let Some(key) = group_key(&fname) else {
            continue;
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(mut cfg) = toml::from_str::<TomlConfig>(&content) else {
            continue;
        };

        let ver = cfg.version.clone().unwrap_or_default();
        let version_label = ver.label();

        cfg.xinfa.profession = key.clone();
        by_group.entry(key.clone()).or_default().push(XinfaSummary {
            value: key.clone(),
            label: cfg.xinfa.xinfa_name,
            nom: cfg.xinfa.xinfa_nom,
            version_label,
            version: ver,
        });
    }

    by_group
        .into_values()
        .filter_map(|mut list| {
            list.sort_by(|a, b| {
                b.version
                    .level
                    .cmp(&a.version.level)
                    .then_with(|| b.version.season.cmp(&a.version.season))
                    .then_with(|| b.version.modified.cmp(&a.version.modified))
            });
            list.into_iter().next()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("jpcg-prof-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn list_merges_across_dirs_user_first() {
        let resources = tmp("lp-res");
        let user = tmp("lp-user");
        std::fs::write(
            resources.join("mowen.toml"),
            "[xinfa]\nxinfa_name = \"莫问\"\nxinfa_nom = \"gengu\"\n",
        )
        .unwrap();
        std::fs::write(
            resources.join("wanhua.toml"),
            "[xinfa]\nxinfa_name = \"万花\"\nxinfa_nom = \"gengu\"\n",
        )
        .unwrap();
        // 用户目录只有 mowen（模拟“保存单个心法”）
        std::fs::write(
            user.join("mowen.toml"),
            "[xinfa]\nxinfa_name = \"莫问改\"\nxinfa_nom = \"gengu\"\n",
        )
        .unwrap();

        let list = list_from_dirs(&[user.clone(), resources.clone()]);
        let mut values: Vec<String> = list.iter().map(|s| s.value.clone()).collect();
        values.sort();
        // 跨目录并集：保存单个心法后其他心法仍在
        assert_eq!(values, vec!["mowen", "wanhua"]);
        // 同文件名用户覆盖优先
        assert_eq!(
            list.iter().find(|s| s.value == "mowen").unwrap().label,
            "莫问改"
        );

        let _ = std::fs::remove_dir_all(&resources);
        let _ = std::fs::remove_dir_all(&user);
    }
}
