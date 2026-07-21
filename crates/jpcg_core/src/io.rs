// ============================================================================
// io — 文件 IO 模块
// 负责 TOML 配置文件的读取、解析、保存。
// 数据文件位于 {exe_dir}/data/shuxing/{心法名}.toml，
// 保存文件位于工作目录下的 saved_config.toml。
// ============================================================================

use crate::log::{error, info, warn};
use crate::type_set::buff::BuffConfig;
use crate::type_set::coefficient::CoefficientConfig;
use crate::type_set::combo::ComboPreset;
use crate::type_set::xinfa::{self, VersionInfo, XinfaSummary};
use crate::type_set::{hostilepile, player, skilltype};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub fn data_dir() -> Option<PathBuf> {
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

// ============================================================================
// toml_input — 读取 .toml 文件内容为字符串
// 若文件不存在或读取失败，返回 "none" 以示区分（而非 panic）。
// ============================================================================

/// 读取 TOML 文件内容
/// - `profession`: 不带扩展名的文件路径（函数内部追加 .toml）
/// - 返回: 文件内容字符串，若文件不存在则返回 "none"
pub fn toml_input(profession: &str) -> String {
    let file_path = format!("{}.toml", profession);
    info(&format!("正在加载配置文件: {}", file_path));
    match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            error(&format!("读取配置文件失败: {}", e));
            "none".into()
        }
    }
}

// ============================================================================
// TomlConfig — TOML 配置顶层结构
// 对应 data/shuxing/{心法名}.toml 文件格式:
//   [xinfa]
//   xinfa_name = "莫问"
//   xinfa_nom = "根骨"
//   ...
//   [[skill]]
//   skill_name = "技能名"
//   ...
// ============================================================================

/// 心法技能配置（从 TOML 文件解析）
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct TomlConfig {
    pub xinfa: xinfa::XinfaConfig,           // 心法基础配置
    pub skill: Vec<skilltype::Skilltype>,     // 技能列表（每个技能一条 [[skill]]）
    pub version: Option<VersionInfo>,         // 赛季版本信息（可选）
}

// ============================================================================
// Config — 完整运行时配置（已废弃/预留）
// 聚合玩家、目标、技能配置，当前仅由内部方法使用。
// ============================================================================

/// 完整运行时配置，包含玩家属性和技能数据
#[derive(Default)]
pub struct Config {
    pub player: player::PlayerConfig,           // 玩家配置
    pub hostilepile: hostilepile::HostilepileConfig, // 目标配置
    pub data: TomlConfig,                       // 技能表配置
}

impl Config {
    /// 从 SaveConfig + 心法名加载完整配置
    pub fn load(x: SaveConfig, fs: &str) -> Self {
        let data = load_config(fs);
        Config {
            player: x.player,
            hostilepile: x.hostilepile,
            data,
        }
    }
}

// ============================================================================
// SaveConfig — 持久化保存的结构
// 保存文件 saved_config.toml 仅包含玩家、目标、心法配置（不含技能表）。
// ============================================================================

/// 持久化保存的配置（不含技能数据，技能数据从各自心法文件加载）
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SaveConfig {
    pub xinfa: xinfa::XinfaConfig,                     // 心法配置
    pub player: player::PlayerConfig,                   // 玩家属性
    pub hostilepile: hostilepile::HostilepileConfig,     // 目标属性
    #[serde(default)]
    pub buff: BuffConfig,                               // 阵眼/奇穴增益
    #[serde(default)]
    pub coefficient: CoefficientConfig,                 // 系数设置
}

// ============================================================================
// load_config — 按心法名加载技能配置表
// 路径: {exe_dir}/data/shuxing/{profession}.toml
// ============================================================================

/// 按心法名称加载对应的技能 TOML 配置
/// - `profession`: 心法名称（同时也是 .toml 文件名，不含扩展名）
/// - 返回: TomlConfig，若文件不存在或解析失败则返回默认空配置
pub fn load_config(profession: &str) -> TomlConfig {
    let dir = match data_dir() {
        Some(d) => d,
        None => return TomlConfig::default(),
    };
    let file_path = dir.join(profession);
    let file_path_str = file_path.to_str().unwrap_or("").to_string();
    if file_path_str.is_empty() {
        error("配置文件路径包含非法 UTF-8 字符");
        return TomlConfig::default();
    }
    // 读取并解析
    let content = toml_input(&file_path_str);
    let mut config: TomlConfig = match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            error(&format!("解析心法 '{}' 的 TOML 配置失败: {}", profession, e));
            return TomlConfig::default();
        }
    };
    config.xinfa.profession = profession.to_string();
    config
}

