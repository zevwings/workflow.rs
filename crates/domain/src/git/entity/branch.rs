//! 分支相关实体

/// 分支信息
///
/// 包含分支的完整元数据和显示信息
#[derive(Debug, Clone)]
pub struct BranchInfo {
    /// 原始分支名称（如 "origin/feature" 或 "feature"）
    pub name: String,
    /// 显示名称（如 "[R] feature" 或 "feature"）
    pub display_name: String,
    /// 是否为远程分支
    pub is_remote: bool,
    /// 是否为当前分支
    pub is_current: bool,
    /// 最新提交 SHA（短格式，可选）
    pub commit_sha: Option<String>,
    /// 最新提交消息（可选）
    pub commit_message: Option<String>,
    /// 上游分支（可选）
    pub upstream: Option<String>,
}

impl BranchInfo {
    /// 创建本地分支信息
    pub fn local(name: String) -> Self {
        Self {
            display_name: name.clone(),
            name,
            is_remote: false,
            is_current: false,
            commit_sha: None,
            commit_message: None,
            upstream: None,
        }
    }

    /// 创建远程分支信息
    pub fn remote(name: String, display_name: String) -> Self {
        Self {
            name,
            display_name,
            is_remote: true,
            is_current: false,
            commit_sha: None,
            commit_message: None,
            upstream: None,
        }
    }
}

/// 分支过滤器
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchFilter {
    /// 仅本地分支
    Local,
    /// 仅远程分支
    Remote,
    /// 所有分支
    All,
}
