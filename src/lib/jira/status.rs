//! Jira 状态管理
//!
//! 本模块提供了 Jira 状态相关的完整功能：
//! - 获取项目状态列表（通过 REST API）
//! - 交互式配置状态映射（PR 创建/合并时的状态）
//! - 读取状态配置

use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::api::project::JiraProjectApi;
use super::config::ConfigManager;
use super::helpers::extract_jira_project;
use crate::base::dialog::SelectDialog;
use crate::base::settings::paths::Paths;
use crate::{trace_debug, trace_info};

// ==================== 返回结构体 ====================

/// 状态配置结果
#[derive(Debug, Clone)]
pub struct StatusConfigResult {
    /// 项目名称
    pub project: String,
    /// PR 创建时的目标状态
    pub created_pull_request_status: String,
    /// PR 合并时的目标状态
    pub merged_pull_request_status: String,
}

/// 项目状态配置
///
/// 存储单个项目的状态配置，包括 PR 创建和合并时的目标状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatusConfig {
    /// PR 创建时的目标状态（JSON 字段名：`created-pr`）
    #[serde(rename = "created-pr")]
    pub created_pull_request_status: Option<String>,
    /// PR 合并时的目标状态（JSON 字段名：`merged-pr`）
    #[serde(rename = "merged-pr")]
    pub merged_pull_request_status: Option<String>,
}

/// Jira 状态配置（兼容现有接口）
///
/// 包含项目名称和对应的状态配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatusConfig {
    /// 项目名称（如 `"PROJ"`）
    pub project: String,
    /// PR 创建时的目标状态
    pub created_pull_request_status: Option<String>,
    /// PR 合并时的目标状态
    pub merged_pull_request_status: Option<String>,
}

/// Jira 状态配置存储格式（TOML 表）
///
/// TOML 格式示例：
/// ```toml
/// [PROJ]
/// created-pr = "In Progress"
/// merged-pr = "Done"
/// ```
type JiraStatusMap = HashMap<String, ProjectStatusConfig>;

/// Jira 状态管理（用于 PR 流程）
///
/// 提供 PR 创建和合并时的状态自动更新功能。
pub struct JiraStatus;

