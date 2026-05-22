// ============================================================================
// io — 文件 IO 模块
// 负责 TOML 配置文件的读取、解析、保存。
// 数据文件位于 {exe_dir}/data/pvp36500/{心法名}.toml，
// 保存文件位于工作目录下的 saved_config.toml。
// ============================================================================

use crate::log::{error, info, warn};
use crate::type_set::skilltype::Skilltype;
use crate::type_set::xinfa::XinfaConfig;
use crate::type_set::{hostilepile, player, skilltype, xinfa};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub fn data_dir() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(|p| p.join("data").join("pvp36500"))
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
// 对应 data/pvp36500/{心法名}.toml 文件格式:
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
pub struct TomlConfig {
    pub xinfa: xinfa::XinfaConfig,         // 心法基础配置
    pub skill: Vec<skilltype::Skilltype>,   // 技能列表（每个技能一条 [[skill]]）
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
}

// ============================================================================
// load_config — 按心法名加载技能配置表
// 路径: {exe_dir}/data/pvp36500/{profession}.toml
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
    match toml::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            error(&format!("解析心法 '{}' 的 TOML 配置失败: {}", profession, e));
            TomlConfig::default()
        }
    }
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
