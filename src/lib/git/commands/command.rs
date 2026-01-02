//! Git 命令统一执行层
//!
//! 提供统一的 Git 命令执行接口，包括：
//! - 命令执行
//! - 错误处理
//! - 输出解析

use crate::base::resilience::{execute_with_timeout, TimeoutConfig};
use color_eyre::Result;
use duct::cmd;
use std::fmt;
use std::path::Path;
use std::process::Output;
use std::time::Duration;

/// Git 命令执行错误
#[derive(Debug, Clone)]
pub enum GitError {
    /// Git 命令执行失败
    CommandFailed {
        command: String,
        stderr: String,
        stdout: String,
    },
    /// 不在 Git 仓库中
    NotGitRepo,
    /// 分支不存在
    BranchNotFound { branch: String },
    /// 分支已存在
    BranchAlreadyExists { branch: String },
    /// 提交不存在
    CommitNotFound { commit: String },
    /// 合并冲突
    MergeConflict { details: String },
    /// Cherry-pick 冲突
    CherryPickConflict,
    /// Stash 冲突
    StashConflict { details: String },
    /// 认证失败
    AuthenticationFailed { reason: String },
    /// 输出解析失败
    ParseError { reason: String },
    /// 其他错误
    Other { message: String },
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::CommandFailed {
                command,
                stderr,
                stdout,
            } => {
                write!(
                    f,
                    "Git command failed: {}\nStderr: {}\nStdout: {}",
                    command, stderr, stdout
                )
            }
            GitError::NotGitRepo => write!(f, "Not in a Git repository"),
            GitError::BranchNotFound { branch } => {
                write!(f, "Branch '{}' does not exist", branch)
            }
            GitError::BranchAlreadyExists { branch } => {
                write!(f, "Branch '{}' already exists", branch)
            }
            GitError::CommitNotFound { commit } => {
                write!(f, "Commit '{}' does not exist", commit)
            }
            GitError::MergeConflict { details } => {
                write!(f, "Merge conflict detected:\n{}", details)
            }
            GitError::CherryPickConflict => write!(
                f,
                "Cherry-pick conflict detected. Please resolve conflicts and continue with 'git cherry-pick --continue'"
            ),
            GitError::StashConflict { details } => {
                write!(f, "Stash apply conflict detected:\n{}", details)
            }
            GitError::AuthenticationFailed { reason } => {
                write!(f, "Authentication failed: {}", reason)
            }
            GitError::ParseError { reason } => {
                write!(f, "Failed to parse git output: {}", reason)
            }
            GitError::Other { message } => write!(f, "Git operation failed: {}", message),
        }
    }
}

impl std::error::Error for GitError {}

/// Git 命令执行器
///
/// 提供统一的 Git 命令执行接口，所有 Git 操作都通过此接口执行。
pub struct GitCommand;