impl JiraStatus {
    /// 交互式配置 Jira 状态
    ///
    /// 通过交互式界面配置指定项目的 PR 创建和合并时的目标状态。
    /// 会从 Jira API 获取项目状态列表，然后让用户选择。
    ///
    /// # 参数
    ///
    /// * `jira_ticket_or_project` - Jira ticket ID（如 `"PROJ-123"`）或项目名称（如 `"PROJ"`）
    ///
    /// # 行为
    ///
    /// 1. 从 ticket 或项目名中提取项目名
    /// 2. 验证项目名格式
    /// 3. 从 Jira API 获取项目状态列表
    /// 4. 交互式选择 PR 创建时的状态
    /// 5. 交互式选择 PR 合并时的状态
    /// 6. 保存配置到本地文件
    ///
    /// # 错误
    ///
    /// 如果项目名格式无效、无法获取状态列表或保存配置失败，返回相应的错误信息。
    pub fn configure_interactive(jira_ticket_or_project: &str) -> Result<StatusConfigResult> {
        let project = extract_jira_project(jira_ticket_or_project);
        let project = if let Some(proj) = project {
            if proj.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                proj
            } else {
                anyhow::bail!(
                    "Invalid Jira project name format: '{}'. Jira project names should contain only ASCII letters, numbers, and underscores (e.g., 'PROJ', 'PROJ-123').",
                    proj
                );
            }
        } else if jira_ticket_or_project.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            jira_ticket_or_project
        } else {
            anyhow::bail!(
                "Invalid Jira ticket or project name: '{}'. Expected format: 'PROJ-123' (ticket) or 'PROJ' (project name). The input contains invalid characters (like '/', Chinese characters, etc.).",
                jira_ticket_or_project
            );
        };

        trace_debug!("Fetching status list for project: {}", project);
        let statuses = JiraProjectApi::get_project_statuses(project)
            .with_context(|| {
                format!(
                    "Failed to fetch project statuses for '{}'. Please check:\n  - The project name is correct\n  - The project exists in your Jira instance\n  - You have access to this project\n  - The project name format is correct (e.g., 'PROJ', not 'zw/修改打包脚本问题')",
                    project
                )
            })
            .context("Failed to get project statuses")?;

        if statuses.is_empty() {
            anyhow::bail!("No statuses found for project: {}", project);
        }

        let statuses_vec: Vec<String> = statuses.to_vec();
        let created_pull_request_status =
            SelectDialog::new("Select status for PR created", statuses_vec.clone())
                .prompt()
                .context("Failed to select status")?;

        let merged_pull_request_status =
            SelectDialog::new("Select status for PR merged", statuses_vec)
                .prompt()
                .context("Failed to select status")?;

        let jira_config = JiraStatusConfig {
            project: project.to_string(),
            created_pull_request_status: Some(created_pull_request_status.clone()),
            merged_pull_request_status: Some(merged_pull_request_status.clone()),
        };

        Self::write_status_config(&jira_config)
            .context("Failed to write Jira status configuration")?;

        trace_info!("Jira status configuration saved");
        trace_debug!("  PR created status: {}", created_pull_request_status);
        trace_debug!("  PR merged status: {}", merged_pull_request_status);

        Ok(StatusConfigResult {
            project: project.to_string(),
            created_pull_request_status,
            merged_pull_request_status,
        })
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
    pub fn read_pull_request_created_status(jira_ticket: &str) -> Result<Option<String>> {
        // 先验证 ticket 格式
        super::helpers::validate_jira_ticket_format(jira_ticket)
            .context("Invalid Jira ticket format")?;

        let project = extract_jira_project(jira_ticket)
            .ok_or_else(|| anyhow::anyhow!("Invalid Jira ticket format: cannot extract project"))?;

        let config = Self::read_status_config(project)
            .context("Failed to read Jira status configuration")?;

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
    pub fn read_pull_request_merged_status(jira_ticket: &str) -> Result<Option<String>> {
        // 先验证 ticket 格式
        super::helpers::validate_jira_ticket_format(jira_ticket)
            .context("Invalid Jira ticket format")?;

        let project = extract_jira_project(jira_ticket)
            .ok_or_else(|| anyhow::anyhow!("Invalid Jira ticket format: cannot extract project"))?;

        let config = Self::read_status_config(project)
            .context("Failed to read Jira status configuration")?;

        Ok(config.merged_pull_request_status)
    }

    /// 读取 Jira 状态配置（内部方法）
    ///
    /// 从配置文件中读取指定项目的状态配置。
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
    fn read_status_config(project: &str) -> Result<JiraStatusConfig> {
        let config_path = Paths::jira_status_config()?;
        let manager = ConfigManager::<JiraStatusMap>::new(config_path);

        let status_map = manager.read()?;
        let project_config = status_map.get(project);

        Ok(JiraStatusConfig {
            project: project.to_string(),
            created_pull_request_status: project_config
                .and_then(|c| c.created_pull_request_status.clone()),
            merged_pull_request_status: project_config
                .and_then(|c| c.merged_pull_request_status.clone()),
        })
    }

    /// 写入 Jira 状态配置（内部方法）
    ///
    /// 将状态配置写入配置文件。
    /// 如果项目配置已存在，则更新；如果不存在，则创建新配置。
    ///
    /// # 参数
    ///
    /// * `config` - Jira 状态配置结构体
    ///
    /// # 行为
    ///
    /// 1. 读取现有的状态配置（如果文件存在）
    /// 2. 更新或插入项目配置
    /// 3. 将更新后的配置写入文件（使用 pretty print 格式）
    ///
    /// # 错误
    ///
    /// 如果读取或写入文件失败，返回相应的错误信息。
    fn write_status_config(config: &JiraStatusConfig) -> Result<()> {
        let config_path = Paths::jira_status_config()?;
        let manager = ConfigManager::<JiraStatusMap>::new(config_path);

        manager.update(|status_map| {
            status_map.insert(
                config.project.clone(),
                ProjectStatusConfig {
                    created_pull_request_status: config.created_pull_request_status.clone(),
                    merged_pull_request_status: config.merged_pull_request_status.clone(),
                },
            );
        })
    }
}
