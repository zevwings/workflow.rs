//! GitHub 配置相关结构体

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// GitHub 账号配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAccount {
    /// 账号名称（用于标识和切换）
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// 账号邮箱（必填，用于显示和区分）
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub email: String,
    /// GitHub API Token
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_token: String,
}

impl GitHubAccount {
    /// 检查 GitHub 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.email.is_empty() && self.api_token.is_empty()
    }
}

/// GitHub 配置（TOML）
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubSettings {
    /// 多个 GitHub 账号列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accounts: Vec<GitHubAccount>,
    /// 当前激活的账号名称
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub current: String,
}

impl GitHubSettings {
    /// 检查 GitHub 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty() && self.current.is_empty()
    }

    /// 获取当前激活的账号
    ///
    /// 如果设置了 `current`，返回对应的账号；否则返回第一个账号。
    /// 如果没有账号，返回 `None`。
    pub fn get_current_account(&self) -> Option<&GitHubAccount> {
        if !self.current.is_empty() {
            self.accounts.iter().find(|acc| acc.name == self.current)
        } else {
            self.accounts.first()
        }
    }

    /// 获取当前账号的 API Token
    pub fn get_current_token(&self) -> Option<&str> {
        self.get_current_account().map(|acc| acc.api_token.as_str())
    }
}
