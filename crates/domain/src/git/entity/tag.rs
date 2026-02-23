//! Tag 相关实体

/// Tag 删除范围
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagDeleteScope {
    /// 只删除本地 tag
    Local,
    /// 只删除远程 tag
    Remote,
    /// 删除本地和远程 tag
    Both,
}

/// Tag 删除结果信息
#[derive(Debug, Clone)]
pub struct TagDeleteInfo {
    /// Tag 名称
    pub name: String,
    /// 是否在本地存在
    pub exists_local: bool,
    /// 是否在远程存在
    pub exists_remote: bool,
}

/// Tag 创建范围
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagCreateScope {
    /// 只创建本地 tag
    Local,
    /// 创建本地 tag 并推送到远程
    Both,
}

/// Tag 创建结果信息
#[derive(Debug, Clone)]
pub struct TagCreateInfo {
    /// Tag 名称
    pub name: String,
    /// 是否在本地创建成功
    pub created_local: bool,
    /// 是否在远程创建成功
    pub created_remote: bool,
}
