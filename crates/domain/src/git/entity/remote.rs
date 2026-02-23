//! 远程仓库相关实体

/// 远程信息
#[derive(Debug, Clone)]
pub struct RemoteInfo {
    /// 远程名称
    pub name: String,
    /// 远程 URL
    pub url: String,
    /// 推送 URL（如果不同）
    pub push_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteDirection {
    Push,
    Fetch,
}
