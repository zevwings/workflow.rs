//! 阶段二 2.2：核心逻辑分析服务
//!
//! 对业务代码、服务层等核心文件的完整 diff 进行深入分析。

use llm::LLMConversation;

use crate::summary::prompt;

/// 核心逻辑分析对话
pub(crate) struct LogicAnalyzeConversation {
    user_prompt: String,
}

impl LogicAnalyzeConversation {
    pub fn new(user_prompt: String) -> Self {
        Self { user_prompt }
    }
}

impl LLMConversation for LogicAnalyzeConversation {
    fn get_system_prompt(&self, _language_code: &str) -> String {
        prompt::analyze_logic().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        self.user_prompt.clone()
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.3)
    }
}
