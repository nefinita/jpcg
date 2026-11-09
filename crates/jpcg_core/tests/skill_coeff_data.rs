//! 防回归：`data/shuxing/mowen.toml` 的倍率必须与截图反推的整数倍率一致。
//!
//! 真源是 `data/skill-shots/mowen-final.json`（tooltip 反推），
//! `scripts/apply-skill-coeff.py` 负责写回 TOML；json-to-toml 重新生成数据会覆盖，
//! 这个测试用来在 CI 上兜底。

use std::path::PathBuf;

const BASE: f64 = 160.0;

fn data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data")
        .join(name)
}

#[test]
fn mowen_coefficients_match_screenshots() {
    let json_text = std::fs::read_to_string(data_path("skill-shots/mowen-final.json"))
        .expect("read mowen-final.json");
    let json: serde_json::Value = serde_json::from_str(&json_text).expect("parse json");
    let toml_text =
        std::fs::read_to_string(data_path("shuxing/mowen.toml")).expect("read mowen.toml");
    let toml: toml::Value = toml::from_str(&toml_text).expect("parse toml");
    let skills = toml["skill"].as_array().expect("skill array");

    let mut checked = 0;
    for entry in json["skills"].as_array().expect("skills array") {
        // 计算使用连续倍率（攻击加成 / 攻击力），比率整数只用于展示。
        let (Some(id), Some(ratio)) = (entry["skill_id"].as_u64(), entry["ratio_raw"].as_f64())
        else {
            continue;
        };
        let sub = entry["sub_id"].as_u64().unwrap_or(0);
        let want = ratio;
        let found = skills.iter().find(|s| {
            s["skill_id"].as_integer() == Some(id as i64)
                && s["sub_id"].as_integer() == Some(sub as i64)
        });
        let Some(skill) = found else {
            panic!("缺少 skill_id={id} sub_id={sub}");
        };
        let got = skill["atk_xishu"]
            .as_float()
            .or_else(|| skill["atk_xishu"].as_integer().map(|v| v as f64))
            .expect("atk_xishu");
        assert!(
            (got - want).abs() < 1e-6,
            "skill_id={id} 倍率不一致：toml {got} != json {want}"
        );
        if let Some(min) = entry["base_min"].as_i64() {
            assert_eq!(skill["base_damage1"].as_integer(), Some(min));
        }
        if let Some(max) = entry["base_max"].as_i64() {
            assert_eq!(skill["base_damage2"].as_integer(), Some(max));
        }
        checked += 1;
    }
    assert!(checked >= 10, "至少校验 10 个技能，实际 {checked}");
}
