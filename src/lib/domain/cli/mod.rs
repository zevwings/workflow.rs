//! CLI 命令结构定义
//!
//! 这个模块定义了 Workflow CLI 的命令结构，供 `main.rs` 和补全生成器使用。
//! 这样可以确保补全脚本与实际命令结构保持同步。

use clap::Parser;

// 导入所有子命令枚举
mod alias;
mod args;
mod branch;
mod commands;
mod commit;
mod config;
mod github;
mod jira;
mod llm;
mod log;
mod pr;
mod repo;
mod stash;
mod tag;

// 重新导出所有子命令枚举和主结构体
// 这些导出是必需的，因为 bin/workflow.rs 需要使用它们进行命令分发
pub use alias::AliasSubcommand;
pub use args::{
    ConfirmationArgs, DryRunArgs, ForceArgs, JiraIdArg, JiraOperationArgs, JiraQueryArgs, LogLevel,
    OperationArgs, OutputFormatArgs, PaginationArgs, QueryDisplayArgs, VerbosityArgs,
};
pub use branch::{BranchSubcommand, IgnoreSubcommand};
pub use commands::Commands;
pub use commit::CommitSubcommand;
pub use config::{CompletionSubcommand, ConfigSubcommand, LogLevelSubcommand};
pub use github::GitHubSubcommand;
pub use jira::JiraSubcommand;
pub use llm::LLMSubcommand;
pub use log::LogSubcommand;
pub use pr::PRCommands;
pub use repo::RepoSubcommand;
pub use stash::StashSubcommand;
pub use tag::TagSubcommand;

/// CLI 主结构体
///
/// 使用 clap 进行命令行参数解析，支持子命令模式。
#[derive(Parser)]
#[command(name = "workflow")]
#[command(about = "Workflow CLI tool", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// 从 Commands 枚举中提取命令路径字符串
///
/// 返回格式：`{command}-{subcommand}`（如 `pr-create`、`jira-info`、`jira-log-download`）
/// 如果没有命令，返回 `None`。
///
/// # 示例
///
/// ```
/// use workflow::cli::{Commands, extract_command_name};
///
/// // 示例：从命令中提取命令名
/// // let command = Commands::Pr { subcommand: PRCommands::Create { ... } };
/// // assert_eq!(extract_command_name(&Some(command)), Some("pr-create".to_string()));
/// ```
pub fn extract_command_name(command: &Option<Commands>) -> Option<String> {
    let cmd = command.as_ref()?;
    let mut parts = Vec::new();

    match cmd {
        Commands::Check => {
            parts.push("check".to_string());
        }
        Commands::Setup => {
            parts.push("setup".to_string());
        }
        Commands::Config { subcommand } => {
            parts.push("config".to_string());
            if let Some(sub) = subcommand {
                parts.push(
                    match sub {
                        ConfigSubcommand::Validate { .. } => "validate",
                        ConfigSubcommand::Export { .. } => "export",
                        ConfigSubcommand::Import { .. } => "import",
                    }
                    .to_string(),
                );
            }
        }
        Commands::Uninstall => {
            parts.push("uninstall".to_string());
        }
        Commands::Version => {
            parts.push("version".to_string());
        }
        Commands::Update { .. } => {
            parts.push("update".to_string());
        }
        Commands::Log { subcommand } => {
            parts.push("log".to_string());
            parts.push(
                match subcommand {
                    LogLevelSubcommand::Set => "set",
                    LogLevelSubcommand::Check => "check",
                    LogLevelSubcommand::TraceConsole => "trace-console",
                }
                .to_string(),
            );
        }
        Commands::GitHub { subcommand } => {
            parts.push("github".to_string());
            parts.push(
                match subcommand {
                    GitHubSubcommand::List => "list",
                    GitHubSubcommand::Current => "current",
                    GitHubSubcommand::Add => "add",
                    GitHubSubcommand::Remove => "remove",
                    GitHubSubcommand::Switch => "switch",
                    GitHubSubcommand::Update => "update",
                }
                .to_string(),
            );
        }
        Commands::Llm { subcommand } => {
            parts.push("llm".to_string());
            parts.push(
                match subcommand {
                    LLMSubcommand::Show => "show",
                    LLMSubcommand::Setup => "setup",
                }
                .to_string(),
            );
        }
        Commands::Completion { subcommand } => {
            parts.push("completion".to_string());
            parts.push(
                match subcommand {
                    CompletionSubcommand::Generate => "generate",
                    CompletionSubcommand::Check => "check",
                    CompletionSubcommand::Remove => "remove",
                }
                .to_string(),
            );
        }
        Commands::Branch { subcommand } => {
            parts.push("branch".to_string());
            parts.push(extract_branch_subcommand_name(subcommand));
        }
        Commands::Commit { subcommand } => {
            parts.push("commit".to_string());
            parts.push(
                match subcommand {
                    CommitSubcommand::Amend { .. } => "amend",
                    CommitSubcommand::Reword { .. } => "reword",
                    CommitSubcommand::Squash => "squash",
                }
                .to_string(),
            );
        }
        Commands::Migrate { .. } => {
            parts.push("migrate".to_string());
        }
        Commands::Pr { subcommand } => {
            parts.push("pr".to_string());
            parts.push(extract_pr_command_name(subcommand));
        }
        Commands::Jira { subcommand } => {
            parts.push("jira".to_string());
            parts.push(extract_jira_subcommand_name(subcommand));
        }
        Commands::Stash { subcommand } => {
            parts.push("stash".to_string());
            parts.push(
                match subcommand {
                    StashSubcommand::List { .. } => "list",
                    StashSubcommand::Apply => "apply",
                    StashSubcommand::Drop => "drop",
                    StashSubcommand::Pop => "pop",
                    StashSubcommand::Push => "push",
                }
                .to_string(),
            );
        }
        Commands::Repo { subcommand } => {
            parts.push("repo".to_string());
            parts.push(
                match subcommand {
                    RepoSubcommand::Setup => "setup",
                    RepoSubcommand::Show => "show",
                    RepoSubcommand::Clean { .. } => "clean",
                }
                .to_string(),
            );
        }
        Commands::Alias { subcommand } => {
            parts.push("alias".to_string());
            parts.push(
                match subcommand {
                    AliasSubcommand::List => "list",
                    AliasSubcommand::Add { .. } => "add",
                    AliasSubcommand::Remove { .. } => "remove",
                }
                .to_string(),
            );
        }
        Commands::Tag { subcommand } => {
            parts.push("tag".to_string());
            parts.push(
                match subcommand {
                    TagSubcommand::Delete { .. } => "delete",
                }
                .to_string(),
            );
        }
    }

    Some(parts.join("-"))
}

