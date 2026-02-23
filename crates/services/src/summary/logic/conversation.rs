//! 阶段二 2.2：核心逻辑分析服务
//!
//! 对业务代码、服务层等核心文件的完整 diff 进行深入分析。

use client::LLMConversation;

/// 核心逻辑分析对话
pub(crate) struct LogicAnalyzeConversation {
    user_prompt: String,
}

impl LogicAnalyzeConversation {
    pub fn new(user_prompt: impl Into<String>) -> Self {
        Self {
            user_prompt: user_prompt.into(),
        }
    }
}

impl LLMConversation for LogicAnalyzeConversation {
    fn get_system_prompt(&self) -> String {
        include_str!("prompt.md").to_string()
    }

    fn get_user_prompt(&self) -> String {
        self.user_prompt.clone()
    }

    fn get_max_tokens(&self) -> Option<u32> {
        // Logic analysis uses sampling but may process many files
        None
    }

    fn get_temperature(&self) -> f32 {
        0.3
    }
}
