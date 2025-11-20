use anyhow::{Context, Result};
use std::fmt::Display;
use std::str::FromStr;

/// Pull Request ID 类型
///
/// 提供类型安全的 PR ID 封装，支持不同平台的 ID 格式
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PullRequestId(String);

impl PullRequestId {
    /// 创建新的 PullRequestId
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// 获取字符串引用
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 解析为 u64（用于 GitHub）
    pub fn parse_u64(&self) -> Result<u64> {
        self.0.parse().context("Invalid PR ID format")
    }

    /// 为 GitHub 平台解析 ID
    pub fn for_github(&self) -> Result<u64> {
        self.parse_u64()
    }

    /// 为 Codeup 平台获取 ID（直接返回字符串）
    pub fn for_codeup(&self) -> &str {
        self.as_str()
    }
}

impl From<String> for PullRequestId {
    fn from(id: String) -> Self {
        Self::new(id)
    }
}

impl From<&str> for PullRequestId {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

impl From<u64> for PullRequestId {
    fn from(id: u64) -> Self {
        Self::new(id.to_string())
    }
}

/// Pull Request 状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PullRequestState {
    Open,
    Closed,
    Merged,
}

impl PullRequestState {
    /// 转换为字符串
    pub fn as_str(&self) -> &str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Merged => "merged",
        }
    }
}

impl Display for PullRequestState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for PullRequestState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(Self::Open),
            "closed" => Ok(Self::Closed),
            "merged" => Ok(Self::Merged),
            _ => Ok(Self::Open), // 默认值
        }
    }
}

impl From<&str> for PullRequestState {
    fn from(s: &str) -> Self {
        Self::from_str(s).unwrap_or(Self::Open)
    }
}

impl From<String> for PullRequestState {
    fn from(s: String) -> Self {
        Self::from_str(&s).unwrap_or(Self::Open)
    }
}
