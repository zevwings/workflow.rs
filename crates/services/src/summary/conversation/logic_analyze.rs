//! 阶段二 2.2：核心逻辑分析对话
//!
//! 对业务代码、服务层等核心文件的完整 diff 进行深入分析。

use llm::LLMConversation;
use crate::summary::prompt::analyze_logic;

/// 核心逻辑分析对话
///
/// 输入为已构建的 user prompt（包含修改文件列表及每个文件的 diff），输出为每个文件的详细分析。
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
    fn get_system_prompt(&self, _language_code: &str) -> String {
        analyze_logic().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        self.user_prompt.clone()
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.3)
    }
}
