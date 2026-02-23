//! Hook 常量定义
//!
//! 定义 Git hooks 和 pre-commit 工具支持的 hook 名称常量。

/// 标准 Git hooks 名称
pub mod git_hooks {
    /// 提交前检查
    pub const PRE_COMMIT: &str = "pre-commit";
    /// 准备提交消息
    pub const PREPARE_COMMIT_MSG: &str = "prepare-commit-msg";
    /// 提交消息验证
    pub const COMMIT_MSG: &str = "commit-msg";
    /// 提交后
    pub const POST_COMMIT: &str = "post-commit";
    /// 推送前检查
    pub const PRE_PUSH: &str = "pre-push";
    /// Rebase 前检查
    pub const PRE_REBASE: &str = "pre-rebase";
    /// 合并后
    pub const POST_MERGE: &str = "post-merge";
    /// 切换分支后
    pub const POST_CHECKOUT: &str = "post-checkout";
    /// 合并提交前
    pub const PRE_MERGE_COMMIT: &str = "pre-merge-commit";
    /// rebase/amend 后
    pub const POST_REWRITE: &str = "post-rewrite";
}

/// pre-commit/prek 工具支持的 hooks
///
/// 这些是 pre-commit 框架官方支持的 hook 类型。
/// 参考: <https://pre-commit.com/#supported-git-hooks>
pub mod pre_commit_hooks {
    pub use crate::git::services::hooks::constants::git_hooks::{
        COMMIT_MSG, POST_CHECKOUT, POST_COMMIT, POST_MERGE, POST_REWRITE, PREPARE_COMMIT_MSG,
        PRE_COMMIT, PRE_MERGE_COMMIT, PRE_PUSH, PRE_REBASE,
    };

    /// pre-commit 工具支持的所有 hooks 列表
    pub const SUPPORTED: &[&str] = &[
        PRE_COMMIT,
        PRE_MERGE_COMMIT,
        PRE_PUSH,
        COMMIT_MSG,
        PREPARE_COMMIT_MSG,
        POST_CHECKOUT,
        POST_COMMIT,
        POST_MERGE,
        POST_REWRITE,
        PRE_REBASE,
    ];

    /// 检查 hook 名称是否被 pre-commit 工具支持
    ///
    /// # 参数
    /// - `hook_name`: Hook 名称
    ///
    /// # 返回
    /// - `true`: 支持
    /// - `false`: 不支持
    #[inline]
    pub fn is_supported(hook_name: impl AsRef<str>) -> bool {
        SUPPORTED.contains(&hook_name.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_commit_hooks_supported() {
        assert!(pre_commit_hooks::is_supported("pre-commit"));
        assert!(pre_commit_hooks::is_supported("commit-msg"));
        assert!(pre_commit_hooks::is_supported("pre-push"));
        assert!(!pre_commit_hooks::is_supported("pre-receive")); // 服务端 hook
        assert!(!pre_commit_hooks::is_supported("invalid-hook"));
    }

    #[test]
    fn test_git_hooks_constants() {
        assert_eq!(git_hooks::PRE_COMMIT, "pre-commit");
        assert_eq!(git_hooks::COMMIT_MSG, "commit-msg");
    }
}
