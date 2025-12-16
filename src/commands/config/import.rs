//! 配置导入命令
//! 从文件导入配置（合并或覆盖模式）

use crate::base::settings::paths::Paths;
use crate::base::settings::settings::Settings;
use crate::commands::config::helpers::{extract_section, parse_config};
use crate::commands::config::validate::ConfigValidateCommand;
use crate::{log_error, log_info, log_message, log_success, log_warning};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// 导入事务结构
/// 用于管理导入过程中的备份和回滚
struct ImportTransaction {
    backup_path: PathBuf,
    original_config: Settings,
    config_path: PathBuf,
}

impl ImportTransaction {
    /// 创建新的事务
    fn new(config_path: PathBuf) -> Result<Self> {
        let original_config = if config_path.exists() {
            Settings::load()
        } else {
            Settings::default()
        };

        // 创建备份
        let backup_path = Self::create_backup(&config_path)?;

        Ok(Self {
            backup_path,
            original_config,
            config_path,
        })
    }

    /// 回滚到原始配置
    fn rollback(&self) -> Result<()> {
        // 从备份恢复
        fs::copy(&self.backup_path, &self.config_path).wrap_err("Failed to restore from backup")?;

        // 设置文件权限（Unix 系统）
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&self.config_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&self.config_path, perms)?;
        }

        // 验证恢复的配置
        let restored = Settings::load();
        let validation = ConfigValidateCommand::validate_config(&restored, &self.config_path)?;

        if !validation.errors.is_empty() {
            log_warning!("Warning: Restored configuration has validation errors");
            for error in &validation.errors {
                log_message!("  - {}: {}", error.field, error.message);
            }
        }

        log_success!("Configuration restored from backup: {:?}", self.backup_path);
        Ok(())
    }

    /// 提交事务（删除备份）
    fn commit(&self) -> Result<()> {
        if self.backup_path.exists() {
            fs::remove_file(&self.backup_path).wrap_err("Failed to remove backup file")?;
        }
        Ok(())
    }

    /// 创建备份文件
    fn create_backup(config_path: &Path) -> Result<PathBuf> {
        if !config_path.exists() {
            return Err(eyre!(
                "Configuration file does not exist: {:?}",
                config_path
            ));
        }

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let backup_filename = format!("config.backup.{}.toml", timestamp);
        let backup_path = config_path
            .parent()
            .ok_or_else(|| eyre!("Config path has no parent directory: {:?}", config_path))?
            .join(backup_filename);

        fs::copy(config_path, &backup_path)
            .wrap_err(format!("Failed to create backup: {:?}", backup_path))?;

        Ok(backup_path)
    }
}

/// 配置导入命令
pub struct ConfigImportCommand;

