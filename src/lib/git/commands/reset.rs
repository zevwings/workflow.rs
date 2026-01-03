//! Git Reset 操作命令封装
//!
//! 提供重置相关的所有 Git 命令操作，包括：
//! - 软重置（--soft）
//! - 混合重置（--mixed，默认）
//! - 硬重置（--hard）

use crate::git::commands::command::GitCommand;
use color_eyre::{eyre::WrapErr, Result};
use std::path::Path;

/// Git Reset 命令操作
pub struct GitResetCommand;

/// Reset 模式
#[derive(Debug, Clone, Copy)]
pub enum ResetMode {
    /// 软重置：只重置 HEAD，保留索引和工作区
    Soft,
    /// 混合重置（默认）：重置 HEAD 和索引，保留工作区
    Mixed,
    /// 硬重置：重置 HEAD、索引和工作区（会丢失未提交的更改）
    Hard,
}

impl GitResetCommand {
    /// 执行 Git reset 操作
    ///
    /// # 参数
    ///
    /// * `mode` - Reset 模式（Soft、Mixed、Hard）
    /// * `target` - 目标引用（如 "HEAD", "HEAD~1", commit SHA, 分支名等）
    ///   如果为 `None`，则重置到当前 HEAD（通常用于取消暂存）
    /// * `cwd` - 工作目录（可选）
    ///
    /// # 警告
    ///
    /// `ResetMode::Hard` 会**永久丢弃**工作区和暂存区的所有未提交更改，请谨慎使用。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::git::commands::{GitResetCommand, ResetMode};
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // 硬重置到上一个提交
    ///     GitResetCommand::reset(ResetMode::Hard, Some("HEAD~1"), None)?;
    ///
    ///     // 软重置到指定分支
    ///     GitResetCommand::reset(ResetMode::Soft, Some("main"), None)?;
    ///
    ///     // 取消暂存（重置索引到 HEAD）
    ///     GitResetCommand::reset(ResetMode::Mixed, None, None)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn reset(mode: ResetMode, target: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let mut args = vec!["reset"];

        // 添加模式参数
        match mode {
            ResetMode::Soft => args.push("--soft"),
            ResetMode::Mixed => args.push("--mixed"),
            ResetMode::Hard => args.push("--hard"),
        }

        // 添加目标（如果提供）
        // 注意：如果 target 为 None，git reset 默认重置到 HEAD
        if let Some(target) = target {
            args.push(target);
        }

        GitCommand::execute(&args, cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| {
                format!(
                    "Failed to reset (mode: {:?}, target: {:?})",
                    mode,
                    target.unwrap_or("HEAD")
                )
            })
    }

    /// 硬重置到指定目标
    ///
    /// 便捷方法，等同于 `reset(ResetMode::Hard, target, cwd)`
    ///
    /// # 警告
    ///
    /// 此操作会**永久丢弃**工作区和暂存区的所有未提交更改，请谨慎使用。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::git::commands::GitResetCommand;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // 硬重置到上一个提交
    ///     GitResetCommand::reset_hard(Some("HEAD~1"), None)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn reset_hard(target: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        Self::reset(ResetMode::Hard, target, cwd)
    }

    /// 软重置到指定目标
    ///
    /// 便捷方法，等同于 `reset(ResetMode::Soft, target, cwd)`
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::git::commands::GitResetCommand;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // 软重置到上一个提交（保留更改在暂存区）
    ///     GitResetCommand::reset_soft(Some("HEAD~1"), None)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn reset_soft(target: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        Self::reset(ResetMode::Soft, target, cwd)
    }

    /// 混合重置到指定目标（默认模式）
    ///
    /// 便捷方法，等同于 `reset(ResetMode::Mixed, target, cwd)`
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::git::commands::GitResetCommand;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // 取消暂存（重置索引到 HEAD）
    ///     GitResetCommand::reset_mixed(None, None)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn reset_mixed(target: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        Self::reset(ResetMode::Mixed, target, cwd)
    }
}
