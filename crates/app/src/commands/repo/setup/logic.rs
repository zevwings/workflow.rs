//! 仓库配置初始化命令
//!
//! 交互式初始化仓库级别的配置。

use std::io::{self, IsTerminal};

use color_eyre::{eyre::WrapErr, Result};

use domain::{
    BranchConfig, BranchTemplates, CommitTemplates, ProjectConfig, PullRequestsTemplates,
    TemplateConfig, UserConfig,
};
use prompt::{br, confirm, info, success, warning};
use prompt::{ConfirmFormField, FormBuilder, FormResult, GroupConfig, InputFormField};
use toolkit::{project_config_file, user_config_file};

use crate::registry;

/// 确保仓库配置存在
///
/// 此函数应在分支/提交/PR 操作开始时调用。
/// 检查 `repo setup` 是否已完成。
///
/// 如果配置不存在，将：
/// 1. 检查是否在交互式环境
/// 2. 提示用户运行 setup
/// 3. 如果用户确认，自动运行 setup
///
/// # 返回
///
/// 如果配置存在或用户选择跳过，返回 `Ok(())`。
/// 仅在 setup 必需且失败时返回错误。
pub fn ensure() -> Result<()> {
    // 1. 检查是否在交互式环境
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Ok(()); // 非交互式环境，跳过检查
    }

    // 2. 检查配置是否存在
    if RepoSetupCommand::check_config_exists()? {
        return Ok(()); // 配置存在，无需 setup
    }

    // 3. 配置不存在或不完整
    br!();
    warning!("Repository configuration not found or incomplete.");
    info!("Project-level configuration helps:");
    info!("  - Share branch prefix and commit template settings with your team");
    info!("  - Automatically configure commit message format");
    info!("  - Manage ignored branches");
    br!();

    // 4. 询问用户是否要运行 setup
    let should_setup = confirm!("Run 'workflow repo setup' to configure this repository?")
        .default(true)
        .prompt()
        .wrap_err("Failed to get user confirmation")?;

    if should_setup {
        // 5. 运行 setup
        br!();
        info!("Running repository setup...");
        br!();

        RepoSetupCommand::new().run().wrap_err("Failed to run repository setup")?;

        br!();
        success!("Repository configuration completed!");
        br!();
    } else {
        info!("Skipping repository setup. You can run 'workflow repo setup' later.");
    }

    Ok(())
}

/// Repo Setup 命令
pub struct RepoSetupCommand;

impl Default for RepoSetupCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl RepoSetupCommand {
    /// 创建新的 RepoSetupCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行仓库配置初始化
    ///
    /// 此方法可以被：
    /// 1. 用户直接调用：`workflow repo setup`
    /// 2. 其他命令调用：`repo::setup::RepoSetupCommand::new().run()`
    pub fn run(&self) -> Result<()> {
        info!("Starting repository configuration setup...");

        // 1. 检查是否在 Git 仓库中
        let repo_path = std::env::current_dir().wrap_err("Failed to get current directory")?;
        let repo_repo = registry::get_git_repo_repository();

        let repo_info = repo_repo.get_repo_info();
        if !repo_info.is_valid {
            return Err(color_eyre::eyre::eyre!(
                "Not in a Git repository. Please run this command in a Git repository."
            ));
        }

        // 获取仓库名
        let repo_name = repo_info.name.unwrap_or_else(|| "unknown".to_string());

        info!("Repository: {}", repo_name);
        br!();

        // 2. 加载现有配置（如果存在）
        let config_repo = registry::get_repo_config_repository();

        let existing_project_config = config_repo.load_project_config().ok();
        let existing_user_config = config_repo.load_user_config().ok();

        // 3. 收集配置信息
        let (project_config, user_config) =
            self.collect_config(&existing_project_config, &existing_user_config)?;

        // 4. 保存配置
        config_repo
            .save_project_config(&project_config)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to save project config: {}", e))?;
        config_repo
            .save_user_config(&user_config)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to save user config: {}", e))?;

        br!();
        success!(
            "Project configuration saved to: {}",
            project_config_file(&repo_path).display()
        );
        success!(
            "Personal configuration saved to: {}",
            user_config_file(&repo_path).display()
        );
        br!();
        success!("Repository configuration completed successfully!");
        br!();
        info!("You can commit the project template configuration to Git to share with your team.");

        Ok(())
    }

