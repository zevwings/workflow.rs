//! 外部工具 Hook 执行器
//!
//! 负责执行 pre-commit/prek 等外部工具管理的 hooks。

use super::context::{HookContext, HookResult};
use domain::git::GitError;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

/// 默认超时时间（5 分钟，外部工具可能需要更长时间）
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

/// 外部工具 Hook 执行器
///
/// 负责调用 pre-commit 或 prek 等外部工具来执行 hooks。
pub struct ToolHookExecutor {
    tool_path: PathBuf,
    /// 配置文件路径（保留以便将来使用）
    #[allow(dead_code)]
    config_path: PathBuf,
    is_prek: bool,
    /// 超时时间
    timeout: Duration,
}

impl Default for ToolHookExecutor {
    fn default() -> Self {
        Self {
            tool_path: PathBuf::new(),
            config_path: PathBuf::new(),
            is_prek: false,
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

impl ToolHookExecutor {
    /// 创建新的外部工具 Hook 执行器
    ///
    /// # 参数
    /// - `tool_path`: 工具可执行文件路径
    /// - `config_path`: 配置文件路径（.pre-commit-config.yaml）
    /// - `is_prek`: 是否为 prek（true）或 pre-commit（false）
    pub fn new(tool_path: PathBuf, config_path: PathBuf, is_prek: bool) -> Self {
        Self {
            tool_path,
            config_path,
            is_prek,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// 执行外部工具管理的 hook
    ///
    /// # 参数
    /// - `hook_name`: Hook 名称（如 "pre-commit"）
    /// - `context`: Hook 上下文信息
    ///
    /// # 返回
    /// - `Ok(HookResult)`: Hook 执行结果
    /// - `Err(GitError)`: Hook 执行失败
    pub fn execute(
        &self,
        hook_name: impl AsRef<str>,
        context: &HookContext,
    ) -> Result<HookResult, GitError> {
        let hook_name = hook_name.as_ref();
        use std::io::Read;

        let tool_name = if self.is_prek { "prek" } else { "pre-commit" };

        let mut cmd = Command::new(&self.tool_path);

        // pre-commit/prek 的命令行参数（两者格式不同）
        if self.is_prek {
            // prek 使用: prek run --stage <STAGE> --commit-msg-filename <FILE>
            let commit_msg_file = context.git_dir.join("COMMIT_EDITMSG");
            cmd.arg("run")
                .arg("--stage")
                .arg(hook_name)
                .arg("--commit-msg-filename")
                .arg(&commit_msg_file)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
        } else {
            // pre-commit 使用: pre-commit run [HOOK_ID] --all-files
            cmd.arg("run")
                .arg(hook_name)
                .arg("--all-files")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
        }

        // 设置工作目录
        cmd.current_dir(&context.repo_path);

        // 设置环境变量
        cmd.env("GIT_DIR", &context.git_dir);
        cmd.env("GIT_WORK_TREE", &context.repo_path);

        // 对于 pre-commit hook，传递暂存区文件列表
        if hook_name == "pre-commit" && !context.staged_files.is_empty() {
            // pre-commit 会自动检测暂存区文件，但我们可以通过环境变量传递
            cmd.env("PRE_COMMIT_ALL_FILES", "1");
        }

        // 启动进程
        let mut child = cmd.spawn().map_err(|e| {
            GitError::OperationFailed(format!("Failed to execute {}: {}", tool_name, e))
        })?;

        // 等待执行完成（带超时）
        let status = child.wait_timeout(self.timeout).map_err(|e| {
            GitError::OperationFailed(format!("Failed to wait for {}: {}", tool_name, e))
        })?;

        match status {
            Some(exit_status) => {
                // 收集输出
                let stdout = child
                    .stdout
                    .take()
                    .and_then(|mut s| {
                        let mut buf = Vec::new();
                        s.read_to_end(&mut buf).ok()?;
                        Some(buf)
                    })
                    .unwrap_or_default();

                let stderr = child
                    .stderr
                    .take()
                    .and_then(|mut s| {
                        let mut buf = Vec::new();
                        s.read_to_end(&mut buf).ok()?;
                        Some(buf)
                    })
                    .unwrap_or_default();

                let stdout_str = String::from_utf8_lossy(&stdout);
                let stderr_str = String::from_utf8_lossy(&stderr);

                // 处理输出
                if !stdout.is_empty() {
                    toolkit::log_info!("{}", stdout_str);
                }
                if !stderr.is_empty() {
                    toolkit::log_info!("{}", stderr_str);
                }

                // 检查结果
                if exit_status.success() {
                    Ok(HookResult::Success)
                } else {
                    // 检测 "files were modified by this hook" 的情况
                    // 这是 pre-commit/prek 的正常行为，不是真正的失败
                    if stdout_str.contains("files were modified by this hook")
                        || stdout_str.contains("files were modified")
                    {
                        return Ok(HookResult::Modified);
                    }

                    // 真正的失败，优先使用 stderr，如果为空则使用 stdout
                    let error_msg = if stderr_str.trim().is_empty() {
                        stdout_str.to_string()
                    } else {
                        stderr_str.to_string()
                    };

                    Err(GitError::HookFailed(format!(
                        "{} hook failed: {}",
                        tool_name, error_msg
                    )))
                }
            }
            None => {
                // 超时，终止进程
                let _ = child.kill();
                let _ = child.wait();
                Err(GitError::OperationFailed(format!(
                    "{} execution timed out after {:?}",
                    tool_name, self.timeout
                )))
            }
        }
    }
}
