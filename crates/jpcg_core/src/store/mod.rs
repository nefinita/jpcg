// ============================================================================
// store — 数据存储子模块
// 原 io.rs 拆分而来，按职责分组：
//   paths      — 路径定位（数据目录/连招目录）
//   toml       — TOML 读取解析（心法技能配置）
//   config     — 玩家配置读写（saved_config.toml 导入导出）
//   combo      — 连招预设 CRUD
//   profession — 门派列表扫描
// ============================================================================

pub mod combo;
pub mod config;
pub mod paths;
pub mod profession;
pub mod toml;

pub use combo::{delete_combo_preset, list_combo_presets, load_combo_preset, save_combo_preset};
pub use config::{
    SaveConfig, export_config_toml, import_config_toml, load_save_config, save_config,
};
pub use paths::data_dir;
pub use profession::list_available_professions;
pub use toml::{TomlConfig, load_config, save_skill_toml, toml_input};
