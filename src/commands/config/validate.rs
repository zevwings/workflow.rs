//! 配置验证命令
//! 验证配置文件的完整性和有效性

use crate::base::settings::paths::Paths;
use crate::base::settings::settings::Settings;
use crate::commands::config::helpers::parse_config;
use crate::{log_error, log_info, log_message, log_success, log_warning};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// 配置验证错误
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub fixable: bool,
    pub fix_suggestion: Option<String>,
}

/// 配置验证警告
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
}

/// 配置验证结果
#[derive(Debug)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

/// 配置验证命令
pub struct ConfigValidateCommand;

impl ConfigValidateCommand {
    /// 验证配置文件
    pub fn validate(config_path: Option<String>, fix: bool, strict: bool) -> Result<()> {
        // 确定要验证的配置文件路径
        let default_path =
            Paths::workflow_config().wrap_err("Failed to get workflow config path")?;
        let config_path = if let Some(path) = config_path {
            std::path::PathBuf::from(path)
        } else {
            default_path.clone()
        };

        if !config_path.exists() {
            log_warning!("Configuration file does not exist: {:?}", config_path);
            if config_path == default_path {
                log_info!("Run 'workflow setup' to initialize configuration.");
            }
            return Ok(());
        }

        // 读取配置文件内容
        let content = fs::read_to_string(&config_path)
            .wrap_err(format!("Failed to read config file: {:?}", config_path))?;

        // 解析配置文件（支持 TOML、JSON、YAML）
        let mut settings = parse_config(&content, &config_path)?;

        // 执行验证
        let mut result = Self::validate_config(&settings, &config_path)?;

        // 如果有修复选项，尝试自动修复
        if fix && !result.errors.is_empty() {
            let fixed_count = Self::attempt_fixes(&mut result, &config_path, &mut settings)?;
            if fixed_count > 0 {
                log_warning!("Found {} issue(s), fixed automatically:", fixed_count);
                // 重新验证以获取更新后的错误列表
                result = Self::validate_config(&settings, &config_path)?;
            }
        }

        // 显示验证结果
        Self::print_validation_result(&result, strict)?;

        // 如果有错误（或在严格模式下有警告），返回错误
        if !result.errors.is_empty() || (strict && !result.warnings.is_empty()) {
            if !fix {
                log_info!("\nRun 'workflow config validate --fix' to attempt automatic fixes.");
            }
            std::process::exit(1);
        }

        log_success!("Configuration file is valid");
        Ok(())
    }

    /// 验证配置
    pub fn validate_config(
        settings: &Settings,
        _config_path: &std::path::Path,
    ) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 验证 JIRA 配置
        if let (Some(ref email), Some(ref api_token), Some(ref service_address)) = (
            &settings.jira.email,
            &settings.jira.api_token,
            &settings.jira.service_address,
        ) {
            // 验证邮箱格式
            if !email.contains('@') {
                errors.push(ValidationError {
                    field: "jira.email".to_string(),
                    message: format!("Invalid email format: '{}'", email),
                    fixable: false,
                    fix_suggestion: None,
                });
            }

            // 验证 URL 格式
            if !service_address.starts_with("http://") && !service_address.starts_with("https://") {
                errors.push(ValidationError {
                    field: "jira.service_address".to_string(),
                    message: format!(
                        "Invalid URL format: '{}' (must start with http:// or https://)",
                        service_address
                    ),
                    fixable: true,
                    fix_suggestion: Some(format!(
                        "Updated 'jira.service_address' from '{}' to 'https://{}'",
                        service_address,
                        service_address
                            .trim_start_matches("http://")
                            .trim_start_matches("https://")
                    )),
                });
            }

            // 验证 API token 不为空
            if api_token.is_empty() {
                errors.push(ValidationError {
                    field: "jira.api_token".to_string(),
                    message: "API token cannot be empty".to_string(),
                    fixable: false,
                    fix_suggestion: None,
                });
            }
        }

