//! 仓库配置检查命令
//!
//! 验证仓库级别的配置文件并运行验证检查。

use domain::{BranchTemplates, CommitTemplates, PullRequestsTemplates};
use prompt::{br, info, print, separator, success, warning, TableBuilder};

use crate::registry::{get_git_repository, get_path_service, get_repo_config_repository};

/// Repo Check 命令
pub struct RepoCheckCommand;

impl Default for RepoCheckCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl RepoCheckCommand {
    /// 创建新的 RepoCheckCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行仓库配置检查
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting repository check");
        br!();

        // 1. 检查是否在 Git 仓库中
        let repo_repo = get_git_repository();

        let repo_info = repo_repo.get_repo_info();
        if !repo_info.is_valid {
            return Err("Not in a Git repository".into());
        }

        // 获取仓库名
        let repo_name = repo_info.name.unwrap_or_else(|| "unknown".to_string());

        info!("Repository: {}", repo_name);
        br!();

        // 2. 显示配置路径
        separator!('=', 80, "Repository Configuration");
        br!();

        let path_service = get_path_service();
        let project_config_path = path_service.get_project_config_filepath()?;
        let user_config_path = path_service.get_user_config_filepath()?;
        info!("Project config: {:?}", project_config_path);
        info!("User config: {:?}", user_config_path);
        br!();

        // 3. 验证配置文件
        self.verify_repo_config()?;

