use serde::{Deserialize, Serialize};

pub(crate) type CommandResult<T> = Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Workspace {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) base_folder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppState {
    pub(crate) global_base_folder: String,
    pub(crate) workspaces: Vec<Workspace>,
    pub(crate) target_base_folders: Vec<String>,
    #[serde(default = "crate::state::default_enabled_target_ids")]
    pub(crate) enabled_target_ids: Vec<String>,
    #[serde(default = "crate::state::default_publish_mode")]
    pub(crate) default_publish_mode: PublishMode,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScopeSelection {
    pub(crate) kind: String,
    pub(crate) workspace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SkillEntry {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) kind: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScanResult {
    pub(crate) skills_path: String,
    pub(crate) exists: bool,
    pub(crate) skills: Vec<SkillEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceGitDetail {
    pub(crate) path: String,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PublishTargetStatus {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) folder_name: String,
    pub(crate) base_folder: String,
    pub(crate) skills_path: String,
    pub(crate) enabled: bool,
    pub(crate) is_source: bool,
    pub(crate) protects_source_children: bool,
    pub(crate) has_skills_folder: bool,
    pub(crate) installed_skill_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TargetScanResult {
    pub(crate) targets: Vec<PublishTargetStatus>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PublishFolderRequest {
    pub(crate) scope: ScopeSelection,
    pub(crate) target_base_folder: String,
    pub(crate) mode: PublishMode,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PublishSkillRequest {
    pub(crate) scope: ScopeSelection,
    pub(crate) skill_name: String,
    pub(crate) target_base_folder: String,
    pub(crate) mode: PublishMode,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RemoveFolderRequest {
    pub(crate) scope: ScopeSelection,
    pub(crate) target_base_folder: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RemoveSkillRequest {
    pub(crate) scope: ScopeSelection,
    pub(crate) skill_name: String,
    pub(crate) target_base_folder: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeleteSkillRequest {
    pub(crate) scope: ScopeSelection,
    pub(crate) skill_name: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PublishMode {
    Symlink,
    Copy,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OperationResult {
    pub(crate) message: String,
    pub(crate) source: String,
    pub(crate) target: String,
}
