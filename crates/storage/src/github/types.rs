//! GitHub API 类型定义
//!
//! 定义所有与 GitHub API 交互时使用的数据结构。

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// GitHub 用户信息
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUserInfo {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

/// 创建 Pull Request 请求
#[derive(Debug, Serialize)]
pub struct CreatePullRequestRequest {
    pub title: String,
    pub body: String,
    pub head: String,
    pub base: String,
}

/// 合并 Pull Request 请求
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct MergePullRequestRequest {
    pub commit_title: Option<String>,
    pub commit_message: Option<String>,
    pub merge_method: String,
}

/// 更新 Pull Request 请求
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct UpdatePullRequestRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub state: Option<String>,
    pub base: Option<String>,
}

/// 创建 Pull Request 响应
#[derive(Debug, Deserialize)]
pub struct CreatePullRequestResponse {
    pub html_url: String,
}

/// Pull Request 信息
#[derive(Debug, Deserialize, Clone)]
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
    pub user: Option<GitHubUserInfo>,
}

/// Pull Request 分支信息
#[derive(Debug, Deserialize, Clone)]
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
    /// 补丁内容（如果文件较小）
    #[serde(default)]
    pub patch: Option<String>,
}
