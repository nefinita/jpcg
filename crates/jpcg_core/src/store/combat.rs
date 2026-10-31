// ============================================================================
// store::combat — 战斗时序 overlay 读取（data/combat/{门派}.toml）
//
// 读取顺序与 shuxing 数据一致（用户数据目录优先，其次随包资源），
// 取第一个存在 `{门派}.toml` 的 combat 目录；文件不存在 = 无 overlay（返回默认空表）。
// 解析失败 / 校验失败都带明确错误信息，便于数据同学定位。
// ============================================================================

use std::path::PathBuf;

use crate::log::{error, info};
use crate::type_set::combat::CombatData;
use crate::type_set::skilltype::Skilltype;

/// 读取某门派的战斗 overlay；无文件时返回空 `CombatData`（不报错）。
/// （B 阶段接 FFI/命令前先保留入口，故允许未使用）
#[allow(dead_code)]
pub fn load_combat(profession: &str) -> CombatData {
    match load_combat_from(&super::paths::combat_dirs(), profession) {
        Ok(data) => data,
        Err(e) => {
            error(&e);
            CombatData::default()
        }
    }
}

/// 显式目录列表版（便于测试）：取首个存在文件的目录；解析后做结构校验。
#[allow(dead_code)]
pub(crate) fn load_combat_from(dirs: &[PathBuf], profession: &str) -> Result<CombatData, String> {
    for dir in dirs {
        let file_path = dir.join(format!("{}.toml", profession));
        if !file_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("读取 combat 数据失败（{}）: {}", file_path.display(), e))?;
        let mut data: CombatData = toml::from_str(&content)
            .map_err(|e| format!("解析 combat 数据失败（{}）: {}", file_path.display(), e))?;
        if data.meta.profession.trim().is_empty() {
            data.meta.profession = profession.to_string();
        }
        // 结构校验（id 唯一 / 引用非空 / 数值合法）
        data.validate()
            .map_err(|e| format!("{}（文件：{}）", e, file_path.display()))?;
        info(&format!("已加载战斗时序数据: {}", file_path.display()));
        return Ok(data);
    }
    Ok(CombatData::default())
}

/// 读取某门派技能表 + combat overlay，并做「技能引用」交叉校验。
///
/// 返回套用 overlay 后的技能表（时序字段已覆盖）与 overlay 数据本身
/// （形态 / GCD 通道 / buff / 秘籍 / 奇穴仍需调用方按需使用）。
#[allow(dead_code)]
pub fn load_skills_with_combat(profession: &str) -> Result<(Vec<Skilltype>, CombatData), String> {
    let toml_cfg = super::load_config(profession);
    let skills = toml_cfg.skill;
    let data = load_combat_from(&super::paths::combat_dirs(), profession)?;
    data.validate_against_skills(&skills)?;
    Ok((data.apply_overlay(&skills), data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_set::combat::CombatData;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("jpcg-combat-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    const SAMPLE: &str = r#"
[meta]
profession = "mowen"

[[form]]
id = "single"
name = "单持"
gcd_base = 1.5
initial = true

[[skill_combat]]
skill_id = 101
sub_id = 0
gcd_lane = "single"
cast_time = 2.5
"#;

    #[test]
    fn missing_file_is_empty_overlay() {
        let dir = tmp("missing");
        let data = load_combat_from(&[dir.clone()], "mowen").unwrap();
        assert!(data.form.is_empty() && data.skill_combat.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn loads_and_validates_overlay() {
        let dir = tmp("load");
        std::fs::write(dir.join("mowen.toml"), SAMPLE).unwrap();
        let data = load_combat_from(&[dir.clone()], "mowen").unwrap();
        assert_eq!(data.meta.profession, "mowen");
        assert_eq!(data.skill_combat.len(), 1);
        assert_eq!(data.skill_combat[0].cast_time, Some(2.5));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_error_names_file() {
        let dir = tmp("bad");
        std::fs::write(dir.join("mowen.toml"), "[[form]]\nid = 1\n").unwrap();
        let err = load_combat_from(&[dir.clone()], "mowen").unwrap_err();
        assert!(err.contains("解析 combat 数据失败"), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn first_existing_dir_wins() {
        let user = tmp("user");
        let resources = tmp("res");
        std::fs::write(user.join("mowen.toml"), SAMPLE).unwrap();
        std::fs::write(
            resources.join("mowen.toml"),
            SAMPLE.replace("gcd_base = 1.5", "gcd_base = 1.0"),
        )
        .unwrap();
        let data = load_combat_from(&[user.clone(), resources.clone()], "mowen").unwrap();
        assert_eq!(data.form[0].gcd_base, 1.5);
        let _ = std::fs::remove_dir_all(&user);
        let _ = std::fs::remove_dir_all(&resources);
    }

    #[test]
    fn roundtrip_serialize() {
        let data: CombatData = toml::from_str(SAMPLE).unwrap();
        let text = toml::to_string(&data).unwrap();
        let back: CombatData = toml::from_str(&text).unwrap();
        assert_eq!(back.skill_combat[0].cast_time, Some(2.5));
    }
}
