//! 网络相关常量
//!
//! 统一管理网络操作相关的错误消息和配置。

/// 网络错误消息
pub mod errors {
    /// 网络超时
    pub const TIMEOUT: &str = "Network timeout";

    /// 连接失败
    pub const CONNECTION_FAILED: &str = "Connection failed";

    /// 速率限制超出
    pub const RATE_LIMIT_EXCEEDED: &str = "Rate limit exceeded";
}
