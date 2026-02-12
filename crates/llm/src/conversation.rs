use crate::LLMRequestParameters;

/// 可转换为 LLM 请求参数的 trait
///
/// 用于快速将任意类型转换为 `LLMRequestParameters`。
/// 对 `LLMConversation` 有默认实现，也可为其他类型单独实现。
pub trait IntoLLMRequestParameters {
    fn to_params(&self, language_code: &str) -> LLMRequestParameters;
}

impl<T: LLMConversation> IntoLLMRequestParameters for T {
    fn to_params(&self, language_code: &str) -> LLMRequestParameters {
        LLMRequestParameters {
            system_prompt: self.get_system_prompt(language_code),
            user_prompt: self.get_user_prompt(language_code),
            max_tokens: self.get_max_tokens(),
            temperature: self.get_temperature(),
        }
    }
}

/// LLM Conversation Trait
///
/// 定义所有 Conversation 的通用接口，包括 prompt 构建。
/// Conversation 只负责定义对话结构，不负责执行逻辑。
///
/// # 方法
///
/// * `get_system_prompt` - 获取 system prompt
/// * `get_user_prompt` - 构建 user prompt
/// * `get_execution_params` - 获取执行参数（max_tokens, temperature）
pub trait LLMConversation {
    /// 获取 system prompt
    ///
    /// 返回用于 LLM 调用的 system prompt 字符串。
    ///
    /// # 参数
    ///
    /// * `language_code` - 语言代码，用于生成对应语言的 prompt
    fn get_system_prompt(&self, language_code: &str) -> String;

    /// 构建 user prompt
    ///
    /// 根据 conversation 中存储的输入数据构建 user prompt 字符串。
    ///
    /// # 参数
    ///
    /// * `language_code` - 语言代码，用于生成对应语言的 prompt
    ///
    /// # 返回
    ///
    /// 返回构建好的 user prompt 字符串
    fn get_user_prompt(&self, language_code: &str) -> String;

    /// 获取最大 token 数
    ///
    /// 返回最大 token 数，None 表示由 LLM 自动决定
    fn get_max_tokens(&self) -> Option<u32> {
        None
    }

    /// 获取温度参数
    ///
    /// 返回温度参数，控制输出的随机性（0.0-1.0）
    ///
    fn get_temperature(&self) -> f32 {
        0.5
    }
}
