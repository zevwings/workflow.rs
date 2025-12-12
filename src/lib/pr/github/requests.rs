use serde::Serialize;
use serde_with::skip_serializing_none;

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
