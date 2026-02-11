//! Jira 状态管理服务
//!
//! 提供 Jira 状态相关的业务逻辑实现，包括：
//! - 获取项目状态列表（通过 REST API）
//! - 读取状态配置
//! - 写入状态配置

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::Path, sync::Arc};

use domain::{
    extract_jira_project, validate_jira_ticket_format, JiraError, JiraStatusConfig, PathService,
    ProjectStatusConfig,
};
use toolkit::{file, log_debug};

use super::entity::JiraConfig;
use crate::jira::JiraClient;

/// 状态服务接口
pub trait StatusService: Send + Sync {
    fn get_project_statuses(&self, project: &str) -> Result<Vec<String>, JiraError>;
    fn write_status_config(&self, config: &JiraStatusConfig) -> Result<(), JiraError>;
    fn read_pull_request_created_status(
        &self,
        jira_ticket: &str,
    ) -> Result<Option<String>, JiraError>;
    fn read_pull_request_merged_status(
        &self,
        jira_ticket: &str,
    ) -> Result<Option<String>, JiraError>;
}

/// Jira 状态管理服务
///
/// 提供 PR 创建和合并时的状态自动更新功能。
pub struct StatusServiceImpl {
    jira_client: Arc<dyn JiraClient>,
    path_service: Arc<dyn PathService>,
}

impl StatusServiceImpl {
    pub fn new(jira_client: Arc<dyn JiraClient>, path_service: Arc<dyn PathService>) -> Self {
        Self {
            jira_client,
            path_service,
        }
    }

    /// 读取 Jira 配置文件
    ///
    /// 如果文件不存在，返回默认配置。
    fn read_jira_config(path: impl AsRef<Path>) -> Result<JiraConfig, JiraError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(JiraConfig::default());
        }
        let content = file::read_string(path)
            .map_err(|e| JiraError::ApiError(format!("Failed to read config file: {}", e)))?;
        toml::from_str(&content).map_err(|e| {
            JiraError::ApiError(format!("Failed to parse TOML config {:?}: {}", path, e))
        })
    }

    /// 写入 Jira 配置文件
    ///
    /// 在 Unix 系统上会自动设置文件权限为 600。
    fn write_jira_config(path: impl AsRef<Path>, config: &JiraConfig) -> Result<(), JiraError> {
        let path = path.as_ref();
        file::write_toml(path, config).map_err(|e| {
            JiraError::ApiError(format!("Failed to write config file {:?}: {}", path, e))
        })?;
        Self::set_permissions(path)
            .map_err(|e| JiraError::ApiError(format!("Failed to set permissions: {}", e)))?;
        Ok(())
    }

    #[cfg(unix)]
    fn set_permissions(path: &Path) -> Result<(), JiraError> {
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|e| {
            JiraError::ApiError(format!("Failed to set config file permissions: {}", e))
        })?;
        Ok(())
    }

    #[cfg(not(unix))]
    fn set_permissions(_path: &Path) -> Result<(), JiraError> {
        Ok(())
    }

    /// 读取 Jira 状态配置（内部方法）
    ///
    /// 从 `jira.toml` 配置文件中读取指定项目的状态配置。
    ///
    /// # 参数
    ///
    /// * `project` - 项目名称（如 `"PROJ"`）
    ///
    /// # 返回
    ///
    /// 返回 `JiraStatusConfig` 结构体。
    /// 如果文件不存在或项目配置不存在，返回空配置（所有字段为 `None`）。
    ///
    /// # 错误
    ///
    /// 如果读取或解析文件失败，返回相应的错误信息。
    fn read_status_config(&self, project: &str) -> Result<JiraStatusConfig, JiraError> {
        let config_path = self
            .path_service
            .get_jira_config_filepath()
            .map_err(|e| JiraError::ApiError(format!("Failed to get config path: {}", e)))?;
        let config = Self::read_jira_config(&config_path)
            .map_err(|e| JiraError::ApiError(format!("Failed to read config: {}", e)))?;

        if let Some(project_config) = config.status.get(project) {
            Ok(JiraStatusConfig {
                project: project.to_string(),
                created_pull_request_status: project_config.created_pull_request_status.clone(),
                merged_pull_request_status: project_config.merged_pull_request_status.clone(),
            })
        } else {
            // 返回空配置
            Ok(JiraStatusConfig {
                project: project.to_string(),
                created_pull_request_status: None,
                merged_pull_request_status: None,
            })
        }
    }
}

