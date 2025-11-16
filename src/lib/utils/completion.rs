//! Shell Completion 管理工具
//!
//! 本模块提供了 Shell Completion 的完整管理功能，包括：
//! - 生成 completion 脚本文件
//! - 配置 shell 配置文件以启用 completion
//! - 创建 completion 配置文件
//! - 删除 completion 配置和文件
//! - 获取 completion 文件列表

use crate::log_debug;
use crate::log_info;
use crate::log_success;
use crate::ShellInfo;
use anyhow::{Context, Result};
use clap::Command;
use clap_complete::{generate, shells::Shell as ClapShell};
use std::fs;
use std::path::{Path, PathBuf};

/// Completion 管理工具
///
/// 提供 Shell Completion 的配置和管理功能。
/// 支持 zsh 和 bash 两种 shell。
pub struct Completion;

impl Completion {
    /// 创建 workflow 配置文件目录
    fn create_workflow_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let workflow_dir = PathBuf::from(&home).join(".workflow");
        fs::create_dir_all(&workflow_dir).context("Failed to create .workflow directory")?;
        Ok(workflow_dir)
    }

    /// 创建并写入 workflow completion 配置文件
    /// 配置文件同时支持 zsh 和 bash
    ///
    /// 注意：`_workflow` 文件包含 `workflow` 命令及其所有子命令的 completion，
    /// 包括 `github`、`proxy`、`log`、`clean` 等子命令。
    /// `_pr` 和 `_qk` 是独立命令的 completion 文件。
    fn create_completion_config_file(shell_info: &ShellInfo) -> Result<PathBuf> {
        let workflow_dir = Self::create_workflow_dir()?;
        let config_file = workflow_dir.join(".completions");

        // 生成同时支持 zsh 和 bash 的配置
        let config_content = format!(
            "# Workflow CLI completions\n\
            # Supports both zsh and bash\n\
            \n\
            # Zsh completion setup\n\
            if [[ -n \"$ZSH_VERSION\" ]]; then\n\
                fpath=({} $fpath)\n\
                if [[ -f {}/_workflow ]]; then\n\
                    source {}/_workflow\n\
                    source {}/_pr\n\
                    source {}/_qk\n\
                fi\n\
            fi\n\
            \n\
            # Bash completion setup\n\
            if [[ -n \"$BASH_VERSION\" ]]; then\n\
                for f in {}/*.bash; do\n\
                    [[ -f \"$f\" ]] && source \"$f\"\n\
                done\n\
            fi\n",
            shell_info.completion_dir.display(),
            shell_info.completion_dir.display(),
            shell_info.completion_dir.display(),
            shell_info.completion_dir.display(),
            shell_info.completion_dir.display(),
            shell_info.completion_dir.display()
        );

        fs::write(&config_file, config_content)
            .context("Failed to write workflow completion config file")?;

        Ok(config_file)
    }

    /// 配置 shell 配置文件以启用 completion
    pub fn configure_shell_config(shell_info: &ShellInfo) -> Result<()> {
        // 创建 workflow completion 配置文件
        let workflow_config_file = Self::create_completion_config_file(shell_info)?;
        let workflow_config_file_str = workflow_config_file.display().to_string();

        // 读取 shell 配置文件
        let config_content =
            fs::read_to_string(&shell_info.config_file).unwrap_or_else(|_| String::new());

        // 检查是否已经引用了 workflow 配置文件
        let source_pattern = "source $HOME/.workflow/.completions";

        // 也检查是否使用了绝对路径
        let source_pattern_abs = format!("source {}", workflow_config_file_str);

        if config_content.contains(source_pattern) || config_content.contains(&source_pattern_abs) {
            log_success!(
                "completion 配置已存在于 {}",
                shell_info.config_file.display()
            );
            return Ok(());
        }

        // 添加 source 语句到 shell 配置文件
        let mut new_content = config_content;
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str("\n# Workflow CLI completions\n");
        new_content.push_str(source_pattern);
        new_content.push('\n');
        new_content.push('\n');

        fs::write(&shell_info.config_file, new_content)
            .context("Failed to write to shell config file")?;

        log_success!(
            "已将 completion 配置添加到 {}",
            shell_info.config_file.display()
        );

        Ok(())
    }

    /// 从 shell 配置文件中移除 completion 配置
    pub fn remove_completion_config(shell_info: &ShellInfo) -> Result<()> {
        let config_content =
            fs::read_to_string(&shell_info.config_file).unwrap_or_else(|_| String::new());

        // 检查是否引用了 workflow 配置文件
        let source_pattern = "source $HOME/.workflow/.completions";

        let has_source = config_content.contains(source_pattern);

        // 也检查绝对路径
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let workflow_config_file = PathBuf::from(&home).join(".workflow").join(".completions");
        let source_pattern_abs = format!("source {}", workflow_config_file.display());
        let has_source_abs = config_content.contains(&source_pattern_abs);

        if !has_source && !has_source_abs {
            log_info!(
                "completion 配置未在 {} 中找到",
                shell_info.config_file.display()
            );
            return Ok(());
        }

        // 删除配置块（包括 marker 和 source 行）
        let marker_start = "# Workflow CLI completions";
        let mut new_content = String::new();
        let lines: Vec<&str> = config_content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // 检查是否是配置块开始
            if line.contains(marker_start) {
                i += 1; // 跳过 marker 行
                        // 跳过 source 行
                while i < lines.len() {
                    let current_line = lines[i];
                    if current_line.contains(source_pattern)
                        || current_line.contains(&source_pattern_abs)
                    {
                        i += 1; // 跳过 source 行
                        break;
                    }
                    // 如果遇到空行，停止
                    if current_line.trim().is_empty() {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                continue;
            }

            // 跳过独立的 source 行（不在配置块内）
            if line.contains(source_pattern) || line.contains(&source_pattern_abs) {
                i += 1;
                continue;
            }

            new_content.push_str(line);
            new_content.push('\n');
            i += 1;
        }

        // 清理末尾的多个空行
        while new_content.ends_with("\n\n") {
            new_content.pop();
        }
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }

        fs::write(&shell_info.config_file, new_content)
            .context("Failed to write to shell config file")?;

        log_success!(
            "已从 {} 中删除 completion 配置",
            shell_info.config_file.display()
        );

        Ok(())
    }

    /// 获取 completion 文件列表（根据 shell 类型）
    ///
    /// 返回独立的 completion 文件列表：
    /// - `_workflow` / `workflow.bash`: 包含 `workflow` 命令及其所有子命令（包括 `github`）
    /// - `_pr` / `pr.bash`: `pr` 独立命令
    /// - `_qk` / `qk.bash`: `qk` 独立命令
    pub fn get_completion_files(shell_info: &ShellInfo) -> Vec<PathBuf> {
        if shell_info.shell_type == "zsh" {
            vec![
                shell_info.completion_dir.join("_workflow"),
                shell_info.completion_dir.join("_pr"),
                shell_info.completion_dir.join("_qk"),
            ]
        } else {
            vec![
                shell_info.completion_dir.join("workflow.bash"),
                shell_info.completion_dir.join("pr.bash"),
                shell_info.completion_dir.join("qk.bash"),
            ]
        }
    }

    /// 删除 completion 文件
    ///
    /// 删除所有 shell 类型的 completion 文件（zsh, bash, fish, powershell, elvish），
    /// 确保卸载时完全清理所有可能存在的 completion 文件。
    pub fn remove_completion_files(shell_info: &ShellInfo) -> Result<usize> {
        // 获取所有 shell 类型的 completion 文件
        let all_files = vec![
            // zsh 文件
            shell_info.completion_dir.join("_workflow"),
            shell_info.completion_dir.join("_pr"),
            shell_info.completion_dir.join("_qk"),
            // bash 文件
            shell_info.completion_dir.join("workflow.bash"),
            shell_info.completion_dir.join("pr.bash"),
            shell_info.completion_dir.join("qk.bash"),
            // fish 文件
            shell_info.completion_dir.join("workflow.fish"),
            shell_info.completion_dir.join("pr.fish"),
            shell_info.completion_dir.join("qk.fish"),
            // powershell 文件
            shell_info.completion_dir.join("_workflow.ps1"),
            shell_info.completion_dir.join("_pr.ps1"),
            shell_info.completion_dir.join("_qk.ps1"),
            // elvish 文件
            shell_info.completion_dir.join("workflow.elv"),
            shell_info.completion_dir.join("pr.elv"),
            shell_info.completion_dir.join("qk.elv"),
        ];

        let mut removed_count = 0;
        for file in &all_files {
            if file.exists() {
                if let Err(e) = fs::remove_file(file) {
                    log_info!("删除失败: {} ({})", file.display(), e);
                } else {
                    log_info!("  Removed: {}", file.display());
                    removed_count += 1;
                }
            }
        }

        if removed_count > 0 {
            log_info!("  Completion script files removed");
        } else {
            log_debug!("  Completion script files not found (may not be installed)");
        }

        Ok(removed_count)
    }

    /// 删除 workflow completion 配置文件
    pub fn remove_completion_config_file() -> Result<()> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let workflow_config_file = PathBuf::from(&home).join(".workflow").join(".completions");

        if workflow_config_file.exists() {
            fs::remove_file(&workflow_config_file)
                .context("Failed to remove workflow completion config file")?;
            log_info!("  Removed: {}", workflow_config_file.display());
        } else {
            log_info!(
                "  Completion config file not found: {}",
                workflow_config_file.display()
            );
        }

        Ok(())
    }

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
        Self::generate_workflow_completion(&shell_type, &output)?;
        Self::generate_pr_completion(&shell_type, &output)?;
        Self::generate_qk_completion(&shell_type, &output)?;

        log_success!("  Shell completion 脚本已生成到: {}", output.display());
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
            .subcommand(Command::new("check").about("Run environment checks"))
            .subcommand(Command::new("setup").about("Initialize or update configuration"))
            .subcommand(Command::new("config").about("View current configuration"))
            .subcommand(
                Command::new("install").about("Install Workflow CLI (binary and completions)"),
            )
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
