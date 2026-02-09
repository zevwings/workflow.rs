//! 分支服务接口
//!
//! 提供基于 LLM 的智能分支名称生成功能。

use thiserror::Error;

/// 分支服务错误类型
///
/// 封装了在分支名称生成过程中可能出现的所有错误。
#[derive(Error, Debug, Clone)]
pub enum BranchServiceError {
    /// LLM API 调用失败
    ///
    /// 当 LLM 服务不可用、网络超时或 API 返回错误时抛出此错误。
    #[error("LLM API 调用失败: {0}")]
    LLMError(String),

    /// 生成分支名失败
    ///
    /// 当 LLM 生成的内容不符合预期格式或无法转换为有效分支名时抛出此错误。
    #[error("生成分支名失败: {0}")]
    GenerateBranchNameFailed(String),

    /// JSON 解析失败
    ///
    /// 当 LLM 返回的 JSON 格式无效或字段缺失时抛出此错误。
    #[error("JSON 解析失败: {0}")]
    JsonParseFailed(String),
}

/// 分支服务接口
///
/// 提供智能分支名称生成功能，通过 LLM 根据标题、描述等信息自动生成符合规范的分支名。
///
/// # 功能特性
///
/// - 自动生成符合 kebab-case 命名规范的分支名
/// - 支持避免与已存在分支重名
/// - 支持根据可选的标题/描述生成语义化分支名
/// - 基于 LLM 的智能推理，理解上下文并生成合适的名称
///
/// # 线程安全
///
/// 实现须满足 [`Send`] + [`Sync`]，以便在多线程或异步上下文中共享。
///
/// # 示例
///
/// ```ignore
/// use domain::BranchService;
///
/// fn example(service: &dyn BranchService) -> Result<(), Box<dyn std::error::Error>> {
///     // 基于标题生成分支名
///     let branch = service.generate_branch_name(
///         Some("Add user authentication"),
///         &[],
///     )?;
///     println!("Generated: {}", branch);
///     // 输出: add-user-authentication
///
///     // 避免重名
///     let existing = ["feature/login".to_string(), "feature/auth".to_string()];
///     let branch = service.generate_branch_name(
///         Some("Add login feature"),
///         &existing,
///     )?;
///     println!("Generated unique: {}", branch);
///
///     Ok(())
/// }
/// ```
pub trait BranchService: Send + Sync {
    /// 使用 LLM 生成符合规范的分支名称
    ///
    /// 根据可选的标题和已存在的分支列表，调用 LLM 生成一个语义清晰、
    /// 符合 kebab-case 命名规范的分支名。
    ///
    /// # 参数
    ///
    /// * `title` - 可选的标题或描述文本，用于生成分支名的语义基础。
    ///   如果为 `None`，LLM 将尝试基于上下文生成通用名称。
    /// * `exists_branches` - 已存在的分支名称切片。如果提供非空切片，
    ///   生成的分支名将避免与列表中的分支重名。
    ///   空切片表示无需检查重名。
    ///
    /// # 返回
    ///
    /// 返回生成的分支名称（kebab-case 格式），例如：
    /// - `"add-user-authentication"`
    /// - `"fix-login-bug"`
    /// - `"feature/implement-oauth"`
    ///
    /// # 错误
    ///
    /// * [`BranchServiceError::LLMError`] - LLM API 调用失败（网络错误、超时、服务不可用等）
    /// * [`BranchServiceError::JsonParseFailed`] - LLM 返回的 JSON 格式无效或字段缺失
    /// * [`BranchServiceError::GenerateBranchNameFailed`] - LLM 返回的内容无法转换为有效分支名
    ///
    /// # 命名规范
    ///
    /// 生成的分支名遵循以下规范：
    /// - 使用 kebab-case（小写字母，单词间用 `-` 连接）
    /// - 仅包含字母、数字、`-` 和 `/`（不包含特殊字符、空格等）
    /// - 长度通常在 3-50 个字符之间
    /// - 语义清晰，能够反映分支的用途
    ///
    /// # 性能考虑
    ///
    /// - 本方法会调用远程 LLM API，可能需要 1-5 秒响应时间
    /// - 建议在异步上下文中调用以避免阻塞
    /// - 考虑缓存生成结果以减少重复调用
    fn generate_branch_name(
        &self,
        title: Option<&str>,
        exists_branches: &[String],
    ) -> Result<String, BranchServiceError>;
}
