//! Git 命令统一执行层
//!
//! 提供统一的 Git 命令执行接口，包括：
//! - 命令执行
//! - 错误处理
//! - 输出解析

use color_eyre::Result;
use duct::cmd;
use std::fmt;
use std::path::Path;
use std::process::Output;

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
    /// 如果命令执行失败，返回 `GitError`
    pub fn run(args: &[&str], cwd: Option<&Path>) -> Result<String, GitError> {
        let mut command = cmd("git", args);

        if let Some(cwd) = cwd {
            command = command.dir(cwd);
        }

        let output = command.stdout_capture().stderr_capture().run().map_err(|e| {
            GitError::CommandFailed {
                command: format!("git {}", args.join(" ")),
                stderr: format!("Failed to execute command: {}", e),
                stdout: String::new(),
            }
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
        let mut command = cmd("git", args).stdout_null().stderr_capture();

        if let Some(cwd) = cwd {
            command = command.dir(cwd);
        }

        let output = command.run().map_err(|e| GitError::CommandFailed {
            command: format!("git {}", args.join(" ")),
            stderr: format!("Failed to execute command: {}", e),
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
    /// 返回 `true` 如果命令成功，`false` 如果失败
    pub fn check(args: &[&str], cwd: Option<&Path>) -> bool {
        let mut command = cmd("git", args).stdout_null().stderr_null();

        if let Some(cwd) = cwd {
            command = command.dir(cwd);
        }

        command.run().map(|output| output.status.success()).unwrap_or(false)
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
