//! Multipart 请求

use std::{path::Path, time::Duration};

use reqwest::{
    blocking::multipart::{Form, Part},
    header::HeaderMap,
};

use crate::{auth::Authorization, error::HttpError};

/// Multipart 请求构建器
///
/// 用于构建 multipart/form-data 请求。
#[derive(Debug)]
pub struct MultipartRequest {
    /// Multipart form
    form: Option<Form>,
    /// 查询参数
    pub(crate) query: Option<serde_json::Value>,
    /// 请求头
    pub(crate) headers: Option<HeaderMap>,
    /// 认证信息
    pub(crate) auth: Option<Authorization>,
    /// 超时时间
    pub(crate) timeout: Option<Duration>,
}

impl Default for MultipartRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl MultipartRequest {
    /// 创建新的 Multipart 请求
    pub fn new() -> Self {
        Self {
            form: Some(Form::new()),
            query: None,
            headers: None,
            auth: None,
            timeout: None,
        }
    }

    /// 添加文本字段
    pub fn text(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        if let Some(form) = self.form.take() {
            self.form = Some(form.text(name.into(), value.into()));
        }
        self
    }

    /// 添加文件
    pub fn file(
        mut self,
        name: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, HttpError> {
        let path = path.as_ref();
        let part = Part::file(path).map_err(|e| {
            HttpError::Other(format!("Failed to read file '{}': {}", path.display(), e))
        })?;

        if let Some(form) = self.form.take() {
            self.form = Some(form.part(name.into(), part));
        }
        Ok(self)
    }

    /// 添加字节数据
    pub fn bytes(
        mut self,
        name: impl Into<String>,
        data: Vec<u8>,
        filename: Option<&str>,
        mime_type: Option<&str>,
    ) -> Self {
        let mut part = Part::bytes(data);

        if let Some(filename) = filename {
            part = part.file_name(filename.to_string());
        }

        // 应用 mime type（如果提供且有效）
        if let Some(mt) = mime_type {
            match part.mime_str(mt) {
                Ok(p) => part = p,
                Err(e) => {
                    tracing::warn!(mime_type = mt, error = %e, "Invalid MIME type, ignoring");
                    return self;
                }
            }
        }

        if let Some(form) = self.form.take() {
            self.form = Some(form.part(name.into(), part));
        }
        self
    }

    /// 设置查询参数
    pub fn query<T: serde::Serialize>(mut self, query: &T) -> Self {
        self.query = serde_json::to_value(query).ok();
        self
    }

    /// 设置请求头
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = Some(headers);
        self
    }

    /// 设置认证
    pub fn auth(mut self, auth: Authorization) -> Self {
        self.auth = Some(auth);
        self
    }

    /// 设置超时
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// 获取 form（内部使用）
    pub(crate) fn into_form(mut self) -> Option<Form> {
        self.form.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multipart_text() {
        let request = MultipartRequest::new().text("field1", "value1").text("field2", "value2");
        assert!(request.form.is_some());
    }

    #[test]
    fn test_multipart_bytes() {
        let request = MultipartRequest::new().bytes(
            "file",
            vec![1, 2, 3],
            Some("test.bin"),
            Some("application/octet-stream"),
        );
        assert!(request.form.is_some());
    }

    #[test]
    fn test_multipart_auth() {
        let request = MultipartRequest::new().auth(Authorization::bearer("token"));
        assert!(request.auth.is_some());
    }

    #[test]
    fn test_multipart_timeout() {
        let request = MultipartRequest::new().timeout(Duration::from_secs(60));
        assert_eq!(request.timeout, Some(Duration::from_secs(60)));
    }
}
