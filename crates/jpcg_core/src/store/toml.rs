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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<jpcg_api::AttributeRules>, // 本心法的显式属性换算规则
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

/// 心法标识只能作为单个文件名使用，不能携带路径、扩展名或空白。
fn validate_profession(profession: &str) -> Result<(), String> {
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

/// 严格读取首个存在的文件；损坏或不可读的高优先级文件不得被低优先级文件遮盖。
fn read_config_strict(dirs: &[PathBuf], profession: &str) -> Result<Option<TomlConfig>, String> {
    validate_profession(profession)?;
    for dir in dirs {
        let path = dir.join(format!("{profession}.toml"));
        match std::fs::symlink_metadata(&path) {
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("读取 {} 元数据失败: {e}", path.display())),
        }
        // 包括悬空符号链接：文件已存在但不可读，不能回退到资源目录。
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取 {} 失败: {e}", path.display()))?;
        let mut config: TomlConfig =
            toml::from_str(&text).map_err(|e| format!("解析 {} 失败: {e}", path.display()))?;
        config.xinfa.profession = profession.to_string();
        return Ok(Some(config));
    }
    Ok(None)
}

fn validate_attribute_rules(config: &TomlConfig) -> Result<(), String> {
    let rules = config
        .attributes
        .as_ref()
        .ok_or("心法 TOML 缺少 [attributes] 规则表")?;
    let version = config
        .version
        .as_ref()
        .ok_or("含属性规则的心法 TOML 必须有 [version].level")?;
    if rules.level == 0 || version.level != rules.level {
        return Err(format!(
            "属性规则等级必须大于零且与同文件 version.level 一致（attributes.level={}, version.level={}）",
            rules.level, version.level
        ));
    }
    let main_stat = match config.xinfa.xinfa_nom.as_str() {
        "力道" | "liliang" => jpcg_api::BasicAttribute::Liliang,
        "身法" | "shenfa" => jpcg_api::BasicAttribute::Shenfa,
        "根骨" | "gengu" => jpcg_api::BasicAttribute::Gengu,
        "元气" | "yuanqi" => jpcg_api::BasicAttribute::Yuanqi,
        _ => {
            return Err(
                "心法 xinfa_nom 必须明确为力道、身法、根骨或元气，不能以体质为主属性".into(),
            );
        }
    };
    if rules.main_stat != main_stat {
        return Err("属性规则 main_stat 与同文件心法 xinfa_nom 不一致".into());
    }
    if rules.id.trim().is_empty()
        || rules.value_set.trim().is_empty()
        || rules.source.trim().is_empty()
    {
        return Err("属性规则 id/value_set/source 不能为空".into());
    }
    if rules.qixue_multiplier.denominator == 0
        || rules
            .conversions
            .iter()
            .any(|conversion| conversion.denominator == 0)
    {
        return Err("属性换算规则分母必须大于零".into());
    }
    Ok(())
}

/// 从当前心法 TOML 严格加载规则；不提供历史快照、常量或默认规则兜底。
/// 值集与请求是否一致由调用的配装层校验。
pub fn load_attribute_rules(profession: &str) -> Result<jpcg_api::AttributeRules, String> {
    load_attribute_rules_from(&super::paths::data_dirs(), profession)
}

pub(crate) fn load_attribute_rules_from(
    dirs: &[PathBuf],
    profession: &str,
) -> Result<jpcg_api::AttributeRules, String> {
    let mut config = read_config_strict(dirs, profession)?
        .ok_or_else(|| format!("找不到心法 {profession} 的 TOML 配置"))?;
    validate_attribute_rules(&config)?;
    config
        .attributes
        .take()
        .ok_or("心法 TOML 缺少 [attributes] 规则表".into())
}

/// 保存技能配置到可写数据根。旧编辑器没有 attributes 字段时保留当前优先配置中的规则。
/// 不能通过省略字段或 null 删除规则；有规则时禁止写入与其等级不一致的版本。
pub fn save_skill_toml(profession: &str, config: TomlConfig) -> Result<(), String> {
    let root = super::paths::data_root_writable().ok_or("无法获取可写数据目录")?;
    save_skill_toml_to(
        &super::paths::data_dirs(),
        &root.join("shuxing"),
        profession,
        config,
    )
}

