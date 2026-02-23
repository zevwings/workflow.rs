//! HTTP 错误类型（定义层）
//!
//! 提供增强的错误类型，包含完整的请求/响应上下文信息。
//! 不依赖 reqwest 等实现层库。

use std::{collections::HashMap, error::Error, fmt, sync::Arc, time::Duration};

use crate::http::{method::ParseMethodError, HttpMethod};

/// 错误上下文
///
/// 包含请求和响应的详细信息，便于调试和日志记录。
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// 请求 URL
    pub url: String,
    /// 请求方法
    pub method: HttpMethod,
    /// 请求头
    pub request_headers: Option<HashMap<String, String>>,
    /// 响应状态码
    pub response_status: Option<u16>,
    /// 响应头
    pub response_headers: Option<HashMap<String, String>>,
    /// 响应体（截断）
    pub response_body: Option<String>,
    /// 请求耗时
    pub duration: Option<Duration>,

    pub error: Option<Arc<dyn Error>>,
}

impl ErrorContext {
    /// 创建新的错误上下文
    pub fn new(url: impl Into<String>, method: HttpMethod) -> Self {
        Self {
            url: url.into(),
            method,
            request_headers: None,
            response_status: None,
            response_headers: None,
            response_body: None,
            duration: None,
            error: None,
        }
    }

    /// 创建新的错误上下文（装箱）
    pub fn boxed(url: impl Into<String>, method: HttpMethod) -> Box<Self> {
        Box::new(Self::new(url, method))
    }

    /// 转换为 Box
    pub fn into_box(self) -> Box<Self> {
        Box::new(self)
    }

    /// 设置请求头
    pub fn with_request_headers(mut self, headers: impl Into<HashMap<String, String>>) -> Self {
        self.request_headers = Some(headers.into());
        self
    }

    /// 设置响应状态码
    pub fn with_response_status(mut self, status: u16) -> Self {
        self.response_status = Some(status);
        self
    }

    /// 设置响应头
    pub fn with_response_headers(mut self, headers: impl Into<HashMap<String, String>>) -> Self {
        self.response_headers = Some(headers.into());
        self
    }

    /// 设置响应体（自动截断）
    pub fn with_response_body(mut self, body: impl Into<String>) -> Self {
        let body = body.into();
        const MAX_BODY_LEN: usize = 1024;
        if body.len() > MAX_BODY_LEN {
            self.response_body = Some(format!("{}...(truncated)", &body[..MAX_BODY_LEN]));
        } else {
            self.response_body = Some(body);
        }
        self
    }

    /// 设置请求耗时
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// 设置错误
    pub fn with_error<E: Error + Send + Sync + 'static>(mut self, error: E) -> Self {
        self.error = Some(Arc::new(error));
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.method, self.url)?;
        if let Some(status) = self.response_status {
            write!(f, " (status: {})", status)?;
        }
        if let Some(duration) = self.duration {
            write!(f, " (took: {:?})", duration)?;
        }
        Ok(())
    }
}

/// HTTP 错误类型
///
/// 语义化变体，不持有 reqwest::Error 等实现层类型。
/// 实现层在捕获具体错误时，将其转换为 message 字符串。
#[derive(Debug)]
pub enum HttpError {
    /// 客户端创建失败
    ClientCreation(String),

    /// 请求构建失败
    RequestBuild {
        message: String,
        context: Box<ErrorContext>,
    },

    /// 连接失败
    Connection { context: Box<ErrorContext> },

    /// 请求超时
    Timeout { context: Box<ErrorContext> },

    /// 请求发送失败
    Request {
        message: String,
        context: Box<ErrorContext>,
    },

    /// HTTP 错误状态码
    Status {
        status: u16,
        context: Box<ErrorContext>,
    },

    /// 响应解析失败
    ResponseParse {
        message: String,
        context: Box<ErrorContext>,
    },

    /// 重试耗尽
    RetryExhausted {
        attempts: u32,
        last_error: Box<HttpError>,
    },

    /// 无效的 HTTP 方法
    InvalidMethod(String),

    /// 无效的 Header 名称
    InvalidHeaderName(String),

    /// 无效的 Header 值
    InvalidHeaderValue(String),

    /// 文件读取失败
    FileReadFailed(String),
}

