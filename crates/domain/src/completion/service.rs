//! 完成服务接口

use crate::errors::ServiceError;

/// 完成服务接口
pub trait CompletionService: Send + Sync {
    /// 生成 completion 脚本
    fn generate_completion(&self, shell: &str) -> Result<(), ServiceError>;

    /// 检查 completion 配置
    fn check_completion(&self) -> Result<bool, ServiceError>;

    /// 删除 completion 配置
    fn remove_completion(&self) -> Result<(), ServiceError>;
}
