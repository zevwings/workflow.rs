//! CNB API 类型定义

use serde::{Deserialize, Serialize};

/// CNB 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CNBUserInfo {
    #[serde(rename = "username")]
    pub login: String,
    #[serde(rename = "nickname")]
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Pull Request 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestInfo {
    pub number: String,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub merged: bool,
    pub merged_at: Option<String>,
    pub base: PullRef,
    pub head: PullRef,
    pub author: Option<UserInfo>,
    pub assignees: Option<Vec<UserInfo>>,
    pub reviewers: Option<Vec<Reviewer>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub html_url: Option<String>,
}

/// Pull Request 引用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRef {
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
    pub repo: Option<RepoInfo>,
}

/// 仓库信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub web_url: Option<String>,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    #[serde(rename = "username")]
    pub login: String,
    #[serde(rename = "nickname")]
    pub name: Option<String>,
    pub email: Option<String>,
}

/// 审查者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reviewer {
    pub user: UserInfo,
    pub review_state: Option<String>,
}

/// Pull Request 创建请求
#[derive(Debug, Serialize)]
pub struct CreatePullRequest {
    pub title: String,
    pub body: String,
    pub head: String,
    pub base: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_repo: Option<String>,
}

/// Pull Request 更新请求
#[derive(Debug, Serialize)]
pub struct UpdatePullRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/// Pull Request 合并请求
#[derive(Debug, Serialize)]
pub struct MergePullRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_message: Option<String>,
}

/// Pull Request 评论请求
#[derive(Debug, Serialize)]
pub struct CreateComment {
    pub body: String,
}

/// Pull Request 审查请求
#[derive(Debug, Serialize)]
pub struct CreateReview {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<Vec<ReviewComment>>,
}

/// 审查评论
#[derive(Debug, Serialize)]
pub struct ReviewComment {
    pub body: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_side: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_side: Option<String>,
}
