use std::{
    collections::{hash_map::DefaultHasher, HashSet},
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
/// The shared Agent Skills directory used by most modern coding agents.
pub(crate) const DEFAULT_BASE_FOLDER_NAME: &str = ".agents";
pub(crate) const SKILLS_FOLDER_NAME: &str = "skills";
/// The manifest file that defines a directory skill.
pub(crate) const SKILL_MAIN_FILE: &str = "SKILL.md";

pub(crate) struct SupportedTarget {
    pub(crate) id: &'static str,
    pub(crate) name: &'static str,
    /// Directory used inside a project workspace.
    pub(crate) project_folder: &'static str,
    /// Candidate directories under the user's home directory, in preference order.
    pub(crate) global_folders: &'static [&'static str],
    /// Directories whose presence means the agent is installed on this machine.
    pub(crate) detection_folders: &'static [&'static str],
}

#[derive(Debug, Clone)]
pub(crate) struct PublishTargetDefinition {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) project_folder: String,
    pub(crate) global_folder: String,
    pub(crate) installed: bool,
}

/// Built-in agents and the directories where they look for skills.
///
/// References:
/// - Claude Code: <https://code.claude.com/docs/en/skills>
/// - Codex: <https://developers.openai.com/codex/skills>
/// - Gemini CLI: <https://geminicli.com/docs/cli/using-agent-skills/>
/// - OpenCode: <https://opencode.ai/docs/en/skills/>
/// - CodeBuddy: <https://www.codebuddy.cn/docs/cli/skills>
/// - Qoder: <https://docs.qoder.com/cli/Skills>
/// - Trae: <https://docs.trae.ai/ide/skills>
/// - Pi: <https://pi.dev/docs/latest/configuration>
/// - DeepSeek Harness: <https://github.com/deepseek-ai/deepseek-harness>
pub(crate) const SUPPORTED_TARGETS: &[SupportedTarget] = &[
    SupportedTarget {
        id: "claude",
        name: "Claude",
        project_folder: ".claude",
        global_folders: &[".claude"],
        detection_folders: &[".claude"],
    },
    SupportedTarget {
        id: "codex",
        name: "Codex",
        // Codex reads repository skills from `.agents/skills` and user skills
        // from `~/.agents/skills`.
        project_folder: ".agents",
        global_folders: &[".agents"],
        detection_folders: &[".codex"],
    },
    SupportedTarget {
        id: "gemini",
        name: "Gemini",
        project_folder: ".gemini",
        global_folders: &[".gemini"],
        detection_folders: &[".gemini"],
    },
    SupportedTarget {
        id: "opencode",
        name: "OpenCode",
        project_folder: ".opencode",
        // `~/.opencode` is an install/bin directory on some setups, so only
        // `~/.config/opencode` is used as the global skills base.
        global_folders: &[".config/opencode"],
        detection_folders: &[".config/opencode", ".opencode"],
    },
    SupportedTarget {
        id: "qoder",
        name: "Qoder",
        project_folder: ".qoder",
        global_folders: &[".qoder"],
        detection_folders: &[".qoder"],
    },
    SupportedTarget {
        id: "trae",
        name: "Trae",
        project_folder: ".trae",
        global_folders: &[".trae", ".trae-cn"],
        detection_folders: &[".trae", ".trae-cn"],
    },
    SupportedTarget {
        id: "codebuddy",
        name: "CodeBuddy",
        project_folder: ".codebuddy",
        global_folders: &[".codebuddy"],
        detection_folders: &[".codebuddy"],
    },
    SupportedTarget {
        id: "pi",
        name: "Pi",
        project_folder: ".pi",
        // Pi's user configuration always lives in the agent directory.
        global_folders: &[".pi/agent"],
        detection_folders: &[".pi"],
    },
    SupportedTarget {
        id: "deepseek",
        name: "DeepSeek",
        project_folder: ".dsh",
        global_folders: &[".dsh"],
        detection_folders: &[".dsh"],
    },
];

