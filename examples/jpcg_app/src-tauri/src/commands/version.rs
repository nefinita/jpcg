use serde::Serialize;

/// 各模块库（dll）版本，供 UI 展示
#[derive(Debug, Serialize)]
pub struct ModuleVersions {
    /// 计算引擎 core 版本（= release tag / 安装包命名源）
    pub core: String,
    /// 更新库 update 版本
    pub update: String,
    /// 常量库 const 版本（等级.赛季.日期，如 130.3.20260602）
    pub r#const: String,
    /// 连招引擎 combo 版本
    pub combo: String,
    /// 当前 app UI 版本
    pub app: String,
}

#[tauri::command]
pub async fn get_module_versions() -> Result<ModuleVersions, String> {
    #[cfg(feature = "static")]
    {
        Ok(ModuleVersions {
            core: jpcg_core::CORE_VERSION.to_string(),
            update: jpcg_update::UPDATE_VERSION.to_string(),
            r#const: jpcg_const::CONST_VERSION.to_string(),
            combo: jpcg_combo::COMBO_VERSION.to_string(),
            app: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    #[cfg(feature = "dynamic")]
    {
        let (core, r#const, update) = crate::commands::ffi_bridge::module_versions();
        let combo = crate::commands::ffi_bridge::combo_version().unwrap_or_default();
        Ok(ModuleVersions {
            core,
            update,
            r#const,
            combo,
            app: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}
