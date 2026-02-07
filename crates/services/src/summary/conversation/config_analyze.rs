//! 阶段二 2.3：配置/文档分析对话
//!
//! 对配置文件、环境变量、文档类文件的修改进行简要总结。

use llm::LLMConversation;
use crate::summary::prompt::analyze_config;

/// 配置/文档分析对话
///
/// 输入为已构建的 user prompt（包含配置或文档文件的 diff 列表），输出为配置变更与文档更新摘要。
pub(crate) struct ConfigAnalyzeConversation {
    user_prompt: String,
}

impl ConfigAnalyzeConversation {
    pub fn new(user_prompt: impl Into<String>) -> Self {
        Self {
            user_prompt: user_prompt.into(),
        }
    }
}

impl LLMConversation for ConfigAnalyzeConversation {
    fn get_system_prompt(&self, _language_code: &str) -> String {
        analyze_config().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        self.user_prompt.clone()
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.3)
    }
}