        Ok(())
    }

    /// 验证仓库配置文件及内容
    fn verify_repo_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_repo = get_repo_config_repository();

        // repo_path 仅用于显示路径，实际加载配置时不需要传入

        // 创建验证表格
        let mut table = TableBuilder::new(vec!["Check Item", "Status", "Description"]);

        let path_service = get_path_service();
        let project_config_path = path_service.get_project_config_filepath()?;
        let user_config_path = path_service.get_user_config_filepath()?;

        // 检查项目配置文件
        let mut public_config_valid = false;
        if project_config_path.exists() {
            match config_repo.load_project_config() {
                Ok(_) => {
                    table = table.add_row(vec![
                        "Project Config File",
                        "✓",
                        "Config file exists and is valid",
                    ]);
                    public_config_valid = true;
                }
                Err(e) => {
                    table = table.add_row(vec![
                        "Project Config File",
                        "✗",
                        &format!("Config file exists but is invalid: {}", e),
                    ]);
                }
            }
        } else {
            table = table.add_row(vec![
                "Project Config File",
                "⚠",
                "Config file not found (optional)",
            ]);
        }

        // 检查用户配置文件
        let mut user_config_valid = false;
        if user_config_path.exists() {
            match config_repo.load_user_config() {
                Ok(config) => {
                    if !config.branch.prefix.is_empty() || !config.branch.ignore.is_empty() {
                        table = table.add_row(vec![
                            "User Config File",
                            "✓",
                            "Config file exists and is valid",
                        ]);
                        user_config_valid = true;
                    } else {
                        table = table.add_row(vec![
                            "User Config File",
                            "⚠",
                            "Config file exists but is empty",
                        ]);
                    }
                }
                Err(e) => {
                    table = table.add_row(vec![
                        "User Config File",
                        "✗",
                        &format!("Config file exists but is invalid: {}", e),
                    ]);
                }
            }
        } else {
            table = table.add_row(vec![
                "User Config File",
                "⚠",
                "Config file not found (optional)",
            ]);
        }

        // 验证分支配置
        let user_config = config_repo.load_user_config().unwrap_or_default();
        let prefix = &user_config.branch.prefix;
        if !prefix.is_empty() {
            table = table.add_row(vec![
                "Branch Prefix",
                "✓",
                &format!("Branch prefix configured: {}", prefix),
            ]);
        } else {
            table = table.add_row(vec![
                "Branch Prefix",
                "⚠",
                "Branch prefix not set (run 'workflow repo setup' to configure)",
            ]);
        }

        let ignore_branches = &user_config.branch.ignore;
        if !ignore_branches.is_empty() {
            table = table.add_row(vec![
                "Ignore Branches",
                "✓",
                &format!(
                    "Ignore branches configured: {} branch(es)",
                    ignore_branches.len()
                ),
            ]);
        } else {
            table = table.add_row(vec![
                "Ignore Branches",
                "⚠",
                "No ignore branches configured",
            ]);
        }

        // 验证模板配置（仅在项目配置有效时）
        let mut commit_template = String::new();
        let mut branch_templates: Vec<(&str, String)> = Vec::new();
        let mut pr_template = String::new();

        if public_config_valid {
            match config_repo.load_project_config() {
                Ok(project_config) => {
                    // 检查 use_scope
                    if project_config.use_scope {
                        table =
                            table.add_row(vec!["Commit Use Scope", "✓", "Use scope is enabled"]);
                    } else {
                        table =
                            table.add_row(vec!["Commit Use Scope", "⚠", "Use scope is disabled"]);
                    }

                    // 检查模板（只显示用户配置的，不显示默认值）
                    let default_commit_message = CommitTemplates::default_message_template();
                    let (new_table, ct) = self.check_template_string(
                        table,
                        "Commit Template",
                        &project_config.template.commit.message,
                        &default_commit_message,
                    );
                    table = new_table;
                    commit_template = ct;

                    // 检查 Branch Templates（各个类型，只显示用户配置的，不显示默认值）
                    let branch_config = &project_config.template.branch;
                    let default_branch = BranchTemplates::default();

                    let branch_types = [
                        ("Feature", &branch_config.feature, &default_branch.feature),
                        ("Bugfix", &branch_config.bugfix, &default_branch.bugfix),
                        ("Hotfix", &branch_config.hotfix, &default_branch.hotfix),
                        (
                            "Refactoring",
                            &branch_config.refactoring,
                            &default_branch.refactoring,
                        ),
                        ("Chore", &branch_config.chore, &default_branch.chore),
                    ];

                    for (name, template, default) in branch_types {
                        let default_val = default.as_str();
                        if !template.is_empty() && template != default_val {
                            table = table.add_row(vec![
                                &format!("Branch Template ({})", name),
                                "✓",
                                &format!("{} branch template is configured", name),
                            ]);
                            branch_templates.push((name, template.clone()));
                        }
                    }

                    // 如果没有任何用户配置的 branch template，不显示警告（因为默认值已经被隐藏）

                    let default_pr_body = PullRequestsTemplates::default_body_template();
                    let (new_table, pt) = self.check_template_string(
                        table,
                        "Pull Request Template",
                        &project_config.template.pull_requests.body,
                        &default_pr_body,
                    );
                    table = new_table;
                    pr_template = pt;
                }
                Err(_) => {
                    table = table.add_row(vec![
                        "Template Configuration",
                        "⚠",
                        "Project config not loaded, skipping template checks",
                    ]);
                }
            }
        } else {
            table = table.add_row(vec![
                "Template Configuration",
                "⚠",
                "Project config not loaded, skipping template checks",
            ]);
        }

        // 渲染表格
        table.print()?;
        br!();

        // 显示模板内容（只有当至少有一个模板被配置时才显示）
        if public_config_valid
            && (!commit_template.is_empty()
                || !branch_templates.is_empty()
                || !pr_template.is_empty())
        {
            info!("Your Template Configuration:");
            br!();

            self.display_template_content("Commit Template", &commit_template);

            if !branch_templates.is_empty() {
                separator!('-', 80, "Branch Template");
                for (template_type, content) in &branch_templates {
                    print!("{}:", template_type.to_lowercase());
                    print!("{}", content);
                    br!();
                }
            }

            self.display_template_content("Pull Request Template", &pr_template);
        }

        // 总结消息
        br!();
        if public_config_valid || user_config_valid {
            success!("Repository configuration check completed!");
        } else {
            warning!("No repository configuration found. Run 'workflow repo setup' to configure this repository.");
        }
        br!();

        Ok(())
    }

    /// 检查模板是否配置并添加表格行
    ///
    /// 只显示用户实际配置的模板，不显示默认值。
    ///
    /// # 参数
    ///
    /// * `table` - 表格构建器
    /// * `name` - 模板名称
    /// * `template` - 当前模板值
    /// * `default_template` - 默认模板值
    ///
    /// # 返回
    ///
    /// 返回 (新的 TableBuilder, 模板内容)。如果模板是默认值，返回空字符串。
    fn check_template_string(
        &self,
        table: TableBuilder,
        name: &str,
        template: &str,
        default_template: &str,
    ) -> (TableBuilder, String) {
        // 如果模板不为空且不等于默认值，说明用户配置了
        if !template.is_empty() && template != default_template {
            let new_table = table.add_row(vec![name, "✓", &format!("{} is configured", name)]);
            (new_table, template.to_string())
        } else {
            // 不显示默认值，所以不添加表格行
            (table, String::new())
        }
    }

    /// 显示模板内容（带分隔符）
    fn display_template_content(&self, name: &str, content: &str) {
        if content.is_empty() {
            return;
        }
        separator!('-', 80, name);
        print!("{}", content);
        br!();
    }
}
