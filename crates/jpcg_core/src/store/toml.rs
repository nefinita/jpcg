// ============================================================================
// toml — TOML 读取解析
// 负责 TOML 配置文件的读取、解析、保存。
// ============================================================================

use crate::log::{error, info};
use crate::type_set::{skilltype, xinfa, xinfa::VersionInfo};
use serde::{Deserialize, Serialize};

use super::paths::data_dir;

// ============================================================================
// toml_input — 读取 .toml 文件内容为字符串
// 文件不存在或读取失败时返回 None（而非哨兵字符串）。
// ============================================================================

/// 读取 TOML 文件内容
/// - `profession`: 不带扩展名的文件路径（函数内部追加 .toml）
/// - 返回: 文件内容字符串；文件不存在或读取失败时返回 None
pub fn toml_input(profession: &str) -> Option<String> {
    let file_path = format!("{}.toml", profession);
    info(&format!("正在加载配置文件: {}", file_path));
    match std::fs::read_to_string(file_path) {
        Ok(content) => Some(content),
        Err(e) => {
            error(&format!("读取配置文件失败: {}", e));
            None
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

/// 心法技能配置（从 TOML 文件解析/写入）
#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct TomlConfig {
    pub xinfa: xinfa::XinfaConfig,        // 心法基础配置
    pub skill: Vec<skilltype::Skilltype>, // 技能列表（每个技能一条 [[skill]]）
    pub version: Option<VersionInfo>,     // 赛季版本信息（可选）
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
    let file_path_str = match file_path.to_str() {
        Some(s) => s.to_string(),
        None => {
            error("配置文件路径包含非法 UTF-8 字符");
            return TomlConfig::default();
        }
    };
    // 读取并解析
    let content = match toml_input(&file_path_str) {
        Some(c) => c,
        None => return TomlConfig::default(),
    };
    let mut config: TomlConfig = match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            error(&format!(
                "解析心法 '{}' 的 TOML 配置失败: {}",
                profession, e
            ));
            return TomlConfig::default();
        }
    };
    config.xinfa.profession = profession.to_string();
    config
}

/// 保存技能配置到心法数据文件
pub fn save_skill_toml(profession: &str, config: TomlConfig) -> Result<(), String> {
    let dir = data_dir().ok_or("无法获取数据目录")?;
    let file_path = dir.join(format!("{}.toml", profession));
    let content = toml::to_string_pretty(&config).map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(&file_path, &content).map_err(|e| format!("写入文件失败: {}", e))?;
    info(&format!("技能数据已保存到: {:?}", file_path));
    Ok(())
}
