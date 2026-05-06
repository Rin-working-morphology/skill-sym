use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use tauri::AppHandle;

use crate::{
    filesystem::{
        delete_scope_skill, publish_scope_folder, publish_scope_skill,
        remove_published_scope_folder, remove_published_scope_skill, scan_scope_publish_targets,
        scan_scope_skills,
    },
    models::{
        AppState, CommandResult, CustomPublishTarget, DeleteSkillRequest, OperationResult,
        PublishFolderRequest, PublishMode, PublishSkillRequest, RemoveFolderRequest,
        RemoveSkillRequest, ScanResult, ScopeSelection, TargetScanResult, Workspace,
        WorkspaceGitDetail,
    },
    paths::{absolute_path, ensure_existing_dir, path_to_string, validate_child_name},
    state::{
        configured_publish_targets, global_base_child, load_and_prepare_state, load_state,
        normalize_enabled_targets, save_state, workspace_id, DEFAULT_BASE_FOLDER_NAME,
    },
};

#[tauri::command]
pub(crate) fn load_manager_state(app: AppHandle) -> CommandResult<AppState> {
    load_and_prepare_state(&app)
}

#[tauri::command]
pub(crate) fn scan_workspace_git_details(
    paths: Vec<String>,
) -> CommandResult<Vec<WorkspaceGitDetail>> {
    Ok(paths
        .into_iter()
        .filter_map(|path| {
            workspace_git_detail(&path).map(|detail| WorkspaceGitDetail { path, detail })
        })
        .collect())
}

#[tauri::command]
pub(crate) fn add_workspace(app: AppHandle, path: String) -> CommandResult<AppState> {
    let workspace_path = ensure_existing_dir(&path, "工作区")?;
    let mut state = load_state(&app)?;
    let normalized_path = path_to_string(&workspace_path);

    if state
        .workspaces
        .iter()
        .any(|item| item.path == normalized_path)
    {
        return Ok(state);
    }

    let name = workspace_path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("工作区")
        .to_string();
    let base_folder = workspace_path
        .join(DEFAULT_BASE_FOLDER_NAME)
        .to_string_lossy()
        .to_string();

    state.workspaces.push(Workspace {
        id: workspace_id(&normalized_path),
        name,
        path: normalized_path,
        base_folder,
    });
    save_state(&app, &state)?;
    Ok(state)
}

#[tauri::command]
pub(crate) fn remove_workspace(app: AppHandle, id: String) -> CommandResult<AppState> {
    let mut state = load_state(&app)?;
    state.workspaces.retain(|workspace| workspace.id != id);
    save_state(&app, &state)?;
    Ok(state)
}

#[tauri::command]
pub(crate) fn open_managed_folder(app: AppHandle, path: String) -> CommandResult<()> {
    let folder = ensure_existing_dir(&path, "打开")?;
    let state = load_state(&app)?;

    if !is_managed_folder(&folder, &state) {
        return Err(format!("不允许打开未登记的路径：{}", folder.display()));
    }

    tauri_plugin_opener::open_path(&folder, None::<&str>)
        .map_err(|error| format!("无法打开文件夹：{error}"))
}

#[tauri::command]
pub(crate) fn set_scope_base_folder(
    app: AppHandle,
    scope: ScopeSelection,
    base_folder: String,
) -> CommandResult<AppState> {
    let base_path = ensure_existing_dir(&base_folder, "基础")?;
    let mut state = load_state(&app)?;
    match scope.kind.as_str() {
        "global" => state.global_base_folder = path_to_string(&base_path),
        "workspace" => {
            let workspace_id = scope
                .workspace_id
                .ok_or_else(|| "工作区范围需要工作区 ID。".to_string())?;
            let workspace = state
                .workspaces
                .iter_mut()
                .find(|item| item.id == workspace_id)
                .ok_or_else(|| "未找到工作区。".to_string())?;
            workspace.base_folder = path_to_string(&base_path);
        }
        _ => return Err("未知的范围类型。".to_string()),
    }
    save_state(&app, &state)?;
    Ok(state)
}

