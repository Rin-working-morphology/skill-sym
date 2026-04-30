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

#[cfg(test)]
mod tests {
    use super::validate_child_name;

    #[test]
    fn validate_child_name_rejects_path_like_values() {
        assert!(validate_child_name("skill-a").is_ok());
        assert!(validate_child_name("../outside").is_err());
        assert!(validate_child_name("nested/skill").is_err());
        assert!(validate_child_name(r"nested\skill").is_err());
        assert!(validate_child_name("   ").is_err());
    }
}
