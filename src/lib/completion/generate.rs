//! Completion 脚本生成工具
//!
//! 提供生成各种 shell 的 completion 脚本文件的功能。

use crate::log_debug;
use crate::log_success;
use anyhow::{Context, Result};
use clap::Command;
use clap_complete::{generate, shells::Shell as ClapShell};
use std::fs;
use std::path::PathBuf;

use crate::completion::helpers::get_completion_filename;

/// Completion 脚本生成器
///
/// 提供生成各种 shell 的 completion 脚本文件的功能。
/// 支持 workflow 命令及其所有子命令的 completion 生成。
pub struct CompletionGenerator {
    shell: ClapShell,
    output_dir: PathBuf,
}

impl CompletionGenerator {
    /// 创建新的 CompletionGenerator 实例
    ///
    /// # 参数
    ///
    /// * `shell_type` - Shell 类型字符串（"zsh", "bash", "fish", "powershell", "elvish"），如果为 None 则自动检测
    /// * `output_dir` - 输出目录路径，如果为 None 则使用默认目录 `~/.workflow/completions`
    ///
    /// # 返回
    ///
    /// 返回 `CompletionGenerator` 实例，如果 shell 类型不支持则返回错误。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::completion::generate::CompletionGenerator;
    ///
    /// let generator = CompletionGenerator::new(
    ///     Some("zsh".to_string()),
    ///     Some("/path/to/completions".to_string()),
    /// )?;
    /// generator.generate_all()?;
    /// ```
    pub fn new(shell_type: Option<String>, output_dir: Option<String>) -> Result<Self> {
        // 解析 shell 类型
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

        let clap_shell = match shell {
            "zsh" => ClapShell::Zsh,
            "bash" => ClapShell::Bash,
            "fish" => ClapShell::Fish,
            "powershell" => ClapShell::PowerShell,
            "elvish" => ClapShell::Elvish,
            _ => {
                anyhow::bail!("Unsupported shell type: {}. Supported shell types: zsh, bash, fish, powershell, elvish", shell);
            }
        };

        // 解析输出目录
        let output = output_dir.map(PathBuf::from).unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
            PathBuf::from(&home).join(".workflow/completions")
        });

        Ok(Self {
            shell: clap_shell,
            output_dir: output,
        })
    }

    /// 生成所有 completion 脚本文件
    ///
    /// 为所有命令生成 completion 脚本：
    /// - `workflow` 命令及其所有子命令（包括 `pr`、`log`、`jira`、`github`、`proxy`、`log-level` 等）
    pub fn generate_all(&self) -> Result<()> {
        log_debug!("Generating shell completion scripts...");
        log_debug!("Shell type: {}", self.shell);
        log_debug!("Output directory: {}", self.output_dir.display());

        // 创建输出目录
        fs::create_dir_all(&self.output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {} (shell: {})",
                self.output_dir.display(),
                self.shell
            )
        })?;

        // 生成 completion 脚本
        self.generate_workflow()?;

        log_success!(
            "  Shell completion scripts generated to: {}",
            self.output_dir.display()
        );
        Ok(())
    }

    /// 生成 workflow 命令的 completion
    fn generate_workflow(&self) -> Result<()> {
        let mut cmd = Command::new("workflow")
            .about("Workflow CLI tool")
            .subcommand(
                Command::new("proxy")
                    .about("Manage proxy settings")
                    .arg(
                        clap::Arg::new("temporary")
                            .long("temporary")
                            .short('t')
                            .action(clap::ArgAction::SetTrue),
                    )
                    .subcommand(Command::new("on").about("Turn proxy on"))
                    .subcommand(Command::new("off").about("Turn proxy off"))
                    .subcommand(Command::new("check").about("Check proxy status")),
            )
            .subcommand(Command::new("check").about("Run environment checks"))
            .subcommand(Command::new("setup").about("Initialize or update configuration"))
            .subcommand(Command::new("config").about("View current configuration"))
            .subcommand(Command::new("uninstall").about("Uninstall Workflow CLI configuration"))
            .subcommand(Command::new("version").about("Show Workflow CLI version"))
            .subcommand(
                Command::new("update").about("Update Workflow CLI").arg(
                    clap::Arg::new("version")
                        .long("version")
                        .short('v')
                        .value_name("VERSION"),
                ),
            )
            .subcommand(
                Command::new("log-level")
                    .about("Manage log level")
                    .subcommand(Command::new("set").about("Set log level"))
                    .subcommand(Command::new("check").about("Check log level")),
            )
            .subcommand(
                Command::new("pr")
                    .about("Pull Request operations")
                    .subcommand(
                        Command::new("create")
                            .about("Create a new Pull Request")
                            .arg(clap::Arg::new("JIRA_TICKET").value_name("JIRA_TICKET"))
                            .arg(
                                clap::Arg::new("title")
                                    .short('t')
                                    .long("title")
                                    .value_name("TITLE"),
                            )
                            .arg(
                                clap::Arg::new("description")
                                    .short('d')
                                    .long("description")
                                    .value_name("DESCRIPTION"),
                            )
                            .arg(
                                clap::Arg::new("dry-run")
                                    .long("dry-run")
                                    .action(clap::ArgAction::SetTrue),
                            ),
                    )
                    .subcommand(
                        Command::new("merge")
                            .about("Merge a Pull Request")
                            .arg(clap::Arg::new("PR_ID").value_name("PR_ID"))
                            .arg(
                                clap::Arg::new("force")
                                    .short('f')
                                    .long("force")
                                    .action(clap::ArgAction::SetTrue),
                            ),
                    )
                    .subcommand(
                        Command::new("status")
                            .about("Show PR status information")
                            .arg(clap::Arg::new("PR_ID_OR_BRANCH").value_name("PR_ID_OR_BRANCH")),
                    )
                    .subcommand(
                        Command::new("list")
                            .about("List PRs")
                            .arg(
                                clap::Arg::new("state")
                                    .short('s')
                                    .long("state")
                                    .value_name("STATE"),
                            )
                            .arg(
                                clap::Arg::new("limit")
                                    .short('l')
                                    .long("limit")
                                    .value_name("LIMIT"),
                            ),
                    )
                    .subcommand(Command::new("update").about("Update code"))
                    .subcommand(
                        Command::new("integrate")
                            .about("Integrate branch to current branch")
                            .arg(
                                clap::Arg::new("SOURCE_BRANCH")
                                    .value_name("SOURCE_BRANCH")
                                    .required(true),
                            )
                            .arg(
                                clap::Arg::new("ff-only")
                                    .long("ff-only")
                                    .action(clap::ArgAction::SetTrue),
                            )
                            .arg(
                                clap::Arg::new("squash")
                                    .long("squash")
                                    .action(clap::ArgAction::SetTrue),
                            )
                            .arg(
                                clap::Arg::new("no-push")
                                    .long("no-push")
                                    .action(clap::ArgAction::SetTrue),
                            ),
                    )
                    .subcommand(
                        Command::new("close")
                            .about("Close a Pull Request")
                            .arg(clap::Arg::new("PR_ID").value_name("PR_ID")),
                    )
                    .subcommand(
                        Command::new("summarize")
                            .about("Summarize a Pull Request")
                            .arg(clap::Arg::new("PR_ID").value_name("PR_ID"))
                            .arg(
                                clap::Arg::new("language")
                                    .short('l')
                                    .long("language")
                                    .value_name("LANGUAGE"),
                            ),
                    ),
            )
            .subcommand(
                Command::new("log")
                    .about("Log operations")
                    .subcommand(
                        Command::new("download")
                            .about("Download log files from Jira ticket")
                            .arg(
                                clap::Arg::new("JIRA_ID")
                                    .value_name("JIRA_ID")
                                    .required(false),
                            ),
                    )
                    .subcommand(
                        Command::new("find")
                            .about("Find request ID in log files")
                            .arg(
                                clap::Arg::new("JIRA_ID")
                                    .value_name("JIRA_ID")
                                    .required(false),
                            )
                            .arg(clap::Arg::new("REQUEST_ID").value_name("REQUEST_ID")),
                    )
                    .subcommand(
                        Command::new("search")
                            .about("Search for keywords in log files")
                            .arg(
                                clap::Arg::new("JIRA_ID")
                                    .value_name("JIRA_ID")
                                    .required(false),
                            )
                            .arg(clap::Arg::new("SEARCH_TERM").value_name("SEARCH_TERM")),
                    ),
            )
            .subcommand(
                Command::new("jira")
                    .about("Jira operations")
                    .subcommand(
                        Command::new("info").about("Show ticket information").arg(
                            clap::Arg::new("JIRA_ID")
                                .value_name("JIRA_ID")
                                .required(false),
                        ),
                    )
                    .subcommand(
                        Command::new("attachments")
                            .about("Download all attachments from Jira ticket")
                            .arg(
                                clap::Arg::new("JIRA_ID")
                                    .value_name("JIRA_ID")
                                    .required(false),
                            ),
                    )
                    .subcommand(
                        Command::new("clean")
                            .about("Clean log directory")
                            .arg(clap::Arg::new("JIRA_ID").value_name("JIRA_ID"))
                            .arg(
                                clap::Arg::new("all")
                                    .long("all")
                                    .short('a')
                                    .action(clap::ArgAction::SetTrue),
                            )
                            .arg(
                                clap::Arg::new("dry-run")
                                    .long("dry-run")
                                    .short('n')
                                    .action(clap::ArgAction::SetTrue),
                            )
                            .arg(
                                clap::Arg::new("list")
                                    .long("list")
                                    .short('l')
                                    .action(clap::ArgAction::SetTrue),
                            ),
                    ),
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
            )
            .subcommand(
                Command::new("completion")
                    .about("Manage shell completion")
                    .subcommand(Command::new("generate").about("Generate completion scripts"))
                    .subcommand(Command::new("check").about("Check completion status"))
                    .subcommand(Command::new("remove").about("Remove completion configuration")),
            )
            .subcommand(
                Command::new("llm")
                    .about("Manage LLM configuration")
                    .subcommand(Command::new("show").about("Show current LLM configuration"))
                    .subcommand(Command::new("setup").about("Setup LLM configuration"))
                    .subcommand(Command::new("language").about("Set summary language")),
            )
            .subcommand(
                Command::new("branch")
                    .about("Manage Git branches")
                    .subcommand(
                        Command::new("clean").about("Clean local branches").arg(
                            clap::Arg::new("dry-run")
                                .long("dry-run")
                                .short('n')
                                .action(clap::ArgAction::SetTrue),
                        ),
                    )
                    .subcommand(
                        Command::new("ignore")
                            .about("Manage branch ignore list")
                            .subcommand(
                                Command::new("add").about("Add branch to ignore list").arg(
                                    clap::Arg::new("BRANCH_NAME")
                                        .value_name("BRANCH_NAME")
                                        .required(false),
                                ),
                            )
                            .subcommand(
                                Command::new("remove")
                                    .about("Remove branch from ignore list")
                                    .arg(
                                        clap::Arg::new("BRANCH_NAME")
                                            .value_name("BRANCH_NAME")
                                            .required(false),
                                    ),
                            )
                            .subcommand(Command::new("list").about("List ignored branches")),
                    ),
            );

        self.generate_completion(&mut cmd, "workflow")
    }

    /// 生成单个命令的 completion（通用方法）
    ///
    /// # 参数
    ///
    /// * `cmd` - clap Command 实例
    /// * `command_name` - 命令名称（"workflow"）
    fn generate_completion(&self, cmd: &mut Command, command_name: &str) -> Result<()> {
        let mut buffer = Vec::new();
        generate(self.shell, cmd, command_name, &mut buffer);

        let shell_type_str = self.shell.to_string();
        let filename = get_completion_filename(&shell_type_str, command_name)?;
        let output_file = self.output_dir.join(&filename);

        fs::write(&output_file, buffer).with_context(|| {
            format!(
                "Failed to write completion file: {} (command: {}, shell: {})",
                output_file.display(),
                command_name,
                self.shell
            )
        })?;
        log_success!("  Generated: {}", output_file.display());

        Ok(())
    }
}
