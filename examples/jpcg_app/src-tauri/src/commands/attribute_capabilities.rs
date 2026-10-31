//! 动态模块的属性语义门禁；不依赖 Tauri，供无 GUI 回归测试直接加载。
use serde_json::Value;

fn converted_player(value: &Value) -> bool {
    value
        .pointer("/player/zhuanhua_gongji")
        .is_some_and(|extra| !extra.is_null())
}

fn converted_config(text: &str) -> Result<bool, String> {
    let value: toml::Value = toml::from_str(text).map_err(|e| format!("配置解析失败: {e}"))?;
    Ok(value
        .get("player")
        .and_then(|player| player.get("zhuanhua_gongji"))
        .is_some())
}

/// `saved_config` 仅用于 load_config/export_config：旧库会在返回前丢失未知 marker。
/// 探测失败不得把请求转交旧库，否则额外攻击可能被重复换算或在保存时丢失。
pub(super) fn guard(
    method: &str,
    request: &str,
    saved_config: Option<&str>,
    probe: impl FnOnce() -> Result<Value, String>,
) -> Result<(), String> {
    let value: Value = serde_json::from_str(request).map_err(|e| format!("请求解析失败: {e}"))?;
    let mut converted = converted_player(&value);
    if method == "import_config"
        && let Some(text) = value.get("toml_str").and_then(Value::as_str)
    {
        converted |= converted_config(text)?;
    }
    if matches!(method, "load_config" | "export_config")
        && let Some(text) = saved_config
    {
        converted |= converted_config(text)?;
    }
    if !converted {
        return Ok(());
    }
    let capabilities = probe().map_err(|_| {
        "当前动态计算模块无法确认装备属性换算支持，请更新 core/combo 模块后重试".to_string()
    })?;
    if capabilities
        .get("supported")
        .and_then(Value::as_array)
        .is_some_and(|supported| {
            supported
                .iter()
                .any(|v| v.as_str() == Some("converted_attack_v1"))
        })
    {
        Ok(())
    } else {
        Err("当前动态计算模块不支持装备属性换算，请更新 core/combo 模块后重试".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn supported() -> Result<Value, String> {
        Ok(serde_json::json!({"supported":["converted_attack_v1"]}))
    }

    #[test]
    fn legacy_input_does_not_require_new_module() {
        for request in [r#"{"player":{}}"#, r#"{"player":{"zhuanhua_gongji":null}}"#] {
            assert!(guard("calculate", request, None, || panic!("无需探测旧输入")).is_ok());
        }
    }

    #[test]
    fn converted_input_requires_confirmed_capability_including_zero() {
        for method in [
            "calculate",
            "compute_derivatives",
            "save_config",
            "calculate_combo",
            "calculate_timeline",
        ] {
            for extra in [0, 251] {
                let request = serde_json::json!({"player":{"zhuanhua_gongji":extra}}).to_string();
                assert!(guard(method, &request, None, supported).is_ok());
                assert!(guard(method, &request, None, || Err("未知方法".into())).is_err());
                assert!(
                    guard(method, &request, None, || Ok(
                        serde_json::json!({"supported":[]})
                    ))
                    .is_err()
                );
                assert!(
                    guard(method, &request, None, || Ok(serde_json::json!([
                        "converted_attack_v1"
                    ])))
                    .is_err()
                );
            }
        }
    }

    #[test]
    fn old_modules_cannot_silently_strip_marker_from_saved_or_imported_config() {
        let text = "[player]\nzhuanhua_gongji = 0\n";
        for method in ["load_config", "export_config"] {
            assert!(guard(method, "{}", Some(text), supported).is_ok());
            assert!(guard(method, "{}", Some(text), || Err("未知方法".into())).is_err());
        }
        let request = serde_json::json!({"toml_str":text}).to_string();
        assert!(guard("import_config", &request, None, || Err("未知方法".into())).is_err());
        assert!(guard("import_config", &request, None, supported).is_ok());
    }
}
