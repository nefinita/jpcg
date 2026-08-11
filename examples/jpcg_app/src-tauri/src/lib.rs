mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::calculate::calculate_damage,
            commands::config::save_config_cmd,
            commands::config::load_config_cmd,
            commands::config::list_professions_cmd,
            commands::config::load_profession_config,
            commands::update::check_update,
            commands::update::perform_update,
            commands::update::perform_app_update,
            commands::update::perform_modules_update,
            commands::forum::forum_list_files,
            commands::forum::forum_list_categories,
            commands::forum::forum_download_file,
            commands::forum::forum_list_downloaded,
            commands::forum::forum_delete_downloaded,
            commands::optimize::compute_derivatives,
            commands::data::load_skill_pool,
            commands::combo::calculate_combo_cmd,
            commands::combo::save_combo_preset,
            commands::combo::list_combo_presets,
            commands::combo::load_combo_preset,
            commands::combo::delete_combo_preset,
            commands::combo::export_config_cmd,
            commands::combo::import_config_cmd,
            commands::skill_editor::load_skill_data,
            commands::skill_editor::save_skill_data,
            commands::version::get_module_versions,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用程序时发生错误");
}
