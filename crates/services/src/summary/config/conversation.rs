//! 阶段二 2.3：配置/文档分析服务
//!
//! 对配置文件、环境变量、文档类文件的修改进行简要总结。

use client::LLMConversation;

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
    fn get_system_prompt(&self) -> String {
        include_str!("prompt.md").to_string()
    }

    fn get_user_prompt(&self) -> String {
        self.user_prompt.clone()
    }

    fn get_max_tokens(&self) -> Option<u32> {
        // Config/doc analysis processes fewer files
        Some(6000)
    }

    fn get_temperature(&self) -> f32 {
        0.3
    }
}
