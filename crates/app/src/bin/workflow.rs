//! Workflow CLI 主入口
//!
//! 这里只负责解析顶层命令，将实际逻辑委托给 `commands` 模块。

use clap::Parser;
use prompt::terminal_state;
use toolkit::{log_info, logger, register_spinner_handlers, LoggerConfig};

use app::cli::{
    AliasCommand, BranchSubcommand, Cli, Command, CompletionCommand, GithubCommand,
    IgnoreSubcommand, JiraCommand, LlmCommand, LogCommand, PrSubcommand, RepoCommand,
    StashSubcommand, TagSubcommand, UninstallArgs, UpdateArgs,
};
#[cfg(feature = "develop")]
use app::cli::{CommitSubcommand, RollbackCommand};
use app::commands;
use app::registry::{get_global_config_repository, get_path_service};

/// 获取命令名称字符串（用于日志文件名）
fn get_command_name(command: &Command) -> Option<&'static str> {
    match command {
        Command::Version => Some("version"),
        Command::Check => Some("check"),
        Command::Setup => Some("setup"),
        Command::Update(_) => Some("update"),
        Command::Uninstall(_) => Some("uninstall"),
        Command::Repo(_) => Some("repo"),
        Command::Log(_) => Some("log"),
        Command::Llm(_) => Some("llm"),
        Command::Github(_) => Some("github"),
        Command::Jira(_) => Some("jira"),
        Command::Branch(_) => Some("branch"),
        #[cfg(feature = "develop")]
        Command::Commit(_) => Some("commit"),
        Command::Stash(_) => Some("stash"),
        Command::Tag(_) => Some("tag"),
        Command::Pr(_) => Some("pr"),
        #[cfg(feature = "develop")]
        Command::Push => Some("push"),
        #[cfg(feature = "develop")]
        Command::Pull => Some("pull"),
        Command::Completion(_) => Some("completion"),
        Command::Alias(_) => Some("alias"),
        #[cfg(feature = "develop")]
        Command::Rollback(_) => Some("rollback"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 shaku 模块（通过 Lazy 自动初始化）
    // 访问模块会触发初始化
    // 模块已通过 Lazy 自动初始化，无需手动调用

    let cli = Cli::parse();

    let global_config_repository = get_global_config_repository();
    let path_service = get_path_service();

    if let Ok(global_config) = global_config_repository.load() {
        // RUST_LOG 优先于配置文件，便于调试时用 RUST_LOG=debug 看详细日志
        let level = std::env::var("RUST_LOG")
            .ok()
            .filter(|s| !s.is_empty())
            .or(global_config.log.level.clone());
        let logger_config = LoggerConfig::new(
            level,
            global_config.log.format.clone(),
            global_config.log.enable_trace_console.unwrap_or(false),
            path_service.get_logs_dir()?,
        );

        // 初始化 logger（从配置文件读取 LogSettings 并转换为 LoggerConfig）
        // 注意：如果配置加载失败或 logger 初始化失败，我们继续执行（不阻塞应用启动）
        let command_name = get_command_name(&cli.command);

        // 忽略初始化错误（可能已经初始化过了，或者日志级别为 off）
        let _ = logger::init(command_name, &logger_config);

        // 注册终端处理器，让 tracing 输出时能协调 spinner/progress
        register_spinner_handlers(terminal_state::suspend, terminal_state::resume);

        log_info!(
            "Logger initialized (console={}, level={})",
            logger_config.enable_console,
            logger_config.level.as_deref().unwrap_or("off")
        );
    }

    // 额外的构建信息通过环境变量注入（如果存在）
    let version = env!("CARGO_PKG_VERSION");
    let build_date = option_env!("WORKFLOW_BUILD_DATE").unwrap_or("unknown");
    let git_commit = option_env!("WORKFLOW_GIT_COMMIT").unwrap_or("unknown");

    match cli.command {
        Command::Version => {
            let cmd = commands::version::VersionCommand::new(version, build_date, git_commit);
            cmd.run()?;
        }
        Command::Check => {
            let cmd = commands::check::CheckCommand::new();
            cmd.run()?;
        }
        Command::Setup => {
            let cmd = commands::setup::SetupCommand::new();
            cmd.run()?;
        }
        Command::Update(UpdateArgs {
            target_version,
            force,
            github_token,
        }) => {
            // 优先使用命令行参数，其次使用环境变量
            let token = github_token.or_else(|| std::env::var("GITHUB_TOKEN").ok());
            let cmd = commands::update::UpdateCommand::new(target_version, force, token);
            cmd.run()?;
        }
        Command::Uninstall(UninstallArgs { keep_config, .. }) => {
            let cmd = commands::uninstall::UninstallCommand::new(keep_config);
            cmd.run()?;
        }
        Command::Repo(repo_cmd) => match repo_cmd {
            RepoCommand::Setup => {
                let cmd = commands::repo::RepoSetupCommand::new();
                cmd.run()?;
            }
            RepoCommand::Status => {
                let cmd = commands::repo::RepoStatusCommand::new();
                cmd.run()?;
            }
            RepoCommand::Check => {
                let cmd = commands::repo::RepoCheckCommand::new();
                cmd.run()?;
            }
        },
        Command::Log(log_cmd) => match log_cmd {
            LogCommand::Setup => {
                let cmd = commands::log::LogSetupCommand::new();
                cmd.run()?;
            }
            LogCommand::Check => {
                let cmd = commands::log::LogCheckCommand::new();
                cmd.run()?;
            }
        },
        Command::Llm(llm_cmd) => match llm_cmd {
            LlmCommand::Check => {
                let cmd = commands::llm::LlmCheckCommand::new();
                cmd.run()?;
            }
            LlmCommand::Setup => {
                let cmd = commands::llm::LlmSetupCommand::new();
                cmd.run()?;
            }
        },
        Command::Github(github_cmd) => match github_cmd {
            GithubCommand::Check => {
                let cmd = commands::github::GithubCheckCommand::new();
                cmd.run()?;
            }
            GithubCommand::Setup => {
                let cmd = commands::github::GithubSetupCommand::new();
                cmd.run()?;
            }
        },
        Command::Jira(jira_cmd) => match jira_cmd {
            JiraCommand::Check => {
                let cmd = commands::jira::JiraCheckCommand::new();
                cmd.run()?;
            }
            JiraCommand::Setup => {
                let cmd = commands::jira::JiraSetupCommand::new();
                cmd.run()?;
            }
            JiraCommand::Info(args) => {
                let format = args.get_format();
                let cmd = commands::jira::JiraInfoCommand::new(args.jira_id.into_option(), format);
                cmd.run()?;
            }
            JiraCommand::Attachments(args) => {
                let cmd = commands::jira::JiraAttachmentsCommand::new(args.jira_id.into_option());
                cmd.run()?;
            }
            JiraCommand::Clean(args) => {
                let cmd =
                    commands::jira::JiraCleanCommand::new(args.jira_id.into_option(), args.all);
                cmd.run()?;
            }
            #[cfg(feature = "develop")]
            JiraCommand::Comment(args) => {
                let cmd = commands::jira::JiraCommentCommand::new(
                    args.jira_id.into_option(),
                    args.message.clone(),
                );
                cmd.run()?;
            }
            #[cfg(feature = "develop")]
            JiraCommand::Status(args) => {
                let cmd = commands::jira::JiraStatusCommand::new(args.jira_id);
                cmd.run()?;
            }
            #[cfg(feature = "develop")]
            JiraCommand::Transition(args) => {
                let cmd = commands::jira::JiraTransitionCommand::new(args.jira_id);
                cmd.run()?;
            }
            #[cfg(feature = "develop")]
            JiraCommand::Assign(args) => {
                let cmd = commands::jira::JiraAssignCommand::new(args.jira_id);
                cmd.run()?;
            }
        },
        Command::Branch(branch_cmd) => match branch_cmd {
            BranchSubcommand::Create {
                jira_id,
                from_default,
                dry_run,
            } => {
                let cmd = commands::branch::create::BranchCreateCommand::new(
                    jira_id.into_option(),
                    from_default,
                    dry_run.is_dry_run(),
                );
                cmd.run()?;
            }
            BranchSubcommand::Switch { branch_name } => {
                let cmd = commands::branch::switch::BranchSwitchCommand::new(branch_name.clone());
                cmd.run()?;
            }
            BranchSubcommand::Rename => {
                let cmd = commands::branch::rename::BranchRenameCommand::new();
                cmd.run()?;
            }
            BranchSubcommand::Clean { dry_run } => {
                let cmd = commands::branch::clean::BranchCleanCommand::new(dry_run.is_dry_run());
                cmd.run()?;
            }
            #[cfg(feature = "develop")]
            BranchSubcommand::InferSource => {
                let cmd = commands::branch::infer_source::BranchInferSourceCommand::new();
                cmd.run()?;
            }
            BranchSubcommand::Ignore(ignore_cmd) => match ignore_cmd {
                IgnoreSubcommand::Add { branch_name } => {
                    let cmd =
                        commands::branch::ignore::BranchIgnoreAddCommand::new(branch_name.clone());
                    cmd.run()?;
                }
                IgnoreSubcommand::Remove { branch_name } => {
                    let cmd = commands::branch::ignore::BranchIgnoreRemoveCommand::new(branch_name);
                    cmd.run()?;
                }
                IgnoreSubcommand::List => {
                    let cmd = commands::branch::ignore::BranchIgnoreListCommand::new();
                    cmd.run()?;
                }
            },
            BranchSubcommand::Remove {
                branch_name,
                local_only,
                remote_only,
                dry_run,
                force,
            } => {
                let cmd = commands::branch::remove::BranchRemoveCommand::new(
                    branch_name.clone(),
                    local_only,
                    remote_only,
                    dry_run.is_dry_run(),
                    force.is_force(),
                );
                cmd.run()?;
            }
        },
        #[cfg(feature = "develop")]
        Command::Commit(commit_cmd) => {
            if let Some(sub) = &commit_cmd.subcommand {
                match sub {
                    CommitSubcommand::CommitToMerge {
                        source_branch,
                        target_branch,
                    } => {
                        let cmd = commands::commit::CommitToMergeCommand::new(
                            source_branch.clone(),
                            target_branch.clone(),
                        );
                        cmd.run()?;
                    }
                    CommitSubcommand::CommitFiles { ref_or_sha } => {
                        let cmd = commands::commit::CommitFilesCommand::new(ref_or_sha.clone());
                        cmd.run()?;
                    }
                    CommitSubcommand::CommitDiff { ref_or_sha } => {
                        let cmd = commands::commit::CommitDiffCommand::new(ref_or_sha.clone());
                        cmd.run()?;
                    }
                }
            } else {
                let cmd = commands::commit::CommitCreateCommand::new(
                    commit_cmd.all,
                    commit_cmd.push,
                    commit_cmd.dry_run.is_dry_run(),
                    commit_cmd.message.clone(),
                );
                cmd.run()?;
            }
        }
        #[cfg(feature = "develop")]
        Command::Push => {
            let cmd = commands::sync::push::PushCommand::new();
            cmd.run()?;
        }
        #[cfg(feature = "develop")]
        Command::Pull => {
            let cmd = commands::sync::pull::PullCommand::new();
            cmd.run()?;
        }
        Command::Stash(stash_cmd) => match stash_cmd {
            StashSubcommand::Push => {
                commands::stash::StashPushCommand::run()?;
            }
            StashSubcommand::Pop => {
                commands::stash::StashPopCommand::run()?;
            }
            StashSubcommand::Apply => {
                commands::stash::StashApplyCommand::run()?;
            }
            StashSubcommand::Drop => {
                commands::stash::StashDropCommand::run()?;
            }
            StashSubcommand::List => {
                commands::stash::StashListCommand::run()?;
            }
        },
        Command::Tag(tag_cmd) => match tag_cmd {
            TagSubcommand::Create {
                tag_name,
                target,
                message,
                local,
                force,
            } => {
                let cmd = commands::tag::TagCreateCommand::new(
                    tag_name,
                    target,
                    message,
                    local,
                    force.is_force(),
                );
                cmd.run()?;
            }
            TagSubcommand::Remove {
                tag_name,
                local,
                remote,
                pattern,
                dry_run,
                force,
            } => {
                let cmd = commands::tag::TagRemoveCommand::new(
                    tag_name,
                    local,
                    remote,
                    pattern,
                    dry_run.is_dry_run(),
                    force.is_force(),
                );
                cmd.run()?;
            }
        },
        Command::Pr(pr_cmd) => match pr_cmd {
            PrSubcommand::Create { jira_id, dry_run } => {
                let cmd = commands::pr::PullRequestCreateCommand::new(
                    jira_id.into_option(),
                    dry_run.is_dry_run(),
                );
                cmd.run()?;
            }
            PrSubcommand::List { state, limit } => {
                let cmd = commands::pr::PullRequestListCommand::new(state.clone(), limit);
                cmd.run()?;
            }
            PrSubcommand::Comment { pr_id, comment } => {
                let cmd =
                    commands::pr::PullRequestCommentCommand::new(pr_id.clone(), comment.clone());
                cmd.run()?;
            }
            PrSubcommand::Update {
                pr_id,
                message,
                dry_run,
            } => {
                let cmd = commands::pr::PullRequestUpdateCommand::new(
                    pr_id.clone(),
                    message.clone(),
                    dry_run.is_dry_run(),
                );
                cmd.run()?;
            }
            PrSubcommand::Merge { pr_id, force } => {
                let cmd = commands::pr::PullRequestMergeCommand::new(
                    Some(pr_id.clone()),
                    force.is_force(),
                );
                cmd.run()?;
            }
            PrSubcommand::Close { pr_id } => {
                let cmd = commands::pr::PullRequestCloseCommand::new(pr_id.clone());
                cmd.run()?;
            }
            PrSubcommand::Approve { pr_id } => {
                let cmd = commands::pr::PullRequestApproveCommand::new(pr_id.clone());
                cmd.run()?;
            }
            PrSubcommand::Reword { dry_run } => {
                let cmd = commands::pr::PullRequestRewordCommand::new(dry_run.is_dry_run());
                cmd.run()?;
            }
        },
        Command::Completion(completion_cmd) => match completion_cmd {
            CompletionCommand::Generate { shell, output } => {
                let cmd = commands::completion::CompletionGenerateCommand::new(
                    shell.clone(),
                    output.clone(),
                );
                cmd.run()?;
            }
            CompletionCommand::Check => {
                let cmd = commands::completion::CompletionCheckCommand::new();
                cmd.run()?;
            }
            CompletionCommand::Remove { all } => {
                let cmd = commands::completion::CompletionRemoveCommand::new(all);
                cmd.run()?;
            }
        },
        Command::Alias(alias_cmd) => match alias_cmd {
            AliasCommand::List => {
                let cmd = commands::alias::AliasListCommand::new();
                cmd.run()?;
            }
            AliasCommand::Add {
                name,
                command,
                force,
            } => {
                let cmd = commands::alias::AliasAddCommand::new(name, command, force);
                cmd.run()?;
            }
            AliasCommand::Remove { name } => {
                let cmd = commands::alias::AliasRemoveCommand::new(name);
                cmd.run()?;
            }
        },
        #[cfg(feature = "develop")]
        Command::Rollback(rollback_cmd) => match rollback_cmd {
            RollbackCommand::Backup => {
                let cmd = commands::rollback::BackupCommand::new();
                cmd.run()?;
            }
            RollbackCommand::Restore { backup_dir } => {
                let cmd = commands::rollback::RestoreCommand::new(backup_dir);
                cmd.run()?;
            }
        },
    }

    Ok(())
}
