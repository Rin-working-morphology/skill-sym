use std::{
    fs,
    path::{Path, PathBuf},
};

use tauri::AppHandle;

use crate::{
    models::{
        AppState, CommandResult, DeleteSkillRequest, OperationResult, PublishFolderRequest,
        PublishMode, PublishSkillRequest, PublishTargetStatus, RemoveFolderRequest,
        RemoveSkillRequest, ScanResult, ScopeSelection, SkillEntry, TargetScanResult,
    },
    paths::{absolute_path, path_to_string, readable_io_error, validate_child_name},
    state::{scope_skills_path, target_root_path, SKILLS_FOLDER_NAME, SUPPORTED_TARGETS},
};

pub(crate) fn scan_scope_skills(
    state: &AppState,
    scope: &ScopeSelection,
) -> CommandResult<ScanResult> {
    let skills_path = scope_skills_path(state, scope)?;

    if !skills_path.exists() {
        return Ok(ScanResult {
            skills_path: path_to_string(&skills_path),
            exists: false,
            skills: Vec::new(),
        });
    }

    if !skills_path.is_dir() {
        return Err(format!(
            "技能路径已存在，但不是文件夹：{}",
            skills_path.display()
        ));
    }

    let mut skills = Vec::new();
    for entry in fs::read_dir(&skills_path).map_err(readable_io_error)? {
        let entry = entry.map_err(readable_io_error)?;
        let entry_path = entry.path();
        let metadata = fs::symlink_metadata(&entry_path).map_err(readable_io_error)?;
        let kind = if metadata.file_type().is_symlink() {
            "symlink"
        } else if metadata.is_dir() {
            "directory"
        } else if metadata.is_file() {
            "file"
        } else {
            "other"
        };
        skills.push(SkillEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: path_to_string(&entry_path),
            kind: kind.to_string(),
        });
    }
    skills.sort_by_key(|skill| skill.name.to_lowercase());

    Ok(ScanResult {
        skills_path: path_to_string(&skills_path),
        exists: true,
        skills,
    })
}

pub(crate) fn scan_scope_publish_targets(
    app: &AppHandle,
    state: &AppState,
    scope: &ScopeSelection,
) -> CommandResult<TargetScanResult> {
    let root = target_root_path(app, state, scope)?;
    let source_skills = absolute_path(&scope_skills_path(state, scope)?)?;
    let mut targets = Vec::new();

    for supported in SUPPORTED_TARGETS {
        let base_folder = root.join(supported.folder_name);
        let skills_path = base_folder.join(SKILLS_FOLDER_NAME);
        let installed_skill_names = installed_skill_names(&skills_path)?;
        let has_skills_folder = fs::symlink_metadata(&skills_path).is_ok();
        let skills_path_abs = absolute_path(&skills_path)?;
        let is_source =
            paths_match_without_following_final_symlink(&source_skills, &skills_path_abs);
        let protects_source_children = paths_equal(&source_skills, &skills_path_abs);

        targets.push(PublishTargetStatus {
            id: supported.id.to_string(),
            name: supported.name.to_string(),
            folder_name: supported.folder_name.to_string(),
            base_folder: path_to_string(&base_folder),
            skills_path: path_to_string(&skills_path),
            enabled: state.enabled_target_ids.iter().any(|id| id == supported.id),
            is_source,
            protects_source_children,
            has_skills_folder,
            installed_skill_names,
        });
    }

    Ok(TargetScanResult { targets })
}

pub(crate) fn publish_scope_folder(
    state: &AppState,
    request: &PublishFolderRequest,
) -> CommandResult<OperationResult> {
    let source = scope_skills_path(state, &request.scope)?;
    let target = absolute_path(Path::new(&request.target_base_folder))?.join(SKILLS_FOLDER_NAME);

    publish_path(&source, &target, request.mode, true).map(|message| OperationResult {
        message,
        source: path_to_string(&source),
        target: path_to_string(&target),
    })
}

