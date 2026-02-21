//! 工作流阶段注册表
//!
//! 提供 Stage 的集中注册与按名称查找，固定顺序为 Jira → GitHub → LLM → Log。

use crate::interactive::platforms::{github_stage, jira_stage, llm_stage, log_stage};
use crate::interactive::WorkflowStage;

// ============================================================================
// Stage 名称常量
// ============================================================================

/// Jira 阶段名称
pub const JIRA_STAGE_NAME: &str = "Jira";

/// GitHub 阶段名称
pub const GITHUB_STAGE_NAME: &str = "GitHub";

/// LLM 阶段名称
pub const LLM_STAGE_NAME: &str = "LLM";

/// Log 阶段名称
pub const LOG_STAGE_NAME: &str = "Log";

// ============================================================================
// Registry 接口
// ============================================================================

/// 工作流阶段注册表
///
/// 提供按固定顺序获取所有 stage，以及按名称查找单个 stage 的能力。
pub trait WorkflowStageManager: Send + Sync {
    /// 返回所有 stage，按固定顺序：Jira → GitHub → LLM → Log
    fn stages(&self) -> Vec<&'static dyn WorkflowStage>;

    /// 按名称查找 stage
    ///
    /// # 参数
    ///
    /// * `name` - 阶段名称，如 "Jira"、"GitHub"
    ///
    /// # 返回
    ///
    /// 找到则返回 `Some(&dyn WorkflowStage)`，否则返回 `None`
    fn stage_by_name(&self, name: &str) -> Option<&'static dyn WorkflowStage> {
        self.stages().into_iter().find(|s| s.stage_name() == name)
    }
}

// ============================================================================
// 默认实现
// ============================================================================

/// 默认的 WorkflowStageRegistry 实现
///
/// Stage 顺序固定为：Jira → GitHub → LLM → Log
pub struct WorkflowStageManagerImpl;

impl WorkflowStageManager for WorkflowStageManagerImpl {
    fn stages(&self) -> Vec<&'static dyn WorkflowStage> {
        vec![jira_stage(), github_stage(), llm_stage(), log_stage()]
    }
}
