use domain::git::GitError;

use super::{
    pre_commit_hooks, HookContext, HookDiscoverer, HookResult, HookTool, HookToolDetector,
    ScriptHookExecutor, ToolHookExecutor,
};
use crate::git::services::context::GitContext;

/// Hook 服务接口
///
/// 定义 Hook 执行的核心接口。
pub trait HookService: Send + Sync {
    /// 执行指定名称的 hook
    ///
    /// # 参数
    /// - `hook_name`: Hook 名称（如 "pre-commit"）
    /// - `context`: Hook 上下文信息
    /// - `skip_if_missing`: 如果 hook 不存在，是否跳过（默认 true）
    ///
    /// # 返回
    /// - `Ok(HookResult)`: Hook 执行结果
    /// - `Err(GitError)`: Hook 执行失败
    fn execute_hook(&self, hook_name: &str, context: &HookContext) -> Result<HookResult, GitError>;
}

/// Hook 服务实现
///
/// 实现 HookService trait，提供完整的 Hook 执行功能。
/// 支持 prek/pre-commit 工具和标准 Git hooks。
pub struct HookServiceImpl {
    discoverer: HookDiscoverer,
    executor: ScriptHookExecutor,
    tool_detector: HookToolDetector,
}

impl HookServiceImpl {
    /// 创建新的 Hook 服务实例
    ///
    /// # 参数
    /// - `ctx`: Git 上下文
    pub fn new(ctx: GitContext) -> Self {
        let repo = ctx.repository();
        let git_dir = repo.path().to_path_buf();
        let repo_path = repo.workdir().map(|p| p.to_path_buf()).unwrap_or_else(|| git_dir.clone());
        drop(repo);

        let discoverer = HookDiscoverer::new(git_dir, repo_path.clone());
        let executor = ScriptHookExecutor::default();
        let tool_detector = HookToolDetector::new(repo_path);

        Self {
            discoverer,
            executor,
            tool_detector,
        }
    }
}

impl HookService for HookServiceImpl {
    fn execute_hook(&self, hook_name: &str, context: &HookContext) -> Result<HookResult, GitError> {
        // 检测工具（按优先级：Prek > PreCommit > Standard）
        let tool_results = self.tool_detector.detect_tools(hook_name);

        // 按优先级执行 hooks
        for tool_result in tool_results {
            match tool_result.tool {
                HookTool::Prek | HookTool::PreCommit => {
                    // 检查 pre-commit 工具是否支持该 hook
                    if !pre_commit_hooks::is_supported(hook_name) {
                        continue;
                    }

                    // 执行 prek/pre-commit
                    if let (Some(config_path), Some(executable_path)) =
                        (tool_result.config_path, tool_result.executable_path)
                    {
                        let is_prek = tool_result.tool == HookTool::Prek;
                        let tool_executor =
                            ToolHookExecutor::new(executable_path, config_path, is_prek);

                        match tool_executor.execute(hook_name, context) {
                            Ok(result) => {
                                // prek/pre-commit 成功，直接返回
                                // 不再执行标准 hooks（因为 .git/hooks/pre-commit 可能是 prek 安装的，会重复执行）
                                return Ok(result);
                            }
                            Err(e) => {
                                // prek/pre-commit 失败，阻止操作
                                return Err(GitError::HookFailed(format!(
                                    "prek/pre-commit hook failed: {}",
                                    e
                                )));
                            }
                        }
                    }
                }

                HookTool::Standard => {
                    // 执行标准 Git hooks
                    // HookDiscoverer 会自动检查 core.hooksPath 配置
                    // 如果 core.hooksPath 已设置，会使用该路径；否则使用 .git/hooks/
                    match self.discoverer.find_hook(hook_name)? {
                        Some(hook_path) => match self.executor.execute(&hook_path, context) {
                            Ok(result) => {
                                return Ok(result);
                            }
                            Err(e) => {
                                return Err(GitError::HookFailed(format!(
                                    "standard git hook failed: {}",
                                    e
                                )));
                            }
                        },
                        None => {
                            return Ok(HookResult::Success);
                        }
                    }
                }
            }
        }

        Ok(HookResult::Success)
    }
}
