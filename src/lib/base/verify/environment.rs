//! 环境检查模块
//!
//! 检查 Git 仓库状态、网络连接和配置文件权限。

use crate::base::constants::{errors::http_client, git::check_errors};
use crate::base::http::client::HttpClient;
use crate::base::http::{HttpMethod, RequestConfig};
use crate::base::settings::settings::Settings;
use crate::git::{GitCommit, GitRepo};
use crate::{br, error, info, success};
use color_eyre::{eyre::WrapErr, Result};
use serde_json::Value;
use std::time::Duration;

/// 环境检查结果
#[derive(Debug, Clone)]
pub struct EnvironmentCheckResult {
    /// Git 状态检查结果
    pub git_status: GitStatusCheck,
    /// 网络连接检查结果
    pub network: NetworkCheck,
    /// 配置文件权限警告（如果有）
    pub config_permissions_warning: Option<String>,
}

/// Git 状态检查结果
#[derive(Debug, Clone)]
pub struct GitStatusCheck {
    /// 是否在 Git 仓库中
    pub is_git_repo: bool,
    /// Git 状态输出（如果有未提交的更改）
    pub status_output: Option<String>,
}

/// 网络连接检查结果
#[derive(Debug, Clone)]
pub struct NetworkCheck {
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    pub error: Option<String>,
}

/// 环境验证器
pub struct EnvironmentVerifier;

impl EnvironmentVerifier {
    /// 执行完整的环境检查
    ///
    /// 包括：
    /// - Git 仓库状态检查
    /// - 网络连接检查（GitHub）
    /// - 配置文件权限检查
    pub fn verify_all() -> Result<EnvironmentCheckResult> {
        info!("Running environment checks...");
        br!();

        // 1. 检查 Git 状态
        let git_status = Self::check_git_status()?;

        br!();

        // 2. 检查网络连接
        let network = Self::check_network()?;

        br!();

        // 3. 检查配置文件权限
        let config_permissions_warning = Self::check_config_permissions();

        Ok(EnvironmentCheckResult {
            git_status,
            network,
            config_permissions_warning,
        })
    }

    /// 检查 Git 仓库状态
    pub fn check_git_status() -> Result<GitStatusCheck> {
        info!("[1/2] Checking Git repository status...");
        if !GitRepo::is_git_repo() {
            error!("Not in a Git repository");
            return Err(color_eyre::eyre::eyre!("{}", check_errors::NOT_GIT_REPO));
        }

        let git_output = GitCommit::status().wrap_err("Failed to check git status")?;
        if git_output.trim().is_empty() {
            success!("Git repository is clean (no uncommitted changes)");
            Ok(GitStatusCheck {
                is_git_repo: true,
                status_output: None,
            })
        } else {
            info!("Git status:\n{}", git_output);
            Ok(GitStatusCheck {
                is_git_repo: true,
                status_output: Some(git_output),
            })
        }
    }

    /// 检查网络连接（GitHub）
    pub fn check_network() -> Result<NetworkCheck> {
        info!("[2/2] Checking network connection to GitHub...");
        let client = HttpClient::global().wrap_err(http_client::CREATE_CLIENT_FAILED)?;
        let config = RequestConfig::<Value, Value>::new().timeout(Duration::from_secs(10));
        match client.stream(HttpMethod::Get, crate::git::github::BASE, config) {
            Ok(response) => {
                if response.status().is_success() {
                    success!("GitHub network is available");
                    Ok(NetworkCheck {
                        success: true,
                        error: None,
                    })
                } else {
                    let error_msg = format!(
                        "GitHub network check failed (status: {})",
                        response.status()
                    );
                    error!("{}", error_msg);
                    Err(color_eyre::eyre::eyre!("Network check failed"))
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to check network connection: {}", e);
                error!("{}", error_msg);
                error!(
                    "  This might be due to network issues, proxy settings, or firewall restrictions"
                );
                Err(color_eyre::eyre::eyre!("Network check failed: {}", e))
            }
        }
    }

    /// 检查配置文件权限
    ///
    /// 返回警告信息（如果有），否则返回 `None`。
    pub fn check_config_permissions() -> Option<String> {
        Settings::check_permissions()
    }
}
