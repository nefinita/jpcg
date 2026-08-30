// ============================================================================
// combo — 连招预设 CRUD
// 预设存于 {combo_presets_dir}/{name}.toml。
// ============================================================================

use crate::type_set::combo::ComboPreset;

use super::paths::combo_presets_dir;

/// 列出所有连招预设文件（不含 .toml 后缀）
pub fn list_combo_presets() -> Vec<String> {
    let dir = match combo_presets_dir() {
        Some(d) => d,
        None => return vec![],
    };
    if !dir.exists() {
        return vec![];
    }
    let mut presets = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml")
                && let Some(name) = path.file_stem().and_then(|n| n.to_str())
            {
                presets.push(name.to_string());
            }
        }
    }
    presets.sort();
    presets
}

/// 加载指定连招预设
pub fn load_combo_preset(name: &str) -> Option<ComboPreset> {
    let dir = combo_presets_dir()?;
    let path = dir.join(format!("{}.toml", name));
    let content = std::fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}

/// 保存连招预设
pub fn save_combo_preset(preset: &ComboPreset) -> Result<(), String> {
    let dir = match combo_presets_dir() {
        Some(d) => d,
        None => return Err("无法获取连招预设目录".to_string()),
    };
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;
    let path = dir.join(format!("{}.toml", preset.name));
    let content = toml::to_string_pretty(preset).map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("写入文件失败: {}", e))
}

/// 删除连招预设
pub fn delete_combo_preset(name: &str) -> Result<(), String> {
    let dir = match combo_presets_dir() {
        Some(d) => d,
        None => return Err("无法获取连招预设目录".to_string()),
    };
    let path = dir.join(format!("{}.toml", name));
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("删除文件失败: {}", e))
    } else {
        Err("预设不存在".to_string())
    }
}
