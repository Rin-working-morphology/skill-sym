use std::path::{Component, Path, PathBuf};

use crate::models::CommandResult;

pub(crate) fn absolute_path(path: &Path) -> CommandResult<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .map_err(readable_io_error)
}

pub(crate) fn ensure_existing_dir(path: &str, label: &str) -> CommandResult<PathBuf> {
    let absolute = absolute_path(Path::new(path))?;
    if !absolute.exists() {
        return Err(format!("{label}文件夹不存在：{}", absolute.display()));
    }
    if !absolute.is_dir() {
        return Err(format!("{label}路径不是文件夹：{}", absolute.display()));
    }
    Ok(absolute)
}

pub(crate) fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

pub(crate) fn readable_io_error(error: std::io::Error) -> String {
    format!("文件系统操作失败：{error}")
}

pub(crate) fn validate_child_name(name: &str) -> CommandResult<()> {
    if name.trim().is_empty() || name.contains('/') || name.contains('\\') {
        return Err("技能名称必须是单个文件或文件夹名称。".to_string());
    }

    let path = Path::new(name);
    let mut components = path.components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(()),
        _ => Err("技能名称必须是单个文件或文件夹名称。".to_string()),
    }
}

/// Validate a folder path relative to a known root (home or workspace).
///
/// Unlike [`validate_child_name`], nested folders such as `.config/opencode`
/// are allowed, but absolute paths and parent-directory escapes are not.
pub(crate) fn validate_relative_path(value: &str) -> CommandResult<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("目录必须是相对路径。".to_string());
    }

    let path = Path::new(trimmed);
    if path.is_absolute() || path.has_root() {
        return Err("目录必须是相对路径。".to_string());
    }

    let mut has_normal_component = false;
    for component in path.components() {
        match component {
            Component::Normal(_) => has_normal_component = true,
            Component::CurDir => {}
            _ => return Err("目录必须是相对路径。".to_string()),
        }
    }

    if !has_normal_component {
        return Err("目录必须是相对路径。".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_child_name, validate_relative_path};

    #[test]
    fn validate_child_name_rejects_path_like_values() {
        assert!(validate_child_name("skill-a").is_ok());
        assert!(validate_child_name("../outside").is_err());
        assert!(validate_child_name("nested/skill").is_err());
        assert!(validate_child_name(r"nested\skill").is_err());
        assert!(validate_child_name("   ").is_err());
    }

    #[test]
    fn validate_relative_path_allows_nested_folders_but_rejects_escapes() {
        assert!(validate_relative_path(".agents").is_ok());
        assert!(validate_relative_path(".config/opencode").is_ok());
        assert!(validate_relative_path(".pi/agent").is_ok());
        assert!(validate_relative_path("../outside").is_err());
        assert!(validate_relative_path("nested/../outside").is_err());
        assert!(validate_relative_path("/etc/skills").is_err());
        assert!(validate_relative_path("").is_err());
        assert!(validate_relative_path("   ").is_err());
    }
}