pub(crate) fn load_and_prepare_state(app: &AppHandle) -> CommandResult<AppState> {
    let home = app_home_dir(app)?;
    let mut state = load_state(app)?;
    migrate_legacy_global_base_folder(app, &mut state)?;
    normalize_enabled_targets(&mut state, &home);
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

/// Serde fallback for states written before the target list became dynamic.
/// Every known target stays enabled; the manager view only renders installed ones.
pub(crate) fn default_enabled_target_ids() -> Vec<String> {
    SUPPORTED_TARGETS
        .iter()
        .map(|target| target.id.to_string())
        .collect()
}

pub(crate) fn normalize_enabled_targets(state: &mut AppState, home: &Path) {
    normalize_custom_publish_targets(state);
    let supported_targets = publish_target_definitions(state, home);
    let detected = detected_target_ids(home);
    let known: HashSet<String> = state.known_detected_target_ids.iter().cloned().collect();

    // Enable an agent the first time it is detected so it appears in the
    // manager view, but never re-enable one the user has explicitly disabled.
    for id in &detected {
        if !known.contains(id) && !state.enabled_target_ids.contains(id) {
            state.enabled_target_ids.push(id.clone());
        }
    }

    state.known_detected_target_ids.extend(detected);
    state.known_detected_target_ids.sort();
    state.known_detected_target_ids.dedup();

    state
        .enabled_target_ids
        .retain(|id| supported_targets.iter().any(|target| &target.id == id));
    state.enabled_target_ids.sort();
    state.enabled_target_ids.dedup();
}

/// All built-in and custom targets, with installation resolved against `home`.
pub(crate) fn publish_target_definitions(
    state: &AppState,
    home: &Path,
) -> Vec<PublishTargetDefinition> {
    let mut targets: Vec<PublishTargetDefinition> = SUPPORTED_TARGETS
        .iter()
        .map(|target| PublishTargetDefinition {
            id: target.id.to_string(),
            name: target.name.to_string(),
            project_folder: target.project_folder.to_string(),
            global_folder: resolved_global_folder(home, target),
            installed: target_is_installed(home, target),
        })
        .collect();

    targets.extend(
        state
            .custom_publish_targets
            .iter()
            .map(|target| PublishTargetDefinition {
                id: target.id.clone(),
                name: target.name.clone(),
                project_folder: target.folder_name.clone(),
                global_folder: target.folder_name.clone(),
                installed: true,
            }),
    );
    targets
}

/// IDs of built-in agents detected under the user's home directory.
pub(crate) fn detected_target_ids(home: &Path) -> Vec<String> {
    SUPPORTED_TARGETS
        .iter()
        .filter(|target| target_is_installed(home, target))
        .map(|target| target.id.to_string())
        .collect()
}

/// Resolve the base folder for one target in the current scope.
pub(crate) fn target_base_folder(
    target: &PublishTargetDefinition,
    root: &Path,
    scope_kind: &str,
) -> PathBuf {
    let folder = if scope_kind == "global" {
        &target.global_folder
    } else {
        &target.project_folder
    };
    root.join(folder)
}

fn resolved_global_folder(home: &Path, target: &SupportedTarget) -> String {
    target
        .global_folders
        .iter()
        .find(|folder| home.join(folder).exists())
        .or_else(|| target.global_folders.first())
        .copied()
        .unwrap_or(target.project_folder)
        .to_string()
}

fn target_is_installed(home: &Path, target: &SupportedTarget) -> bool {
    let folders = if target.detection_folders.is_empty() {
        target.global_folders
    } else {
        target.detection_folders
    };

    folders.iter().any(|folder| home.join(folder).exists())
}

fn normalize_custom_publish_targets(state: &mut AppState) {
    let mut seen_ids: HashSet<String> = SUPPORTED_TARGETS
        .iter()
        .map(|target| target.id.to_lowercase())
        .collect();
    let mut seen_folder_names: HashSet<String> = SUPPORTED_TARGETS
        .iter()
        .flat_map(|target| {
            std::iter::once(target.project_folder).chain(target.global_folders.iter().copied())
        })
        .map(|folder_name| folder_name.to_lowercase())
        .collect();

    state.custom_publish_targets.retain(|target| {
        let id = target.id.trim().to_lowercase();
        let folder_name = target.folder_name.trim().to_lowercase();

        !id.is_empty()
            && !target.name.trim().is_empty()
            && !folder_name.is_empty()
            && seen_ids.insert(id)
            && seen_folder_names.insert(folder_name)
    });
    state
        .custom_publish_targets
        .sort_by_key(|target| target.name.to_lowercase());
}

pub(crate) fn app_home_dir(app: &AppHandle) -> CommandResult<PathBuf> {
    app.path()
        .home_dir()
        .map_err(|error| format!("无法定位用户主目录：{error}"))
}

pub(crate) fn global_base_child(app: &AppHandle, folder_name: &str) -> CommandResult<PathBuf> {
    Ok(global_base_child_from_home(
        &app_home_dir(app)?,
        folder_name,
    ))
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
    let home = app_home_dir(app)?;
    let global_base_folder = home.join(DEFAULT_BASE_FOLDER_NAME);
    let detected = detected_target_ids(&home);

    Ok(AppState {
        global_base_folder: path_to_string(&global_base_folder),
        workspaces: Vec::new(),
        target_base_folders: Vec::new(),
        custom_publish_targets: Vec::new(),
        enabled_target_ids: detected.clone(),
        known_detected_target_ids: detected,
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
    let home = app_home_dir(app)?;
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
    for folder_name in [DEFAULT_BASE_FOLDER_NAME, ".claude"] {
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
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        default_enabled_target_ids, default_publish_mode, detected_target_ids,
        global_base_child_from_home, legacy_global_base_child_from_app_data,
        migrate_legacy_global_base_folder_from_paths, normalize_enabled_targets,
        publish_target_definitions, target_base_folder, AppState, DEFAULT_BASE_FOLDER_NAME,
    };
    use crate::{
        models::{PublishMode, Workspace},
        paths::path_to_string,
    };

    fn temp_home(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after UNIX_EPOCH")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "skillsym-state-{name}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("test temp directory should be created");
        path
    }

    fn state_for(home: &Path) -> AppState {
        AppState {
            global_base_folder: path_to_string(&home.join(DEFAULT_BASE_FOLDER_NAME)),
            workspaces: Vec::<Workspace>::new(),
            target_base_folders: Vec::new(),
            custom_publish_targets: Vec::new(),
            enabled_target_ids: default_enabled_target_ids(),
            known_detected_target_ids: Vec::new(),
            default_publish_mode: default_publish_mode(),
        }
    }

    #[test]
    fn default_base_folder_is_the_shared_agents_directory() {
        assert_eq!(DEFAULT_BASE_FOLDER_NAME, ".agents");

        let home = Path::new(r"C:\Users\Rin");

        assert_eq!(
            global_base_child_from_home(home, DEFAULT_BASE_FOLDER_NAME),
            home.join(".agents")
        );
    }

    #[test]
    fn legacy_global_base_folder_migrates_to_home_directory() {
        let home = Path::new(r"C:\Users\Rin");
        let app_data = Path::new(r"C:\Users\Rin\AppData\Roaming\skillsym");
        let mut state = AppState {
            global_base_folder: path_to_string(&legacy_global_base_child_from_app_data(
                app_data, ".claude",
            )),
            workspaces: Vec::<Workspace>::new(),
            target_base_folders: Vec::new(),
            custom_publish_targets: Vec::new(),
            enabled_target_ids: default_enabled_target_ids(),
            known_detected_target_ids: Vec::new(),
            default_publish_mode: default_publish_mode(),
        };

        migrate_legacy_global_base_folder_from_paths(&mut state, home, app_data);

        assert_eq!(
            state.global_base_folder,
            path_to_string(&home.join(".claude"))
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
            global_base_folder: r"C:\Users\Rin\.agents".to_string(),
            workspaces: Vec::<Workspace>::new(),
            target_base_folders: vec![r"C:\Users\Rin\.codex".to_string()],
            custom_publish_targets: Vec::new(),
            enabled_target_ids: default_enabled_target_ids(),
            known_detected_target_ids: Vec::new(),
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
            custom_publish_targets: Vec::new(),
            enabled_target_ids: default_enabled_target_ids(),
            known_detected_target_ids: Vec::new(),
            default_publish_mode: default_publish_mode(),
        };

        migrate_legacy_global_base_folder_from_paths(&mut state, home, app_data);

        assert_eq!(state.global_base_folder, path_to_string(custom));
    }

    #[test]
    fn detected_agents_follow_home_config_folders() {
        let home = temp_home("detect");
        for folder in [".claude", ".codex", ".config/opencode", ".dsh", ".trae-cn"] {
            fs::create_dir_all(home.join(folder)).expect("agent config dir should be created");
        }

        let state = state_for(&home);
        let targets = publish_target_definitions(&state, &home);

        let target = |id: &str| {
            targets
                .iter()
                .find(|target| target.id == id)
                .unwrap_or_else(|| panic!("missing target {id}"))
        };

        assert!(target("claude").installed);
        assert_eq!(target("claude").global_folder, ".claude");

        assert!(target("codex").installed);
        assert_eq!(
            target("codex").global_folder,
            ".agents",
            "Codex should write to the shared .agents directory"
        );
        assert_eq!(target("codex").project_folder, ".agents");

        assert!(target("opencode").installed);
        assert_eq!(target("opencode").global_folder, ".config/opencode");
        assert_eq!(target("opencode").project_folder, ".opencode");

        assert!(target("trae").installed);
        assert_eq!(
            target("trae").global_folder,
            ".trae-cn",
            "Trae CN should fall back to its own home directory"
        );

        assert!(target("deepseek").installed);
        assert_eq!(target("deepseek").global_folder, ".dsh");

        assert!(!target("gemini").installed);
        assert!(!target("qoder").installed);
        assert!(!target("codebuddy").installed);
        assert!(!target("pi").installed);

        assert_eq!(
            detected_target_ids(&home),
            vec![
                "claude".to_string(),
                "codex".to_string(),
                "opencode".to_string(),
                "trae".to_string(),
                "deepseek".to_string()
            ]
        );

        fs::remove_dir_all(home).expect("test temp directory should be removed");
    }

    #[test]
    fn target_base_folder_differs_between_global_and_workspace_scope() {
        let home = temp_home("target-base");
        fs::create_dir_all(home.join(".config/opencode"))
            .expect("opencode config dir should be created");
        let workspace = home.join("project");

        let state = state_for(&home);
        let target = publish_target_definitions(&state, &home)
            .into_iter()
            .find(|target| target.id == "opencode")
            .expect("opencode target should exist");

        assert_eq!(
            target_base_folder(&target, &home, "global"),
            home.join(".config/opencode")
        );
        assert_eq!(
            target_base_folder(&target, &workspace, "workspace"),
            workspace.join(".opencode")
        );

        fs::remove_dir_all(home).expect("test temp directory should be removed");
    }

    #[test]
    fn detection_only_folders_do_not_change_the_global_skills_base() {
        let home = temp_home("detection-only");
        fs::create_dir_all(home.join(".opencode")).expect("opencode bin dir should be created");
        fs::create_dir_all(home.join(".pi")).expect("pi home dir should be created");

        let state = state_for(&home);
        let targets = publish_target_definitions(&state, &home);
        let target = |id: &str| {
            targets
                .iter()
                .find(|target| target.id == id)
                .unwrap_or_else(|| panic!("missing target {id}"))
        };

        assert!(target("opencode").installed);
        assert_eq!(
            target("opencode").global_folder,
            ".config/opencode",
            "the bin directory must not become the global skills base"
        );

        assert!(target("pi").installed);
        assert_eq!(
            target("pi").global_folder,
            ".pi/agent",
            "Pi always reads user skills from its agent directory"
        );

        fs::remove_dir_all(home).expect("test temp directory should be removed");
    }

    #[test]
    fn newly_detected_agent_is_enabled_once_and_respects_a_later_disable() {
        let home = temp_home("auto-enable");
        fs::create_dir_all(home.join(".claude")).expect("claude config dir should be created");
        let mut state = state_for(&home);
        state.enabled_target_ids = vec!["claude".to_string()];
        state.known_detected_target_ids = vec!["claude".to_string()];

        fs::create_dir_all(home.join(".codex")).expect("codex config dir should be created");
        normalize_enabled_targets(&mut state, &home);
        assert!(
            state.enabled_target_ids.contains(&"codex".to_string()),
            "a newly installed agent should appear without a manual enable"
        );
        assert!(state
            .known_detected_target_ids
            .contains(&"codex".to_string()));

        state.enabled_target_ids.retain(|id| id != "codex");
        normalize_enabled_targets(&mut state, &home);
        assert!(
            !state.enabled_target_ids.contains(&"codex".to_string()),
            "an agent the user disabled must stay disabled"
        );

        fs::remove_dir_all(home).expect("test temp directory should be removed");
    }
}
