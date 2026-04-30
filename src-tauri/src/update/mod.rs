use crate::models::UpdateStatus;

pub(crate) fn check_for_updates() -> UpdateStatus {
    // Future release publishing can set this to:
    // https://github.com/<owner>/<repo>/releases/latest/download/latest.json
    // Once configured, wire this command to Tauri's updater plugin and signed artifacts.
    const UPDATE_ENDPOINT: &str = "";

    if UPDATE_ENDPOINT.is_empty() {
        return UpdateStatus {
            status: "notConfigured".to_string(),
            endpoint: None,
            message: "尚未配置更新检查。".to_string(),
            integration_note: "发布流程配置完成后，设置 GitHub 发布版本 latest.json 端点。"
                .to_string(),
        };
    }

    UpdateStatus {
        status: "ready".to_string(),
        endpoint: Some(UPDATE_ENDPOINT.to_string()),
        message: "更新端点已配置。".to_string(),
        integration_note: "使用 Tauri 更新元数据和签名发布产物进行实际检查。".to_string(),
    }
}
