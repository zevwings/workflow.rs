//! 仓库配置管理实现
//!
//! 提供 ProjectConfig、UserConfig 和 RepoConfig 的加载和保存功能。

use std::sync::Arc;

use domain::{
    BranchTemplates, CommitTemplates, ConfigError, MCPConfig, PathService, ProjectConfig,
    PullRequestsTemplates, RepoConfig, RepoConfigRepository, TemplateConfig, UserConfig,
};
use toml::{map::Map, Value};
use toolkit::{file, log_warn};

/// 仓库配置仓储实现
///
/// 实现 `RepoConfigRepository` trait，提供仓库配置的持久化操作。
pub(crate) struct RepoConfigRepositoryImpl {
    path_service: Arc<dyn PathService>,
}

impl RepoConfigRepositoryImpl {
    /// 创建新的仓库配置仓储实例
    pub fn new(path_service: Arc<dyn PathService>) -> Self {
        Self { path_service }
    }

    /// 清理模板配置中的默认值
    ///
    /// 将所有等于程序默认值的字段清除，避免保存默认值到配置文件。
    fn clean_template_defaults(&self, template: &mut TemplateConfig) {
        // engine 字段通过 skip_serializing 跳过序列化，不需要清理

        let default_branch = BranchTemplates::default();
        let default_commit = CommitTemplates::default();
        let default_pr = PullRequestsTemplates::default();

        // 使用宏来减少重复代码
        macro_rules! clean_if_default {
            ($field:expr, $default:expr) => {
                if $field == $default {
                    $field = String::new();
                }
            };
        }

        // 清理 branch templates
        clean_if_default!(template.branch.feature, default_branch.feature);
        clean_if_default!(template.branch.bugfix, default_branch.bugfix);
        clean_if_default!(template.branch.hotfix, default_branch.hotfix);
        clean_if_default!(template.branch.refactoring, default_branch.refactoring);
        clean_if_default!(template.branch.chore, default_branch.chore);

        // 清理 commit templates
        clean_if_default!(template.commit.message, default_commit.message);

        // 清理 PR templates
        clean_if_default!(template.pull_requests.title, default_pr.title);
        clean_if_default!(template.pull_requests.body, default_pr.body);
    }
}

impl RepoConfigRepository for RepoConfigRepositoryImpl {
    fn load_project_config(&self) -> Result<ProjectConfig, ConfigError> {
        let config_path = self.path_service.get_project_config_filepath().map_err(|e| {
            ConfigError::OperationFailed(format!("Failed to get project config path: {}", e))
        })?;

        if !config_path.exists() {
            return Ok(ProjectConfig::default());
        }

        let value: Value = file::read_toml(&config_path).map_err(|e| {
            ConfigError::OperationFailed(format!("Failed to read project config: {}", e))
        })?;

        let mut config = ProjectConfig::default();

        // 解析 use_scope
        if let Some(use_scope) = value.get("use_scope") {
            if let Some(use_scope_bool) = use_scope.as_bool() {
                config.use_scope = use_scope_bool;
            }
        }

        // 解析 template
        if let Some(template_section) = value.get("template") {
            let template_str = toml::to_string(template_section).map_err(|e| {
                ConfigError::OperationFailed(format!("Failed to serialize template section: {}", e))
            })?;
            config.template = toml::from_str(&template_str).map_err(|e| {
                ConfigError::OperationFailed(format!("Failed to parse template config: {}", e))
            })?;
        }

        Ok(config)
    }

