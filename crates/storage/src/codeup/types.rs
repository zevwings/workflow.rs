//! Codeup API 类型定义

use serde::{Deserialize, Serialize};

/// 创建 Pull Request 请求
#[derive(Debug, Serialize)]
pub struct CreatePullRequestRequest {
    pub title: String,
    pub description: String,
    pub source_branch: String,
    pub target_branch: String,
}

/// 创建 Pull Request 响应
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreatePullRequestResponse {
    pub id: i64,
    pub iid: i64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub source_branch: String,
    pub target_branch: String,
    pub web_url: String,
}

/// 更新 Pull Request 请求
#[derive(Debug, Serialize, Default)]
pub struct UpdatePullRequestRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
}

/// 合并 Pull Request 请求
#[derive(Debug, Serialize)]
pub struct MergePullRequestRequest {
    pub merge_commit_message: Option<String>,
    pub should_remove_source_branch: bool,
}

/// Pull Request 信息
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct PullRequestInfo {
    pub id: i64,
    pub iid: i64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub source_branch: String,
    pub target_branch: String,
    pub web_url: String,
    #[serde(default)]
    pub merged: bool,
    #[serde(default)]
    pub mergeable: Option<bool>,
    pub author: Option<CodeupUser>,
}

/// Codeup 用户信息
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct CodeupUser {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
}

/// 添加评论请求
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct AddCommentRequest {
    pub body: String,
}
