//! 原生脚本 Hook 执行器
//!
//! 负责执行 Git hooks 脚本，包括环境变量设置、标准输入处理和输出捕获。

use super::context::{HookContext, HookResult};
use domain::GitError;
use std::process::{Command, Stdio};
use std::time::Duration;
use toolkit::log_info;
use wait_timeout::ChildExt;

/// 默认超时时间（30 秒）
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// 原生脚本 Hook 执行器
///
/// 负责执行 Git hooks 脚本，设置正确的环境变量，处理标准输入输出。
pub struct ScriptHookExecutor {
    /// 超时时间
    timeout: Duration,
}

impl Default for ScriptHookExecutor {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

impl ScriptHookExecutor {
    /// 执行 hook 脚本
    ///
    /// # 参数
    /// - `hook_path`: Hook 脚本路径
    /// - `context`: Hook 上下文信息
    ///
    /// # 返回
    /// - `Ok(HookResult)`: Hook 执行结果
    /// - `Err(GitError)`: Hook 执行失败
    pub fn execute(
        &self,
        hook_path: &std::path::Path,
        context: &HookContext,
    ) -> Result<HookResult, GitError> {
        // 准备环境变量
        let env_vars = self.prepare_env_vars(context);

        // 准备标准输入（某些 hook 需要）
        let stdin_content = self.prepare_stdin(context, hook_path)?;

        // 执行脚本
        let mut cmd = Command::new(hook_path);
        cmd.current_dir(&context.repo_path)
            .envs(env_vars)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // 设置超时（使用 spawn + wait_timeout）
        let mut child = cmd
            .spawn()
            .map_err(|e| GitError::OperationFailed(format!("Failed to execute hook: {}", e)))?;

        // 写入标准输入
        if let Some(ref stdin) = stdin_content {
            if let Some(mut stdin_handle) = child.stdin.take() {
                use std::io::Write;
                stdin_handle.write_all(stdin.as_bytes()).map_err(|e| {
                    GitError::OperationFailed(format!("Failed to write stdin: {}", e))
                })?;
                // 关闭 stdin，让子进程知道输入结束
                drop(stdin_handle);
            }
        } else {
            // 如果没有 stdin 内容，关闭 stdin
            drop(child.stdin.take());
        }

        // 等待执行完成（带超时）
        let output = self.wait_with_timeout(&mut child)?;

        // 处理输出
        self.handle_output(&output)?;

        // 检查结果
        if output.status.success() {
            Ok(HookResult::Success)
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(GitError::HookFailed(format!(
                "Hook {} failed: {}",
                hook_path.display(),
                error_msg
            )))
        }
    }

    /// 准备环境变量
    ///
    /// 根据 Git hooks 规范设置必要的环境变量。
    fn prepare_env_vars(&self, context: &HookContext) -> Vec<(String, String)> {
        let mut env_vars = Vec::new();

        env_vars.push((
            "GIT_DIR".to_string(),
            context.git_dir.to_string_lossy().to_string(),
        ));
        env_vars.push((
            "GIT_WORK_TREE".to_string(),
            context.repo_path.to_string_lossy().to_string(),
        ));

        // 为 pre-commit 传递暂存区文件列表
        if !context.staged_files.is_empty() {
            env_vars.push((
                "GIT_STAGED_FILES".to_string(),
                context.staged_files.join("\n"),
            ));
        }

        // 为 commit-msg 传递提交消息
        if let Some(ref msg) = context.commit_message {
            env_vars.push(("GIT_COMMIT_MSG".to_string(), msg.clone()));
        }

        // 为 pre-push 传递分支和提交信息
        if let Some(ref branch) = context.branch_name {
            env_vars.push(("GIT_BRANCH".to_string(), branch.clone()));
        }

        if !context.commits_to_push.is_empty() {
            env_vars.push((
                "GIT_COMMITS_TO_PUSH".to_string(),
                context.commits_to_push.join("\n"),
            ));
        }

        env_vars
    }

    /// 准备标准输入内容
    ///
    /// 某些 hooks（如 commit-msg, prepare-commit-msg）需要从标准输入读取数据。
    fn prepare_stdin(
        &self,
        context: &HookContext,
        hook_path: &std::path::Path,
    ) -> Result<Option<String>, GitError> {
        let hook_name = hook_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // commit-msg hook 需要传递提交消息
        if hook_name == "commit-msg" {
            if let Some(ref msg) = context.commit_message {
                return Ok(Some(msg.clone()));
            }
        }

        // prepare-commit-msg hook 需要传递提交消息
        if hook_name == "prepare-commit-msg" {
            if let Some(ref msg) = context.commit_message {
                return Ok(Some(msg.clone()));
            }
        }

        Ok(None)
    }

    /// 等待进程完成（带超时）
    ///
    /// 使用 wait-timeout crate 实现跨平台超时控制。
    fn wait_with_timeout(
        &self,
        child: &mut std::process::Child,
    ) -> Result<std::process::Output, GitError> {
        use std::io::Read;

        // 使用 wait-timeout 等待进程完成
        let status = child
            .wait_timeout(self.timeout)
            .map_err(|e| GitError::OperationFailed(format!("Failed to wait for hook: {}", e)))?;

        match status {
            Some(exit_status) => {
                // 进程在超时前结束，收集输出
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

                Ok(std::process::Output {
                    status: exit_status,
                    stdout,
                    stderr,
                })
            }
            None => {
                // 超时，终止进程
                let _ = child.kill();
                let _ = child.wait();
                Err(GitError::OperationFailed(format!(
                    "Hook execution timed out after {:?}",
                    self.timeout
                )))
            }
        }
    }

    /// 处理 hook 输出
    ///
    /// 将 hook 的输出打印到日志。
    fn handle_output(&self, output: &std::process::Output) -> Result<(), GitError> {
        // 输出到日志
        if !output.stdout.is_empty() {
            log_info!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            log_info!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }
}