pub(crate) fn publish_scope_skill(
    state: &AppState,
    request: &PublishSkillRequest,
) -> CommandResult<OperationResult> {
    validate_child_name(&request.skill_name)?;
    let source = scope_skills_path(state, &request.scope)?.join(&request.skill_name);
    let target = absolute_path(Path::new(&request.target_base_folder))?
        .join(SKILLS_FOLDER_NAME)
        .join(&request.skill_name);

    publish_path(&source, &target, request.mode, false).map(|message| OperationResult {
        message,
        source: path_to_string(&source),
        target: path_to_string(&target),
    })
}

pub(crate) fn remove_published_scope_folder(
    state: &AppState,
    request: &RemoveFolderRequest,
) -> CommandResult<OperationResult> {
    let source = scope_skills_path(state, &request.scope)?;
    let target = absolute_path(Path::new(&request.target_base_folder))?.join(SKILLS_FOLDER_NAME);

    remove_published_path(&source, &target).map(|message| OperationResult {
        message,
        source: path_to_string(&source),
        target: path_to_string(&target),
    })
}

pub(crate) fn remove_published_scope_skill(
    state: &AppState,
    request: &RemoveSkillRequest,
) -> CommandResult<OperationResult> {
    validate_child_name(&request.skill_name)?;
    let source = scope_skills_path(state, &request.scope)?.join(&request.skill_name);
    let target = absolute_path(Path::new(&request.target_base_folder))?
        .join(SKILLS_FOLDER_NAME)
        .join(&request.skill_name);

    remove_published_path(&source, &target).map(|message| OperationResult {
        message,
        source: path_to_string(&source),
        target: path_to_string(&target),
    })
}

pub(crate) fn delete_scope_skill(
    app: &AppHandle,
    state: &AppState,
    request: &DeleteSkillRequest,
) -> CommandResult<OperationResult> {
    validate_child_name(&request.skill_name)?;
    let source = scope_skills_path(state, &request.scope)?.join(&request.skill_name);
    let source_abs = absolute_path(&source)?;
    let removed_targets = delete_skill_from_target_bases(
        &source_abs,
        &publish_target_base_paths(app, state, &request.scope)?,
        &request.skill_name,
    )?;

    Ok(OperationResult {
        message: format!("已删除技能，并清理 {removed_targets} 个发布目标。"),
        source: path_to_string(&source_abs),
        target: path_to_string(&source_abs),
    })
}

fn delete_skill_from_target_bases(
    source_abs: &Path,
    target_bases: &[PathBuf],
    skill_name: &str,
) -> CommandResult<usize> {
    let mut removed_targets = 0usize;

    for target_base in target_bases {
        let target = absolute_path(target_base)?
            .join(SKILLS_FOLDER_NAME)
            .join(skill_name);
        if paths_match_without_following_final_symlink(source_abs, &target) {
            continue;
        }

        if path_exists_or_is_symlink(&target)? {
            remove_existing(&target)?;
            removed_targets += 1;
        }
    }

    if path_exists_or_is_symlink(source_abs)? {
        remove_existing(source_abs)?;
    }

    Ok(removed_targets)
}

fn publish_path(
    source: &Path,
    target: &Path,
    mode: PublishMode,
    replace_any_existing_target: bool,
) -> CommandResult<String> {
    let source_abs = ensure_source_exists(source)?;
    let target_abs = absolute_path(target)?;

    if paths_equal(&source_abs, &target_abs) {
        return Err("源路径和目标路径相同。".to_string());
    }
    if target_is_inside_source(&source_abs, &target_abs) {
        return Err("目标路径位于源路径内。".to_string());
    }

    let source_meta = fs::metadata(&source_abs).map_err(readable_io_error)?;
    let source_is_dir = source_meta.is_dir();
    ensure_target_parent_is_directory(&target_abs)?;

    match mode {
        PublishMode::Copy => {
            delete_existing_target(&target_abs, source_is_dir, replace_any_existing_target)?;
            copy_item(&source_abs, &target_abs)?;
            Ok("已将技能文件复制到目标。".to_string())
        }
        PublishMode::Symlink => {
            delete_existing_target(&target_abs, source_is_dir, replace_any_existing_target)?;
            create_relative_symlink(&source_abs, &target_abs, source_is_dir)?;
            Ok("已在目标创建相对符号链接。".to_string())
        }
    }
}

