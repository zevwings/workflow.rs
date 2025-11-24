use serde::Serialize;

/// 创建 Pull Request 请求
#[derive(Debug, Serialize)]
pub struct CreatePullRequestRequest {
    pub source_project_id: u64,
    pub target_project_id: u64,
    pub source_branch: String,
    pub target_branch: String,
    pub title: String,
    pub description: String,
    pub tb_user_ids: Vec<u64>,
    pub reviewer_user_ids: Vec<u64>,
    pub create_from: String,
}

/// 合并 Pull Request 请求
#[derive(Debug, Serialize)]
pub struct MergePullRequestRequest {
    pub merge_method: String,
    pub delete_source_branch: bool,
}

/// 关闭 Pull Request 请求
#[derive(Debug, Serialize)]
pub struct ClosePullRequestRequest {
    pub state: String,
}
