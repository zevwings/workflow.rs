use serde::Deserialize;
use serde_with::skip_serializing_none;

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
    pub merged: Option<bool>,
    #[serde(rename = "merged_at", default)]
    pub merged_at: Option<String>,
    #[serde(default)]
    pub mergeable: Option<bool>,
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
#[skip_serializing_none]
#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Pull Request 文件信息
#[derive(Debug, Deserialize)]
pub struct PullRequestFile {
    /// 文件路径
    pub filename: String,
    /// 文件状态（added, removed, modified, renamed, etc.）
    pub status: String,
    /// 添加的行数
    pub additions: u32,
    /// 删除的行数
    pub deletions: u32,
    /// 变更的行数
    pub changes: u32,
    /// 文件的 SHA（base）
    #[serde(default)]
    pub sha: Option<String>,
    /// 文件的 blob URL
    #[serde(default)]
    pub blob_url: Option<String>,
    /// 原始文件内容 URL（base）
    #[serde(default)]
    pub contents_url: Option<String>,
    /// 补丁内容（如果文件较小）
    #[serde(default)]
    pub patch: Option<String>,
}
