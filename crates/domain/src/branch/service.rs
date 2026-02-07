//! 分支服务接口

use thiserror::Error;

/// LLM 服务错误
#[derive(Error, Debug, Clone)]
pub enum BranchServiceError {
    #[error("LLM API 调用失败: {0}")]
    LLMError(String),

    #[error("生成分支名失败: {0}")]
    GenerateBranchNameFailed(String),

    #[error("JSON 解析失败: {0}")]
    JsonParseFailed(String),
}

/// 分支服务接口
pub trait BranchService: Send + Sync {
    /// 创建分支
    fn generate_branch_name(
        &self,
        title: Option<&str>,
        exists_branches: Option<Vec<String>>,
    ) -> Result<String, BranchServiceError>;
}
