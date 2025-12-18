//! API URL 和端点常量
//!
//! 统一管理各种 API 服务的 URL 常量，便于配置管理和环境切换。

/// GitHub API 相关常量
pub mod github {
    /// GitHub API 基础 URL
    pub const API_BASE: &str = "https://api.github.com";

    /// GitHub 网站基础 URL
    pub const BASE: &str = "https://github.com";

    /// GitHub 域名
    pub const DOMAIN: &str = "github.com";
}
