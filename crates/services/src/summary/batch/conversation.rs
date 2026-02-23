//! 阶段二 2.1：批量操作分析服务
//!
//! 当阶段一检测到批量重命名、格式化、配置更新等模式时，对样本 diff 进行分析。

use client::LLMConversation;

// ── Conversation ──────────────────────────────────────────────

/// 批量操作分析对话
pub(crate) struct BatchAnalyzeConversation {
    user_prompt: String,
}

impl BatchAnalyzeConversation {
    pub fn new(user_prompt: impl Into<String>) -> Self {
        Self {
            user_prompt: user_prompt.into(),
        }
    }
}

impl LLMConversation for BatchAnalyzeConversation {
    fn get_system_prompt(&self) -> String {
        include_str!("prompt.md").to_string()
    }

    fn get_user_prompt(&self) -> String {
        self.user_prompt.clone()
    }

    fn get_max_tokens(&self) -> Option<u32> {
        // Batch analysis for mass operations
        None
    }

    fn get_temperature(&self) -> f32 {
        0.3
    }
}
