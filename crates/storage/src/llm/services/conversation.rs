use domain::LLMError;

/// LLM Conversation Trait
///
/// 定义所有 Conversation 的通用接口，包括 prompt 构建和响应解析。
/// Conversation 只负责定义对话结构，不负责执行逻辑。
///
/// # 关联类型
///
/// * `Input` - Conversation 的输入参数类型（用于构建 user prompt）
/// * `Output` - Conversation 的输出类型
pub trait LLMConversation {
    /// Conversation 的输入参数类型
    ///
    /// 用于 `get_user_prompt` 方法的参数。可以是元组、结构体等。
    type Input;

    /// Conversation 的输出类型
    type Output;

    /// 获取 system prompt
    ///
    /// 返回用于 LLM 调用的 system prompt 字符串。
    fn get_system_prompt(&self, language_code: &str) -> String;

    /// 构建 user prompt
    ///
    /// 根据 conversation 中存储的 input 构建 user prompt 字符串。
    ///
    /// # 返回
    ///
    /// 返回构建好的 user prompt 字符串
    fn get_user_prompt(&self, language_code: &str) -> String;

    /// 获取执行参数
    ///
    /// 返回 (max_tokens, temperature) 元组。
    /// - `max_tokens`: 最大 token 数，None 表示由 LLM 自动决定
    /// - `temperature`: 温度参数，控制输出的随机性
    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.5) // 默认值
    }

    /// 解析 LLM 响应
    ///
    /// 将 LLM 返回的原始响应解析为 Conversation 的输出类型。
    ///
    /// # 参数
    ///
    /// * `response` - LLM 返回的原始响应字符串
    ///
    /// # 返回
    ///
    /// 返回解析后的结果
    fn parse_response(&self, response: String) -> Result<Self::Output, LLMError>;
}