impl ConfigImportCommand {
    /// 导入配置文件
    pub fn import(
        input_path: String,
        overwrite: bool,
        section: Option<String>,
        dry_run: bool,
    ) -> Result<()> {
        let input_path = PathBuf::from(input_path);

        if !input_path.exists() {
            return Err(eyre!("Input file does not exist: {:?}", input_path));
        }

        // 读取并解析输入文件
        let content = fs::read_to_string(&input_path)
            .wrap_err(format!("Failed to read input file: {:?}", input_path))?;

        let imported_settings = parse_config(&content, &input_path)?;

        // 提取要导入的配置段
        let imported = if let Some(ref section_name) = section {
            extract_section(&imported_settings, section_name)?
        } else {
            imported_settings
        };

        // 验证导入的配置
        let validation_result = ConfigValidateCommand::validate_config(&imported, &input_path)?;
        if !validation_result.errors.is_empty() {
            log_error!("Configuration validation failed");
            for error in &validation_result.errors {
                log_message!("  - {}: {}", error.field, error.message);
            }
            return Err(eyre!("Import cancelled. Configuration validation failed."));
        }

        if dry_run {
            log_info!("Dry run mode - previewing changes:");
            Self::preview_changes(&imported, section.as_deref())?;
            log_info!("\nNo changes were made. Remove --dry-run to apply changes.");
            return Ok(());
        }

        // 获取配置文件路径
        let current_config_path =
            Paths::workflow_config().wrap_err("Failed to get workflow config path")?;

        // 创建事务（包含备份）
        let transaction = ImportTransaction::new(current_config_path.clone())?;
        log_success!(
            "Configuration backup created: {:?}",
            transaction.backup_path
        );

        // 加载当前配置
        let current_settings = transaction.original_config.clone();

        // 执行导入（合并或覆盖）
        let final_settings = if overwrite {
            if let Some(ref section_name) = section {
                Self::merge_section(&current_settings, &imported, section_name)
            } else {
                imported.clone()
            }
        } else if let Some(ref section_name) = section {
            Self::merge_section(&current_settings, &imported, section_name)
        } else {
            Self::merge_configs(&current_settings, &imported)
        };

        // 验证最终配置
        let final_validation =
            ConfigValidateCommand::validate_config(&final_settings, &current_config_path)?;
        if !final_validation.errors.is_empty() {
            log_error!("Configuration validation failed after import");
            for error in &final_validation.errors {
                log_message!("  - {}: {}", error.field, error.message);
            }
            // 自动回滚
            match transaction.rollback() {
                Ok(_) => {
                    log_success!("Successfully rolled back to original configuration");
                }
                Err(e) => {
                    log_error!("Failed to rollback configuration: {}", e);
                    log_error!("Backup file is available at: {:?}", transaction.backup_path);
                    log_error!("Please manually restore from backup if needed");
                }
            }
            return Err(eyre!("Import cancelled. Configuration validation failed."));
        }

        // 保存配置
        if let Err(e) = Self::save_config(&final_settings, &current_config_path) {
            log_error!("Failed to save configuration: {}", e);
            // 保存失败，回滚
            match transaction.rollback() {
                Ok(_) => {
                    log_success!("Successfully rolled back due to save failure");
                }
                Err(rollback_err) => {
                    log_error!("Failed to rollback: {}", rollback_err);
                    log_error!("Backup file is available at: {:?}", transaction.backup_path);
                }
            }
            return Err(e.wrap_err("Failed to save configuration"));
        }

        // 验证保存后的配置（确保文件写入正确）
        if !Self::verify_saved_config(&current_config_path)? {
            log_error!("Post-save validation failed");
            // 保存后验证失败，回滚
            match transaction.rollback() {
                Ok(_) => {
                    log_success!("Successfully rolled back due to post-save validation failure");
                }
                Err(e) => {
                    log_error!("Failed to rollback: {}", e);
                    log_error!("Backup file is available at: {:?}", transaction.backup_path);
                }
            }
            return Err(eyre!("Post-save validation failed"));
        }

        // 提交事务（删除备份）
        transaction.commit()?;

        // 显示结果
        if overwrite {
            log_success!("Configuration imported successfully (overwrite mode)");
        } else {
            log_success!("Configuration imported successfully (merge mode)");
            Self::show_changes(&current_settings, &final_settings, section.as_deref())?;
        }

        Ok(())
    }

    /// 合并配置段
    fn merge_section(current: &Settings, imported: &Settings, section: &str) -> Settings {
        let mut merged = current.clone();

        match section.to_lowercase().as_str() {
            "jira" => {
                merged.jira = imported.jira.clone();
            }
            "github" => {
                merged.github = imported.github.clone();
            }
            "log" => {
                merged.log = imported.log.clone();
            }
            "llm" => {
                merged.llm = imported.llm.clone();
            }
            _ => {}
        }

        merged
    }

    /// 深度合并配置
    fn merge_configs(current: &Settings, imported: &Settings) -> Settings {
        let mut merged = current.clone();

        // 合并 JIRA 配置
        if imported.jira.email.is_some() {
            merged.jira.email = imported.jira.email.clone();
        }
        if imported.jira.api_token.is_some() {
            merged.jira.api_token = imported.jira.api_token.clone();
        }
        if imported.jira.service_address.is_some() {
            merged.jira.service_address = imported.jira.service_address.clone();
        }

        // 合并 GitHub 配置（完全替换）
        if !imported.github.accounts.is_empty() {
            merged.github.accounts = imported.github.accounts.clone();
        }
        if imported.github.current.is_some() {
            merged.github.current = imported.github.current.clone();
        }

        // 合并日志配置
        merged.log.output_folder_name = imported.log.output_folder_name.clone();
        if imported.log.download_base_dir.is_some() {
            merged.log.download_base_dir = imported.log.download_base_dir.clone();
        }
        if imported.log.level.is_some() {
            merged.log.level = imported.log.level.clone();
        }
        if imported.log.enable_trace_console.is_some() {
            merged.log.enable_trace_console = imported.log.enable_trace_console;
        }

        // 合并 LLM 配置
        merged.llm.provider = imported.llm.provider.clone();
        if !imported.llm.language.is_empty() {
            merged.llm.language = imported.llm.language.clone();
        }
        // 合并各 provider 的配置
        if imported.llm.openai.key.is_some() {
            merged.llm.openai.key = imported.llm.openai.key.clone();
        }
        if imported.llm.openai.model.is_some() {
            merged.llm.openai.model = imported.llm.openai.model.clone();
        }
        if imported.llm.deepseek.key.is_some() {
            merged.llm.deepseek.key = imported.llm.deepseek.key.clone();
        }
        if imported.llm.deepseek.model.is_some() {
            merged.llm.deepseek.model = imported.llm.deepseek.model.clone();
        }
        if imported.llm.proxy.url.is_some() {
            merged.llm.proxy.url = imported.llm.proxy.url.clone();
        }
        if imported.llm.proxy.key.is_some() {
            merged.llm.proxy.key = imported.llm.proxy.key.clone();
        }
        if imported.llm.proxy.model.is_some() {
            merged.llm.proxy.model = imported.llm.proxy.model.clone();
        }

        merged
    }