    /// 收集配置信息（交互式）
    fn collect_config(
        &self,
        existing_project: &Option<ProjectConfig>,
        existing_user: &Option<UserConfig>,
    ) -> Result<(ProjectConfig, UserConfig)> {
        // 准备现有值
        let current_prefix = existing_user.as_ref().and_then(|c| {
            if c.branch.prefix.is_empty() {
                None
            } else {
                Some(c.branch.prefix.clone())
            }
        });
        let current_use_scope = existing_project.as_ref().map(|c| c.use_scope).unwrap_or(false);

        // 检查模板配置是否存在
        let has_commit_template = existing_project
            .as_ref()
            .map(|c| !c.template.commit.default.is_empty())
            .unwrap_or(false);
        let has_branch_template = existing_project
            .as_ref()
            .map(|c| {
                !c.template.branch.feature.is_empty()
                    || !c.template.branch.bugfix.is_empty()
                    || !c.template.branch.hotfix.is_empty()
                    || !c.template.branch.refactoring.is_empty()
                    || !c.template.branch.chore.is_empty()
            })
            .unwrap_or(false);
        let has_pr_template = existing_project
            .as_ref()
            .map(|c| !c.template.pull_requests.default.is_empty())
            .unwrap_or(false);

        // 只使用现有配置作为默认值，如果不存在则不设置默认值（空字符串）
        let custom_commit_template = existing_project
            .as_ref()
            .and_then(|c| {
                let default = &c.template.commit.default;
                if !default.is_empty() && default != &CommitTemplates::default().default {
                    Some(default.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        // Branch templates: 收集各个类型的现有配置
        let existing_branch_templates = existing_project.as_ref().map(|c| {
            (
                c.template.branch.feature.clone(),
                c.template.branch.bugfix.clone(),
                c.template.branch.hotfix.clone(),
                c.template.branch.refactoring.clone(),
                c.template.branch.chore.clone(),
            )
        });

        let custom_pr_template = existing_project
            .as_ref()
            .and_then(|c| {
                let default = &c.template.pull_requests.default;
                if !default.is_empty() && default != &PullRequestsTemplates::default().default {
                    Some(default.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        // 显示当前配置状态
        if let Some(ref prefix) = current_prefix {
            info!(
                "Branch prefix configuration detected (current: {}).",
                prefix
            );
        } else {
            info!("No branch prefix configuration detected.");
        }

        if existing_project.is_some() {
            if has_commit_template || has_branch_template || has_pr_template {
                info!("Project template configuration detected.");
                if current_use_scope {
                    info!("  - Use scope: true");
                }
                if has_commit_template {
                    info!("  - Commit template: configured");
                }
                if has_branch_template {
                    info!("  - Branch template: configured");
                }
                if has_pr_template {
                    info!("  - Pull request template: configured");
                }
            } else {
                info!("No project template configuration detected.");
            }
        } else {
            info!("No project template configuration detected.");
        }

        br!();

        // 使用 FormBuilder 收集所有配置
        let form_result = FormBuilder::new()
        // Group 1: Personal Preference Configuration
        .add_group(
            "personal_preference",
            |g| {
                g.add_step(|s| {
                    let mut field = InputFormField::new("branch_prefix", "Please enter your branch prefix").result_title("Your branch prefix");
                    if let Some(ref prefix) = current_prefix {
                        field = field.default(prefix.clone());
                    }
                    // 不调用 required() 表示字段是可选的
                    s.add_input(field)
                })
            },
            GroupConfig::required()
                .with_title("Personal Preference Configuration")
                .with_description("These settings are personal preferences and will be saved to .workflow/user.toml (not committed to Git)."),
        )
        // Group 2: Project Template Configuration
        .add_group(
            "project_template",
            |g| {
                g.add_step(|s| {
                    // Use scope
                    s.add_confirm(ConfirmFormField::new("use_scope", "Use scope for commit messages?")
                        .default(current_use_scope))
                })
                .add_step(|s| {
                    // Commit template configuration
                    s.add_confirm(ConfirmFormField::new("configure_commit_template", "Do you want to configure commit templates?")
                        .default(has_commit_template))
                })
                .add_step_if(
                    |result| result.get_bool("configure_commit_template"),
                    |s| {
                        let mut field = InputFormField::new("custom_commit_template", "Please enter your custom commit template")
                            .result_title("Your custom commit template");
                        if !custom_commit_template.is_empty() {
                            field = field.default(custom_commit_template);
                        }
                        s.add_input(field)
                    },
                )
                .add_step(|s| {
                    // Branch template configuration
                    s.add_confirm(ConfirmFormField::new("configure_branch_template", "Do you want to configure branch templates?")
                        .default(has_branch_template))
                })
                .add_step_if(
                    |result| result.get_bool("configure_branch_template"),
                    |s| {
                        let branch_types = [
                            ("feature", "Feature"),
                            ("bugfix", "Bugfix"),
                            ("hotfix", "Hotfix"),
                            ("refactoring", "Refactoring"),
                            ("chore", "Chore"),
                        ];
                        branch_types.iter().fold(s, |step, (key, name)| {
                            let mut field = InputFormField::new(
                                format!("branch_template_{}", key),
                                format!("Please enter your {} branch template", name.to_lowercase()),
                            )
                            .result_title(format!("{} branch template", name));
                            if let Some(ref templates) = existing_branch_templates {
                                let template = match *key {
                                    "feature" => &templates.0,
                                    "bugfix" => &templates.1,
                                    "hotfix" => &templates.2,
                                    "refactoring" => &templates.3,
                                    "chore" => &templates.4,
                                    _ => &String::new(),
                                };
                                if !template.is_empty() {
                                    field = field.default(template.clone());
                                }
                            }
                            step.add_input(field)
                        })
                    },
                )
                .add_step(|s| {
                    // PR template configuration
                    s.add_confirm(ConfirmFormField::new("configure_pr_template", "Do you want to configure pull request templates?")
                        .default(has_pr_template))
                })
                .add_step_if(
                    |result| result.get_bool("configure_pr_template"),
                    |s| {
                        let mut field = InputFormField::new("custom_pr_template", "Please enter your custom pull request template")
                            .result_title("Your custom pull request template");
                        if !custom_pr_template.is_empty() {
                            field = field.default(custom_pr_template);
                        }
                        s.add_input(field)
                    },
                )
            },
            GroupConfig::required()
                .with_title("Project Template Configuration")
                .with_description("These settings are project standards and will be saved to .workflow/config.toml (can be committed to Git)."),
        )
        .run()
        .wrap_err("Failed to collect repository configuration")?;

        // 处理结果：构建 UserConfig
        let branch_prefix = {
            let prefix_str = form_result.get_string("branch_prefix");
            let trimmed = prefix_str.trim();
            if !trimmed.is_empty() {
                Some(trimmed.to_string())
            } else {
                current_prefix.clone()
            }
        };

        let user_config = UserConfig {
            branch: BranchConfig {
                prefix: branch_prefix.unwrap_or_default(),
                ignore: existing_user.as_ref().map(|c| c.branch.ignore.clone()).unwrap_or_default(),
            },
        };

        // 处理结果：构建 ProjectConfig
        // ConfirmFormField 总是会设置一个布尔值，所以直接使用 get_bool
        // 如果字段不存在（不应该发生），get_bool 返回 false，但我们有默认值作为后备
        let use_scope = form_result.get_bool("use_scope");

        // 构建模板配置
        // 从现有配置开始，如果不存在则创建空的（不包含默认值）
        let mut template =
            existing_project.as_ref().map(|c| c.template.clone()).unwrap_or_else(|| {
                // 创建空的模板配置，不包含默认值
                TemplateConfig {
                    engine: "handlebars".to_string(),
                    branch: BranchTemplates {
                        feature: String::new(),
                        bugfix: String::new(),
                        hotfix: String::new(),
                        refactoring: String::new(),
                        chore: String::new(),
                    },
                    commit: CommitTemplates {
                        default: String::new(),
                    },
                    pull_requests: PullRequestsTemplates {
                        default: String::new(),
                    },
                }
            });

        // 更新模板配置
        self.update_template_from_form(&mut template, &form_result);

        let project_config = ProjectConfig {
            use_scope,
            template,
        };

        Ok((project_config, user_config))
    }

    /// 检查配置是否存在
    ///
    /// 检查用户配置中是否有 prefix 或 ignore 配置。
    pub fn check_config_exists() -> Result<bool> {
        let config_repo = registry::get_repo_config_repository();

        let user_config = config_repo
            .load_user_config()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to load user config: {}", e))?;

        Ok(!user_config.branch.prefix.is_empty() || !user_config.branch.ignore.is_empty())
    }

    /// 从表单结果更新模板配置
    fn update_template_from_form(&self, template: &mut TemplateConfig, form_result: &FormResult) {
        // 更新 commit template
        if form_result.get_bool("configure_commit_template") {
            let template_str = form_result.get_string("custom_commit_template");
            template.commit.default = template_str.trim().to_string();
        }

        // 更新 branch templates
        if form_result.get_bool("configure_branch_template") {
            let branch_types = [
                ("feature", &mut template.branch.feature),
                ("bugfix", &mut template.branch.bugfix),
                ("hotfix", &mut template.branch.hotfix),
                ("refactoring", &mut template.branch.refactoring),
                ("chore", &mut template.branch.chore),
            ];

            for (key, field) in branch_types {
                let value = form_result.get_string(&format!("branch_template_{}", key));
                *field = value.trim().to_string();
            }
        }

        // 更新 PR template
        if form_result.get_bool("configure_pr_template") {
            let template_str = form_result.get_string("custom_pr_template");
            template.pull_requests.default = template_str.trim().to_string();
        }
    }
}
