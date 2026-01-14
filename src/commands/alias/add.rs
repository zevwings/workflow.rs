//! 别名添加命令
//!
//! 支持直接添加和交互式添加别名。

use crate::alias::{AliasManager, CommandsConfig};
use crate::prompt::{FormBuilder, GroupConfig, InputFormField, SelectFormField};
use crate::{info, success, warning};
use color_eyre::{eyre::WrapErr, Result};
use std::sync::Arc;

/// 别名添加命令
pub struct AliasAddCommand;

impl AliasAddCommand {
    /// 添加别名
    ///
    /// 支持两种模式：
    /// - 直接模式：提供 name 和 command 参数
    /// - 交互式模式：不提供参数，通过对话框输入
    pub fn add(name: Option<String>, command: Option<String>) -> Result<()> {
        let (alias_name, alias_command, is_direct_mode) = if let (Some(name), Some(cmd)) =
            (name, command)
        {
            // 直接添加模式
            (name, cmd, true)
        } else {
            // 交互式添加模式
            let aliases = AliasManager::list()?;

            // 收集别名名称
            let name = crate::input!("Enter alias name")
                .validator(|input: &str| {
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
                .wrap_err("Failed to get alias name")?
                .trim()
                .to_string();

            // 检查别名是否已存在
            if aliases.contains_key(&name) {
                let should_overwrite =
                    crate::confirm!("Alias '{}' already exists. Overwrite? (y/N)", name)
                        .default(false)
                        .prompt()
                        .unwrap_or(false);

                if !should_overwrite {
                    info!("Operation cancelled");
                    return Ok(());
                }
            }

            // 继续收集命令信息
            let command_form_result = FormBuilder::new()
                .add_group(
                    "command_input",
                    |g| {
                        g.add_step(|s| {
                            s.add_select(SelectFormField::new(
                                "input_method",
                                "How do you want to enter the command?",
                                vec![
                                    "Select from common commands".to_string(),
                                    "Enter manually".to_string(),
                                ],
                            ))
                        })
                        .step_if("input_method", "Select from common commands", |s| {
                            // 从常用命令列表选择
                            let commands = CommandsConfig::get_common_commands()
                                .unwrap_or_else(|_| Vec::new());
                            s.add_select(SelectFormField::new(
                                "selected_command",
                                "Select a command",
                                commands,
                            ))
                        })
                        .step_if("input_method", "Enter manually", |s| {
                            let field = InputFormField::new("manual_command", "Enter command")
                                .required()
                                .validator(Arc::new(|input: &str| {
                                    if input.trim().is_empty() {
                                        Err("Command cannot be empty".to_string())
                                    } else {
                                        Ok(())
                                    }
                                }));
                            s.add_input(field)
                        })
                    },
                    GroupConfig::required(),
                )
                .run()
                .wrap_err("Failed to collect command information")?;

            // 提取命令
            // 注意：Select 字段现在返回选项值（String），而不是索引
            let cmd = if command_form_result.get("input_method")
                == Some("Select from common commands".to_string())
            {
                command_form_result
                    .get_required("selected_command")
                    .wrap_err("Selected command is required")?
            } else {
                command_form_result
                    .get_required("manual_command")
                    .wrap_err("Manual command is required")?
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
            let should_overwrite =
                crate::confirm!("Alias '{}' already exists. Overwrite? (y/N)", alias_name)
                    .default(false)
                    .prompt()
                    .unwrap_or(false);

            if !should_overwrite {
                info!("Operation cancelled");
                return Ok(());
            }
        }

        // 保存别名
        AliasManager::add(&alias_name, &alias_command)?;
        success!(
            "Alias '{}' = '{}' added successfully",
            alias_name,
            alias_command
        );

        // 询问是否更新补全脚本
        let should_update = crate::confirm!("Update completion scripts?")
            .default(true)
            .prompt()
            .unwrap_or(false);

        if should_update {
            match crate::Completion::generate_all_completions(None, None) {
                Ok(_) => {
                    success!("Completion scripts updated successfully");
                }
                Err(e) => {
                    warning!("Failed to update completion scripts: {}", e);
                    info!("You can manually update them later with: workflow completion generate");
                }
            }
        }

        Ok(())
    }
}