#[tauri::command]
pub(crate) fn set_scope_base_child(
    app: AppHandle,
    scope: ScopeSelection,
    folder_name: String,
) -> CommandResult<AppState> {
    validate_child_name(&folder_name)?;
    let mut state = load_state(&app)?;
    match scope.kind.as_str() {
        "global" => {
            state.global_base_folder = path_to_string(&global_base_child(&app, &folder_name)?);
        }
        "workspace" => {
            let workspace_id = scope
                .workspace_id
                .ok_or_else(|| "工作区范围需要工作区 ID。".to_string())?;
            let workspace = state
                .workspaces
                .iter_mut()
                .find(|item| item.id == workspace_id)
                .ok_or_else(|| "未找到工作区。".to_string())?;
            workspace.base_folder =
                path_to_string(&std::path::Path::new(&workspace.path).join(folder_name));
        }
        _ => return Err("未知的范围类型。".to_string()),
    }
    save_state(&app, &state)?;
    Ok(state)
}

#[tauri::command]
pub(crate) fn add_target_base_folder(app: AppHandle, path: String) -> CommandResult<AppState> {
    let target_path = ensure_existing_dir(&path, "目标基础")?;
    let normalized_path = path_to_string(&target_path);
    let mut state = load_state(&app)?;

    if !state
        .target_base_folders
        .iter()
        .any(|item| item == &normalized_path)
    {
        state.target_base_folders.push(normalized_path);
        state.target_base_folders.sort();
        save_state(&app, &state)?;
    }

    Ok(state)
}

#[tauri::command]
pub(crate) fn remove_target_base_folder(app: AppHandle, path: String) -> CommandResult<AppState> {
    let mut state = load_state(&app)?;
    state.target_base_folders.retain(|item| item != &path);
    save_state(&app, &state)?;
    Ok(state)
}

#[tauri::command]
pub(crate) fn add_custom_publish_target(
    app: AppHandle,
    name: String,
    folder_name: String,
) -> CommandResult<AppState> {
    let target_name = name.trim();
    let target_folder_name = folder_name.trim();

    if target_name.is_empty() {
        return Err("目标名称不能为空。".to_string());
    }
    validate_child_name(target_folder_name)
        .map_err(|_| "存放位置必须是单个文件夹名称。".to_string())?;

    let mut state = load_state(&app)?;
    let existing_targets = configured_publish_targets(&state);
    if existing_targets
        .iter()
        .any(|target| target.name.eq_ignore_ascii_case(target_name))
    {
        return Err("已存在同名发布目标。".to_string());
    }
    if existing_targets
        .iter()
        .any(|target| target.folder_name.eq_ignore_ascii_case(target_folder_name))
    {
        return Err("已存在相同存放位置的发布目标。".to_string());
    }

    let id = custom_publish_target_id(target_name, target_folder_name);
    state.custom_publish_targets.push(CustomPublishTarget {
        id: id.clone(),
        name: target_name.to_string(),
        folder_name: target_folder_name.to_string(),
    });
    state.enabled_target_ids.push(id);
    normalize_enabled_targets(&mut state);
    save_state(&app, &state)?;
    Ok(state)
}

#[tauri::command]
pub(crate) fn set_default_publish_mode(
    app: AppHandle,
    mode: PublishMode,
) -> CommandResult<AppState> {
    let mut state = load_state(&app)?;
    state.default_publish_mode = mode;
    save_state(&app, &state)?;
    Ok(state)
}

#[tauri::command]
pub(crate) fn set_enabled_publish_targets(
    app: AppHandle,
    target_ids: Vec<String>,
) -> CommandResult<AppState> {
    let mut state = load_state(&app)?;
    let supported_targets = configured_publish_targets(&state);
    if target_ids
        .iter()
        .any(|id| !supported_targets.iter().any(|target| &target.id == id))
    {
        return Err("包含不支持的发布目标。".to_string());
    }

    state.enabled_target_ids = target_ids;
    normalize_enabled_targets(&mut state);
    save_state(&app, &state)?;
    Ok(state)
}

