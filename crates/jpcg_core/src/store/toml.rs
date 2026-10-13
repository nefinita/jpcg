// ============================================================================
// toml — TOML 读取解析
// 负责 TOML 配置文件的读取、解析、保存。
// ============================================================================

use crate::log::{error, info};
use crate::type_set::{skilltype, xinfa, xinfa::VersionInfo};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

/// 按心法名加载技能配置表；在 `data_dirs()` 中取**首个**存在该文件的目录
/// （安装版用户覆盖优先，其后随包资源）——修复“保存单个心法后遮蔽其他心法”。
pub fn load_config(profession: &str) -> TomlConfig {
    load_config_from(&super::paths::data_dirs(), profession)
}

/// 显式目录列表版（便于测试多目录合并）
pub(crate) fn load_config_from(dirs: &[PathBuf], profession: &str) -> TomlConfig {
    for dir in dirs {
        let file_path = dir.join(format!("{}.toml", profession));
        if !file_path.exists() {
            continue;
        }
        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                error(&format!("读取配置文件失败: {}", e));
                return TomlConfig::default();
            }
        };
        match toml::from_str::<TomlConfig>(&content) {
            Ok(mut config) => {
                config.xinfa.profession = profession.to_string();
                return config;
            }
            Err(e) => {
                error(&format!(
                    "解析心法 '{}' 的 TOML 配置失败: {}",
                    profession, e
                ));
                return TomlConfig::default();
            }
        }
    }
    TomlConfig::default()
}

/// 保存技能配置到心法数据文件
/// 写入**可写数据根**的 `shuxing/`（安装版 bundle 资源只读，不能直接写）
pub fn save_skill_toml(profession: &str, config: TomlConfig) -> Result<(), String> {
    let root = super::paths::data_root_writable().ok_or("无法获取可写数据目录")?;
    let dir = root.join("shuxing");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建数据目录失败: {}", e))?;
    let file_path = dir.join(format!("{}.toml", profession));
    let content = toml::to_string_pretty(&config).map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(&file_path, &content).map_err(|e| format!("写入文件失败: {}", e))?;
    info(&format!("技能数据已保存到: {:?}", file_path));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("jpcg-toml-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn load_config_merges_across_dirs_user_first() {
        let resources = tmp("lc-res");
        let user = tmp("lc-user");
        std::fs::write(
            resources.join("a.toml"),
            "[xinfa]\nxinfa_name = \"A资源\"\nxinfa_nom = \"gengu\"\n",
        )
        .unwrap();
        std::fs::write(
            resources.join("b.toml"),
            "[xinfa]\nxinfa_name = \"B资源\"\nxinfa_nom = \"gengu\"\n",
        )
        .unwrap();
        std::fs::write(
            user.join("a.toml"),
            "[xinfa]\nxinfa_name = \"A用户\"\nxinfa_nom = \"gengu\"\n",
        )
        .unwrap();

        let dirs = vec![user.clone(), resources.clone()];
        // 用户覆盖优先
        assert_eq!(load_config_from(&dirs, "a").xinfa.xinfa_name, "A用户");
        // 用户目录缺失的仍从资源读取（不再被遮蔽）
        assert_eq!(load_config_from(&dirs, "b").xinfa.xinfa_name, "B资源");
        // 都不存在 → 默认空
        assert_eq!(load_config_from(&dirs, "c").xinfa.xinfa_name, "");

        let _ = std::fs::remove_dir_all(&resources);
        let _ = std::fs::remove_dir_all(&user);
    }
}
