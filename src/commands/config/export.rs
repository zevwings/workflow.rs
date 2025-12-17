//! 配置导出命令
//! 导出配置文件用于备份和迁移

use crate::base::settings::paths::Paths;
use crate::base::settings::settings::Settings;
use crate::base::util::file::FileWriter;
use crate::commands::config::helpers::extract_section;
use crate::commands::config::validate::ConfigValidateCommand;
use crate::{log_error, log_info, log_message, log_success, log_warning};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
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
        let config_path =
            Paths::workflow_config().wrap_err("Failed to get workflow config path")?;

        if !config_path.exists() {
            return Err(eyre!(
                "Configuration file does not exist: {:?}",
                config_path
            ));
        }

        // 加载配置
        let settings = Settings::load();

        // 如果指定了 section，验证该 section 的配置
        if let Some(ref section_name) = section {
            let export_config = extract_section(&settings, section_name)?;
            log_info!("Validating configuration before export...");
            let validation_result =
                ConfigValidateCommand::validate_config(&export_config, &config_path)?;

            if !validation_result.errors.is_empty() {
                log_error!("Configuration validation failed");
                for error in &validation_result.errors {
                    log_message!("  - {}: {}", error.field, error.message);
                }
                return Err(eyre!(
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
        } else {
            // 导出完整配置时验证
            log_info!("Validating configuration before export...");
            let validation_result =
                ConfigValidateCommand::validate_config(&settings, &config_path)?;

            if !validation_result.errors.is_empty() {
                log_error!("Configuration validation failed");
                for error in &validation_result.errors {
                    log_message!("  - {}: {}", error.field, error.message);
                }
                return Err(eyre!(
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
        }

        // 过滤敏感信息（只在需要时处理，save_config 会处理 section 提取）
        let filtered_count = if no_secrets {
            if let Some(ref _section_name) = section {
                // 对于 section 导出，在 save_config 中处理过滤
                0 // 将在 save_config 中计算
            } else {
                let (_, count) = Self::filter_secrets(settings.clone());
                count
            }
        } else {
            0
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

        // 导出到文件（直接使用原始 settings，save_config 会处理 section 提取和过滤）
        Self::save_config(
            &settings,
            section.as_deref(),
            &output_path,
            format,
            no_secrets,
        )?;

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

        // 如果指定了 section，需要重新计算 filtered_count
        let actual_filtered_count = if no_secrets {
            if let Some(ref section_name) = section {
                // 对于 section 导出，计算该 section 中被过滤的字段数
                let export_config = extract_section(&settings, section_name)?;
                let (_, count) = Self::filter_secrets(export_config);
                count
            } else {
                filtered_count
            }
        } else {
            0
        };

        if actual_filtered_count > 0 {
            log_warning!(
                "Sensitive information has been filtered ({} field(s))",
                actual_filtered_count
            );
        }

        Ok(())
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

        // 过滤 LLM keys（所有 provider）
        if filtered.llm.openai.key.is_some() {
            filtered.llm.openai.key = Some("***FILTERED***".to_string());
            count += 1;
        }
        if filtered.llm.deepseek.key.is_some() {
            filtered.llm.deepseek.key = Some("***FILTERED***".to_string());
            count += 1;
        }
        if filtered.llm.proxy.key.is_some() {
            filtered.llm.proxy.key = Some("***FILTERED***".to_string());
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
    fn save_config(
        config: &Settings,
        section: Option<&str>,
        path: &PathBuf,
        format: &str,
        no_secrets: bool,
    ) -> Result<()> {
        // 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .wrap_err(format!("Failed to create directory: {:?}", parent))?;
        }

        let content = match format.to_lowercase().as_str() {
            "toml" => {
                // 如果指定了 section，只序列化该 section
                if let Some(section_name) = section {
                    // 直接从原始 config 中提取 section，然后过滤（如果需要），并序列化
                    let section_str = match section_name.to_lowercase().as_str() {
                        "jira" => {
                            let mut jira = config.jira.clone();
                            if no_secrets && jira.api_token.is_some() {
                                jira.api_token = Some("***FILTERED***".to_string());
                            }
                            toml::to_string_pretty(&jira)
                                .wrap_err("Failed to serialize jira config to TOML")?
                        }
                        "github" => {
                            let mut github = config.github.clone();
                            if no_secrets {
                                for account in &mut github.accounts {
                                    if !account.api_token.is_empty() {
                                        account.api_token = "***FILTERED***".to_string();
                                    }
                                }
                            }
                            toml::to_string_pretty(&github)
                                .wrap_err("Failed to serialize github config to TOML")?
                        }
                        "log" => toml::to_string_pretty(&config.log)
                            .wrap_err("Failed to serialize log config to TOML")?,
                        "llm" => {
                            let mut llm = config.llm.clone();
                            if no_secrets {
                                if llm.openai.key.is_some() {
                                    llm.openai.key = Some("***FILTERED***".to_string());
                                }
                                if llm.deepseek.key.is_some() {
                                    llm.deepseek.key = Some("***FILTERED***".to_string());
                                }
                                if llm.proxy.key.is_some() {
                                    llm.proxy.key = Some("***FILTERED***".to_string());
                                }
                            }
                            toml::to_string_pretty(&llm)
                                .wrap_err("Failed to serialize llm config to TOML")?
                        }
                        _ => {
                            return Err(eyre!("Unknown section: '{}'", section_name));
                        }
                    };

                    // 解析为 toml::Value，然后构建包含 section 标题的 TOML
                    let section_value: toml::Value = toml::from_str(&section_str)
                        .wrap_err("Failed to parse section config as TOML value")?;

                    // 构建包含 section 标题的 TOML 表
                    let mut table = toml::map::Map::new();
                    table.insert(section_name.to_string(), section_value);
                    let root_table = toml::Value::Table(table);

                    toml::to_string_pretty(&root_table)
                        .wrap_err("Failed to serialize section config to TOML")?
                } else {
                    // 导出完整配置
                    let config_to_serialize = if no_secrets {
                        let (filtered, _) = Self::filter_secrets(config.clone());
                        filtered
                    } else {
                        config.clone()
                    };
                    toml::to_string_pretty(&config_to_serialize)
                        .wrap_err("Failed to serialize config to TOML")?
                }
            }
            "json" => {
                // 如果指定了 section，只序列化该 section
                if let Some(section_name) = section {
                    let json_value = match section_name.to_lowercase().as_str() {
                        "jira" => {
                            let mut jira = config.jira.clone();
                            if no_secrets && jira.api_token.is_some() {
                                jira.api_token = Some("***FILTERED***".to_string());
                            }
                            serde_json::to_value(&jira)
                                .wrap_err("Failed to serialize jira config to JSON")?
                        }
                        "github" => {
                            let mut github = config.github.clone();
                            if no_secrets {
                                for account in &mut github.accounts {
                                    if !account.api_token.is_empty() {
                                        account.api_token = "***FILTERED***".to_string();
                                    }
                                }
                            }
                            serde_json::to_value(&github)
                                .wrap_err("Failed to serialize github config to JSON")?
                        }
                        "log" => serde_json::to_value(&config.log)
                            .wrap_err("Failed to serialize log config to JSON")?,
                        "llm" => {
                            let mut llm = config.llm.clone();
                            if no_secrets {
                                if llm.openai.key.is_some() {
                                    llm.openai.key = Some("***FILTERED***".to_string());
                                }
                                if llm.deepseek.key.is_some() {
                                    llm.deepseek.key = Some("***FILTERED***".to_string());
                                }
                                if llm.proxy.key.is_some() {
                                    llm.proxy.key = Some("***FILTERED***".to_string());
                                }
                            }
                            serde_json::to_value(&llm)
                                .wrap_err("Failed to serialize llm config to JSON")?
                        }
                        _ => {
                            return Err(eyre!("Unknown section: '{}'", section_name));
                        }
                    };
                    serde_json::to_string_pretty(&json_value).wrap_err("Failed to format JSON")?
                } else {
                    let config_to_serialize = if no_secrets {
                        let (filtered, _) = Self::filter_secrets(config.clone());
                        filtered
                    } else {
                        config.clone()
                    };
                    serde_json::to_string_pretty(&config_to_serialize)
                        .wrap_err("Failed to serialize config to JSON")?
                }
            }
            "yaml" | "yml" => {
                // 如果指定了 section，只序列化该 section
                if let Some(section_name) = section {
                    match section_name.to_lowercase().as_str() {
                        "jira" => {
                            let mut jira = config.jira.clone();
                            if no_secrets && jira.api_token.is_some() {
                                jira.api_token = Some("***FILTERED***".to_string());
                            }
                            serde_saphyr::to_string(&jira)
                                .wrap_err("Failed to serialize jira config to YAML")?
                        }
                        "github" => {
                            let mut github = config.github.clone();
                            if no_secrets {
                                for account in &mut github.accounts {
                                    if !account.api_token.is_empty() {
                                        account.api_token = "***FILTERED***".to_string();
                                    }
                                }
                            }
                            serde_saphyr::to_string(&github)
                                .wrap_err("Failed to serialize github config to YAML")?
                        }
                        "log" => serde_saphyr::to_string(&config.log)
                            .wrap_err("Failed to serialize log config to YAML")?,
                        "llm" => {
                            let mut llm = config.llm.clone();
                            if no_secrets {
                                if llm.openai.key.is_some() {
                                    llm.openai.key = Some("***FILTERED***".to_string());
                                }
                                if llm.deepseek.key.is_some() {
                                    llm.deepseek.key = Some("***FILTERED***".to_string());
                                }
                                if llm.proxy.key.is_some() {
                                    llm.proxy.key = Some("***FILTERED***".to_string());
                                }
                            }
                            serde_saphyr::to_string(&llm)
                                .wrap_err("Failed to serialize llm config to YAML")?
                        }
                        _ => {
                            return Err(eyre!("Unknown section: '{}'", section_name));
                        }
                    }
                } else {
                    let config_to_serialize = if no_secrets {
                        let (filtered, _) = Self::filter_secrets(config.clone());
                        filtered
                    } else {
                        config.clone()
                    };
                    serde_saphyr::to_string(&config_to_serialize)
                        .wrap_err("Failed to serialize config to YAML")?
                }
            }
            _ => {
                return Err(eyre!(
                    "Unsupported format: '{}'. Supported formats: toml, json, yaml",
                    format
                ));
            }
        };

        FileWriter::new(path)
            .write_str(&content)
            .wrap_err_with(|| format!("Failed to write config file: {:?}", path))?;

        Ok(())
    }
}