fn save_skill_toml_to(
    dirs: &[PathBuf],
    writable_dir: &std::path::Path,
    profession: &str,
    mut config: TomlConfig,
) -> Result<(), String> {
    validate_profession(profession)?;
    if config.attributes.is_none()
        && let Some(existing) = read_config_strict(dirs, profession)?
    {
        config.attributes = existing.attributes;
        if config.attributes.is_some() && config.version.is_none() {
            config.version = existing.version;
        }
    }
    if config.attributes.is_some() {
        validate_attribute_rules(&config)?;
    }
    let content = toml::to_string_pretty(&config).map_err(|e| format!("序列化失败: {e}"))?;
    std::fs::create_dir_all(writable_dir).map_err(|e| format!("创建数据目录失败: {e}"))?;
    let file_path = writable_dir.join(format!("{profession}.toml"));
    std::fs::write(&file_path, &content).map_err(|e| format!("写入文件失败: {e}"))?;
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

#[cfg(test)]
mod attribute_rule_tests {
    use super::*;

    // 专用合成测试数据；数字用于验证数据流，不代表任何游戏版本或门派。
    const RULES: &str = r#"
[xinfa]
xinfa_nom = "根骨"
[version]
level = 50
season = 99
modified = 20261031
[attributes]
id = "synthetic-rule"
level = 50
value_set = "synthetic-50"
source = "unit test only; not game coefficients"
main_stat = "gengu"
damage_kind = "lunar"
qixue_multiplier = { numerator = 1, denominator = 1 }
conversions = [{ from = "tizhi", to = "qixue_base", numerator = 2, denominator = 1 }]
"#;

    fn tmp(tag: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("jpcg-attribute-rules-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn strict_rules_follow_priority_and_never_fallback_from_bad_override() {
        let user = tmp("priority-user");
        let bundle = tmp("priority-bundle");
        std::fs::write(bundle.join("mowen.toml"), RULES).unwrap();
        let dirs = [user.clone(), bundle.clone()];
        assert_eq!(
            load_attribute_rules_from(&dirs, "mowen").unwrap().id,
            "synthetic-rule"
        );
        std::fs::write(
            user.join("mowen.toml"),
            RULES.replace("synthetic-rule", "user-rule"),
        )
        .unwrap();
        assert_eq!(
            load_attribute_rules_from(&dirs, "mowen").unwrap().id,
            "user-rule"
        );
        for bad in ["[attributes", "[version]\nlevel = 50\n", ""] {
            std::fs::write(user.join("mowen.toml"), bad).unwrap();
            assert!(load_attribute_rules_from(&dirs, "mowen").is_err());
        }
        std::fs::remove_file(user.join("mowen.toml")).unwrap();
        std::fs::create_dir(user.join("mowen.toml")).unwrap();
        assert!(
            load_attribute_rules_from(&dirs, "mowen")
                .unwrap_err()
                .contains("读取")
        );
        #[cfg(unix)]
        {
            std::fs::remove_dir(user.join("mowen.toml")).unwrap();
            std::os::unix::fs::symlink("missing-target", user.join("mowen.toml")).unwrap();
            assert!(load_attribute_rules_from(&dirs, "mowen").is_err());
        }
        assert!(load_attribute_rules_from(&dirs, "missing").is_err());
        let _ = std::fs::remove_dir_all(user);
        let _ = std::fs::remove_dir_all(bundle);
    }

    #[test]
    fn rules_require_explicit_coefficients_matching_positive_level_and_safe_id() {
        let dir = tmp("invalid");
        let dirs = [dir.clone()];
        let invalid = [
            RULES.replace("level = 50", "level = 0"),
            RULES.replacen("level = 50", "level = 130", 1),
            RULES.replace(
                "[version]\nlevel = 50\nseason = 99\nmodified = 20261031\n",
                "",
            ),
            RULES.replace("denominator = 1", "denominator = 0"),
            RULES.replace("numerator = 2, ", ""),
            RULES.replace("numerator = 1, ", ""),
            RULES.replace("id = \"synthetic-rule\"", "id = \"\""),
            RULES.replace(
                "source = \"unit test only; not game coefficients\"",
                "source = \"\"",
            ),
            RULES.replace("main_stat =", "unknown = 1\nmain_stat ="),
            RULES.replace("qixue_multiplier = {", "qixue_multiplier = { unknown = 1,"),
            RULES.replace("conversions = [{", "conversions = [{ unknown = 1,"),
        ];
        for text in invalid {
            std::fs::write(dir.join("mowen.toml"), text).unwrap();
            assert!(load_attribute_rules_from(&dirs, "mowen").is_err());
        }
        for id in [
            "",
            "../mowen",
            "a/b",
            "a\\b",
            "mowen.toml",
            " mowen",
            "/tmp/mowen",
        ] {
            assert!(
                load_attribute_rules_from(&dirs, id)
                    .unwrap_err()
                    .contains("心法标识")
            );
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn rules_main_stat_must_match_same_file_profession() {
        let dir = tmp("main-stat");
        let dirs = [dir.clone()];
        for (name, main_stat) in [
            ("力道", "liliang"),
            ("liliang", "liliang"),
            ("身法", "shenfa"),
            ("shenfa", "shenfa"),
            ("根骨", "gengu"),
            ("gengu", "gengu"),
            ("元气", "yuanqi"),
            ("yuanqi", "yuanqi"),
        ] {
            let text = RULES
                .replace("xinfa_nom = \"根骨\"", &format!("xinfa_nom = {name:?}"))
                .replace(
                    "main_stat = \"gengu\"",
                    &format!("main_stat = {main_stat:?}"),
                );
            std::fs::write(dir.join("mowen.toml"), text).unwrap();
            assert!(load_attribute_rules_from(&dirs, "mowen").is_ok());
        }
        for text in [
            RULES.replace("main_stat = \"gengu\"", "main_stat = \"yuanqi\""),
            RULES.replace("main_stat = \"gengu\"", "main_stat = \"tizhi\""),
            RULES.replace("xinfa_nom = \"根骨\"", "xinfa_nom = \"unknown\""),
            RULES.replace("xinfa_nom = \"根骨\"", "xinfa_nom = \"体质\""),
            RULES.replace("xinfa_nom = \"根骨\"", ""),
        ] {
            std::fs::write(dir.join("mowen.toml"), text).unwrap();
            assert!(load_attribute_rules_from(&dirs, "mowen").is_err());
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn old_editor_save_preserves_rules_and_refuses_incompatible_or_corrupt_source() {
        let user = tmp("save-user");
        let bundle = tmp("save-bundle");
        let file = user.join("mowen.toml");
        std::fs::write(bundle.join("mowen.toml"), RULES).unwrap();
        let dirs = [user.clone(), bundle.clone()];
        let mut legacy = TomlConfig::default();
        legacy.xinfa.xinfa_name = "legacy editor changed name".into();
        legacy.xinfa.xinfa_nom = "根骨".into();
        save_skill_toml_to(&dirs, &user, "mowen", legacy).unwrap();
        let loaded = read_config_strict(&dirs, "mowen").unwrap().unwrap();
        assert_eq!(loaded.xinfa.xinfa_name, "legacy editor changed name");
        assert_eq!(loaded.version.as_ref().unwrap().level, 50);
        assert_eq!(loaded.attributes.as_ref().unwrap().id, "synthetic-rule");
        let original = std::fs::read_to_string(&file).unwrap();
        let mismatch = TomlConfig {
            version: Some(VersionInfo {
                level: 130,
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(save_skill_toml_to(&dirs, &user, "mowen", mismatch).is_err());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), original);
        std::fs::write(&file, "[attributes\n").unwrap();
        assert!(save_skill_toml_to(&dirs, &user, "mowen", TomlConfig::default()).is_err());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "[attributes\n");
        let valid: TomlConfig = toml::from_str(RULES).unwrap();
        save_skill_toml_to(&dirs, &user, "mowen", valid).unwrap();
        assert!(load_attribute_rules_from(&dirs, "mowen").is_ok());
        let _ = std::fs::remove_dir_all(user);
        let _ = std::fs::remove_dir_all(bundle);
    }
}
