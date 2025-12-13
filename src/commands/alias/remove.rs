//! 别名删除命令
//!
//! 支持直接删除和交互式多选删除别名。

use crate::base::alias::AliasManager;
use crate::base::dialog::{ConfirmDialog, MultiSelectDialog};
use crate::{log_info, log_message, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};

/// 别名删除命令
pub struct AliasRemoveCommand;

impl AliasRemoveCommand {
    /// 删除别名
    ///
    /// 支持两种模式：
    /// - 直接模式：提供 name 参数
    /// - 交互式模式：不提供参数，通过多选对话框选择
    pub fn remove(name: Option<String>) -> Result<()> {
        let aliases = AliasManager::list()?;

        if aliases.is_empty() {
            log_info!("No aliases defined");
            return Ok(());
        }

        let names_to_remove = if let Some(name) = name {
            // 直接删除模式
            if !aliases.contains_key(&name) {
                return Err(color_eyre::eyre::eyre!("Alias '{}' not found", name));
            }
            vec![name]
        } else {
            // 交互式选择模式
            let options: Vec<String> = aliases
                .iter()
                .map(|(alias_name, command)| format!("{} = {}", alias_name, command))
                .collect();

            let selected = MultiSelectDialog::new("Select aliases to remove", options)
                .prompt()
                .wrap_err("Failed to select aliases")?;

            if selected.is_empty() {
                log_info!("No aliases selected");
                return Ok(());
            }

            // 从选中的字符串中提取别名名称（格式：alias_name = command）
            selected
                .iter()
                .filter_map(|s| s.split('=').next().map(|n| n.trim().to_string()))
                .collect()
        };

        // 显示将要删除的别名
        log_message!("Aliases to be removed:");
        for name in &names_to_remove {
            if let Some(command) = aliases.get(name) {
                log_info!("  {} = {}", name, command);
            }
        }

        // 确认删除
        let confirmed = ConfirmDialog::new(format!(
            "Are you sure you want to remove {} alias/aliases?",
            names_to_remove.len()
        ))
        .with_default(false)
        .prompt()
        .wrap_err("Failed to get user confirmation")?;

        if !confirmed {
            log_info!("Operation cancelled");
            return Ok(());
        }

        // 批量删除
        let mut removed_count = 0;
        for name in &names_to_remove {
            match AliasManager::remove(name) {
                Ok(true) => {
                    log_success!("Alias '{}' removed successfully", name);
                    removed_count += 1;
                }
                Ok(false) => {
                    log_warning!("Alias '{}' not found (may have been removed already)", name);
                }
                Err(e) => {
                    log_warning!("Failed to remove alias '{}': {}", name, e);
                }
            }
        }

        if removed_count > 0 {
            log_success!("Successfully removed {} alias/aliases", removed_count);

            // 询问是否更新补全脚本
            let should_update = ConfirmDialog::new("Update completion scripts? (Y/n)")
                .with_default(true)
                .prompt()
                .unwrap_or(false);

            if should_update {
                match crate::Completion::generate_all_completions(None, None) {
                    Ok(_) => {
                        log_success!("Completion scripts updated successfully");
                    }
                    Err(e) => {
                        log_warning!("Failed to update completion scripts: {}", e);
                        log_info!(
                            "You can manually update them later with: workflow completion generate"
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
