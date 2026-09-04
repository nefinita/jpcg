// ============================================================================
// profession — 门派列表扫描
// 扫描 data/shuxing/ 目录，按前缀分组，每组取赛季版本最新的一份。
// ============================================================================

use std::collections::HashMap;

use crate::type_set::xinfa::XinfaSummary;

use super::paths::data_dir;
use super::toml::TomlConfig;

fn group_key(filename: &str) -> Option<String> {
    let stem = filename.strip_suffix(".toml")?;
    if stem.starts_with('_') {
        return None;
    }
    Some(stem.split('_').next().unwrap_or(stem).to_string())
}

pub fn list_available_professions() -> Vec<XinfaSummary> {
    let dir = match data_dir() {
        Some(d) => d,
        None => return vec![],
    };

    let mut by_group: HashMap<String, Vec<XinfaSummary>> = HashMap::new();

    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let fname = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let key = match group_key(&fname) {
            Some(k) => k,
            None => continue,
        };

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut cfg: TomlConfig = match toml::from_str(&content) {
            Ok(c) => c,
            Err(_) => continue,
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
