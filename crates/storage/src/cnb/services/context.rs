//! CNB 服务上下文
//!
//! 提供 CNB 服务共用的上下文和辅助方法

use std::sync::Arc;

use domain::{CNBContext, CNBError};

pub trait ServiceContext: Send + Sync {
    fn project_path(&self) -> Result<String, CNBError>;

    /// 解析 PR ID 为 PR number
    fn parse_pr_number(&self, pr_id: &str) -> Result<String, CNBError> {
        // CNB 的 PR number 可能是字符串格式，直接返回
        Ok(pr_id.to_string())
    }
}

/// CNB 服务上下文
///
/// 封装服务共用的依赖和辅助方法
pub struct ServiceContextImpl {
    cnb_context: Arc<dyn CNBContext>,
}

impl ServiceContextImpl {
    /// 创建新的服务上下文
    pub fn new(cnb_context: Arc<dyn CNBContext>) -> Self {
        Self { cnb_context }
    }
}

impl ServiceContext for ServiceContextImpl {
    /// 从 CNBContext 获取项目路径
    fn project_path(&self) -> Result<String, CNBError> {
        self.cnb_context.get_project_path()
    }
}
