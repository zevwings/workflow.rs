//! 别名添加命令
//!
//! 添加新的别名。

use clap::CommandFactory;
use color_eyre::{eyre::WrapErr, Result};
use prompt::{br, info, success, warning, InputBuilder, SelectBuilder};

use crate::cli::Cli;
use crate::registry::get_alias_service;

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
    pub fn run(&self) -> Result<()> {
        // 获取别名名称和命令
        let (name, command) = if self.name.is_some() && self.command.is_some() {
            // 直接模式
            (self.name.clone().unwrap(), self.command.clone().unwrap())
        } else {
            // 交互模式
            self.interactive_input()?
        };

        // 添加别名
        let service = get_alias_service();
        let result = service.add(&name, &command, self.force).wrap_err("添加别名失败")?;

        // 显示结果
        br!();
        if result.overwritten {
            warning!("别名 '{}' 已更新", result.name);
        } else {
            success!("别名 '{}' 已添加", result.name);
        }
        info!("  {} -> {}", result.name, result.command);
        br!();
        info!("使用 'workflow {}' 执行此别名", result.name);

        Ok(())
    }

    /// 交互式输入
    fn interactive_input(&self) -> Result<(String, String)> {
        info!("添加新别名（交互模式）");
        br!();

        // 获取别名名称
        let name = if let Some(ref n) = self.name {
            n.clone()
        } else {
            let input = InputBuilder::new("请输入别名名称")
                .placeholder("例如: ci")
                .prompt()
                .wrap_err("获取别名名称失败")?;

            if input.is_empty() {
                color_eyre::eyre::bail!("别名名称不能为空");
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

            let selected = SelectBuilder::new("请选择要关联的命令", display_options)
                .prompt()
                .wrap_err("选择命令失败")?;

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
fn collect_commands(cmd: &clap::Command, prefix: &str, commands: &mut Vec<(String, String)>) {
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