fn extract_branch_subcommand_name(subcommand: &BranchSubcommand) -> String {
    match subcommand {
        BranchSubcommand::Ignore {
            subcommand: ignore_sub,
        } => {
            format!(
                "ignore-{}",
                match ignore_sub {
                    IgnoreSubcommand::Add { .. } => "add",
                    IgnoreSubcommand::Remove { .. } => "remove",
                    IgnoreSubcommand::List => "list",
                }
            )
        }
        BranchSubcommand::Create { .. } => "create".to_string(),
        BranchSubcommand::Rename => "rename".to_string(),
        BranchSubcommand::Switch { .. } => "switch".to_string(),
        BranchSubcommand::Sync { .. } => "sync".to_string(),
        BranchSubcommand::Delete { .. } => "delete".to_string(),
    }
}

fn extract_pr_command_name(subcommand: &PRCommands) -> String {
    match subcommand {
        PRCommands::Create { .. } => "create",
        PRCommands::Merge { .. } => "merge",
        PRCommands::Status { .. } => "status",
        PRCommands::List { .. } => "list",
        PRCommands::Update => "update",
        PRCommands::Sync { .. } => "sync",
        PRCommands::Rebase { .. } => "rebase",
        PRCommands::Close { .. } => "close",
        PRCommands::Summarize { .. } => "summarize",
        PRCommands::Approve { .. } => "approve",
        PRCommands::Comment { .. } => "comment",
        PRCommands::Pick { .. } => "pick",
        PRCommands::Reword { .. } => "reword",
    }
    .to_string()
}

fn extract_jira_subcommand_name(subcommand: &JiraSubcommand) -> String {
    match subcommand {
        JiraSubcommand::Info { .. } => "info".to_string(),
        JiraSubcommand::Related { .. } => "related".to_string(),
        JiraSubcommand::Changelog { .. } => "changelog".to_string(),
        JiraSubcommand::Comment { .. } => "comment".to_string(),
        JiraSubcommand::Comments { .. } => "comments".to_string(),
        JiraSubcommand::Attachments { .. } => "attachments".to_string(),
        JiraSubcommand::Clean { .. } => "clean".to_string(),
        JiraSubcommand::Log {
            subcommand: log_sub,
        } => {
            format!(
                "log-{}",
                match log_sub {
                    LogSubcommand::Download { .. } => "download",
                    LogSubcommand::Find { .. } => "find",
                    LogSubcommand::Search { .. } => "search",
                }
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{
        BranchSubcommand, Commands, IgnoreSubcommand, JiraSubcommand, LogSubcommand, PRCommands,
    };

    /// 测试提取简单命令名
    #[test]
    fn test_extract_command_name_simple() {
        let command = Some(Commands::Check);
        let result = extract_command_name(&command);
        assert_eq!(result, Some("check".to_string()));
    }

    /// 测试提取 PR 命令名
    #[test]
    fn test_extract_pr_command_name() {
        use crate::cli::args::DryRunArgs;
        use crate::cli::args::JiraIdArg;
        let command = Some(Commands::Pr {
            subcommand: PRCommands::Create {
                jira_id: JiraIdArg { jira_id: None },
                title: None,
                description: None,
                dry_run: DryRunArgs { dry_run: false },
            },
        });
        let result = extract_command_name(&command);
        assert_eq!(result, Some("pr-create".to_string()));
    }

    /// 测试提取嵌套命令名（jira log download）
    #[test]
    fn test_extract_nested_command_name() {
        use crate::cli::args::JiraIdArg;
        let command = Some(Commands::Jira {
            subcommand: JiraSubcommand::Log {
                subcommand: LogSubcommand::Download {
                    jira_id: JiraIdArg { jira_id: None },
                },
            },
        });
        let result = extract_command_name(&command);
        assert_eq!(result, Some("jira-log-download".to_string()));
    }

    /// 测试提取分支命令名（branch ignore add）
    #[test]
    fn test_extract_branch_ignore_command_name() {
        let command = Some(Commands::Branch {
            subcommand: BranchSubcommand::Ignore {
                subcommand: IgnoreSubcommand::Add { branch_name: None },
            },
        });
        let result = extract_command_name(&command);
        assert_eq!(result, Some("branch-ignore-add".to_string()));
    }

    /// 测试无命令时返回 None
    #[test]
    fn test_extract_command_name_none() {
        let command = None;
        let result = extract_command_name(&command);
        assert_eq!(result, None);
    }
}