fn remove_published_path(source: &Path, target: &Path) -> CommandResult<String> {
    let source_abs = absolute_path(source)?;
    let target_abs = absolute_path(target)?;

    if paths_match_without_following_final_symlink(&source_abs, &target_abs) {
        return Err("不能从发布目标移除当前源路径。".to_string());
    }

    if !path_exists_or_is_symlink(&target_abs)? {
        return Ok("目标中未找到已发布项。".to_string());
    }

    remove_existing(&target_abs)?;
    Ok("已从目标移除发布项。".to_string())
}

fn ensure_source_exists(path: &Path) -> CommandResult<PathBuf> {
    if !path.exists() {
        return Err(format!("源不存在：{}", path.display()));
    }
    fs::canonicalize(path).map_err(readable_io_error)
}

fn ensure_target_parent_is_directory(target: &Path) -> CommandResult<()> {
    let parent = target
        .parent()
        .ok_or_else(|| "目标路径没有父文件夹。".to_string())?;
    if parent.exists() && !parent.is_dir() {
        return Err(format!("目标父路径不是文件夹：{}", parent.display()));
    }
    Ok(())
}

fn target_is_inside_source(source: &Path, target: &Path) -> bool {
    if target.starts_with(source) {
        return true;
    }

    let mut ancestor = target.parent();
    while let Some(path) = ancestor {
        if path.exists() {
            return fs::canonicalize(path)
                .map(|canonical| canonical == source || canonical.starts_with(source))
                .unwrap_or(false);
        }
        ancestor = path.parent();
    }

    false
}

fn delete_existing_target(
    target: &Path,
    source_is_dir: bool,
    replace_any_existing_target: bool,
) -> CommandResult<()> {
    let metadata = match fs::symlink_metadata(target) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(readable_io_error(error)),
    };
    let file_type = metadata.file_type();
    let target_is_dir = if file_type.is_symlink() {
        fs::metadata(target)
            .map(|metadata| metadata.is_dir())
            .unwrap_or(source_is_dir)
    } else {
        metadata.is_dir()
    };

    if !replace_any_existing_target && source_is_dir != target_is_dir {
        return Err(format!("目标已存在且类型不兼容：{}", target.display()));
    }

    remove_existing_with_metadata(target, &metadata)
}

fn remove_existing(target: &Path) -> CommandResult<()> {
    let metadata = fs::symlink_metadata(target).map_err(readable_io_error)?;
    remove_existing_with_metadata(target, &metadata)
}

fn path_exists_or_is_symlink(target: &Path) -> CommandResult<bool> {
    match fs::symlink_metadata(target) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(readable_io_error(error)),
    }
}

fn remove_existing_with_metadata(target: &Path, metadata: &fs::Metadata) -> CommandResult<()> {
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        remove_symlink(target)?;
    } else if metadata.is_dir() {
        fs::remove_dir_all(target).map_err(readable_io_error)?;
    } else {
        fs::remove_file(target).map_err(readable_io_error)?;
    }
    Ok(())
}

fn installed_skill_names(skills_path: &Path) -> CommandResult<Vec<String>> {
    if !skills_path.exists() || !skills_path.is_dir() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(skills_path).map_err(readable_io_error)? {
        let entry = entry.map_err(readable_io_error)?;
        names.push(entry.file_name().to_string_lossy().to_string());
    }
    names.sort_by_key(|name| name.to_lowercase());
    Ok(names)
}

fn publish_target_base_paths(
    app: &AppHandle,
    state: &AppState,
    scope: &ScopeSelection,
) -> CommandResult<Vec<PathBuf>> {
    let root = target_root_path(app, state, scope)?;
    let mut paths: Vec<PathBuf> = SUPPORTED_TARGETS
        .iter()
        .map(|target| root.join(target.folder_name))
        .chain(state.target_base_folders.iter().map(PathBuf::from))
        .collect();
    paths.sort_by_key(|path| path_to_string(path).to_lowercase());
    paths.dedup_by(|left, right| paths_match_without_following_final_symlink(left, right));
    Ok(paths)
}

fn remove_symlink(path: &Path) -> CommandResult<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(file_error) => match fs::remove_dir(path) {
            Ok(()) => Ok(()),
            Err(dir_error) => Err(format!(
                "无法移除现有符号链接：{}；{}",
                file_error, dir_error
            )),
        },
    }
}

