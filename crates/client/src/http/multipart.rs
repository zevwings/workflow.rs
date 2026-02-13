//! Multipart 请求抽象（定义层）
//!
//! 纯数据结构，不依赖 reqwest。实现层负责转换为具体 HTTP 库的 multipart 格式。

use std::{collections::HashMap, path::PathBuf, time::Duration};

use super::{Authorization, ErrorContext, HttpError, HttpMethod};

/// Multipart 表单部分
#[derive(Debug, Clone)]
pub enum MultipartPart {
    /// 文本字段
    Text { name: String, value: String },
    /// 文件（路径）
    File { name: String, path: PathBuf },
    /// 字节数据
    Bytes {
        name: String,
        data: Vec<u8>,
        filename: Option<String>,
        mime_type: Option<String>,
    },
}

/// Multipart 请求（纯数据）
///
// 定义层类型，实现层负责将其转换为具体 HTTP 库的 multipart 格式。
#[derive(Debug, Clone, Default)]
pub struct MultipartRequest {
    pub parts: Vec<MultipartPart>,
    pub query: Option<serde_json::Value>,
    pub headers: HashMap<String, String>,
    pub auth: Option<Authorization>,
    pub timeout: Option<Duration>,
}

impl MultipartRequest {
    /// 创建新的 Multipart 请求
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加文本字段
    pub fn text(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.parts.push(MultipartPart::Text {
            name: name.into(),
            value: value.into(),
        });
        self
    }

    /// 添加文件
    pub fn file(mut self, name: impl Into<String>, path: impl AsRef<std::path::Path>) -> Self {
        self.parts.push(MultipartPart::File {
            name: name.into(),
            path: path.as_ref().to_path_buf(),
        });
        self
    }

    /// 添加字节数据
    pub fn bytes(
        mut self,
        name: impl Into<String>,
        data: Vec<u8>,
        filename: Option<impl Into<String>>,
        mime_type: Option<impl Into<String>>,
    ) -> Self {
        self.parts.push(MultipartPart::Bytes {
            name: name.into(),
            data,
            filename: filename.map(Into::into),
            mime_type: mime_type.map(Into::into),
        });
        self
    }

    /// 设置查询参数
    pub fn query<T: serde::Serialize>(mut self, query: &T) -> Result<Self, HttpError> {
        self.query = Some(
            serde_json::to_value(query).map_err(|e| HttpError::RequestBuild {
                message: e.to_string(),
                context: ErrorContext::new("multipart-query", HttpMethod::POST).into_box(),
            })?,
        );
        Ok(self)
    }

    /// 设置请求头
    pub fn header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.headers.insert(name.as_ref().to_lowercase(), value.as_ref().to_string());
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
}
