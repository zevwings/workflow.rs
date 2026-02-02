//! CNB 配置上下文实现
//!
//! 实现 `domain::CNBContext` trait，提供配置获取逻辑。

use std::sync::Arc;

use domain::{CNBContext, CNBError, GitRepository, GlobalConfigRepository};

/// CNB 配置上下文实现
///
/// 实现 `CNBContext` trait，提供基于配置和 Git 仓库的配置获取逻辑。
pub struct CNBContextImpl {
    git_repo: Arc<dyn GitRepository>,
    config_repo: Arc<dyn GlobalConfigRepository>,
}

impl CNBContextImpl {
    pub fn new(
        git_repo: Arc<dyn GitRepository>,
        config_repo: Arc<dyn GlobalConfigRepository>,
    ) -> Self {
        Self {
            git_repo,
            config_repo,
        }
    }

    /// 从 Git 远程 URL 提取项目路径
    ///
    /// 支持的格式：
    /// - https://cnb.cool/owner/project
    /// - https://oauth2:token@cnb.cool/owner/project (带认证信息)
    /// - git@cnb.cool:owner/project.git
    fn extract_project_path_from_url(url: &str) -> Option<String> {
        // 移除 .git 后缀
        let url = url.trim_end_matches(".git");

        // 处理 HTTPS URL（可能包含认证信息）
        // 格式: https://[user[:password]@]cnb.cool/path
        if url.contains("cnb.cool/") {
            // 使用正则表达式或字符串处理来提取路径
            // 先尝试找到 "cnb.cool/" 的位置
            if let Some(pos) = url.find("cnb.cool/") {
                let path = &url[pos + "cnb.cool/".len()..];
                // 移除可能的查询参数和片段
                let path = path.split('?').next().unwrap_or(path);
                let path = path.split('#').next().unwrap_or(path);
                if !path.is_empty() {
                    return Some(path.to_string());
                }
            }
        }

        // 处理 SSH URL
        if url.starts_with("git@cnb.cool:") {
            return url.strip_prefix("git@cnb.cool:").map(|s| s.to_string());
        }

        None
    }
}

impl CNBContext for CNBContextImpl {
    fn get_name(&self) -> Result<String, CNBError> {
        let config = self
            .config_repo
            .load()
            .map_err(|e| CNBError::Other(format!("Failed to load config: {}", e)))?;

        let account = config
            .cnb
            .get_current_account()
            .ok_or_else(|| CNBError::Other("No CNB account configured".to_string()))?;

        Ok(account.name.clone())
    }

    fn get_login(&self) -> Result<String, CNBError> {
        let config = self
            .config_repo
            .load()
            .map_err(|e| CNBError::Other(format!("Failed to load config: {}", e)))?;

        let account = config
            .cnb
            .get_current_account()
            .ok_or_else(|| CNBError::Other("No CNB account configured".to_string()))?;

        Ok(account.login.clone())
    }

    fn get_email(&self) -> Result<String, CNBError> {
        let config = self
            .config_repo
            .load()
            .map_err(|e| CNBError::Other(format!("Failed to load config: {}", e)))?;

        let account = config
            .cnb
            .get_current_account()
            .ok_or_else(|| CNBError::Other("No CNB account configured".to_string()))?;

        Ok(account.email.clone())
    }

    fn get_api_token(&self) -> Result<String, CNBError> {
        let config = self
            .config_repo
            .load()
            .map_err(|e| CNBError::Other(format!("Failed to load config: {}", e)))?;

        let account = config
            .cnb
            .get_current_account()
            .ok_or_else(|| CNBError::Other("No CNB account configured".to_string()))?;

        Ok(account.api_token.clone())
    }

    fn get_project_path(&self) -> Result<String, CNBError> {
        // 获取仓库信息
        let repo_info = self.git_repo.get_repo_info();

        // 获取远程 URL
        let remote_url = repo_info
            .origin_url
            .ok_or_else(|| CNBError::Other("Failed to get origin URL".to_string()))?;

        // 从 URL 中提取项目路径
        Self::extract_project_path_from_url(&remote_url).ok_or_else(|| {
            CNBError::Other(format!(
                "Failed to extract project path from URL: {}",
                remote_url
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_project_path_from_https_url() {
        let url = "https://cnb.cool/owner/project";
        let result = CNBContextImpl::extract_project_path_from_url(url);
        assert_eq!(result, Some("owner/project".to_string()));
    }

    #[test]
    fn test_extract_project_path_from_https_url_with_git() {
        let url = "https://cnb.cool/owner/project.git";
        let result = CNBContextImpl::extract_project_path_from_url(url);
        assert_eq!(result, Some("owner/project".to_string()));
    }

    #[test]
    fn test_extract_project_path_from_ssh_url() {
        let url = "git@cnb.cool:owner/project.git";
        let result = CNBContextImpl::extract_project_path_from_url(url);
        assert_eq!(result, Some("owner/project".to_string()));
    }

    #[test]
    fn test_extract_project_path_from_https_url_with_auth() {
        let url = "https://oauth2:token@cnb.cool/zevwings.com/workflow/Workflow";
        let result = CNBContextImpl::extract_project_path_from_url(url);
        assert_eq!(result, Some("zevwings.com/workflow/Workflow".to_string()));
    }

    #[test]
    fn test_extract_project_path_from_https_url_with_auth_and_git() {
        let url = "https://oauth2:token@cnb.cool/zevwings.com/workflow/Workflow.git";
        let result = CNBContextImpl::extract_project_path_from_url(url);
        assert_eq!(result, Some("zevwings.com/workflow/Workflow".to_string()));
    }

    #[test]
    fn test_extract_project_path_from_invalid_url() {
        let url = "https://github.com/owner/repo";
        let result = CNBContextImpl::extract_project_path_from_url(url);
        assert_eq!(result, None);
    }
}
