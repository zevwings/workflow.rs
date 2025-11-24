use serde::Deserialize;

/// 创建 Pull Request 响应
#[derive(Debug, Deserialize)]
pub struct CreatePullRequestResponse {
    pub detail_url: Option<String>,
}

/// Pull Request 信息
#[derive(Debug, Deserialize)]
pub struct PullRequestInfo {
    #[allow(dead_code)]
    pub id: Option<u64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub source_branch: Option<String>,
    pub target_branch: Option<String>,
    pub state: Option<String>,
    pub detail_url: Option<String>,
    #[serde(rename = "iid")]
    pub pull_request_number: Option<u64>,
}

/// Codeup 用户信息
#[derive(Debug, Deserialize)]
pub struct CodeupUser {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
}
