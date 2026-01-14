//! HTTP 响应解析器

use color_eyre::Result;
use serde::Deserialize;

/// 响应解析器 Trait
///
/// 定义如何将响应体字节解析为特定类型。
/// 不同的格式（JSON、Text、XML 等）可以实现此 Trait 来提供解析逻辑。
///
/// # 类型参数
///
/// * `T` - 解析后的目标类型
pub trait ResponseParser<T> {
    /// 解析响应体
    ///
    /// 将响应体字节解析为特定类型。
    ///
    /// # 参数
    ///
    /// * `bytes` - 响应体字节
    /// * `status` - HTTP 状态码（用于错误处理和验证）
    ///
    /// # 返回
    ///
    /// 返回解析后的数据。
    ///
    /// # 错误
    ///
    /// 如果解析失败，返回相应的错误信息。
    fn parse(bytes: &[u8], status: u16) -> Result<T>
    where
        Self: Sized;
}

/// JSON 解析器
///
/// 将响应体解析为 JSON 格式。
pub struct JsonParser;

impl<T> ResponseParser<T> for JsonParser
where
    T: for<'de> Deserialize<'de>,
{
    fn parse(bytes: &[u8], status: u16) -> Result<T> {
        use crate::core::http::response::error::HttpResponseError;
        // 处理空响应
        if bytes.is_empty() || bytes.iter().all(|&b| b.is_ascii_whitespace()) {
            serde_json::from_slice(b"null")
                .or_else(|_| serde_json::from_slice(b"{}"))
                .map_err(|_| HttpResponseError::ParseEmptyJsonFailed.into())
        } else {
            serde_json::from_slice(bytes).map_err(|_source| {
                let response_text = String::from_utf8_lossy(bytes);
                let preview = if response_text.len() > 200 {
                    format!("{}...", &response_text[..200])
                } else {
                    response_text.to_string()
                };
                HttpResponseError::ParseJsonFailed { status, preview }.into()
            })
        }
    }
}

/// 文本解析器
///
/// 将响应体解析为 UTF-8 文本字符串。
pub struct TextParser;

impl ResponseParser<String> for TextParser {
    fn parse(bytes: &[u8], status: u16) -> Result<String> {
        use crate::core::http::response::error::HttpResponseError;
        // 检查状态码
        if !(200..300).contains(&status) {
            return Err(HttpResponseError::HttpRequestFailed(status).into());
        }

        String::from_utf8(bytes.to_vec()).map_err(|e| HttpResponseError::DecodeUtf8Failed(e).into())
    }
}
