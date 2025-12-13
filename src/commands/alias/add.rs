//! 别名添加命令
//!
//! 支持直接添加和交互式添加别名。

use crate::base::alias::{AliasManager, CommandsConfig};
use crate::base::dialog::{ConfirmDialog, InputDialog, SelectDialog};
use crate::{log_info, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};

/// 别名添加命令
pub struct AliasAddCommand;

impl AliasAddCommand {
    /// 添加别名
    ///
    /// 支持两种模式：
    /// - 直接模式：提供 name 和 command 参数
    /// - 交互式模式：不提供参数，通过对话框输入
    pub fn add(name: Option<String>, command: Option<String>) -> Result<()> {
        let (alias_name, alias_command, is_direct_mode) =
            if let (Some(name), Some(cmd)) = (name, command) {
                // 直接添加模式
                (name, cmd, true)
            } else {
                // 交互式添加模式
                let aliases = AliasManager::list()?;

                // 步骤1：输入别名名称
                let name = InputDialog::new("Enter alias name")
                    .with_validator(|input: &str| {
                        let trimmed = input.trim();
                        if trimmed.is_empty() {
                            Err("Alias name cannot be empty".to_string())
                        } else if trimmed.contains(' ') {
                            Err("Alias name cannot contain spaces".to_string())
                        } else {
                            Ok(())
                        }
                    })
                    .prompt()
                    .wrap_err("Failed to get alias name")?;

                let name = name.trim().to_string();

                // 检查别名是否已存在
                if aliases.contains_key(&name) {
                    let should_overwrite = ConfirmDialog::new(format!(
                        "Alias '{}' already exists. Overwrite? (y/N)",
                        name
                    ))
                    .with_default(false)
                    .prompt()
                    .unwrap_or(false);

                    if !should_overwrite {
                        log_info!("Operation cancelled");
                        return Ok(());
                    }
                }

                // 步骤2：选择命令输入方式
                let input_method = SelectDialog::new(
                    "How do you want to enter the command?",
                    vec!["Select from common commands", "Enter manually"],
                )
                .prompt()
                .wrap_err("Failed to select input method")?;

                // 步骤3：获取命令
                let cmd = if input_method == "Select from common commands" {
                    // 从常用命令列表选择
                    let commands = CommandsConfig::get_common_commands()?;

                    SelectDialog::new("Select a command", commands)
                        .prompt()
                        .wrap_err("Failed to select command")?
                } else {
                    // 手动输入
                    InputDialog::new("Enter command")
                        .with_validator(|input: &str| {
                            if input.trim().is_empty() {
                                Err("Command cannot be empty".to_string())
                            } else {
                                Ok(())
                            }
                        })
                        .prompt()
                        .wrap_err("Failed to get command")?
                        .trim()
                        .to_string()
                };

                (name, cmd, false)
            };

        // 检查循环别名
        if AliasManager::check_circular(&alias_name, &alias_command)? {
            return Err(color_eyre::eyre::eyre!(
                "Circular alias detected: adding '{}' -> '{}' would create a circular reference",
                alias_name,
                alias_command
            ));
        }

        // 检查别名是否已存在（直接模式）
        if is_direct_mode && AliasManager::exists(&alias_name)? {
            let should_overwrite = ConfirmDialog::new(format!(
                "Alias '{}' already exists. Overwrite? (y/N)",
                alias_name
            ))
            .with_default(false)
            .prompt()
            .unwrap_or(false);

            if !should_overwrite {
                log_info!("Operation cancelled");
                return Ok(());
            }
        }

        // 保存别名
        AliasManager::add(&alias_name, &alias_command)?;
        log_success!(
            "Alias '{}' = '{}' added successfully",
            alias_name,
            alias_command
        );

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

        Ok(())
    }
}