fn copy_item(source: &Path, target: &Path) -> CommandResult<()> {
    let metadata = fs::metadata(source).map_err(readable_io_error)?;
    if metadata.is_dir() {
        fs::create_dir_all(target).map_err(readable_io_error)?;
        for entry in fs::read_dir(source).map_err(readable_io_error)? {
            let entry = entry.map_err(readable_io_error)?;
            copy_item(&entry.path(), &target.join(entry.file_name()))?;
        }
        Ok(())
    } else if metadata.is_file() {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(readable_io_error)?;
        }
        fs::copy(source, target)
            .map(|_| ())
            .map_err(readable_io_error)
    } else {
        Err(format!("源类型不支持复制：{}", source.display()))
    }
}

fn create_relative_symlink(source: &Path, target: &Path, source_is_dir: bool) -> CommandResult<()> {
    let parent = target
        .parent()
        .ok_or_else(|| "目标路径没有父文件夹。".to_string())?;
    fs::create_dir_all(parent).map_err(readable_io_error)?;

    let parent_abs = fs::canonicalize(parent).map_err(readable_io_error)?;
    let relative_source = pathdiff::diff_paths(source, &parent_abs)
        .ok_or_else(|| "无法计算相对符号链接目标。".to_string())?;

    platform_symlink(&relative_source, target, source_is_dir).map_err(symlink_error)
}

#[cfg(unix)]
fn platform_symlink(source: &Path, target: &Path, _source_is_dir: bool) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, target)
}

#[cfg(windows)]
fn platform_symlink(source: &Path, target: &Path, source_is_dir: bool) -> std::io::Result<()> {
    if source_is_dir {
        std::os::windows::fs::symlink_dir(source, target)
    } else {
        std::os::windows::fs::symlink_file(source, target)
    }
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }

    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left_canonical), Ok(right_canonical)) => left_canonical == right_canonical,
        (Ok(left_canonical), Err(_)) => left_canonical == right,
        (Err(_), Ok(right_canonical)) => left == right_canonical,
        (Err(_), Err(_)) => false,
    }
}

fn paths_match_without_following_final_symlink(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }

    let Some(left_name) = left.file_name() else {
        return false;
    };
    let Some(right_name) = right.file_name() else {
        return false;
    };
    if left_name != right_name {
        return false;
    }

    match (left.parent(), right.parent()) {
        (Some(left_parent), Some(right_parent)) => {
            match (
                fs::canonicalize(left_parent),
                fs::canonicalize(right_parent),
            ) {
                (Ok(left_parent), Ok(right_parent)) => left_parent == right_parent,
                _ => false,
            }
        }
        _ => false,
    }
}

