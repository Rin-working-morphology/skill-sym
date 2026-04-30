use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Manager};

use crate::{
    models::{AppState, CommandResult, PublishMode, ScopeSelection},
    paths::{path_to_string, readable_io_error},
};

const STATE_FILE_NAME: &str = "skillsym-state.json";
pub(crate) const DEFAULT_BASE_FOLDER_NAME: &str = ".claude";
pub(crate) const SKILLS_FOLDER_NAME: &str = "skills";

pub(crate) struct SupportedTarget {
    pub(crate) id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) folder_name: &'static str,
}

pub(crate) const SUPPORTED_TARGETS: &[SupportedTarget] = &[
    SupportedTarget {
        id: "claude",
        name: "Claude",
        folder_name: ".claude",
    },
    SupportedTarget {
        id: "codex",
        name: "Codex",
        folder_name: ".codex",
    },
    SupportedTarget {
        id: "gemini",
        name: "Gemini",
        folder_name: ".gemini",
    },
    SupportedTarget {
        id: "qoder",
        name: "Qoder",
        folder_name: ".qoder",
    },
];

pub(crate) fn load_and_prepare_state(app: &AppHandle) -> CommandResult<AppState> {
    let mut state = load_state(app)?;
    migrate_legacy_global_base_folder(app, &mut state)?;
    normalize_enabled_targets(&mut state);
    save_state(app, &state)?;
    ensure_global_skills_dir(&state)?;
    Ok(state)
}

pub(crate) fn load_state(app: &AppHandle) -> CommandResult<AppState> {
    let path = state_file_path(app)?;
    if !path.exists() {
        return default_state(app);
    }

    let content = fs::read_to_string(&path).map_err(readable_io_error)?;
    serde_json::from_str(&content).map_err(|error| format!("无法读取已保存的应用状态：{error}"))
}

pub(crate) fn save_state(app: &AppHandle, state: &AppState) -> CommandResult<()> {
    let path = state_file_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(readable_io_error)?;
    }
    let content = serde_json::to_string_pretty(state)
        .map_err(|error| format!("无法序列化应用状态：{error}"))?;
    fs::write(path, content).map_err(readable_io_error)
}

pub(crate) fn default_publish_mode() -> PublishMode {
    PublishMode::Symlink
}

pub(crate) fn default_enabled_target_ids() -> Vec<String> {
    SUPPORTED_TARGETS
        .iter()
        .map(|target| target.id.to_string())
        .collect()
}

pub(crate) fn normalize_enabled_targets(state: &mut AppState) {
    state
        .enabled_target_ids
        .retain(|id| SUPPORTED_TARGETS.iter().any(|target| target.id == id));
    state.enabled_target_ids.sort();
    state.enabled_target_ids.dedup();
}

pub(crate) fn global_base_child(app: &AppHandle, folder_name: &str) -> CommandResult<PathBuf> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| format!("无法定位用户主目录：{error}"))?;
    Ok(global_base_child_from_home(&home, folder_name))
}

pub(crate) fn scope_skills_path(
    state: &AppState,
    scope: &ScopeSelection,
) -> CommandResult<PathBuf> {
    Ok(scope_base_path(state, scope)?.join(SKILLS_FOLDER_NAME))
}

pub(crate) fn workspace_id(path: &str) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("workspace-{:x}", hasher.finish())
}

pub(crate) fn target_root_path(
    app: &AppHandle,
    state: &AppState,
    scope: &ScopeSelection,
) -> CommandResult<PathBuf> {
    match scope.kind.as_str() {
        "global" => app
            .path()
            .home_dir()
            .map_err(|error| format!("无法定位用户主目录：{error}")),
        "workspace" => {
            let workspace_id = scope
                .workspace_id
                .as_ref()
                .ok_or_else(|| "工作区范围需要工作区 ID。".to_string())?;
            let workspace = state
                .workspaces
                .iter()
                .find(|item| &item.id == workspace_id)
                .ok_or_else(|| "未找到工作区。".to_string())?;
            Ok(PathBuf::from(&workspace.path))
        }
        _ => Err("未知的范围类型。".to_string()),
    }
}

fn default_state(app: &AppHandle) -> CommandResult<AppState> {
    let global_base_folder = global_base_child(app, DEFAULT_BASE_FOLDER_NAME)?;

    Ok(AppState {
        global_base_folder: path_to_string(&global_base_folder),
        workspaces: Vec::new(),
        target_base_folders: Vec::new(),
        enabled_target_ids: default_enabled_target_ids(),
        default_publish_mode: default_publish_mode(),
    })
}

fn state_file_path(app: &AppHandle) -> CommandResult<PathBuf> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法定位应用数据目录：{error}"))?;
    Ok(app_data.join(STATE_FILE_NAME))
}

fn global_base_child_from_home(home: &Path, folder_name: &str) -> PathBuf {
    home.join(folder_name)
}

fn legacy_global_base_child_from_app_data(app_data: &Path, folder_name: &str) -> PathBuf {
    app_data.join("global").join(folder_name)
}

