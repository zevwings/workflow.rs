use serde::Deserialize;

/// 创建 Pull Request 响应
#[derive(Debug, Deserialize)]
pub struct CreatePullRequestResponse {
    pub html_url: String,
}

/// Pull Request 信息
#[derive(Debug, Deserialize)]
pub struct PullRequestInfo {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    #[serde(default)]
    pub merged: bool,
    #[serde(rename = "merged_at", default)]
    pub merged_at: Option<String>,
    pub html_url: String,
    pub head: PullRequestBranch,
    pub base: PullRequestBranch,
    pub user: Option<GitHubUser>,
}

/// Pull Request 分支信息
#[derive(Debug, Deserialize)]
pub struct PullRequestBranch {
    #[serde(rename = "ref")]
    pub ref_name: String,
}

/// 仓库信息
#[derive(Debug, Deserialize)]
pub struct RepositoryInfo {
    #[serde(rename = "allow_squash_merge")]
    pub allow_squash_merge: Option<bool>,
    #[serde(rename = "allow_merge_commit")]
    pub allow_merge_commit: Option<bool>,
    #[serde(rename = "allow_rebase_merge")]
    pub allow_rebase_merge: Option<bool>,
}

/// GitHub 用户信息
#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}
