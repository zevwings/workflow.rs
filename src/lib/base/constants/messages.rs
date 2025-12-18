//! 消息常量
//!
//! 统一管理用户交互消息、日志消息等跨模块使用的消息常量。

/// GitHub PR 相关常量
pub mod pull_requests {
    /// PR 批准事件
    pub const APPROVE_EVENT: &str = "APPROVE";

    /// PR 请求修改事件
    pub const REQUEST_CHANGES_EVENT: &str = "REQUEST_CHANGES";

    /// PR 评论事件
    pub const COMMENT_EVENT: &str = "COMMENT";

    /// PR 批准 emoji
    pub const APPROVE_EMOJI: &str = "👍";
}

/// 用户交互消息
pub mod user {
    /// 操作已取消
    pub const OPERATION_CANCELLED: &str = "Operation cancelled";

    /// 不存在
    pub const NOT_EXISTS: &str = "Not exists";

    /// 未设置
    pub const NOT_SET: &str = "Not set";

    /// 下载完成
    pub const DOWNLOAD_COMPLETE: &str = "Download complete";

    /// 更新已取消
    pub const UPDATE_CANCELLED: &str = "Update cancelled";

    /// 安装失败
    pub const INSTALLATION_FAILED: &str = "Installation failed";

    /// 更新失败
    pub const UPDATE_FAILED: &str = "Update failed";

    /// 回滚完成
    pub const ROLLBACK_COMPLETED: &str = "Rollback completed";
}

/// 日志消息
pub mod log {
    /// 分支重命名
    pub const BRANCH_RENAME: &str = "Branch Rename";

    /// 测试失败
    pub const TESTS_FAILED: &str = "Tests failed";

    /// 配置保存消息前缀
    pub const CONFIG_SAVED_PREFIX: &str = "Configuration saved to";
}