        // 验证 GitHub 配置
        if !settings.github.accounts.is_empty() {
            for (idx, account) in settings.github.accounts.iter().enumerate() {
                // 验证账号名称不为空
                if account.name.is_empty() {
                    errors.push(ValidationError {
                        field: format!("github.accounts[{}].name", idx),
                        message: "Account name cannot be empty".to_string(),
                        fixable: false,
                        fix_suggestion: None,
                    });
                }

                // 验证邮箱格式
                if !account.email.contains('@') {
                    errors.push(ValidationError {
                        field: format!("github.accounts[{}].email", idx),
                        message: format!("Invalid email format: '{}'", account.email),
                        fixable: false,
                        fix_suggestion: None,
                    });
                }

                // 验证 API token 不为空
                if account.api_token.is_empty() {
                    errors.push(ValidationError {
                        field: format!("github.accounts[{}].api_token", idx),
                        message: "API token cannot be empty".to_string(),
                        fixable: false,
                        fix_suggestion: None,
                    });
                }
            }

            // 检查 current 账号是否存在
            if let Some(ref current) = settings.github.current {
                if !settings.github.accounts.iter().any(|acc| acc.name == *current) {
                    warnings.push(ValidationWarning {
                        field: "github.current".to_string(),
                        message: format!(
                            "Current account '{}' does not exist in accounts list",
                            current
                        ),
                    });
                }
            }
        }

        // 验证 LLM 配置
        let valid_providers = ["openai", "deepseek", "proxy"];
        if !valid_providers.contains(&settings.llm.provider.as_str()) {
            errors.push(ValidationError {
                field: "llm.provider".to_string(),
                message: format!(
                    "Invalid provider: '{}' (expected: {})",
                    settings.llm.provider,
                    valid_providers.join(", ")
                ),
                fixable: false,
                fix_suggestion: None,
            });
        }

        // 验证当前 provider 的配置
        match settings.llm.provider.as_str() {
            "openai" => {
                // OpenAI 需要 key（可选，但建议配置）
                // model 可选，有默认值
            }
            "deepseek" => {
                // DeepSeek 需要 key（可选，但建议配置）
                // model 可选，有默认值
            }
            "proxy" => {
                // Proxy 需要 URL
                if settings.llm.proxy.url.is_none()
                    || settings.llm.proxy.url.as_ref().map(|s| s.is_empty()).unwrap_or(true)
                {
                    errors.push(ValidationError {
                        field: "llm.proxy.url".to_string(),
                        message: "LLM proxy URL is required when provider is 'proxy'".to_string(),
                        fixable: false,
                        fix_suggestion: None,
                    });
                }

                // Proxy 需要 model
                if settings.llm.proxy.model.is_none() {
                    errors.push(ValidationError {
                        field: "llm.proxy.model".to_string(),
                        message: "LLM model is required when provider is 'proxy'".to_string(),
                        fixable: false,
                        fix_suggestion: None,
                    });
                }
            }
            _ => {}
        }

