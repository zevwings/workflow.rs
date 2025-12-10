//! 配置导出命令
//! 导出配置文件用于备份和迁移

use crate::base::settings::paths::Paths;
use crate::base::settings::settings::Settings;
use crate::commands::config::validate::ConfigValidateCommand;
use crate::{log_error, log_info, log_message, log_success, log_warning};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// 配置导出命令
pub struct ConfigExportCommand;

impl ConfigExportCommand {
    /// 导出配置文件
    pub fn export(
        output_path: String,
        section: Option<String>,
        no_secrets: bool,
        _toml: bool,
        json: bool,
        yaml: bool,
    ) -> Result<()> {
        // 确定导出格式（默认 toml）
        let format = if json {
            "json"
        } else if yaml {
            "yaml"
        } else {
            "toml" // 默认格式
        };
        let config_path = Paths::workflow_config().context("Failed to get workflow config path")?;

        if !config_path.exists() {
            return Err(anyhow::anyhow!(
                "Configuration file does not exist: {:?}",
                config_path
            ));
        }

        // 加载配置
        let settings = Settings::load();

        // 提取要导出的配置
        let export_config = if let Some(ref section_name) = section {
            Self::extract_section(&settings, section_name)?
        } else {
            settings
        };

        // 导出前验证配置有效性
        log_info!("Validating configuration before export...");
        let validation_result =
            ConfigValidateCommand::validate_config(&export_config, &config_path)?;

        if !validation_result.errors.is_empty() {
            log_error!("Configuration validation failed");
            for error in &validation_result.errors {
                log_message!("  - {}: {}", error.field, error.message);
            }
            return Err(anyhow::anyhow!(
                "Export cancelled. Please fix configuration errors before exporting."
            ));
        }

        if !validation_result.warnings.is_empty() {
            log_warning!("Configuration validation warnings:");
            for warning in &validation_result.warnings {
                log_message!("  - {}: {}", warning.field, warning.message);
            }
            log_info!("Continuing with export despite warnings...");
        }

        // 过滤敏感信息
        let (filtered_config, filtered_count) = if no_secrets {
            Self::filter_secrets(export_config)
        } else {
            (export_config, 0)
        };

        // 确定输出路径
        let mut output_path = PathBuf::from(&output_path);

        // 如果路径是目录，或者路径不存在且没有扩展名（可能是目录），自动生成文件名
        let is_directory = if output_path.exists() {
            output_path.is_dir()
        } else {
            // 路径不存在时，通过检查是否有扩展名来判断
            // 如果路径以 / 结尾，或者没有扩展名，认为是目录
            output_path.extension().is_none() || output_path.to_string_lossy().ends_with('/')
        };

        if is_directory {
            let filename = Self::generate_filename(section.as_deref(), format);
            output_path = output_path.join(filename);
        }

        // 导出到文件
        Self::save_config(&filtered_config, &output_path, format)?;

        // 显示结果
        if let Some(ref section_name) = section {
            log_success!(
                "{} configuration exported to {:?}",
                section_name,
                output_path
            );
        } else {
            log_success!("Configuration exported to {:?}", output_path);
        }

        if filtered_count > 0 {
            log_warning!(
                "Sensitive information has been filtered ({} field(s))",
                filtered_count
            );
        }

        Ok(())
    }

    /// 提取特定配置段
    fn extract_section(settings: &Settings, section: &str) -> Result<Settings> {
        let mut extracted = Settings::default();

        match section.to_lowercase().as_str() {
            "jira" => {
                extracted.jira = settings.jira.clone();
            }
            "github" => {
                extracted.github = settings.github.clone();
            }
            "log" => {
                extracted.log = settings.log.clone();
            }
            "llm" => {
                extracted.llm = settings.llm.clone();
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unknown section: '{}'. Valid sections: jira, github, log, llm",
                    section
                ));
            }
        }

        Ok(extracted)
    }

    /// 过滤敏感信息
    fn filter_secrets(settings: Settings) -> (Settings, usize) {
        let mut filtered = settings;
        let mut count = 0;

        // 过滤 JIRA API token
        if filtered.jira.api_token.is_some() {
            filtered.jira.api_token = Some("***FILTERED***".to_string());
            count += 1;
        }

        // 过滤 GitHub API tokens
        for account in &mut filtered.github.accounts {
            if !account.api_token.is_empty() {
                account.api_token = "***FILTERED***".to_string();
                count += 1;
            }
        }

        // 过滤 LLM key
        if filtered.llm.key.is_some() {
            filtered.llm.key = Some("***FILTERED***".to_string());
            count += 1;
        }

        (filtered, count)
    }

    /// 生成文件名
    fn generate_filename(section: Option<&str>, format: &str) -> String {
        let extension = match format {
            "json" => "json",
            "yaml" | "yml" => "yaml",
            _ => "toml", // 默认 toml
        };

        if let Some(section_name) = section {
            format!("config.{}.{}", section_name, extension)
        } else {
            format!("config.{}", extension)
        }
    }

    /// 保存配置到文件
    fn save_config(config: &Settings, path: &PathBuf, format: &str) -> Result<()> {
        // 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create directory: {:?}", parent))?;
        }

        let content = match format.to_lowercase().as_str() {
            "toml" => {
                toml::to_string_pretty(config).context("Failed to serialize config to TOML")?
            }
            "json" => serde_json::to_string_pretty(config)
                .context("Failed to serialize config to JSON")?,
            "yaml" | "yml" => {
                serde_yaml::to_string(config).context("Failed to serialize config to YAML")?
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported format: '{}'. Supported formats: toml, json, yaml",
                    format
                ));
            }
        };

        fs::write(path, content).context(format!("Failed to write config file: {:?}", path))?;

        Ok(())
    }
}
