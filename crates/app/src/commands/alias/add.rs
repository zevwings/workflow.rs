//! 别名添加命令
//!
//! 添加新的别名。

use clap::{Command, CommandFactory};
use prompt::{br, info, success, warning, InputBuilder, SelectBuilder};

use crate::bootstrap::get_alias_service;
use crate::commands::cli::Cli;

/// 别名添加命令
pub struct AliasAddCommand {
    /// 别名名称（可选，为空时进入交互模式）
    name: Option<String>,
    /// 对应的命令（可选，为空时进入交互模式）
    command: Option<String>,
    /// 是否强制覆盖
    force: bool,
}

impl AliasAddCommand {
    /// 创建新的 AliasAddCommand 实例
    pub fn new(name: Option<String>, command: Option<String>, force: bool) -> Self {
        Self {
            name,
            command,
            force,
        }
    }

    /// 运行添加命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 获取别名名称和命令
        let interactive_result;
        let (name, command) = match (&self.name, &self.command) {
            (Some(n), Some(c)) => (n.as_str(), c.as_str()),
            _ => {
                interactive_result = self.interactive_input()?;
                (interactive_result.0.as_str(), interactive_result.1.as_str())
            }
        };

        // 添加别名
        let service = get_alias_service();
        let result = service
            .add(name, command, self.force)
            .map_err(|e| format!("Failed to add alias: {}", e))?;

        // 显示结果
        br!();
        if result.overwritten {
            warning!("Alias '{}' updated", result.name);
        } else {
            success!("Alias '{}' added", result.name);
        }
        info!("  {} -> {}", result.name, result.command);
        br!();
        info!("Use 'workflow {}' to run this alias", result.name);

        Ok(())
    }

    /// 交互式输入
    fn interactive_input(&self) -> Result<(String, String), Box<dyn std::error::Error>> {
        info!("Add new alias (interactive mode)");
        br!();

        // 获取别名名称
        let name = if let Some(ref n) = self.name {
            n.clone()
        } else {
            let input = InputBuilder::new("Enter alias name")
                .placeholder("e.g.: ci")
                .prompt()
                .map_err(|e| format!("Failed to get alias name: {}", e))?;

            if input.is_empty() {
                return Err("Alias name cannot be empty".into());
            }
            input
        };

        // 获取命令 - 使用选择器
        let command = if let Some(ref c) = self.command {
            c.clone()
        } else {
            let commands = get_available_commands();
            let display_options: Vec<String> = commands
                .iter()
                .map(|(cmd, desc)| {
                    if desc.is_empty() {
                        cmd.clone()
                    } else {
                        format!("{} - {}", cmd, desc)
                    }
                })
                .collect();

            let selected = SelectBuilder::new("Select the command to associate", display_options)
                .prompt()
                .map_err(|e| format!("Failed to select command: {}", e))?;

            // 从选中的显示文本中提取命令部分
            selected.split(" - ").next().unwrap_or(&selected).to_string()
        };

        Ok((name, command))
    }
}

/// 获取所有可用的 CLI 命令（动态从 clap 获取）
fn get_available_commands() -> Vec<(String, String)> {
    let cmd = Cli::command();
    let mut commands = Vec::new();
    collect_commands(&cmd, "", &mut commands);
    commands
}

/// 递归收集所有命令
fn collect_commands(cmd: &Command, prefix: &str, commands: &mut Vec<(String, String)>) {
    for subcmd in cmd.get_subcommands() {
        let name = subcmd.get_name();

        // 跳过 alias 相关命令（避免循环引用）
        if name == "alias" {
            continue;
        }

        let full_name = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{} {}", prefix, name)
        };

        // 获取命令描述
        let about = subcmd.get_about().map(|s| s.to_string()).unwrap_or_default();

        // 检查是否有子命令
        let has_subcommands = subcmd.get_subcommands().next().is_some();

        if has_subcommands {
            // 递归收集子命令
            collect_commands(subcmd, &full_name, commands);
        } else {
            // 叶子命令，添加到列表
            commands.push((full_name, about));
        }
    }
}