impl From<ParseMethodError> for HttpError {
    fn from(err: ParseMethodError) -> Self {
        Self::InvalidMethod(err.to_string())
    }
}

impl HttpError {
    /// 获取错误上下文（如果有）
    pub fn context(&self) -> Option<&ErrorContext> {
        match self {
            Self::RequestBuild { context, .. }
            | Self::Connection { context }
            | Self::Timeout { context }
            | Self::Request { context, .. }
            | Self::Status { context, .. }
            | Self::ResponseParse { context, .. } => Some(context.as_ref()),
            Self::RetryExhausted { last_error, .. } => last_error.context(),
            _ => None,
        }
    }

    /// 是否是超时错误
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout { .. })
    }

    /// 是否是连接错误
    pub fn is_connection(&self) -> bool {
        matches!(self, Self::Connection { .. })
    }

    /// 是否是状态码错误
    pub fn is_status(&self) -> bool {
        matches!(self, Self::Status { .. })
    }

    /// 获取状态码（如果有）
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Status { status, .. } => Some(*status),
            _ => None,
        }
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientCreation(e) => write!(f, "Failed to create HTTP client: {}", e),
            Self::RequestBuild { message, context } => {
                write!(f, "Failed to build request: {} ({})", message, context)
            }
            Self::Connection { context } => {
                write!(f, "Connection failed: {}", context)
            }
            Self::Timeout { context } => {
                write!(f, "Request timed out: {}", context)
            }
            Self::Request { message, context } => {
                write!(f, "Request failed: {} ({})", message, context)
            }
            Self::Status { status, context } => {
                write!(f, "HTTP error {}: {}", status, context)
            }
            Self::ResponseParse { message, context } => {
                write!(f, "Failed to parse response: {} ({})", message, context)
            }
            Self::RetryExhausted {
                attempts,
                last_error,
            } => {
                write!(
                    f,
                    "Retry exhausted after {} attempts: {}",
                    attempts, last_error
                )
            }
            Self::InvalidMethod(method) => write!(f, "Invalid HTTP method: {}", method),
            Self::InvalidHeaderName(name) => write!(f, "Invalid header name: {}", name),
            Self::InvalidHeaderValue(value) => write!(f, "Invalid header value: {}", value),
            Self::FileReadFailed(e) => write!(f, "Failed to read file: {}", e),
        }
    }
}

impl std::error::Error for HttpError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_display() {
        let context = ErrorContext::new("https://api.example.com/users", HttpMethod::GET)
            .with_response_status(404)
            .with_duration(Duration::from_millis(150));

        let display = format!("{}", context);
        assert!(display.contains("GET"));
        assert!(display.contains("https://api.example.com/users"));
        assert!(display.contains("404"));
    }

    #[test]
    fn test_error_context_with_hashmap_headers() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        let context = ErrorContext::new("https://api.example.com", HttpMethod::POST)
            .with_request_headers(headers);
        assert!(context.request_headers.is_some());
    }

    #[test]
    fn test_error_context_body_truncation() {
        let long_body = "x".repeat(2000);
        let context = ErrorContext::new("https://api.example.com", HttpMethod::POST)
            .with_response_body(long_body);

        let body = context.response_body.unwrap();
        assert!(body.len() < 2000);
        assert!(body.ends_with("...(truncated)"));
    }

    #[test]
    fn test_http_error_is_timeout() {
        let error = HttpError::Timeout {
            context: ErrorContext::boxed("https://example.com", HttpMethod::GET),
        };
        assert!(error.is_timeout());
        assert!(!error.is_connection());
    }

    #[test]
    fn test_http_error_status() {
        let error = HttpError::Status {
            status: 500,
            context: ErrorContext::boxed("https://example.com", HttpMethod::GET),
        };
        assert_eq!(error.status(), Some(500));
        assert!(error.is_status());
    }

    #[test]
    fn test_parse_method_error_converts_to_http_error() {
        use crate::http::HttpMethod;

        let err: HttpError = "INVALID".parse::<HttpMethod>().unwrap_err().into();
        assert!(matches!(err, HttpError::InvalidMethod(_)));
        assert!(err.to_string().contains("Invalid HTTP method"));
    }
}