    /// 验证保存后的配置
    fn verify_saved_config(config_path: &Path) -> Result<bool> {
        // 重新读取配置文件
        let content = fs::read_to_string(config_path).wrap_err(format!(
            "Failed to read saved config file: {:?}",
            config_path
        ))?;

        let saved_settings = parse_config(&content, config_path)?;

        // 验证配置
        let validation = ConfigValidateCommand::validate_config(&saved_settings, config_path)?;

        if !validation.errors.is_empty() {
            log_warning!("Post-save validation found errors:");
            for error in &validation.errors {
                log_message!("  - {}: {}", error.field, error.message);
            }
            return Ok(false);
        }

        Ok(true)
    }

    /// 保存配置
    fn save_config(settings: &Settings, path: &PathBuf) -> Result<()> {
        use crate::base::util::file::write_toml_file;
        write_toml_file(path, settings)?;

        // 设置文件权限（Unix 系统）
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    /// 预览变更
    fn preview_changes(imported: &Settings, section: Option<&str>) -> Result<()> {
        let current = Settings::load();

        if let Some(section_name) = section {
            log_message!("  Section: {}", section_name);
            match section_name.to_lowercase().as_str() {
                "jira" => {
                    if imported.jira.email.is_some() {
                        log_message!("    - jira.email: will be updated");
                    }
                    if imported.jira.service_address.is_some() {
                        log_message!("    - jira.service_address: will be updated");
                    }
                }
                "github" => {
                    if !imported.github.accounts.is_empty() {
                        log_message!(
                            "    - github.accounts: {} account(s) will be imported",
                            imported.github.accounts.len()
                        );
                    }
                }
                "log" => {
                    log_message!("    - log.output_folder_name: will be updated");
                }
                "llm" => {
                    if imported.llm.provider != current.llm.provider {
                        log_message!(
                            "    - llm.provider: {} -> {}",
                            current.llm.provider,
                            imported.llm.provider
                        );
                    }
                }
                _ => {}
            }
        } else {
            log_message!("  Full configuration will be imported");
        }

        Ok(())
    }

    /// 显示变更
    fn show_changes(
        current: &Settings,
        final_settings: &Settings,
        section: Option<&str>,
    ) -> Result<()> {
        let mut changes = Vec::new();

        if section.is_none() || section == Some("jira") {
            if current.jira.email != final_settings.jira.email {
                changes.push("  - Updated: jira.email".to_string());
            }
            if current.jira.service_address != final_settings.jira.service_address {
                changes.push("  - Updated: jira.service_address".to_string());
            }
        }

        if (section.is_none() || section == Some("github"))
            && current.github.accounts.len() != final_settings.github.accounts.len()
        {
            changes.push(format!(
                "  - Updated: github.accounts ({} account(s))",
                final_settings.github.accounts.len()
            ));
        }

        if (section.is_none() || section == Some("log"))
            && current.log.output_folder_name != final_settings.log.output_folder_name
        {
            changes.push("  - Updated: log.output_folder_name".to_string());
        }

        if section.is_none() || section == Some("llm") {
            if current.llm.provider != final_settings.llm.provider {
                changes.push("  - Updated: llm.provider".to_string());
            }
            if current.llm.language != final_settings.llm.language {
                changes.push("  - Updated: llm.language".to_string());
            }
            // 检查各 provider 的配置变更
            if current.llm.openai.key != final_settings.llm.openai.key
                || current.llm.openai.model != final_settings.llm.openai.model
            {
                changes.push("  - Updated: llm.openai".to_string());
            }
            if current.llm.deepseek.key != final_settings.llm.deepseek.key
                || current.llm.deepseek.model != final_settings.llm.deepseek.model
            {
                changes.push("  - Updated: llm.deepseek".to_string());
            }
            if current.llm.proxy.url != final_settings.llm.proxy.url
                || current.llm.proxy.key != final_settings.llm.proxy.key
                || current.llm.proxy.model != final_settings.llm.proxy.model
            {
                changes.push("  - Updated: llm.proxy".to_string());
            }
        }

        if !changes.is_empty() {
            log_info!("Changes applied:");
            for change in changes {
                log_message!("{}", change);
            }
        }

        Ok(())
    }
}
