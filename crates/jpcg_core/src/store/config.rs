// ============================================================================
// config — 玩家配置读写
// 保存文件 saved_config.toml 仅包含玩家、目标、心法配置（不含技能表）。
// ============================================================================

use crate::log::{error, info, warn};
use crate::type_set::buff::BuffConfig;
use crate::type_set::coefficient::CoefficientConfig;
use crate::type_set::{hostilepile, player, xinfa};
use serde::{Deserialize, Serialize};

use super::toml::toml_input;

// ============================================================================
// SaveConfig — 持久化保存的结构
// 保存文件 saved_config.toml 仅包含玩家、目标、心法配置（不含技能表）。
// ============================================================================

/// 持久化保存的配置（不含技能数据，技能数据从各自心法文件加载）
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SaveConfig {
    pub xinfa: xinfa::XinfaConfig,                   // 心法配置
    pub player: player::PlayerConfig,                // 玩家属性
    pub hostilepile: hostilepile::HostilepileConfig, // 目标属性
    #[serde(default)]
    pub buff: BuffConfig,       // 阵眼/奇穴增益
    #[serde(default)]
    pub coefficient: CoefficientConfig, // 系数设置
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
    let content = match toml_input("saved_config") {
        Some(c) => c,
        None => {
            warn("未找到已保存的配置，使用默认值。");
            return SaveConfig::default();
        }
    };
    let config: SaveConfig = match toml::from_str(&content) {
        Ok(data) => data,
        Err(_) => {
            warn("未找到已保存的配置，使用默认值。");
            SaveConfig::default()
        }
    };
    config
}

/// 导出当前配置为 TOML 字符串（不含技能表）
pub fn export_config_toml() -> Result<String, String> {
    let config = load_save_config();
    toml::to_string_pretty(&config).map_err(|e| format!("序列化失败: {}", e))
}

/// 导入配置 TOML 字符串并写入 saved_config.toml
pub fn import_config_toml(toml_str: &str) -> Result<(), String> {
    let config: SaveConfig =
        toml::from_str(toml_str).map_err(|e| format!("解析配置失败: {}", e))?;
    let content = toml::to_string_pretty(&config).map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write("saved_config.toml", content).map_err(|e| format!("写入文件失败: {}", e))
}
