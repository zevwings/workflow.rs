//! Codeup 服务上下文

use std::sync::Arc;

use domain::{CodeupError, CodeupSettings};

pub trait ServiceContext: Send + Sync {
    /// 获取项目 ID
    fn get_project_id(&self) -> Result<String, CodeupError>;
    /// 解析 PR ID 为 iid
    fn parse_pr_id(&self, pr_id: &str) -> Result<i64, CodeupError> {
        pr_id.parse::<i64>().map_err(|_| {
            CodeupError::ApiError("无效的 PR ID: 应为数字 ID (例如: '123')".to_string())
        })
    }
}

/// Codeup 服务上下文
pub struct ServiceContextImpl {
    settings: Arc<CodeupSettings>,
}

impl ServiceContextImpl {
    /// 创建新的服务上下文
    pub fn new(settings: Arc<CodeupSettings>) -> Self {
        Self { settings }
    }
}

impl ServiceContext for ServiceContextImpl {
    /// 从配置获取项目 ID
    fn get_project_id(&self) -> Result<String, CodeupError> {
        if self.settings.project_id.is_empty() {
            return Err(CodeupError::ConfigurationIncomplete);
        }
        Ok(self.settings.project_id.clone())
    }
}