impl StatusService for StatusServiceImpl {
    /// 获取项目状态列表
    ///
    /// 从 Jira API 获取指定项目的所有可用状态列表。
    ///
    /// # 参数
    ///
    /// * `project` - 项目名称（如 `"PROJ"`）
    ///
    /// # 返回
    ///
    /// 返回项目状态名称列表。
    ///
    /// # 错误
    ///
    /// 如果项目名格式无效、无法获取状态列表或解析失败，返回相应的错误信息。
    fn get_project_statuses(&self, project: &str) -> Result<Vec<String>, JiraError> {
        log_debug!("Fetching status list for project: {}", project);
        let path = format!("project/{}/statuses", project);
        let response = self
            .jira_client
            .get(&path, None)
            .map_err(|e| {
                JiraError::ApiError(format!(
                    "Failed to fetch project statuses for '{}'. Please check:\n  - The project name is correct\n  - The project exists in your Jira instance\n  - You have access to this project\n  - The project name format is correct (e.g., 'PROJ', not 'zw/修改打包脚本问题')\nError: {}",
                    project, e
                ))
            })?;

        let data = response.data;
        let statuses = data
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|version| version.get("statuses"))
            .and_then(|s| s.as_array())
            .ok_or_else(|| {
                JiraError::ApiError(format!(
                    "Invalid statuses JSON structure for project '{}'. The API response format may have changed. Response: {}",
                    project,
                    serde_json::to_string_pretty(&data).unwrap_or_else(|_| "Unable to serialize response".to_string())
                ))
            })?;

        let status_names: Vec<String> = statuses
            .iter()
            .filter_map(|s| s.get("name"))
            .filter_map(|n| n.as_str())
            .map(|s| s.to_string())
            .collect();

        if status_names.is_empty() {
            return Err(JiraError::ApiError(format!(
                "No statuses found for project: {}",
                project
            )));
        }

        Ok(status_names)
    }

    /// 写入 Jira 状态配置
    ///
    /// 将状态配置写入 `jira.toml` 配置文件。
    /// 如果项目配置已存在，则更新；如果不存在，则创建新配置。
    ///
    /// # 参数
    ///
    /// * `config` - Jira 状态配置结构体
    ///
    /// # 行为
    ///
    /// 1. 读取现有的 Jira 配置（如果文件存在）
    /// 2. 更新或插入项目状态配置
    /// 3. 将更新后的配置写入文件（使用 pretty print 格式）
    ///
    /// # 错误
    ///
    /// 如果读取或写入文件失败，返回相应的错误信息。
    fn write_status_config(&self, config: &JiraStatusConfig) -> Result<(), JiraError> {
        let config_path = self
            .path_service
            .get_jira_config_filepath()
            .map_err(|e| JiraError::ApiError(format!("Failed to get config path: {}", e)))?;

        let mut jira_config = Self::read_jira_config(&config_path)
            .map_err(|e| JiraError::ApiError(format!("Failed to read config: {}", e)))?;

        jira_config.status.insert(
            config.project.clone(),
            ProjectStatusConfig {
                created_pull_request_status: config.created_pull_request_status.clone(),
                merged_pull_request_status: config.merged_pull_request_status.clone(),
            },
        );

        Self::write_jira_config(&config_path, &jira_config)
            .map_err(|e| JiraError::ApiError(format!("Failed to write status config: {}", e)))?;

        Ok(())
    }

    /// 读取 PR 创建时的状态
    ///
    /// 从配置文件中读取指定 Jira ticket 所属项目的 PR 创建时的目标状态。
    ///
    /// # 参数
    ///
    /// * `jira_ticket` - Jira ticket ID（如 `"PROJ-123"`）
    ///
    /// # 返回
    ///
    /// 返回 PR 创建时的目标状态名称（如果已配置），否则返回 `None`。
    ///
    /// # 错误
    ///
    /// 如果 ticket 格式无效或读取配置失败，返回相应的错误信息。
    fn read_pull_request_created_status(
        &self,
        jira_ticket: &str,
    ) -> Result<Option<String>, JiraError> {
        // 先验证 ticket 格式
        validate_jira_ticket_format(jira_ticket)
            .map_err(|e| JiraError::ApiError(format!("Invalid Jira ticket format: {}", e)))?;

        let project = extract_jira_project(jira_ticket).ok_or_else(|| {
            JiraError::ApiError("Invalid Jira ticket format: cannot extract project".to_string())
        })?;

        let config = self.read_status_config(project).map_err(|e| {
            JiraError::ApiError(format!("Failed to read Jira status configuration: {}", e))
        })?;

        Ok(config.created_pull_request_status)
    }

    /// 读取 PR 合并时的状态
    ///
    /// 从配置文件中读取指定 Jira ticket 所属项目的 PR 合并时的目标状态。
    ///
    /// # 参数
    ///
    /// * `jira_ticket` - Jira ticket ID（如 `"PROJ-123"`）
    ///
    /// # 返回
    ///
    /// 返回 PR 合并时的目标状态名称（如果已配置），否则返回 `None`。
    ///
    /// # 错误
    ///
    /// 如果 ticket 格式无效或读取配置失败，返回相应的错误信息。
    fn read_pull_request_merged_status(
        &self,
        jira_ticket: &str,
    ) -> Result<Option<String>, JiraError> {
        // 先验证 ticket 格式
        validate_jira_ticket_format(jira_ticket)
            .map_err(|e| JiraError::ApiError(format!("Invalid Jira ticket format: {}", e)))?;

        let project = extract_jira_project(jira_ticket).ok_or_else(|| {
            JiraError::ApiError("Invalid Jira ticket format: cannot extract project".to_string())
        })?;

        let config = self.read_status_config(project).map_err(|e| {
            JiraError::ApiError(format!("Failed to read Jira status configuration: {}", e))
        })?;

        Ok(config.merged_pull_request_status)
    }
}
