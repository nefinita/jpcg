//! 桌面宿主入口。前端只能传配装或 TOML 内容，不能指定文件路径。
use crate::{Catalog, Composition, Loadout};
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;

pub fn catalog_dir() -> Result<PathBuf, String> {
    jpcg_core::data_paths::gear_dir()
        .ok_or("找不到配装模板，请安装 gear 数据或设置 JPCG_DATA_DIR".into())
}

pub fn catalog() -> Result<Catalog, String> {
    Catalog::load(&catalog_dir()?)
}
pub fn compose(catalog: &Catalog, loadout: &Loadout) -> Result<Composition, String> {
    let composed = catalog.compose(loadout)?;
    Ok(Composition {
        player: crate::player(&composed),
        composed,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Import {
    toml: String,
}

pub fn dispatch(catalog: &Catalog, method: &str, input: &str) -> Result<Value, String> {
    let value = match method {
        "catalog" => {
            let _: Empty = serde_json::from_str(input).map_err(|e| e.to_string())?;
            catalog.validate()?;
            serde_json::to_value(catalog)
        }
        "compose" => {
            let loadout: Loadout = serde_json::from_str(input).map_err(|e| e.to_string())?;
            serde_json::to_value(compose(catalog, &loadout)?)
        }
        "export" => {
            let loadout: Loadout = serde_json::from_str(input).map_err(|e| e.to_string())?;
            catalog.compose(&loadout)?;
            serde_json::to_value(toml::to_string_pretty(&loadout).map_err(|e| e.to_string())?)
        }
        "import" => {
            let request: Import = serde_json::from_str(input).map_err(|e| e.to_string())?;
            let loadout: Loadout = toml::from_str(&request.toml).map_err(|e| e.to_string())?;
            serde_json::to_value(compose(catalog, &loadout)?)
        }
        _ => return Err(format!("未知配装方法: {method}")),
    };
    value.map_err(|e| e.to_string())
}

pub fn call(method: &str, input: &str) -> Result<Value, String> {
    dispatch(&catalog()?, method, input)
}
