mod commands;
mod filesystem;
mod models;
mod paths;
mod state;

use commands::{
    add_custom_publish_target, add_target_base_folder, add_workspace, delete_skill,
    load_manager_state, open_managed_folder, publish_skill, publish_skills_folder,
    remove_published_skill, remove_published_skills_folder, remove_target_base_folder,
    remove_workspace, scan_publish_targets, scan_skills, scan_workspace_git_details,
    set_default_publish_mode, set_enabled_publish_targets, set_scope_base_child,
    set_scope_base_folder,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_manager_state,
            scan_workspace_git_details,
            add_workspace,
            remove_workspace,
            open_managed_folder,
            set_scope_base_folder,
            set_scope_base_child,
            add_target_base_folder,
            remove_target_base_folder,
            add_custom_publish_target,
            set_default_publish_mode,
            set_enabled_publish_targets,
            scan_skills,
            scan_publish_targets,
            publish_skills_folder,
            publish_skill,
            remove_published_skills_folder,
            remove_published_skill,
            delete_skill
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
