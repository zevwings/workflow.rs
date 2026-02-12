//! 阶段二 2.3：配置/文档分析服务
//!
//! 对配置文件、环境变量、文档类文件的修改进行简要总结。

use llm::LLMConversation;

use crate::summary::prompt;

/// 配置/文档分析对话
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
        prompt::analyze_config().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        self.user_prompt.clone()
    }

    fn get_max_tokens(&self) -> Option<u32> {
        None
    }

    fn get_temperature(&self) -> f32 {
        0.3
    }
}
