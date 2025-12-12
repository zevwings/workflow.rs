use serde::Serialize;

/// 创建 Pull Request 请求
#[derive(Debug, Serialize)]
pub struct CreatePullRequestRequest {
    pub title: String,
    pub body: String,
    pub head: String,
    pub base: String,
}

/// 合并 Pull Request 请求
#[derive(Debug, Serialize)]
pub struct MergePullRequestRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_message: Option<String>,
    pub merge_method: String,
}

/// 更新 Pull Request 请求
#[derive(Debug, Serialize)]
pub struct UpdatePullRequestRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
}