fn migrate_legacy_global_base_folder(app: &AppHandle, state: &mut AppState) -> CommandResult<()> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| format!("无法定位用户主目录：{error}"))?;
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法定位应用数据目录：{error}"))?;
    migrate_legacy_global_base_folder_from_paths(state, &home, &app_data);
    Ok(())
}

fn migrate_legacy_global_base_folder_from_paths(
    state: &mut AppState,
    home: &Path,
    app_data: &Path,
) {
    for folder_name in [DEFAULT_BASE_FOLDER_NAME, ".agents"] {
        if state.global_base_folder
            == path_to_string(&legacy_global_base_child_from_app_data(
                app_data,
                folder_name,
            ))
        {
            state.global_base_folder =
                path_to_string(&global_base_child_from_home(home, folder_name));
            break;
        }
    }
}

fn ensure_global_skills_dir(state: &AppState) -> CommandResult<()> {
    fs::create_dir_all(Path::new(&state.global_base_folder).join(SKILLS_FOLDER_NAME))
        .map_err(readable_io_error)
}

fn scope_base_path(state: &AppState, scope: &ScopeSelection) -> CommandResult<PathBuf> {
    match scope.kind.as_str() {
        "global" => Ok(PathBuf::from(&state.global_base_folder)),
        "workspace" => {
            let workspace_id = scope
                .workspace_id
                .as_ref()
                .ok_or_else(|| "工作区范围需要工作区 ID。".to_string())?;
            let workspace = state
                .workspaces
                .iter()
                .find(|item| &item.id == workspace_id)
                .ok_or_else(|| "未找到工作区。".to_string())?;
            Ok(PathBuf::from(&workspace.base_folder))
        }
        _ => Err("未知的范围类型。".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        default_enabled_target_ids, default_publish_mode, global_base_child_from_home,
        legacy_global_base_child_from_app_data, migrate_legacy_global_base_folder_from_paths,
        AppState, DEFAULT_BASE_FOLDER_NAME,
    };
    use crate::{
        models::{PublishMode, Workspace},
        paths::path_to_string,
    };

    #[test]
    fn global_base_child_uses_home_directory() {
        let home = Path::new(r"C:\Users\Rin");

        assert_eq!(
            global_base_child_from_home(home, DEFAULT_BASE_FOLDER_NAME),
            home.join(DEFAULT_BASE_FOLDER_NAME)
        );
    }

    #[test]
    fn legacy_global_base_folder_migrates_to_home_directory() {
        let home = Path::new(r"C:\Users\Rin");
        let app_data = Path::new(r"C:\Users\Rin\AppData\Roaming\skillsym");
        let mut state = AppState {
            global_base_folder: path_to_string(&legacy_global_base_child_from_app_data(
                app_data,
                DEFAULT_BASE_FOLDER_NAME,
            )),
            workspaces: Vec::<Workspace>::new(),
            target_base_folders: Vec::new(),
            enabled_target_ids: default_enabled_target_ids(),
            default_publish_mode: default_publish_mode(),
        };

        migrate_legacy_global_base_folder_from_paths(&mut state, home, app_data);

        assert_eq!(
            state.global_base_folder,
            path_to_string(&home.join(DEFAULT_BASE_FOLDER_NAME))
        );
    }

    #[test]
    fn legacy_state_without_publish_mode_defaults_to_symlink() {
        let state: AppState = serde_json::from_str(
            r#"{
                "globalBaseFolder":"C:\\Users\\Rin\\.claude",
                "workspaces":[],
                "targetBaseFolders":[]
            }"#,
        )
        .expect("legacy state should deserialize");

        assert_eq!(state.default_publish_mode, PublishMode::Symlink);
        assert_eq!(state.enabled_target_ids, default_enabled_target_ids());
    }

    #[test]
    fn state_round_trip_preserves_publish_mode() {
        let state = AppState {
            global_base_folder: r"C:\Users\Rin\.claude".to_string(),
            workspaces: Vec::<Workspace>::new(),
            target_base_folders: vec![r"C:\Users\Rin\.codex".to_string()],
            enabled_target_ids: default_enabled_target_ids(),
            default_publish_mode: PublishMode::Copy,
        };

        let json = serde_json::to_string(&state).expect("state should serialize");
        let restored: AppState = serde_json::from_str(&json).expect("state should deserialize");

        assert_eq!(restored.default_publish_mode, PublishMode::Copy);
    }

    #[test]
    fn custom_global_base_folder_is_not_migrated() {
        let home = Path::new(r"C:\Users\Rin");
        let app_data = Path::new(r"C:\Users\Rin\AppData\Roaming\skillsym");
        let custom = Path::new(r"D:\skills\.claude");
        let mut state = AppState {
            global_base_folder: path_to_string(custom),
            workspaces: Vec::<Workspace>::new(),
            target_base_folders: Vec::new(),
            enabled_target_ids: default_enabled_target_ids(),
            default_publish_mode: default_publish_mode(),
        };

        migrate_legacy_global_base_folder_from_paths(&mut state, home, app_data);

        assert_eq!(state.global_base_folder, path_to_string(custom));
    }
}
