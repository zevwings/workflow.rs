//! Completion 脚本生成工具
//!
//! 提供生成各种 shell 的 completion 脚本文件的功能。

use crate::log_debug;
use crate::log_success;
use anyhow::{Context, Result};
use clap::Command;
use clap_complete::{generate, shells::Shell as ClapShell};
use std::fs;
use std::path::{Path, PathBuf};

use crate::completion::files::get_completion_filename;

/// 生成所有 completion 脚本文件
///
/// 为所有命令生成 completion 脚本：
/// - `workflow` 命令及其所有子命令（包括 `github`、`proxy`、`log`、`clean` 等）
/// - `pr` 独立命令
/// - `qk` 独立命令
pub fn generate_all_completions(
    shell_type: Option<String>,
    output_dir: Option<String>,
) -> Result<()> {
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

    log_debug!("生成 shell completion 脚本...");
    log_debug!("Shell 类型: {}", shell);
    log_debug!("输出目录: {}", output.display());

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
    generate_workflow_completion(&shell_type, &output)?;
    generate_pr_completion(&shell_type, &output)?;
    generate_qk_completion(&shell_type, &output)?;

    log_success!("  Shell completion 脚本已生成到: {}", output.display());
    Ok(())
}

/// 生成 workflow 命令的 completion
pub fn generate_workflow_completion(shell: &ClapShell, output_dir: &Path) -> Result<()> {
    let mut cmd = Command::new("workflow")
        .about("Workflow CLI tool")
        .subcommand(
            Command::new("proxy")
                .about("Manage proxy settings")
                .subcommand(Command::new("on").about("Turn proxy on"))
                .subcommand(Command::new("off").about("Turn proxy off"))
                .subcommand(Command::new("check").about("Check proxy status")),
        )
        .subcommand(Command::new("check").about("Run environment checks"))
        .subcommand(Command::new("setup").about("Initialize or update configuration"))
        .subcommand(Command::new("config").about("View current configuration"))
        .subcommand(Command::new("install").about("Install Workflow CLI (binary and completions)"))
        .subcommand(Command::new("uninstall").about("Uninstall Workflow CLI configuration"))
        .subcommand(
            Command::new("clean")
                .about("Clean log directory")
                .arg(clap::Arg::new("dry-run").long("dry-run").short('n'))
                .arg(clap::Arg::new("list").long("list").short('l')),
        )
        .subcommand(
            Command::new("log")
                .about("Manage log level")
                .subcommand(Command::new("set").about("Set log level"))
                .subcommand(Command::new("check").about("Check log level")),
        )
        .subcommand(
            Command::new("github")
                .about("Manage GitHub accounts")
                .subcommand(Command::new("list").about("List all GitHub accounts"))
                .subcommand(Command::new("current").about("Show current GitHub account"))
                .subcommand(Command::new("add").about("Add a new GitHub account"))
                .subcommand(Command::new("remove").about("Remove a GitHub account"))
                .subcommand(Command::new("switch").about("Switch GitHub account"))
                .subcommand(Command::new("update").about("Update GitHub account")),
        );

    let mut buffer = Vec::new();
    generate(*shell, &mut cmd, "workflow", &mut buffer);

    let shell_type_str = shell.to_string();
    let filename = get_completion_filename(&shell_type_str, "workflow")?;
    let output_file = output_dir.join(&filename);

    fs::write(&output_file, buffer).context("Failed to write completion file")?;
    log_success!("  生成: {}", output_file.display());

    Ok(())
}

/// 生成 pr 命令的 completion
pub fn generate_pr_completion(shell: &ClapShell, output_dir: &Path) -> Result<()> {
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

    let shell_type_str = shell.to_string();
    let filename = get_completion_filename(&shell_type_str, "pr")?;
    let output_file = output_dir.join(&filename);

    fs::write(&output_file, buffer).context("Failed to write completion file")?;
    log_success!("  生成: {}", output_file.display());

    Ok(())
}

/// 生成 qk 命令的 completion
pub fn generate_qk_completion(shell: &ClapShell, output_dir: &Path) -> Result<()> {
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

    let shell_type_str = shell.to_string();
    let filename = get_completion_filename(&shell_type_str, "qk")?;
    let output_file = output_dir.join(&filename);

    fs::write(&output_file, buffer).context("Failed to write completion file")?;
    log_success!("  生成: {}", output_file.display());

    Ok(())
}
