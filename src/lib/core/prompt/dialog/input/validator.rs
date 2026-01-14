//! 验证器模块
//!
//! 提供输入验证功能，包括 Validator trait 和内置验证器

use std::result;

/// 验证结果类型别名
pub type ValidationResult = result::Result<(), String>;

/// 验证器 Trait
pub trait Validator: Send + Sync {
    /// 验证输入，返回错误消息（如果验证失败）
    fn validate(&self, input: &str) -> ValidationResult;
}

/// 函数式验证器
impl<F> Validator for F
where
    F: Fn(&str) -> ValidationResult + Send + Sync,
{
    fn validate(&self, input: &str) -> ValidationResult {
        self(input)
    }
}

/// 内置验证器
pub mod validators {
    use super::Validator;

    /// 必填字段验证器
    ///
    /// 验证输入不能为空（去除首尾空格后）。
    ///
    /// # 返回
    ///
    /// 返回一个验证器，如果输入为空则返回错误。
    pub fn required() -> impl Validator {
        move |input: &str| {
            if input.trim().is_empty() {
                Err("此字段为必填项".to_string())
            } else {
                Ok(())
            }
        }
    }

    /// 邮箱地址验证器
    ///
    /// 简单的邮箱格式验证，检查是否包含 `@` 和 `.`。
    ///
    /// # 注意
    ///
    /// 这是一个简单的验证，不进行完整的 RFC 5322 验证。
    /// 如需更严格的验证，请使用 `regex` 验证器。
    ///
    /// # 返回
    ///
    /// 返回一个验证器，如果输入不符合邮箱格式则返回错误。
    pub fn email() -> impl Validator {
        move |input: &str| {
            if input.contains('@') && input.contains('.') {
                Ok(())
            } else {
                Err("请输入有效的邮箱地址".to_string())
            }
        }
    }

    /// 最小长度验证器
    ///
    /// 验证输入的长度至少为指定值。
    ///
    /// # 参数
    ///
    /// * `min` - 最小长度（字符数）
    ///
    /// # 返回
    ///
    /// 返回一个验证器，如果输入长度小于最小值则返回错误。
    ///
    /// # 注意
    ///
    /// 空输入会被允许（空输入由 `required()` 验证器处理）。
    pub fn min_length(min: usize) -> impl Validator {
        move |input: &str| {
            // 允许空输入（空输入由 required() 验证器处理）
            if input.is_empty() || input.len() >= min {
                Ok(())
            } else {
                Err(format!("长度至少为 {} 个字符", min))
            }
        }
    }

    /// 最大长度验证器
    ///
    /// 验证输入的长度不超过指定值。
    ///
    /// # 参数
    ///
    /// * `max` - 最大长度（字符数）
    ///
    /// # 返回
    ///
    /// 返回一个验证器，如果输入长度超过最大值则返回错误。
    pub fn max_length(max: usize) -> impl Validator {
        move |input: &str| {
            if input.len() <= max {
                Ok(())
            } else {
                Err(format!("长度不能超过 {} 个字符", max))
            }
        }
    }

    /// 长度范围验证器
    ///
    /// 验证输入的长度在指定范围内。
    ///
    /// # 参数
    ///
    /// * `min` - 最小长度（字符数）
    /// * `max` - 最大长度（字符数）
    ///
    /// # 返回
    ///
    /// 返回一个验证器，如果输入长度不在范围内则返回错误。
    ///
    /// # Panics
    ///
    /// 如果 `min > max`，验证器行为未定义。
    pub fn length(min: usize, max: usize) -> impl Validator {
        move |input: &str| {
            let len = input.len();
            if len >= min && len <= max {
                Ok(())
            } else {
                Err(format!("长度必须在 {} 到 {} 个字符之间", min, max))
            }
        }
    }

    /// URL 地址验证器
    ///
    /// 验证输入是否为有效的 HTTP/HTTPS URL。
    ///
    /// # 返回
    ///
    /// 返回一个验证器，如果输入不是有效的 URL 则返回错误。
    ///
    /// # 注意
    ///
    /// 这是一个简单的验证，只检查基本格式（必须以 http:// 或 https:// 开头，包含域名等）。
    /// 如需更严格的验证，请使用 `regex` 验证器。
    pub fn url() -> impl Validator {
        const ERROR_MSG: &str = "请输入有效的 URL 地址";
        const ERROR_MSG_SCHEME: &str = "请输入有效的 URL 地址（必须使用 http:// 或 https://）";
        const HTTP_SCHEME: &str = "http://";
        const HTTPS_SCHEME: &str = "https://";
        const SCHEME_SEPARATOR: &str = "://";

        move |input: &str| {
            if input.trim().is_empty() {
                return Err(ERROR_MSG.to_string());
            }
            // 检查是否包含空格（URL 不应该包含未编码的空格）
            if input.contains(' ') {
                return Err(ERROR_MSG.to_string());
            }
            // 简单的 URL 验证（不依赖外部 crate）
            // 检查是否以 http:// 或 https:// 开头
            let input_lower = input.to_lowercase();
            if !input_lower.starts_with(HTTP_SCHEME) && !input_lower.starts_with(HTTPS_SCHEME) {
                return Err(ERROR_MSG_SCHEME.to_string());
            }
            // 检查是否有 host（在 :// 之后至少有一个字符）
            if let Some(after_scheme) = input.split(SCHEME_SEPARATOR).nth(1) {
                if after_scheme.trim().is_empty() {
                    return Err(ERROR_MSG.to_string());
                }
                // 检查是否包含至少一个点（表示域名）
                if !after_scheme.contains('.') {
                    return Err(ERROR_MSG.to_string());
                }
            } else {
                return Err(ERROR_MSG.to_string());
            }
            Ok(())
        }
    }

    /// 正则表达式验证器
    ///
    /// 使用正则表达式验证输入格式。
    ///
    /// # 参数
    ///
    /// * `pattern` - 正则表达式模式（静态字符串，编译时验证）
    /// * `error_msg` - 可选的错误消息，如果未提供则使用默认消息
    ///
    /// # 返回
    ///
    /// 返回一个验证器，如果输入不匹配正则表达式则返回错误。
    ///
    /// # 错误
    ///
    /// 如果正则表达式编译失败，会在创建验证器时返回错误。
    /// 这通常表示正则表达式模式本身有问题。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::prompt::validators;
    ///
    /// // 验证数字
    /// let validator = validators::regex(r"^\d+$", Some("请输入数字"))?;
    ///
    /// // 验证邮箱（更严格）
    /// let validator = validators::regex(
    ///     r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
    ///     Some("请输入有效的邮箱地址")
    /// )?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn regex(
        pattern: &'static str,
        error_msg: Option<&'static str>,
    ) -> Result<impl Validator, String> {
        use regex::Regex;
        let re =
            Regex::new(pattern).map_err(|e| format!("无效的正则表达式 '{}': {}", pattern, e))?;

        let error_msg = error_msg
            .map(String::from)
            .unwrap_or_else(|| format!("输入格式不正确，必须匹配: {}", pattern));

        Ok(move |input: &str| {
            if re.is_match(input) {
                Ok(())
            } else {
                Err(error_msg.clone())
            }
        })
    }
}
