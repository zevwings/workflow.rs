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

/// Git 引用常量
pub mod git_refs {
    /// HEAD 引用
    pub const HEAD: &str = "HEAD";
    /// 当前分支引用
    pub const CURRENT_BRANCH: &str = "HEAD";
}

/// Git 命令选项常量
pub mod git_options {
    /// --porcelain 选项（用于 status 命令）
    pub const PORCELAIN: &str = "--porcelain";
    /// --show-current 选项（用于 branch 命令）
    pub const SHOW_CURRENT: &str = "--show-current";
    /// --force-with-lease 选项（用于 push 命令）
    pub const FORCE_WITH_LEASE: &str = "--force-with-lease";
    /// --no-verify 选项（用于 commit 命令）
    pub const NO_VERIFY: &str = "--no-verify";
    /// --no-ff 选项（用于 merge 命令）
    pub const NO_FF: &str = "--no-ff";
    /// --ff-only 选项（用于 merge 命令）
    pub const FF_ONLY: &str = "--ff-only";
    /// --squash 选项（用于 merge 命令）
    pub const SQUASH: &str = "--squash";
}

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

    /// 默认远程仓库名称
    pub const DEFAULT_REMOTE: &'static str = "origin";

    /// 默认分支名称（按优先级排序）
    pub const DEFAULT_BRANCHES: &[&str] = &["main", "master", "develop", "dev"];

    /// 将 GitError 转换为 color_eyre::eyre::Error
    ///
    /// 这是一个通用的错误转换方法，用于将 `GitError` 转换为 `color_eyre::eyre::Error`。
    pub fn to_eyre_error(e: GitError) -> color_eyre::eyre::Error {
        color_eyre::eyre::eyre!("{}", e)
    }

    /// 处理认证错误
    ///
    /// 如果错误是认证失败，返回专门的认证错误消息；否则返回通用错误。
    pub fn handle_auth_error(e: GitError) -> color_eyre::eyre::Error {
        match e {
            GitError::AuthenticationFailed { reason } => {
                color_eyre::eyre::eyre!("Authentication failed: {}", reason)
            }
            _ => Self::to_eyre_error(e),
        }
    }

    /// 处理合并冲突错误
    ///
    /// 如果错误是合并冲突，返回专门的合并冲突错误消息；否则返回通用错误。
    pub fn handle_merge_error(e: GitError) -> color_eyre::eyre::Error {
        match e {
            GitError::MergeConflict { details } => {
                color_eyre::eyre::eyre!("Merge conflict detected:\n{}", details)
            }
            _ => Self::to_eyre_error(e),
        }
    }

    /// 处理 Stash 冲突错误
    ///
    /// 如果错误是 stash 冲突，返回专门的 stash 冲突错误消息；否则返回通用错误。
    pub fn handle_stash_error(e: GitError) -> color_eyre::eyre::Error {
        match e {
            GitError::StashConflict { details } => {
                color_eyre::eyre::eyre!("Stash apply conflict detected:\n{}", details)
            }
            _ => Self::to_eyre_error(e),
        }
    }

    /// 处理 Cherry-pick 冲突错误
    ///
    /// 如果错误是 cherry-pick 冲突，返回专门的 cherry-pick 冲突错误消息；否则返回通用错误。
    pub fn handle_cherry_pick_error(e: GitError) -> color_eyre::eyre::Error {
        match e {
            GitError::CherryPickConflict => color_eyre::eyre::eyre!(
                "Cherry-pick conflict detected. Please resolve conflicts and continue with 'git cherry-pick --continue'"
            ),
            _ => Self::to_eyre_error(e),
        }
    }

    /// 查找 git 命令的绝对路径
    ///
    /// 尝试多种方法查找 git 命令：
    /// 1. 使用 which/where 命令查找
    /// 2. 尝试常见的 git 路径
    /// 3. 如果都失败，返回 "git"（依赖 PATH）
    ///
    /// # 返回
    ///
    /// 返回 git 命令的路径（可能是绝对路径或 "git"）
    fn find_git_command() -> String {
        // 首先尝试使用 which/where 命令查找
        #[cfg(target_os = "windows")]
        let which_cmd = "where";
        #[cfg(not(target_os = "windows"))]
        let which_cmd = "which";

        if let Ok(output) = std::process::Command::new(which_cmd).arg("git").output() {
            if output.status.success() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    let path = path.trim();
                    if !path.is_empty() {
                        return path.to_string();
                    }
                }
            }
        }

        // 尝试常见的 git 路径
        let common_paths = if cfg!(target_os = "windows") {
            vec![
                "C:\\Program Files\\Git\\cmd\\git.exe",
                "C:\\Program Files (x86)\\Git\\cmd\\git.exe",
            ]
        } else {
            vec![
                "/usr/bin/git",
                "/usr/local/bin/git",
                "/opt/homebrew/bin/git",
            ]
        };

        for path in common_paths {
            if std::path::Path::new(path).exists() {
                return path.to_string();
            }
        }

        // 如果都失败，返回 "git"（依赖 PATH）
        "git".to_string()
    }

    /// 解析 Git 命令输出为行列表
    ///
    /// 将 Git 命令的输出按行分割，去除空白行和前后空格。
    ///
    /// # 参数
    ///
    /// * `output` - Git 命令的输出字符串
    ///
    /// # 返回
    ///
    /// 返回非空行的向量，每行已去除前后空格
    pub fn parse_lines(output: &str) -> Vec<String> {
        output.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    }

    /// 解析 Git 命令输出为键值对列表
    ///
    /// 将 Git 命令的输出按行分割，每行按指定分隔符分割为键值对。
    ///
    /// # 参数
    ///
    /// * `output` - Git 命令的输出字符串
    /// * `separator` - 键值对分隔符（如 `'='`, `'\t'`）
    ///
    /// # 返回
    ///
    /// 返回键值对向量，键和值都已去除前后空格
    pub fn parse_key_value(output: &str, separator: char) -> Vec<(String, String)> {
        output
            .lines()
            .filter_map(|line| {
                line.split_once(separator)
                    .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
            })
            .collect()
    }

    /// 解析 Git 命令输出为行列表（带自定义处理）
    ///
    /// 将 Git 命令的输出按行分割，允许对每行进行自定义处理。
    ///
    /// # 参数
    ///
    /// * `output` - Git 命令的输出字符串
    /// * `mapper` - 对每行进行处理的函数
    ///
    /// # 返回
    ///
    /// 返回处理后的行向量
    pub fn parse_lines_with<F>(output: &str, mapper: F) -> Vec<String>
    where
        F: Fn(&str) -> String,
    {
        output.lines().map(|s| mapper(s.trim())).filter(|s| !s.is_empty()).collect()
    }

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

        // 获取 git 命令的绝对路径，避免依赖 PATH 环境变量
        // 这在并行测试时很重要，因为某些测试可能会修改 PATH
        let git_path = Self::find_git_command();

        // 使用超时机制执行命令
        let output =
            execute_with_timeout(TimeoutConfig::new(timeout), move || -> Result<Output> {
                // 将 Vec<String> 转换为 &[&str]
                let args_refs: Vec<&str> = args_owned.iter().map(|s| s.as_str()).collect();
                let mut command = cmd(&git_path, &args_refs);

                // 设置环境变量以避免 Git 等待终端输入
                // GIT_TERMINAL_PROMPT=0: 禁用终端提示，避免 Git 等待用户输入
                // GIT_PAGER=cat: 使用 cat 作为分页器，避免等待用户交互
                command = command.env("GIT_TERMINAL_PROMPT", "0").env("GIT_PAGER", "cat");

                // 显式传递 GIT_CONFIG 环境变量（如果存在），确保配置隔离正常工作
                // 这对于 GitConfigGuard 等测试基础设施很重要
                if let Ok(git_config) = std::env::var("GIT_CONFIG") {
                    command = command.env("GIT_CONFIG", git_config);
                }

                if let Some(cwd) = cwd_clone.as_ref() {
                    command = command.dir(cwd);
                }

                command
                    .stdout_capture()
                    .stderr_capture()
                    .run()
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to execute command: {}", e))
            })
            .map_err(|e| {
                let error_msg = format!("{}", e);
                // 检查错误类型并格式化错误消息
                let stderr = if error_msg.contains("Operation timed out")
                    || error_msg.contains("timed out after")
                {
                    // 真正的超时错误，使用原始错误消息
                    error_msg
                } else if error_msg.contains("Too many concurrent timeout operations") {
                    // 并发限制错误，使用原始错误消息
                    error_msg
                } else if error_msg.contains("Failed to create timeout thread") {
                    // 线程创建失败错误，使用原始错误消息
                    error_msg
                } else {
                    // 其他错误（如命令执行失败），不提及 timeout，因为这可能不是超时问题
                    format!("Command execution failed: {}", e)
                };
                GitError::CommandFailed {
                    command: command_str.clone(),
                    stderr,
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

        // 获取 git 命令的绝对路径，避免依赖 PATH 环境变量
        // 这在并行测试时很重要，因为某些测试可能会修改 PATH
        let git_path = Self::find_git_command();

        // 使用超时机制执行命令
        let output =
            execute_with_timeout(TimeoutConfig::new(timeout), move || -> Result<Output> {
                // 将 Vec<String> 转换为 &[&str]
                let args_refs: Vec<&str> = args_owned.iter().map(|s| s.as_str()).collect();
                let mut command = cmd(&git_path, &args_refs)
                    .stdout_null()
                    .stderr_capture()
                    .env("GIT_TERMINAL_PROMPT", "0")
                    .env("GIT_PAGER", "cat");

                // 显式传递 GIT_CONFIG 环境变量（如果存在），确保配置隔离正常工作
                if let Ok(git_config) = std::env::var("GIT_CONFIG") {
                    command = command.env("GIT_CONFIG", git_config);
                }

                if let Some(cwd) = cwd_clone.as_ref() {
                    command = command.dir(cwd);
                }

                command
                    .run()
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to execute command: {}", e))
            })
            .map_err(|e| {
                let error_msg = format!("{}", e);
                // 检查错误类型并格式化错误消息
                let stderr = if error_msg.contains("Operation timed out")
                    || error_msg.contains("timed out after")
                {
                    // 真正的超时错误，使用原始错误消息
                    error_msg
                } else if error_msg.contains("Too many concurrent timeout operations") {
                    // 并发限制错误，使用原始错误消息
                    error_msg
                } else if error_msg.contains("Failed to create timeout thread") {
                    // 线程创建失败错误，使用原始错误消息
                    error_msg
                } else {
                    // 其他错误（如命令执行失败），不提及 timeout，因为这可能不是超时问题
                    format!("Command execution failed: {}", e)
                };
                GitError::CommandFailed {
                    command: command_str.clone(),
                    stderr,
                    stdout: String::new(),
                }
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

            // 显式传递 GIT_CONFIG 环境变量（如果存在），确保配置隔离正常工作
            if let Ok(git_config) = std::env::var("GIT_CONFIG") {
                command = command.env("GIT_CONFIG", git_config);
            }

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

/// Git 命令参数构建器
///
/// 提供流畅的 API 来构建 Git 命令参数，简化条件参数添加。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::git::commands::command::GitArgsBuilder;
///
/// let args = GitArgsBuilder::new("push")
///     .flag_if(true, "--force-with-lease")
///     .arg("origin")
///     .arg("main")
///     .build();
/// ```
pub struct GitArgsBuilder {
    args: Vec<String>,
}

impl GitArgsBuilder {
    /// 创建新的参数构建器
    ///
    /// # 参数
    ///
    /// * `command` - Git 命令名称（如 "push", "merge"）
    pub fn new(command: &str) -> Self {
        Self {
            args: vec![command.to_string()],
        }
    }

    /// 添加标志参数（如果条件为真）
    ///
    /// # 参数
    ///
    /// * `condition` - 是否添加该标志
    /// * `flag` - 标志名称（如 "--force-with-lease"）
    pub fn flag_if(mut self, condition: bool, flag: &str) -> Self {
        if condition {
            self.args.push(flag.to_string());
        }
        self
    }

    /// 添加标志参数
    ///
    /// # 参数
    ///
    /// * `flag` - 标志名称（如 "--force-with-lease"）
    pub fn flag(mut self, flag: &str) -> Self {
        self.args.push(flag.to_string());
        self
    }

    /// 添加参数
    ///
    /// # 参数
    ///
    /// * `arg` - 参数值
    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    /// 添加可选参数
    ///
    /// # 参数
    ///
    /// * `arg` - 可选的参数值
    pub fn arg_opt(mut self, arg: Option<&str>) -> Self {
        if let Some(a) = arg {
            self.args.push(a.to_string());
        }
        self
    }

    /// 构建参数数组
    ///
    /// # 返回
    ///
    /// 返回 `Vec<&str>` 格式的参数数组，可直接用于 `GitCommand::run()` 或 `GitCommand::execute()`
    pub fn build(&self) -> Vec<&str> {
        self.args.iter().map(|s| s.as_str()).collect()
    }

    /// 构建参数数组（移动语义）
    ///
    /// # 返回
    ///
    /// 返回 `Vec<String>` 格式的参数数组
    pub fn build_owned(self) -> Vec<String> {
        self.args
    }
}