impl GitCommand {
    /// Git 命令默认超时时间
    ///
    /// - Windows 平台：120秒（因为 Windows 上 Git 操作可能较慢）
    /// - 其他平台：60秒
    ///
    /// 大多数 Git 命令应该在几秒内完成，但对于可能较慢的操作（如网络操作、配置读取），
    /// 使用平台相关的超时时间可以避免不必要的超时错误。
    #[cfg(target_os = "windows")]
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);

    #[cfg(not(target_os = "windows"))]
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

    /// 执行 Git 命令并返回标准输出
    ///
    /// # 参数
    ///
    /// * `args` - Git 命令参数（不包含 "git" 本身）
    /// * `cwd` - 工作目录（可选，默认为当前目录）
    ///
    /// # 返回
    ///
    /// 返回命令的标准输出（UTF-8 字符串）
    ///
    /// # 错误
    ///
    /// 如果命令执行失败或超时，返回 `GitError`
    pub fn run(args: &[&str], cwd: Option<&Path>) -> Result<String, GitError> {
        Self::run_with_timeout(args, cwd, Self::DEFAULT_TIMEOUT)
    }

    /// 执行 Git 命令并返回标准输出（带超时）
    ///
    /// # 参数
    ///
    /// * `args` - Git 命令参数（不包含 "git" 本身）
    /// * `cwd` - 工作目录（可选，默认为当前目录）
    /// * `timeout` - 超时时间
    ///
    /// # 返回
    ///
    /// 返回命令的标准输出（UTF-8 字符串）
    ///
    /// # 错误
    ///
    /// 如果命令执行失败或超时，返回 `GitError`
    pub fn run_with_timeout(
        args: &[&str],
        cwd: Option<&Path>,
        timeout: Duration,
    ) -> Result<String, GitError> {
        let command_str = format!("git {}", args.join(" "));
        let cwd_clone = cwd.map(|p| p.to_path_buf());
        // 将 args 转换为拥有所有权的 Vec<String>
        let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();

        // 使用超时机制执行命令
        let output =
            execute_with_timeout(TimeoutConfig::new(timeout), move || -> Result<Output> {
                // 将 Vec<String> 转换为 &[&str]
                let args_refs: Vec<&str> = args_owned.iter().map(|s| s.as_str()).collect();
                let mut command = cmd("git", &args_refs);

                // 设置环境变量以避免 Git 等待终端输入
                // GIT_TERMINAL_PROMPT=0: 禁用终端提示，避免 Git 等待用户输入
                // GIT_PAGER=cat: 使用 cat 作为分页器，避免等待用户交互
                command = command.env("GIT_TERMINAL_PROMPT", "0").env("GIT_PAGER", "cat");

                if let Some(cwd) = cwd_clone.as_ref() {
                    command = command.dir(cwd);
                }

                command
                    .stdout_capture()
                    .stderr_capture()
                    .run()
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to execute command: {}", e))
            })
            .map_err(|e| GitError::CommandFailed {
                command: command_str.clone(),
                stderr: format!("Command timed out after {:?}: {}", timeout, e),
                stdout: String::new(),
            })?;

        if !output.status.success() {
            return Err(Self::handle_error(args, &output));
        }

        String::from_utf8(output.stdout).map_err(|e| GitError::ParseError {
            reason: format!("Failed to parse output as UTF-8: {}", e),
        })
    }

    /// 执行 Git 命令（静默模式，不返回输出）
    ///
    /// # 参数
    ///
    /// * `args` - Git 命令参数
    /// * `cwd` - 工作目录（可选）
    ///
    /// # 返回
    ///
    /// 成功返回 `Ok(())`，失败返回 `GitError`
    pub fn execute(args: &[&str], cwd: Option<&Path>) -> Result<(), GitError> {
        Self::execute_with_timeout(args, cwd, Self::DEFAULT_TIMEOUT)
    }

    /// 执行 Git 命令（静默模式，不返回输出，带超时）
    ///
    /// # 参数
    ///
    /// * `args` - Git 命令参数
    /// * `cwd` - 工作目录（可选）
    /// * `timeout` - 超时时间
    ///
    /// # 返回
    ///
    /// 成功返回 `Ok(())`，失败返回 `GitError`
    pub fn execute_with_timeout(
        args: &[&str],
        cwd: Option<&Path>,
        timeout: Duration,
    ) -> Result<(), GitError> {
        let command_str = format!("git {}", args.join(" "));
        let cwd_clone = cwd.map(|p| p.to_path_buf());
        // 将 args 转换为拥有所有权的 Vec<String>
        let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();

        // 使用超时机制执行命令
        let output =
            execute_with_timeout(TimeoutConfig::new(timeout), move || -> Result<Output> {
                // 将 Vec<String> 转换为 &[&str]
                let args_refs: Vec<&str> = args_owned.iter().map(|s| s.as_str()).collect();
                let mut command = cmd("git", &args_refs)
                    .stdout_null()
                    .stderr_capture()
                    .env("GIT_TERMINAL_PROMPT", "0")
                    .env("GIT_PAGER", "cat");

                if let Some(cwd) = cwd_clone.as_ref() {
                    command = command.dir(cwd);
                }

                command
                    .run()
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to execute command: {}", e))
            })
            .map_err(|e| GitError::CommandFailed {
                command: command_str.clone(),
                stderr: format!("Command timed out after {:?}: {}", timeout, e),
                stdout: String::new(),
            })?;

        if !output.status.success() {
            return Err(Self::handle_error(args, &output));
        }

        Ok(())
    }

    /// 检查 Git 命令是否成功（静默检查）
    ///
    /// # 参数
    ///
    /// * `args` - Git 命令参数
    /// * `cwd` - 工作目录（可选）
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果命令成功，`false` 如果失败或超时
    pub fn check(args: &[&str], cwd: Option<&Path>) -> bool {
        Self::check_with_timeout(args, cwd, Self::DEFAULT_TIMEOUT)
    }

    /// 检查 Git 命令是否成功（静默检查，带超时）
    ///
    /// # 参数
    ///
    /// * `args` - Git 命令参数
    /// * `cwd` - 工作目录（可选）
    /// * `timeout` - 超时时间
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果命令成功，`false` 如果失败或超时
    pub fn check_with_timeout(args: &[&str], cwd: Option<&Path>, timeout: Duration) -> bool {
        let cwd_clone = cwd.map(|p| p.to_path_buf());
        // 将 args 转换为拥有所有权的 Vec<String>
        let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();

        execute_with_timeout(TimeoutConfig::new(timeout), move || -> Result<bool> {
            // 将 Vec<String> 转换为 &[&str]
            let args_refs: Vec<&str> = args_owned.iter().map(|s| s.as_str()).collect();
            let mut command = cmd("git", &args_refs)
                .stdout_null()
                .stderr_null()
                .env("GIT_TERMINAL_PROMPT", "0")
                .env("GIT_PAGER", "cat");

            if let Some(cwd) = cwd_clone.as_ref() {
                command = command.dir(cwd);
            }

            Ok(command.run().map(|output| output.status.success()).unwrap_or(false))
        })
        .unwrap_or(false)
    }

    /// 处理错误
    ///
    /// 根据命令输出和错误信息，转换为相应的 `GitError`
    fn handle_error(args: &[&str], output: &Output) -> GitError {
        let command = format!("git {}", args.join(" "));
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        let stderr_str = stderr.to_string();
        let stdout_str = stdout.to_string();

        // 根据错误信息分类处理
        if stderr_str.contains("not a git repository")
            || stderr_str.contains("Not a git repository")
        {
            return GitError::NotGitRepo;
        }

        if stderr_str.contains("fatal: A branch named") && stderr_str.contains("already exists") {
            // 提取分支名
            let branch = args.last().unwrap_or(&"unknown");
            return GitError::BranchAlreadyExists {
                branch: branch.to_string(),
            };
        }

        if stderr_str.contains("fatal: branch") && stderr_str.contains("not found") {
            // 提取分支名
            let branch = args.last().unwrap_or(&"unknown");
            return GitError::BranchNotFound {
                branch: branch.to_string(),
            };
        }

        if stderr_str.contains("merge conflict") || stderr_str.contains("CONFLICT") {
            return GitError::MergeConflict {
                details: stderr_str,
            };
        }

        if stderr_str.contains("cherry-pick")
            && (stderr_str.contains("conflict") || stderr_str.contains("CONFLICT"))
        {
            return GitError::CherryPickConflict;
        }

        if stderr_str.contains("stash") && stderr_str.contains("conflict") {
            return GitError::StashConflict {
                details: stderr_str,
            };
        }

        if stderr_str.contains("authentication") || stderr_str.contains("Permission denied") {
            return GitError::AuthenticationFailed { reason: stderr_str };
        }

        // 默认错误
        GitError::CommandFailed {
            command,
            stderr: stderr_str,
            stdout: stdout_str,
        }
    }
}
