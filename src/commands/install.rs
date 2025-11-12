//! 安装命令
//! 提供安装、生成和安装 shell completion 的功能

use crate::{log_break, log_info, log_success, log_warning, Completion, Shell};
use anyhow::{Context, Result};
use clap::Command;
use clap_complete::{generate, shells::Shell as ClapShell};
use std::fs;
use std::path::{Path, PathBuf};

/// 安装命令
#[allow(dead_code)]
pub struct InstallCommand;

#[allow(dead_code)]
impl InstallCommand {
    /// 生成 shell completion 脚本（内部方法）
    /// 参数: shell_type (zsh/bash), output_dir
    fn generate_completions(shell_type: Option<String>, output_dir: Option<String>) -> Result<()> {
        let shell = shell_type.as_deref().unwrap_or_else(|| {
            let shell_env = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
            if shell_env.contains("zsh") {
                "zsh"
            } else if shell_env.contains("bash") {
                "bash"
            } else {
                "zsh" // 默认
            }
        });

        let output = output_dir.map(PathBuf::from).unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
            PathBuf::from(&home).join(".workflow/completions")
        });

        log_info!("生成 shell completion 脚本...");
        log_info!("Shell 类型: {}", shell);
        log_info!("输出目录: {}", output.display());

        // 解析 shell 类型
        let shell_type = match shell {
            "zsh" => ClapShell::Zsh,
            "bash" => ClapShell::Bash,
            "fish" => ClapShell::Fish,
            "powershell" => ClapShell::PowerShell,
            "elvish" => ClapShell::Elvish,
            _ => {
                anyhow::bail!("不支持的 shell: {}", shell);
            }
        };

        // 创建输出目录
        fs::create_dir_all(&output).context("Failed to create output directory")?;

        // 生成 completion 脚本
        Self::generate_workflow_completion(&shell_type, &output)?;
        Self::generate_pr_completion(&shell_type, &output)?;
        Self::generate_qk_completion(&shell_type, &output)?;

        log_success!("  Shell completion 脚本已生成到: {}", output.display());
        Ok(())
    }

    /// 安装 shell completion 脚本（自动检测 shell，内部方法）
    fn install_completions() -> Result<()> {
        let shell_info = Shell::detect()?;

        log_info!("检测 shell 类型...");
        log_info!("检测到: {}", shell_info.shell_type);

        // 创建 completion 目录
        fs::create_dir_all(&shell_info.completion_dir)
            .context("Failed to create completion directory")?;
        log_info!("Completion 目录: {}", shell_info.completion_dir.display());

        // 生成 completion 脚本
        log_info!("正在生成 completion 脚本...");
        Self::generate_completions(
            Some(shell_info.shell_type.clone()),
            Some(shell_info.completion_dir.to_string_lossy().to_string()),
        )?;

        // 配置 shell 配置文件
        log_info!("正在配置 shell 配置文件...");
        Completion::configure_shell_config(&shell_info)?;

        log_success!("  shell completion 安装完成");
        log_info!("请运行以下命令重新加载配置:");
        log_info!("  source {}", shell_info.config_file.display());

        Ok(())
    }

    /// 安装 shell completion 脚本（用于 Makefile 调用）
    pub fn install() -> Result<()> {
        log_info!("安装 shell completion 脚本...");
        if let Err(e) = Self::install_completions() {
            log_warning!("⚠  shell completion 安装失败: {}", e);
            log_info!("  可稍后手动安装 completion 脚本");
            return Err(e);
        }
        log_success!("  shell completion 安装完成");
        log_break!();
        log_info!("提示：请运行以下命令重新加载配置:");
        log_info!("  source ~/.zshrc  # 或 source ~/.bashrc");
        Ok(())
    }

    /// 生成 workflow 命令的 completion
    fn generate_workflow_completion(shell: &ClapShell, output_dir: &Path) -> Result<()> {
        let mut cmd = Command::new("workflow")
            .about("Workflow CLI tool")
            .subcommand(
                Command::new("proxy")
                    .about("Manage proxy settings")
                    .subcommand(Command::new("on").about("Turn proxy on"))
                    .subcommand(Command::new("off").about("Turn proxy off"))
                    .subcommand(Command::new("check").about("Check proxy status")),
            )
            .subcommand(
                Command::new("check")
                    .about("Run checks")
                    .subcommand(Command::new("git_status").about("Check git status"))
                    .subcommand(Command::new("network").about("Check network connection")),
            )
            .subcommand(Command::new("setup").about("Initialize or update configuration"))
            .subcommand(Command::new("config").about("View current configuration"))
            .subcommand(
                Command::new("install").about("Install Workflow CLI (binary and completions)"),
            )
            .subcommand(Command::new("uninstall").about("Uninstall Workflow CLI configuration"));

        let mut buffer = Vec::new();
        generate(*shell, &mut cmd, "workflow", &mut buffer);

        let output_file = match shell {
            ClapShell::Zsh => output_dir.join("_workflow"),
            ClapShell::Bash => output_dir.join("workflow.bash"),
            ClapShell::Fish => output_dir.join("workflow.fish"),
            ClapShell::PowerShell => output_dir.join("_workflow.ps1"),
            ClapShell::Elvish => output_dir.join("workflow.elv"),
            _ => {
                anyhow::bail!("不支持的 shell 类型");
            }
        };

        fs::write(&output_file, buffer).context("Failed to write completion file")?;
        log_success!("  生成: {}", output_file.display());

        Ok(())
    }

    /// 生成 pr 命令的 completion
    fn generate_pr_completion(shell: &ClapShell, output_dir: &Path) -> Result<()> {
        let mut cmd = Command::new("pr")
            .about("Pull Request operations")
            .subcommand(
                Command::new("create")
                    .about("Create a new Pull Request")
                    .arg(clap::Arg::new("JIRA_TICKET").value_name("JIRA_TICKET"))
                    .arg(clap::Arg::new("title").short('t').long("title"))
                    .arg(clap::Arg::new("description").short('d').long("description"))
                    .arg(clap::Arg::new("dry-run").long("dry-run")),
            )
            .subcommand(
                Command::new("merge")
                    .about("Merge a Pull Request")
                    .arg(clap::Arg::new("PR_ID").value_name("PR_ID"))
                    .arg(clap::Arg::new("force").short('f').long("force")),
            )
            .subcommand(
                Command::new("status")
                    .about("Show PR status information")
                    .arg(clap::Arg::new("PR_ID_OR_BRANCH").value_name("PR_ID_OR_BRANCH")),
            )
            .subcommand(
                Command::new("list")
                    .about("List PRs")
                    .arg(clap::Arg::new("state").short('s').long("state"))
                    .arg(clap::Arg::new("limit").short('l').long("limit")),
            )
            .subcommand(Command::new("update").about("Update code"))
            .subcommand(
                Command::new("close")
                    .about("Close a Pull Request")
                    .arg(clap::Arg::new("PR_ID").value_name("PR_ID")),
            );

        let mut buffer = Vec::new();
        generate(*shell, &mut cmd, "pr", &mut buffer);

        let output_file = match shell {
            ClapShell::Zsh => output_dir.join("_pr"),
            ClapShell::Bash => output_dir.join("pr.bash"),
            ClapShell::Fish => output_dir.join("pr.fish"),
            ClapShell::PowerShell => output_dir.join("_pr.ps1"),
            ClapShell::Elvish => output_dir.join("pr.elv"),
            _ => {
                anyhow::bail!("不支持的 shell 类型");
            }
        };

        fs::write(&output_file, buffer).context("Failed to write completion file")?;
        log_success!("  生成: {}", output_file.display());

        Ok(())
    }

    /// 生成 qk 命令的 completion
    fn generate_qk_completion(shell: &ClapShell, output_dir: &Path) -> Result<()> {
        let mut cmd = Command::new("qk")
            .about("Quick log operations")
            .arg(
                clap::Arg::new("JIRA_ID")
                    .value_name("JIRA_ID")
                    .required(true),
            )
            .subcommand(Command::new("download").about("Download logs"))
            .subcommand(
                Command::new("find")
                    .about("Find request by ID")
                    .arg(clap::Arg::new("REQUEST_ID").value_name("REQUEST_ID")),
            )
            .subcommand(
                Command::new("search")
                    .about("Search in logs")
                    .arg(clap::Arg::new("SEARCH_TERM").value_name("SEARCH_TERM")),
            );

        let mut buffer = Vec::new();
        generate(*shell, &mut cmd, "qk", &mut buffer);

        let output_file = match shell {
            ClapShell::Zsh => output_dir.join("_qk"),
            ClapShell::Bash => output_dir.join("qk.bash"),
            ClapShell::Fish => output_dir.join("qk.fish"),
            ClapShell::PowerShell => output_dir.join("_qk.ps1"),
            ClapShell::Elvish => output_dir.join("qk.elv"),
            _ => {
                anyhow::bail!("不支持的 shell 类型");
            }
        };

        fs::write(&output_file, buffer).context("Failed to write completion file")?;
        log_success!("  生成: {}", output_file.display());

        Ok(())
    }
}