fn symlink_error(error: std::io::Error) -> String {
    let base = format!("符号链接创建失败：{error}");
    #[cfg(windows)]
    {
        format!("{base}。在 Windows 上，请启用开发者模式或以提升的权限运行应用。")
    }
    #[cfg(not(windows))]
    {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::{
        delete_skill_from_target_bases, paths_match_without_following_final_symlink, publish_path,
        scan_scope_skills,
    };
    use crate::{
        filesystem::{
            publish_scope_folder, remove_published_scope_folder, remove_published_scope_skill,
        },
        models::{
            AppState, PublishFolderRequest, PublishMode, RemoveFolderRequest, RemoveSkillRequest,
            ScopeSelection, Workspace,
        },
        paths::path_to_string,
        state::{default_enabled_target_ids, default_publish_mode},
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after UNIX_EPOCH")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "skillsym-{name}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("test temp directory should be created");
        path
    }

    fn global_scope() -> ScopeSelection {
        ScopeSelection {
            kind: "global".to_string(),
            workspace_id: None,
        }
    }

    fn state_for_global(base_folder: &Path) -> AppState {
        AppState {
            global_base_folder: path_to_string(base_folder),
            workspaces: Vec::<Workspace>::new(),
            target_base_folders: Vec::new(),
            enabled_target_ids: default_enabled_target_ids(),
            default_publish_mode: default_publish_mode(),
        }
    }

    #[cfg(unix)]
    fn create_test_file_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(source, target)
    }

    #[cfg(windows)]
    fn create_test_file_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
        std::os::windows::fs::symlink_file(source, target)
    }

    #[cfg(unix)]
    fn create_test_dir_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(source, target)
    }

    #[cfg(windows)]
    fn create_test_dir_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
        std::os::windows::fs::symlink_dir(source, target)
    }

    #[test]
    fn scan_missing_skills_folder_returns_empty_result() {
        let root = temp_root("scan-missing");
        let base = root.join(".claude");
        let state = state_for_global(&base);

        let result = scan_scope_skills(&state, &global_scope())
            .expect("scan should succeed for missing dir");

        assert_eq!(result.skills_path, path_to_string(&base.join("skills")));
        assert!(
            !result.exists,
            "missing skills dir should be reported as absent"
        );
        assert!(
            result.skills.is_empty(),
            "missing skills dir should return no entries"
        );

        fs::remove_dir_all(root).expect("test temp directory should be removed");
    }

    #[test]
    fn scan_populated_skills_folder_returns_sorted_entries() {
        let root = temp_root("scan-populated");
        let skills = root.join(".claude").join("skills");
        fs::create_dir_all(skills.join("bravo-skill")).expect("directory skill should be created");
        fs::write(skills.join("alpha-skill.md"), "alpha").expect("file skill should be written");
        let state = state_for_global(&root.join(".claude"));

        let result = scan_scope_skills(&state, &global_scope())
            .expect("scan should succeed for populated dir");

        assert!(
            result.exists,
            "existing skills dir should be reported as present"
        );
        assert_eq!(
            result.skills.len(),
            2,
            "both skill entries should be returned"
        );
        assert_eq!(result.skills[0].name, "alpha-skill.md");
        assert_eq!(result.skills[0].kind, "file");
        assert_eq!(result.skills[1].name, "bravo-skill");
        assert_eq!(result.skills[1].kind, "directory");

        fs::remove_dir_all(root).expect("test temp directory should be removed");
    }

    #[test]
    fn publish_copy_rejects_target_inside_source() {
        let root = temp_root("nested-copy");
        let source = root.join("skills");
        fs::create_dir_all(&source).expect("source directory should be created");
        fs::write(source.join("demo.txt"), "demo").expect("source file should be written");

        let target = source.join("published");
        let result = publish_path(&source, &target, PublishMode::Copy, false);

        assert_eq!(
            result.expect_err("nested target should be rejected"),
            "目标路径位于源路径内。"
        );
        assert!(
            !target.exists(),
            "target directory should not be created before validation fails"
        );

        fs::remove_dir_all(root).expect("test temp directory should be removed");
    }

    #[test]
    fn publish_copy_replaces_existing_directory_contents() {
        let root = temp_root("copy-replace");
        let source = root.join("source").join("skills");
        fs::create_dir_all(&source).expect("source directory should be created");
        fs::write(source.join("fresh.txt"), "fresh").expect("source file should be written");

        let target = root.join("target").join("skills");
        fs::create_dir_all(&target).expect("target directory should be created");
        fs::write(target.join("stale.txt"), "stale").expect("stale target file should be written");

        publish_path(&source, &target, PublishMode::Copy, false)
            .expect("copy publish should replace existing directory");

        assert!(
            !target.join("stale.txt").exists(),
            "stale target contents should be removed before copy"
        );
        assert_eq!(
            fs::read_to_string(target.join("fresh.txt")).expect("fresh target file should exist"),
            "fresh"
        );

        fs::remove_dir_all(root).expect("test temp directory should be removed");
    }

    #[test]
    fn publish_folder_copy_replaces_existing_skills_file() {
        let root = temp_root("folder-replace-file");
        let source_base = root.join("source").join(".claude");
        let source_skills = source_base.join("skills");
        fs::create_dir_all(&source_skills).expect("source skills directory should be created");
        fs::write(source_skills.join("fresh.txt"), "fresh").expect("source file should be written");

        let target_base = root.join("target").join(".codex");
        fs::create_dir_all(&target_base).expect("target base should be created");
        fs::write(target_base.join("skills"), "stale").expect("target skills file should exist");

        let state = state_for_global(&source_base);
        let request = PublishFolderRequest {
            scope: global_scope(),
            target_base_folder: path_to_string(&target_base),
            mode: PublishMode::Copy,
        };

        publish_scope_folder(&state, &request)
            .expect("whole-folder publish should replace an existing skills file");

        assert_eq!(
            fs::read_to_string(target_base.join("skills").join("fresh.txt"))
                .expect("fresh target file should exist"),
            "fresh"
        );

        fs::remove_dir_all(root).expect("test temp directory should be removed");
    }

    #[test]
    fn remove_published_skill_deletes_only_target_child() {
        let root = temp_root("remove-published-skill");
        let source_base = root.join("source").join(".claude");
        let source_skills = source_base.join("skills");
        fs::create_dir_all(&source_skills).expect("source skills directory should be created");
        fs::write(source_skills.join("demo.md"), "source").expect("source skill should be written");

        let target_base = root.join("target").join(".codex");
        let target_skills = target_base.join("skills");
        fs::create_dir_all(&target_skills).expect("target skills directory should be created");
        fs::write(target_skills.join("demo.md"), "target").expect("target skill should be written");

        let state = state_for_global(&source_base);
        let request = RemoveSkillRequest {
            scope: global_scope(),
            skill_name: "demo.md".to_string(),
            target_base_folder: path_to_string(&target_base),
        };

        remove_published_scope_skill(&state, &request)
            .expect("published skill removal should succeed");

        assert!(
            source_skills.join("demo.md").exists(),
            "source skill must not be removed"
        );
        assert!(
            !target_skills.join("demo.md").exists(),
            "target skill should be removed"
        );

        fs::remove_dir_all(root).expect("test temp directory should be removed");
    }

    #[test]
    fn remove_published_skill_allows_final_symlink_to_source() {
        let root = temp_root("remove-final-symlink");
        let source_base = root.join("source").join(".claude");
        let source_skills = source_base.join("skills");
        fs::create_dir_all(&source_skills).expect("source skills directory should be created");
        let source_skill = source_skills.join("demo.md");
        fs::write(&source_skill, "source").expect("source skill should be written");

        let target_base = root.join("target").join(".codex");
        let target_skills = target_base.join("skills");
        fs::create_dir_all(&target_skills).expect("target skills directory should be created");
        let target_skill = target_skills.join("demo.md");
        if create_test_file_symlink(&source_skill, &target_skill).is_err() {
            fs::remove_dir_all(root).expect("test temp directory should be removed");
            return;
        }

        let state = state_for_global(&source_base);
        let request = RemoveSkillRequest {
            scope: global_scope(),
            skill_name: "demo.md".to_string(),
            target_base_folder: path_to_string(&target_base),
        };

        remove_published_scope_skill(&state, &request)
            .expect("published final symlink removal should succeed");

        assert!(source_skill.exists(), "source skill must not be removed");
        assert!(
            fs::symlink_metadata(&target_skill).is_err(),
            "target symlink should be removed"
        );

        fs::remove_dir_all(root).expect("test temp directory should be removed");
    }

    #[test]
    fn remove_published_skill_rejects_parent_symlink_to_source() {
        let root = temp_root("remove-parent-symlink");
        let source_base = root.join("source").join(".claude");
        let source_skills = source_base.join("skills");
        fs::create_dir_all(&source_skills).expect("source skills directory should be created");
        let source_skill = source_skills.join("demo.md");
        fs::write(&source_skill, "source").expect("source skill should be written");

        let target_base = root.join("target").join(".codex");
        fs::create_dir_all(&target_base).expect("target base should be created");
        let target_skills = target_base.join("skills");
        if create_test_dir_symlink(&source_skills, &target_skills).is_err() {
            fs::remove_dir_all(root).expect("test temp directory should be removed");
            return;
        }

        let state = state_for_global(&source_base);
        let request = RemoveSkillRequest {
            scope: global_scope(),
            skill_name: "demo.md".to_string(),
            target_base_folder: path_to_string(&target_base),
        };

        let result = remove_published_scope_skill(&state, &request);

        assert_eq!(
            result.expect_err("parent source symlink removal should be rejected"),
            "不能从发布目标移除当前源路径。"
        );
        assert!(source_skill.exists(), "source skill must not be removed");

        fs::remove_dir_all(root).expect("test temp directory should be removed");
    }

    #[test]
    fn delete_skill_cleans_targets_before_source() {
        let root = temp_root("delete-skill-cleanup");
        let source_base = root.join("source").join(".claude");
        let source_skills = source_base.join("skills");
        fs::create_dir_all(&source_skills).expect("source skills directory should be created");
        let source_skill = source_skills.join("demo.md");
        fs::write(&source_skill, "source").expect("source skill should be written");

        let copy_target_base = root.join("target").join(".codex");
        let copy_target_skills = copy_target_base.join("skills");
        fs::create_dir_all(&copy_target_skills).expect("copy target should be created");
        let copy_target_skill = copy_target_skills.join("demo.md");
        fs::write(&copy_target_skill, "copy").expect("copy target skill should be written");

        let link_target_base = root.join("custom").join(".agents");
        let link_target_skills = link_target_base.join("skills");
        fs::create_dir_all(&link_target_skills).expect("link target should be created");
        let link_target_skill = link_target_skills.join("demo.md");
        let has_link = create_test_file_symlink(&source_skill, &link_target_skill).is_ok();

        let source_alias_base = root.join("alias").join(".claude");
        fs::create_dir_all(&source_alias_base).expect("source alias base should be created");
        let source_alias_skills = source_alias_base.join("skills");
        if create_test_dir_symlink(&source_skills, &source_alias_skills).is_err() {
            fs::remove_dir_all(root).expect("test temp directory should be removed");
            return;
        }

        let mut target_bases = vec![copy_target_base, source_alias_base];
        let expected_removed = if has_link {
            target_bases.push(link_target_base);
            2
        } else {
            1
        };

        let removed_targets =
            delete_skill_from_target_bases(&source_skill, &target_bases, "demo.md")
                .expect("skill deletion should succeed");

        assert_eq!(removed_targets, expected_removed);
        assert!(
            !copy_target_skill.exists(),
            "copied target skill should be removed"
        );
        if has_link {
            assert!(
                fs::symlink_metadata(&link_target_skill).is_err(),
                "final target symlink should be removed"
            );
        }
        assert!(
            !source_skill.exists(),
            "source skill should be removed after targets"
        );

        fs::remove_dir_all(root).expect("test temp directory should be removed");
    }

    #[test]
    fn source_match_follows_parent_but_not_final_symlink() {
        let root = temp_root("source-match-final-symlink");
        let source_parent = root.join("source").join("skills");
        let target_parent = root.join("target").join("skills");
        fs::create_dir_all(&source_parent).expect("source parent should be created");
        fs::create_dir_all(&target_parent).expect("target parent should be created");
        let source_skill = source_parent.join("demo.md");
        let target_skill = target_parent.join("demo.md");
        fs::write(&source_skill, "source").expect("source skill should be written");

        if create_test_file_symlink(&source_skill, &target_skill).is_ok() {
            assert!(
                !paths_match_without_following_final_symlink(&source_skill, &target_skill),
                "a target file symlink is a removable published item, not the source path"
            );
        }

        assert!(
            paths_match_without_following_final_symlink(&source_skill, &source_skill),
            "the literal source path remains protected"
        );

        fs::remove_dir_all(root).expect("test temp directory should be removed");
    }

    #[test]
    fn remove_published_folder_rejects_current_source_target() {
        let root = temp_root("remove-source-folder");
        let source_base = root.join("source").join(".claude");
        let source_skills = source_base.join("skills");
        fs::create_dir_all(&source_skills).expect("source skills directory should be created");
        fs::write(source_skills.join("demo.md"), "source").expect("source skill should be written");

        let state = state_for_global(&source_base);
        let request = RemoveFolderRequest {
            scope: global_scope(),
            target_base_folder: path_to_string(&source_base),
        };

        let result = remove_published_scope_folder(&state, &request);

        assert_eq!(
            result.expect_err("source target removal should be rejected"),
            "不能从发布目标移除当前源路径。"
        );
        assert!(
            source_skills.exists(),
            "source skills folder must not be removed"
        );

        fs::remove_dir_all(root).expect("test temp directory should be removed");
    }
}
