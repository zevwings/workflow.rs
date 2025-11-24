//! Git 操作辅助函数
//!
//! 提供通用的 Git 命令执行辅助函数，减少代码重复。

use anyhow::{Context, Result};
use duct::cmd;

/// Git 命令执行常量
pub(crate) const COMMON_DEFAULT_BRANCHES: &[&str] = &["main", "master", "develop", "dev"];

/// 执行 Git 命令并读取输出
///
/// # 参数
///
/// * `args` - Git 命令参数
///
/// # 返回
///
/// 返回命令输出的字符串（去除首尾空白）
pub(crate) fn cmd_read(args: &[&str]) -> Result<String> {
    let output = cmd("git", args)
        .read()
        .with_context(|| format!("Failed to run: git {}", args.join(" ")))?;
    Ok(output.trim().to_string())
}

/// 执行 Git 命令（不读取输出）
///
/// # 参数
///
/// * `args` - Git 命令参数
pub(crate) fn cmd_run(args: &[&str]) -> Result<()> {
    cmd("git", args)
        .run()
        .with_context(|| format!("Failed to run: git {}", args.join(" ")))?;
    Ok(())
}

/// 静默执行 Git 命令并检查是否成功
///
/// # 参数
///
/// * `args` - Git 命令参数
///
/// # 返回
///
/// 如果命令成功执行（退出码为 0），返回 `true`，否则返回 `false`
pub(crate) fn check_success(args: &[&str]) -> bool {
    cmd("git", args).stdout_null().stderr_null().run().is_ok()
}

/// 检查 Git 引用是否存在
///
/// # 参数
///
/// * `ref_path` - Git 引用路径（如 `refs/heads/main`）
///
/// # 返回
///
/// 如果引用存在，返回 `true`，否则返回 `false`
pub(crate) fn check_ref_exists(ref_path: &str) -> bool {
    check_success(&["rev-parse", "--verify", ref_path])
}

/// 尝试使用 git switch，失败时回退到 git checkout
///
/// # 参数
///
/// * `switch_args` - git switch 的参数
/// * `checkout_args` - git checkout 的回退参数
/// * `error_msg` - 错误消息
pub(crate) fn switch_or_checkout(
    switch_args: &[&str],
    checkout_args: &[&str],
    error_msg: impl AsRef<str>,
) -> Result<()> {
    let result = cmd("git", switch_args).stdout_null().stderr_null().run();

    if result.is_err() {
        cmd_run(checkout_args).with_context(|| error_msg.as_ref().to_string())?;
    }
    Ok(())
}

/// 移除分支名称的前缀
///
/// 从完整的分支名中提取基础名称，支持两种格式：
/// - `prefix/branch-name`（使用 `/` 分割）
/// - `ticket--branch-name`（使用 `--` 分割）
///
/// # 参数
///
/// * `branch` - 完整的分支名称
///
/// # 返回
///
/// 返回基础分支名称
pub(crate) fn remove_branch_prefix(branch: &str) -> &str {
    branch
        .rsplit('/')
        .next()
        .unwrap_or(branch)
        .rsplit("--")
        .next()
        .unwrap_or(branch)
}
