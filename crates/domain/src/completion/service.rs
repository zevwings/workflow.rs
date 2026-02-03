//! Shell Completion 服务接口
//!
//! 提供 Shell Completion 的生成、检查和移除功能。

use crate::completion::entity::{
    CompletionCheckResult, CompletionGenerateResult, CompletionRemoveResult,
};
use crate::errors::ServiceError;

/// Shell Completion 服务接口
///
/// 负责 Shell Completion 的配置管理，包括：
/// - 保存 completion 脚本并配置 shell
/// - 检查各个 shell 的 completion 状态
/// - 移除 completion 配置和脚本文件
pub trait CompletionService: Send + Sync {
    /// 保存 completion 脚本并配置 shell
    ///
    /// # 参数
    /// - `shell`: shell 类型字符串（如 "zsh", "bash"）
    /// - `script_content`: 脚本内容（字节数组）
    /// - `output_dir`: 输出目录，None 表示使用默认目录
    ///
    /// # 返回
    /// 生成结果，包含脚本路径、配置状态等信息
    fn save_and_configure(
        &self,
        shell: &str,
        script_content: &[u8],
        output_dir: Option<&str>,
    ) -> Result<CompletionGenerateResult, ServiceError>;

    /// 检查 completion 配置状态
    ///
    /// 检查各个 shell 的 completion 配置状态，包括：
    /// - 配置文件是否已添加 source
    /// - completion 脚本文件是否存在
    ///
    /// # 返回
    /// 检查结果，包含各个 shell 的状态信息
    fn check_status(&self) -> Result<CompletionCheckResult, ServiceError>;

    /// 移除 completion 配置
    ///
    /// # 参数
    /// - `remove_all`: true 表示移除所有 shell 的配置，false 表示只移除当前 shell
    ///
    /// # 返回
    /// 移除结果，包含移除的配置和文件列表
    fn remove(&self, remove_all: bool) -> Result<CompletionRemoveResult, ServiceError>;
}
