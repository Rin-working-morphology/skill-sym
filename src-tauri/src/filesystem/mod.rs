use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use tauri::AppHandle;

use crate::{
    models::{
        AppState, CommandResult, DeleteSkillRequest, OperationResult, PublishFolderRequest,
        PublishMode, PublishSkillRequest, PublishTargetStatus, RemoveFolderRequest,
        RemoveSkillRequest, ScanResult, ScopeSelection, SkillDiffEntry, SkillDiffResult,
        SkillEntry, TargetScanResult,
    },
    paths::{absolute_path, path_to_string, readable_io_error, validate_child_name},
    state::{
        app_home_dir, publish_target_definitions, scope_skills_path, target_base_folder,
        target_root_path, SKILL_MAIN_FILE, SKILLS_FOLDER_NAME,
    },
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
    let home = app_home_dir(app)?;
    let source_skills = absolute_path(&scope_skills_path(state, scope)?)?;
    let mut targets = Vec::new();

    for supported in publish_target_definitions(state, &home) {
        let base_folder = target_base_folder(&supported, &root, &scope.kind);
        let skills_path = base_folder.join(SKILLS_FOLDER_NAME);
        let installed_skill_names = installed_skill_names(&skills_path)?;
        let has_skills_folder = fs::symlink_metadata(&skills_path).is_ok();
        let skills_path_abs = absolute_path(&skills_path)?;
        let is_source =
            paths_match_without_following_final_symlink(&source_skills, &skills_path_abs);
        let protects_source_children = paths_equal(&source_skills, &skills_path_abs);
        let enabled = state
            .enabled_target_ids
            .iter()
            .any(|id| id == &supported.id);
        let folder_name = if scope.kind == "global" {
            &supported.global_folder
        } else {
            &supported.project_folder
        };

        targets.push(PublishTargetStatus {
            id: supported.id,
            name: supported.name,
            folder_name: folder_name.clone(),
            global_folder_name: supported.global_folder,
            project_folder_name: supported.project_folder,
            base_folder: path_to_string(&base_folder),
            skills_path: path_to_string(&skills_path),
            installed: supported.installed,
            enabled,
            is_source,
            protects_source_children,
            has_skills_folder,
            installed_skill_names,
        });
    }

    Ok(TargetScanResult { targets })
}

/// 比对当前范围与各发布目标之间同名技能的物理副本差异。
///
/// 只统计物理副本：符号链接副本跟随源目录，天然保持一致，不参与比对。
/// 每个技能先收集各副本的内容摘要与文件树，再在拥有两份及以上副本时对比：
/// 内容指 SKILL.md（单文件技能则为整个文件），文件树指其余文件的相对路径集合。
pub(crate) fn scan_scope_skill_differences(
    app: &AppHandle,
    state: &AppState,
    scope: &ScopeSelection,
) -> CommandResult<SkillDiffResult> {
    let source_skills = scope_skills_path(state, scope)?;
    let source_base = source_skills
        .parent()
        .unwrap_or(&source_skills)
        .to_path_buf();
    let target_bases = publish_target_base_paths(app, state, scope)?;
    compare_skill_copies(&source_base, &source_skills, &target_bases)
}

/// 以源目录为基准，对比各目标基础目录中同名技能的物理副本。
fn compare_skill_copies(
    source_base: &Path,
    source_skills: &Path,
    target_bases: &[PathBuf],
) -> CommandResult<SkillDiffResult> {
    // 源目录本身也作为一份参与对比，保证只发布到部分目标时仍能发现差异。
    // 源可能同时是某个发布目标的目录（例如全局源就是 .agents），所以插入后
    // 需要重新去重，避免同一位置被算成两份副本。
    let mut bases: Vec<PathBuf> = Vec::with_capacity(target_bases.len() + 1);
    bases.push(source_base.to_path_buf());
    bases.extend(target_bases.iter().cloned());
    bases.sort_by_key(|path| path_to_string(path).to_lowercase());
    bases.dedup_by(|left, right| paths_match_without_following_final_symlink(left, right));

    let mut entries = Vec::new();
    for name in skill_names_in(source_skills)? {
        let mut digests: Vec<SkillCopyDigest> = Vec::new();
        for base in &bases {
            if let Some(digest) = skill_copy_digest(base, &name)? {
                digests.push(digest);
            }
        }

        // 至少两份副本才谈得上"不同步"；单独一份没有可比对象。
        if digests.len() < 2 {
            continue;
        }

        let mut content_differs = false;
        let mut tree_differs = false;
        for pair in digests.windows(2) {
            if pair[0].content != pair[1].content {
                content_differs = true;
            }
            if pair[0].tree != pair[1].tree {
                tree_differs = true;
            }
        }
        if content_differs || tree_differs {
            entries.push(SkillDiffEntry {
                skill_name: name,
                content_differs,
                tree_differs,
                copy_count: digests.len(),
            });
        }
    }

    entries.sort_by(|left, right| left.skill_name.cmp(&right.skill_name));
    Ok(SkillDiffResult { entries })
}

