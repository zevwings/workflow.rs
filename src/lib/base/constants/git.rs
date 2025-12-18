//! Git 操作相关常量
//!
//! 统一管理 Git 相关的错误消息和检查信息。

/// Git 检查错误消息
pub mod check_errors {
    /// 不在 Git 仓库中
    pub const NOT_GIT_REPO: &str = "Git check failed: Not in a Git repository";

    /// 网络检查失败
    pub const NETWORK_CHECK_FAILED: &str = "Network check failed";

    /// Lint 检查失败
    pub const LINT_CHECK_FAILED: &str = "Lint check failed";
}