fn validate_profession_name(profession: &str) -> Result<(), String> {
    if profession.is_empty()
        || profession.starts_with('_')
        || profession == "."
        || profession == ".."
        || profession.chars().any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'))
    {
        return Err("心法文件名无效".to_string());
    }
    Ok(())
}

/// 读取属性编辑器使用的心法 TOML 原文。
pub fn read_attribute_config(profession: &str) -> Result<String, String> {
    validate_profession_name(profession)?;
    let dir = data_dir().ok_or_else(|| "无法定位应用数据目录".to_string())?;
    let path = dir.join(format!("{}.toml", profession));
    std::fs::read_to_string(&path).map_err(|e| format!("读取配置文件失败: {}", e))
}

/// 将属性编辑器内容保存回现有心法文件。
/// 只允许覆盖已有文件，避免编辑器无意中创建无法被心法列表识别的新配置。
pub fn write_attribute_config(profession: &str, content: &str) -> Result<(), String> {
    validate_profession_name(profession)?;
    let config: TomlConfig = toml::from_str(content).map_err(|e| format!("配置格式无效: {}", e))?;
    if config.xinfa.xinfa_name.trim().is_empty() || config.xinfa.xinfa_nom.trim().is_empty() {
        return Err("配置缺少 [xinfa] 心法信息".to_string());
    }
    if config.skill.is_empty() {
        return Err("配置至少需要一个 [[skill]] 技能条目".to_string());
    }

    let dir = data_dir().ok_or_else(|| "无法定位应用数据目录".to_string())?;
    let path = dir.join(format!("{}.toml", profession));
    if !path.is_file() {
        return Err("只能保存已有心法配置".to_string());
    }

    std::fs::write(&path, content).map_err(|e| format!("写入配置文件失败: {}", e))
}

// ============================================================================
// save_config — 保存玩家配置到本地文件
// 输出文件: 工作目录下的 saved_config.toml
// ============================================================================

/// 将当前玩家、目标、心法配置保存到 saved_config.toml
pub fn save_config(
    player: player::PlayerConfig,
    hostilepile: hostilepile::HostilepileConfig,
    xinfa: xinfa::XinfaConfig,
) {
    let save_config = SaveConfig {
        player,
        hostilepile,
        xinfa,
        buff: BuffConfig::default(),
        coefficient: CoefficientConfig::default(),
    };
    // 序列化为 TOML 字符串
    match toml::to_string(&save_config) {
        Ok(toml_str) => {
            // 写入文件
            if let Err(e) = std::fs::write("saved_config.toml", toml_str) {
                error(&format!("保存配置到文件失败: {}", e));
            } else {
                info("配置已成功保存到 saved_config.toml");
            }
        }
        Err(e) => error(&format!("配置序列化为 TOML 失败: {}", e)),
    }
}

// ============================================================================
// load_save_config — 加载已保存的默认配置
// 读取 saved_config.toml，若文件不存在则返回默认 SaveConfig。
// ============================================================================

/// 加载已持久化的配置，若未找到则返回默认值
pub fn load_save_config() -> SaveConfig {
    let content = toml_input("saved_config");
    let config: SaveConfig = match toml::from_str(&content) {
        Ok(data) => data,
        Err(_) => {
            warn("未找到已保存的配置，使用默认值。");
            SaveConfig::default()
        }
    };
    config
}

/// 连招预设目录路径（开发模式: exe_dir/data/combo；.app bundle: 用户数据目录/combo）
pub fn combo_presets_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;

    let dir = if exe_dir.ends_with("MacOS") {
        // macOS .app bundle — 用用户数据目录（可写）
        let home = std::env::var("HOME").ok()?;
        PathBuf::from(home).join("Library").join("Application Support").join("com.qinthirteen.jpcg").join("combo")
    } else {
        // 开发模式: exe_dir/data/combo
        exe_dir.join("data").join("combo")
    };
    Some(dir)
}

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
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    presets.push(name.to_string());
                }
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

/// 导出当前配置为 TOML 字符串（不含技能表）
pub fn export_config_toml() -> Result<String, String> {
    let config = load_save_config();
    toml::to_string_pretty(&config).map_err(|e| format!("序列化失败: {}", e))
}

/// 导入配置 TOML 字符串并写入 saved_config.toml
pub fn import_config_toml(toml_str: &str) -> Result<(), String> {
    let config: SaveConfig = toml::from_str(toml_str)
        .map_err(|e| format!("解析配置失败: {}", e))?;
    let content = toml::to_string_pretty(&config)
        .map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write("saved_config.toml", content)
        .map_err(|e| format!("写入文件失败: {}", e))
}

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

    let mut by_group: std::collections::HashMap<String, Vec<XinfaSummary>> =
        std::collections::HashMap::new();

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
        let version_label = if ver.level > 0 {
            Some(format!("{}级第{}赛季", ver.level, ver.season))
        } else {
            None
        };

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