struct SkillCopyDigest {
    content: Vec<u8>,
    tree: BTreeMap<String, u64>,
}

/// 读取一份技能副本的比对材料：内容主文件字节 + 其余文件树（相对路径 -> 大小）。
/// 符号链接副本返回 None（链接跟随源，不需要比对）；源位置本身始终返回 Some。
fn skill_copy_digest(base: &Path, skill_name: &str) -> CommandResult<Option<SkillCopyDigest>> {
    let skill_path = base.join(SKILLS_FOLDER_NAME).join(skill_name);
    let Ok(metadata) = fs::symlink_metadata(&skill_path) else {
        return Ok(None);
    };
    if metadata.file_type().is_symlink() {
        return Ok(None);
    }

    if metadata.is_file() {
        return Ok(Some(SkillCopyDigest {
            content: fs::read(&skill_path).map_err(readable_io_error)?,
            tree: BTreeMap::new(),
        }));
    }
    if !metadata.is_dir() {
        return Ok(None);
    }

    let main_file = skill_path.join(SKILL_MAIN_FILE);
    let content = if main_file.is_file() {
        fs::read(&main_file).map_err(readable_io_error)?
    } else {
        Vec::new()
    };

    let mut tree = BTreeMap::new();
    collect_tree_entries(&skill_path, &skill_path, &mut tree)?;
    Ok(Some(SkillCopyDigest { content, tree }))
}

/// 递归记录相对路径与文件大小；跳过符号链接，避免把链接目标误当成树结构。
fn collect_tree_entries(
    root: &Path,
    dir: &Path,
    tree: &mut BTreeMap<String, u64>,
) -> CommandResult<()> {
    for entry in fs::read_dir(dir).map_err(readable_io_error)? {
        let entry = entry.map_err(readable_io_error)?;
        let entry_path = entry.path();
        let metadata = fs::symlink_metadata(&entry_path).map_err(readable_io_error)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        let relative = entry_path
            .strip_prefix(root)
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        if metadata.is_dir() {
            collect_tree_entries(root, &entry_path, tree)?;
        } else {
            tree.insert(relative, metadata.len());
        }
    }
    Ok(())
}

