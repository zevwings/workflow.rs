//! 文本响应解析器
//!
//! 提供从 LLM 响应中解析纯文本的功能。

use crate::LLMError;

/// 文本响应解析器
///
/// 负责从 LLM 响应中提取和清理纯文本数据。
pub struct TextParser;

impl TextParser {
    /// 清理文本响应
    ///
    /// 移除多余的引号、空白字符等。
    ///
    /// # 参数
    ///
    /// * `response` - LLM 返回的响应字符串
    ///
    /// # 返回
    ///
    /// 返回清理后的文本
    pub fn clean(response: impl AsRef<str>) -> String {
        response
            .as_ref()
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim()
            .to_string()
    }

    /// 清理并验证文本响应
    ///
    /// 清理文本并验证不为空。
    ///
    /// # 参数
    ///
    /// * `response` - LLM 返回的响应字符串
    ///
    /// # 返回
    ///
    /// 返回清理后的文本
    ///
    /// # 错误
    ///
    /// 如果清理后的文本为空，返回相应的错误信息。
    pub fn clean_and_validate(response: impl AsRef<str>) -> Result<String, LLMError> {
        let cleaned = Self::clean(response);

        if cleaned.is_empty() {
            return Err(LLMError::ApiError(
                "LLM returned empty response".to_string(),
            ));
        }

        Ok(cleaned)
    }
}