    fn save_project_config(&self, config: &ProjectConfig) -> Result<(), ConfigError> {
        let config_path = self.path_service.get_project_config_filepath().map_err(|e| {
            ConfigError::OperationFailed(format!("Failed to get project config path: {}", e))
        })?;

        // 读取现有配置（如果存在）
        let mut existing_value: Value = if config_path.exists() {
            file::read_toml(&config_path).map_err(|e| {
                ConfigError::OperationFailed(format!("Failed to read existing config: {}", e))
            })?
        } else {
            Value::Table(Map::new())
        };

        // 更新 use_scope
        if let Some(table) = existing_value.as_table_mut() {
            if config.use_scope {
                table.insert("use_scope".to_string(), Value::Boolean(config.use_scope));
            } else {
                table.remove("use_scope");
            }

            // 更新 template（清理默认值后再序列化）
            let mut template_to_save = config.template.clone();
            self.clean_template_defaults(&mut template_to_save);

            let template_str = toml::to_string(&template_to_save).map_err(|e| {
                ConfigError::OperationFailed(format!("Failed to serialize template config: {}", e))
            })?;
            let mut template_value: Value = toml::from_str(&template_str).map_err(|e| {
                ConfigError::OperationFailed(format!("Failed to parse template config: {}", e))
            })?;

            // 检查并删除空的 branch、commit 和 pull_requests 节
            if let Some(template_table) = template_value.as_table_mut() {
                // 检查 branch 是否为空（所有字段都为空）
                if let Some(branch_value) = template_table.get("branch") {
                    if let Some(branch_table) = branch_value.as_table() {
                        if branch_table.is_empty() {
                            template_table.remove("branch");
                        }
                    }
                }

                // 检查 commit 是否为空（所有字段都为空）
                if let Some(commit_value) = template_table.get("commit") {
                    if let Some(commit_table) = commit_value.as_table() {
                        if commit_table.is_empty() {
                            template_table.remove("commit");
                        }
                    }
                }

                // 检查 pull_requests 是否为空（所有字段都为空）
                if let Some(pr_value) = template_table.get("pull_requests") {
                    if let Some(pr_table) = pr_value.as_table() {
                        if pr_table.is_empty() {
                            template_table.remove("pull_requests");
                        }
                    }
                }

                // 如果 template 为空（所有字段都是默认值），则删除 template 节
                if template_table.is_empty() {
                    table.remove("template");
                } else {
                    table.insert("template".to_string(), template_value);
                }
            } else {
                table.insert("template".to_string(), template_value);
            }
        }

        // 写入文件
        file::write_toml(&config_path, &existing_value).map_err(|e| {
            ConfigError::OperationFailed(format!("Failed to write project config: {}", e))
        })?;

        Ok(())
    }

    fn load_user_config(&self) -> Result<UserConfig, ConfigError> {
        let config_path = self.path_service.get_user_config_filepath().map_err(|e| {
            ConfigError::OperationFailed(format!("Failed to get user config path: {}", e))
        })?;

        // 如果文件不存在，返回默认配置
        if !config_path.exists() {
            return Ok(UserConfig::default());
        }

        // 直接解析整个文件为 UserConfig
        let config: UserConfig = file::read_toml(&config_path).map_err(|e| {
            ConfigError::OperationFailed(format!("Failed to read user config: {}", e))
        })?;

        Ok(config)
    }

    fn save_user_config(&self, config: &UserConfig) -> Result<(), ConfigError> {
        let config_path = self.path_service.get_user_config_filepath().map_err(|e| {
            ConfigError::OperationFailed(format!("Failed to get user config path: {}", e))
        })?;

        // 如果配置为空，删除文件（如果存在）并返回
        if config.is_empty() {
            if config_path.exists() {
                std::fs::remove_file(&config_path).map_err(|e| {
                    ConfigError::OperationFailed(format!(
                        "Failed to remove empty user config file: {}",
                        e
                    ))
                })?;
            }
            return Ok(());
        }

        // 直接序列化 UserConfig 并写入文件
        file::write_toml(&config_path, config).map_err(|e| {
            ConfigError::OperationFailed(format!("Failed to write user config: {}", e))
        })?;

        Ok(())
    }

    fn load(&self) -> Result<RepoConfig, ConfigError> {
        let project = self.load_project_config()?;
        let user = self.load_user_config()?;

        // 加载 MCP 配置
        let mcp_config_path =
            self.path_service.get_mcp_config_filepath().map_err(ConfigError::from)?;
        let mcp = if mcp_config_path.exists() {
            match file::read_json::<MCPConfig>(&mcp_config_path) {
                Ok(config) => config,
                Err(e) => {
                    // MCP 配置加载失败不影响其他配置，记录错误但使用默认值
                    log_warn!("Failed to load MCP config: {}", e);
                    MCPConfig::default()
                }
            }
        } else {
            MCPConfig::default()
        };

        Ok(RepoConfig { project, user, mcp })
    }

    fn save(&self, config: &RepoConfig) -> Result<(), ConfigError> {
        self.save_project_config(&config.project)?;
        self.save_user_config(&config.user)?;

        // 保存 MCP 配置（只有当有配置时才保存）
        if !config.mcp.mcp_servers.is_empty() {
            let mcp_config_path =
                self.path_service.get_mcp_config_filepath().map_err(ConfigError::from)?;
            file::write_json_secure(&mcp_config_path, &config.mcp).map_err(|e| {
                ConfigError::OperationFailed(format!("Failed to save MCP config: {}", e))
            })?;
        }

        Ok(())
    }
}