/// 当前范围内技能名的并集：源目录加上所有目标目录中出现的名字。
fn skill_names_in(source_skills: &Path) -> CommandResult<Vec<String>> {
    let mut names = Vec::new();
    if source_skills.is_dir() {
        for entry in fs::read_dir(source_skills).map_err(readable_io_error)? {
            let entry = entry.map_err(readable_io_error)?;
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    names.sort_by_key(|name| name.to_lowercase());
    names.dedup();
    Ok(names)
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
    let home = app_home_dir(app)?;
    let mut paths: Vec<PathBuf> = publish_target_definitions(state, &home)
        .into_iter()
        .map(|target| target_base_folder(&target, &root, &scope.kind))
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
        compare_skill_copies, delete_skill_from_target_bases,
        paths_match_without_following_final_symlink, publish_path, scan_scope_skills,
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
        let path =
            std::env::temp_dir().join(format!("skillsym-{name}-{}-{nanos}", std::process::id()));
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
            custom_publish_targets: Vec::new(),
            enabled_target_ids: default_enabled_target_ids(),
            known_detected_target_ids: Vec::new(),
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
    fn diff_scan_reports_content_and_tree_differences_between_copies() {
        let root = temp_root("diff-scan");
        let source_base = root.join("source").join(".claude");
        let source_skills = source_base.join("skills");
        let demo = source_skills.join("demo");
        fs::create_dir_all(&demo).expect("source skill directory should be created");
        fs::write(demo.join("SKILL.md"), "version a").expect("source SKILL.md should be written");
        fs::write(demo.join("extra.txt"), "extra").expect("source extra file should be written");

        // 副本一：内容不同
        let copy_a_base = root.join("target").join(".codex");
        let copy_a = copy_a_base.join("skills").join("demo");
        fs::create_dir_all(&copy_a).expect("copy a should be created");
        fs::write(copy_a.join("SKILL.md"), "version b").expect("copy a SKILL.md should be written");
        fs::write(copy_a.join("extra.txt"), "extra").expect("copy a extra file should be written");

        // 副本二：内容相同但文件树缺少 extra.txt
        let copy_b_base = root.join("target").join(".gemini");
        let copy_b = copy_b_base.join("skills").join("demo");
        fs::create_dir_all(&copy_b).expect("copy b should be created");
        fs::write(copy_b.join("SKILL.md"), "version a").expect("copy b SKILL.md should be written");

        // 单副本技能：没有可比对象，不应出现在结果里。
        let solo_base = root.join("target").join(".qoder");
        let solo = solo_base.join("skills").join("demo");
        fs::create_dir_all(&solo).expect("solo copy should be created");
        fs::write(solo.join("SKILL.md"), "unique").expect("solo SKILL.md should be written");

        let mut state = state_for_global(&source_base);
        state.target_base_folders =
            vec![path_to_string(&copy_a_base), path_to_string(&copy_b_base)];
        let target_bases: Vec<PathBuf> = state
            .target_base_folders
            .iter()
            .map(PathBuf::from)
            .collect();

        let result = compare_skill_copies(&source_base, &source_skills, &target_bases)
            .expect("diff scan should succeed");

        assert_eq!(result.entries.len(), 1, "only multi-copy skills are compared");
        let entry = &result.entries[0];
        assert_eq!(entry.skill_name, "demo");
        assert!(entry.content_differs, "SKILL.md mismatch should be reported");
        assert!(entry.tree_differs, "missing extra.txt should be reported");
        assert_eq!(entry.copy_count, 3, "source plus two copies take part");

        fs::remove_dir_all(root).expect("test temp directory should be removed");
    }

    #[test]
    fn diff_scan_does_not_double_count_when_source_is_also_a_target() {
        // 全局源目录常常同时是某个发布目标的目录（例如 .agents），
        // 此时它只能算一份副本，而不是两份。
        let root = temp_root("diff-scan-duplicate");
        let source_base = root.join("target").join(".agents");
        let source_skills = source_base.join("skills");
        let demo = source_skills.join("demo");
        fs::create_dir_all(&demo).expect("source skill directory should be created");
        fs::write(demo.join("SKILL.md"), "version a").expect("source SKILL.md should be written");

        let copy_base = root.join("target").join(".claude");
        let copy = copy_base.join("skills").join("demo");
        fs::create_dir_all(&copy).expect("copy should be created");
        fs::write(copy.join("SKILL.md"), "version b").expect("copy SKILL.md should be written");

        let target_bases = vec![source_base.clone(), copy_base.clone()];
        let result = compare_skill_copies(&source_base, &source_skills, &target_bases)
            .expect("diff scan should succeed");

        assert_eq!(result.entries.len(), 1, "the mismatch should be flagged");
        assert_eq!(
            result.entries[0].copy_count, 2,
            "the duplicated source location should be counted once"
        );

        fs::remove_dir_all(root).expect("test temp directory should be removed");
    }

    #[test]
    fn diff_scan_ignores_symlink_copies_and_identical_sets() {
        let root = temp_root("diff-scan-symlink");
        let source_base = root.join("source").join(".claude");
        let source_skills = source_base.join("skills");
        let demo = source_skills.join("demo");
        fs::create_dir_all(&demo).expect("source skill directory should be created");
        fs::write(demo.join("SKILL.md"), "same").expect("source SKILL.md should be written");

        // 符号链接副本跟随源目录，天然一致，不应触发差异。
        let link_base = root.join("target").join(".codex");
        let link_skills = link_base.join("skills");
        fs::create_dir_all(&link_skills).expect("link base should be created");
        let has_link = create_test_dir_symlink(&demo, &link_skills.join("demo")).is_ok();

        // 一份内容一致的物理副本。
        let copy_base = root.join("target").join(".gemini");
        let copy = copy_base.join("skills").join("demo");
        fs::create_dir_all(&copy).expect("physical copy should be created");
        fs::write(copy.join("SKILL.md"), "same").expect("copy SKILL.md should be written");

        let mut state = state_for_global(&source_base);
        state.target_base_folders =
            vec![path_to_string(&link_base), path_to_string(&copy_base)];
        let target_bases: Vec<PathBuf> = state
            .target_base_folders
            .iter()
            .map(PathBuf::from)
            .collect();

        let result = compare_skill_copies(&source_base, &source_skills, &target_bases)
            .expect("diff scan should succeed");

        if has_link {
            assert!(
                result.entries.is_empty(),
                "identical copies and symlinks should not be flagged"
            );
        } else {
            assert_eq!(result.entries.len(), 0, "no differences exist");
        }

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