#[tauri::command]
pub(crate) fn scan_skills(app: AppHandle, scope: ScopeSelection) -> CommandResult<ScanResult> {
    let state = load_state(&app)?;
    scan_scope_skills(&state, &scope)
}

#[tauri::command]
pub(crate) fn scan_publish_targets(
    app: AppHandle,
    scope: ScopeSelection,
) -> CommandResult<TargetScanResult> {
    let state = load_state(&app)?;
    scan_scope_publish_targets(&app, &state, &scope)
}

#[tauri::command]
pub(crate) fn publish_skills_folder(
    app: AppHandle,
    request: PublishFolderRequest,
) -> CommandResult<OperationResult> {
    let state = load_state(&app)?;
    publish_scope_folder(&state, &request)
}

#[tauri::command]
pub(crate) fn publish_skill(
    app: AppHandle,
    request: PublishSkillRequest,
) -> CommandResult<OperationResult> {
    let state = load_state(&app)?;
    publish_scope_skill(&state, &request)
}

#[tauri::command]
pub(crate) fn remove_published_skills_folder(
    app: AppHandle,
    request: RemoveFolderRequest,
) -> CommandResult<OperationResult> {
    let state = load_state(&app)?;
    remove_published_scope_folder(&state, &request)
}

#[tauri::command]
pub(crate) fn remove_published_skill(
    app: AppHandle,
    request: RemoveSkillRequest,
) -> CommandResult<OperationResult> {
    let state = load_state(&app)?;
    remove_published_scope_skill(&state, &request)
}

#[tauri::command]
pub(crate) fn delete_skill(
    app: AppHandle,
    request: DeleteSkillRequest,
) -> CommandResult<OperationResult> {
    let state = load_state(&app)?;
    delete_scope_skill(&app, &state, &request)
}

fn is_managed_folder(path: &Path, state: &AppState) -> bool {
    managed_path_matches(path, &state.global_base_folder)
        || state.workspaces.iter().any(|workspace| {
            managed_path_matches(path, &workspace.path)
                || managed_path_matches(path, &workspace.base_folder)
        })
}

fn workspace_git_detail(workspace_path: &str) -> Option<String> {
    let workspace_path = absolute_path(Path::new(workspace_path)).ok()?;
    if has_git_marker(&workspace_path) {
        return None;
    }

    let mut ancestor = workspace_path.parent();
    while let Some(path) = ancestor {
        if has_git_marker(path) {
            return path
                .file_name()
                .map(|name| name.to_string_lossy().to_string());
        }
        ancestor = path.parent();
    }

    None
}

fn has_git_marker(path: &Path) -> bool {
    path.join(".git").exists()
}

fn managed_path_matches(path: &Path, allowed_path: &str) -> bool {
    match (
        existing_canonical_path(path),
        existing_canonical_path(Path::new(allowed_path)),
    ) {
        (Some(path), Some(allowed_path)) => path == allowed_path,
        _ => absolute_path_or_self(path) == absolute_path_or_self(Path::new(allowed_path)),
    }
}

fn custom_publish_target_id(name: &str, folder_name: &str) -> String {
    let slug: String = name
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .take(24)
        .collect();
    let slug = if slug.is_empty() {
        "target".to_string()
    } else {
        slug
    };
    let mut hasher = DefaultHasher::new();
    name.to_lowercase().hash(&mut hasher);
    folder_name.to_lowercase().hash(&mut hasher);
    format!("custom-{slug}-{:x}", hasher.finish())
}

fn existing_canonical_path(path: &Path) -> Option<PathBuf> {
    fs::canonicalize(path).ok()
}

fn absolute_path_or_self(path: &Path) -> PathBuf {
    crate::paths::absolute_path(path).unwrap_or_else(|_| path.to_path_buf())
}
