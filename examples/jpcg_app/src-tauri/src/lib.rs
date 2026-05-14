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
            commands::calculate_damage,       // 伤害计算
            commands::save_config_cmd,        // 保存配置
            commands::load_config_cmd,        // 加载配置
            commands::load_profession_config,  // 按心法加载
            commands::check_update,           // 检查更新
            commands::perform_update,         // 执行更新
            commands::forum_list_files,       // 论坛文件列表
            commands::forum_download_file,    // 论坛文件下载
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用程序时发生错误");
}
