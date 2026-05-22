// ============================================================================
// lib.rs — Tauri 桌面应用初始化
// 注册 Tauri 插件和所有自定义命令（invoke handler），
// 启动 Tauri 运行时。
// ============================================================================

mod commands;

/// Tauri 应用入口点（移动端也由此进入）
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 注册插件：系统默认打开文件/URL
        .plugin(tauri_plugin_opener::init())
        // 注册命令处理函数（前端通过 invoke 调用）
        .invoke_handler(tauri::generate_handler![
            commands::calculate::calculate_damage,
            commands::config::save_config_cmd,
            commands::config::load_config_cmd,
            commands::config::load_profession_config,
            commands::update::check_update,
            commands::update::perform_update,
            commands::forum::forum_list_files,
            commands::forum::forum_download_file,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用程序时发生错误");
}