        // 验证所有 provider 的 key（建议配置，但不强制）
        // 这里只验证当前 provider 的 key，其他 provider 的 key 是可选的
        let current = settings.llm.current_provider();
        if current.key.is_none() || current.key.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
            warnings.push(ValidationWarning {
                field: format!("llm.{}.key", settings.llm.provider),
                message: format!(
                    "LLM API key is not configured for provider '{}'. Some features may not work.",
                    settings.llm.provider
                ),
            });
        }

        Ok(ValidationResult { errors, warnings })
    }

    /// 尝试自动修复配置错误
    fn attempt_fixes(
        result: &mut ValidationResult,
        config_path: &Path,
        settings: &mut Settings,
    ) -> Result<usize> {
        let mut fixed_count = 0;
        let mut backup_created = false;
        let mut backup_path = None;

        // 创建备份（如果配置文件存在）
        if config_path.exists() {
            match Self::create_backup(config_path) {
                Ok(backup) => {
                    backup_path = Some(backup.clone());
                    backup_created = true;
                    log_info!("Backup created: {:?}", backup);
                }
                Err(e) => {
                    log_warning!("Failed to create backup: {}. Continuing without backup.", e);
                }
            }
        }

        // 修复可修复的错误
        let fixable_errors: Vec<_> = result.errors.iter().filter(|e| e.fixable).cloned().collect();

        for error in fixable_errors {
            let fixed = Self::apply_fix(settings, &error)?;
            if fixed {
                fixed_count += 1;
                if let Some(ref suggestion) = error.fix_suggestion {
                    log_info!("  - Fixed: {}", suggestion);
                } else {
                    log_info!("  - Fixed: {}", error.field);
                }
            }
        }

        // 如果修复了错误，保存配置文件
        if fixed_count > 0 {
            match Self::save_config(settings, config_path) {
                Ok(_) => {
                    log_success!("Fixed {} issue(s) and saved configuration", fixed_count);
                    // 删除备份（修复成功）
                    if let Some(backup) = backup_path {
                        fs::remove_file(&backup).ok();
                    }
                }
                Err(e) => {
                    log_error!("Failed to save fixed configuration: {}", e);
                    // 保存失败，尝试恢复备份
                    if backup_created {
                        if let Some(backup) = backup_path {
                            if let Err(restore_err) =
                                Self::restore_from_backup(&backup, config_path)
                            {
                                log_error!("Failed to restore from backup: {}", restore_err);
                                log_error!("Backup file is available at: {:?}", backup);
                            } else {
                                log_success!("Restored original configuration from backup");
                            }
                        }
                    }
                    return Err(e.wrap_err("Failed to save fixed configuration"));
                }
            }
        } else if backup_created {
            // 如果没有修复任何错误，删除备份
            if let Some(backup) = backup_path {
                fs::remove_file(&backup).ok();
            }
        }

        // 从错误列表中移除已修复的错误
        result.errors.retain(|e| !e.fixable || !Self::can_fix_error(e));

        Ok(fixed_count)
    }

    /// 应用修复
    fn apply_fix(settings: &mut Settings, error: &ValidationError) -> Result<bool> {
        match error.field.as_str() {
            "jira.service_address" => Self::fix_jira_url(settings, error),
            "github.current" => Self::fix_github_current(settings, error),
            _ => Ok(false),
        }
    }

    /// 检查是否可以修复错误
    fn can_fix_error(error: &ValidationError) -> bool {
        matches!(
            error.field.as_str(),
            "jira.service_address" | "github.current"
        )
    }

    /// 修复 JIRA URL 格式
    fn fix_jira_url(settings: &mut Settings, _error: &ValidationError) -> Result<bool> {
        if let Some(ref mut service_address) = settings.jira.service_address {
            if !service_address.starts_with("http://") && !service_address.starts_with("https://") {
                *service_address = format!("https://{}", service_address);
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// 修复 GitHub current 账号
    fn fix_github_current(settings: &mut Settings, _error: &ValidationError) -> Result<bool> {
        if let Some(ref current) = settings.github.current {
            // 检查 current 账号是否存在
            if !settings.github.accounts.iter().any(|acc| acc.name == *current) {
                // 如果不存在，设置为第一个账号或清除
                if let Some(first_account) = settings.github.accounts.first() {
                    settings.github.current = Some(first_account.name.clone());
                    return Ok(true);
                } else {
                    // 没有账号，清除 current
                    settings.github.current = None;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// 创建备份
    fn create_backup(config_path: &Path) -> Result<PathBuf> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let backup_filename = format!(
            "{}.backup.{}",
            config_path.file_name().and_then(|n| n.to_str()).unwrap_or("config"),
            timestamp
        );
        let backup_path = config_path
            .parent()
            .ok_or_else(|| eyre!("Config path has no parent directory: {:?}", config_path))?
            .join(backup_filename);

        fs::copy(config_path, &backup_path)
            .wrap_err(format!("Failed to create backup: {:?}", backup_path))?;

        Ok(backup_path)
    }

    /// 从备份恢复
    fn restore_from_backup(backup_path: &Path, config_path: &Path) -> Result<()> {
        fs::copy(backup_path, config_path)
            .wrap_err(format!("Failed to restore from backup: {:?}", backup_path))?;

        // 设置文件权限（Unix 系统）
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(config_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(config_path, perms)?;
        }

        Ok(())
    }

    /// 保存配置
    fn save_config(settings: &Settings, path: &Path) -> Result<()> {
        // 根据文件格式保存
        let content = match path.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::to_string_pretty(settings)
                .wrap_err("Failed to serialize config to JSON")?,
            Some("yaml") | Some("yml") => {
                serde_saphyr::to_string(settings).wrap_err("Failed to serialize config to YAML")?
            }
            _ => toml::to_string_pretty(settings).wrap_err("Failed to serialize config to TOML")?,
        };

        fs::write(path, content).wrap_err(format!("Failed to write config file: {:?}", path))?;

        // 设置文件权限（Unix 系统）
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    /// 打印验证结果
    fn print_validation_result(result: &ValidationResult, strict: bool) -> Result<()> {
        if result.errors.is_empty() && result.warnings.is_empty() {
            return Ok(());
        }

        if !result.errors.is_empty() {
            log_error!("Configuration validation failed\n");
            log_message!("Errors:");
            for error in &result.errors {
                log_message!(
                    "  - Missing or invalid field: '{}' - {}",
                    error.field,
                    error.message
                );
            }
        }

        if !result.warnings.is_empty() {
            if strict {
                log_error!("\nWarnings (treated as errors in strict mode):");
            } else {
                log_warning!("\nWarnings:");
            }
            for warning in &result.warnings {
                log_message!("  - {}: {}", warning.field, warning.message);
            }
        }

        Ok(())
    }
}
