mod commands;
mod filesystem;
mod models;
mod paths;
mod state;
mod update;

use commands::{
    add_target_base_folder, add_workspace, check_for_updates, delete_skill, load_manager_state,
    open_managed_folder, publish_skill, publish_skills_folder, remove_published_skill,
    remove_published_skills_folder, remove_target_base_folder, remove_workspace,
    scan_publish_targets, scan_skills, scan_workspace_git_details, set_default_publish_mode,
    set_enabled_publish_targets, set_scope_base_child, set_scope_base_folder,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
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
            set_default_publish_mode,
            set_enabled_publish_targets,
            scan_skills,
            scan_publish_targets,
            publish_skills_folder,
            publish_skill,
            remove_published_skills_folder,
            remove_published_skill,
            delete_skill,
            check_for_updates
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
