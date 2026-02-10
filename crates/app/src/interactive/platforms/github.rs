//! GitHub 工作流阶段 (v2)

use crate::interactive::core::context::{WorkflowContext, WorkflowMode};
use crate::interactive::core::platform::{
    add_account_generic, configure_platform, AccountSetMode, GlobalConfigAccessor, PlatformAccount,
    PlatformConfigurator, PlatformSettings,
};
use crate::interactive::core::stage::WorkflowStage;
use crate::interactive::display::VerificationResultFormatter;
use domain::{GitHubAccount, GitHubSettings, GlobalConfig, VerificationService};
use prompt::{
    br, info, success, warning, FormBuilder, FormResult, InputFormField, PasswordFormField,
    PromptError, SelectBuilder,
};
use std::error::Error;

/// GitHub 工作流阶段
pub struct GitHubStage;

impl WorkflowStage for GitHubStage {
    fn stage_name(&self) -> &'static str {
        "GitHub"
    }

    fn configure(&self, context: &mut WorkflowContext) -> Result<(), Box<dyn Error>> {
        configure_platform::<GitHubSettings, _, _, _>(
            context,
            &GitHubConfigurator,
            add_new_github_account,
            update_github_account,
        )
        .map_err(|e| e.into())
    }

    fn is_configured(&self, settings: &GlobalConfig) -> bool {
        !settings.github.current().is_empty()
    }

    fn verify(
        &self,
        service: &dyn VerificationService,
    ) -> Result<Box<dyn VerificationResultFormatter>, Box<dyn Error>> {
        service
            .verify_github_config()
            .map(|r| Box::new(r) as Box<dyn VerificationResultFormatter>)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

/// 获取 GitHub 阶段实例
pub fn github_stage() -> &'static dyn WorkflowStage {
    &GitHubStage
}

// =================================================================================
// GitHub 配置器和访问器
// =================================================================================

struct GitHubConfigurator;

impl PlatformConfigurator for GitHubConfigurator {
    fn platform_name(&self) -> &str {
        "GitHub"
    }
}

impl GitHubConfigurator {
    fn build_account_form_fields(&self, builder: FormBuilder) -> FormBuilder {
        builder
            .add_input(
                InputFormField::new("name", "请输入您的 GitHub 账户名称")
                    .result_title("您的 GitHub 账户名称")
                    .required(),
            )
            .add_input(
                InputFormField::new("email", "请输入您的 GitHub 邮箱")
                    .result_title("您的 GitHub 邮箱")
                    .required(),
            )
            .add_password(
                PasswordFormField::new("api_token", "请输入您的 GitHub Personal Access Token")
                    .result_title("您的 GitHub Personal Access Token")
                    .required(),
            )
    }

    fn build_update_form_fields(
        &self,
        builder: FormBuilder,
        current_name: String,
        current_email: String,
        current_token: String,
    ) -> FormBuilder {
        builder
            .add_input(
                InputFormField::new("name", "请输入您的 GitHub 账户名称")
                    .default(current_name)
                    .result_title("您的 GitHub 账户名称")
                    .required(),
            )
            .add_input(
                InputFormField::new("email", "请输入您的 GitHub 邮箱")
                    .default(current_email)
                    .result_title("您的 GitHub 邮箱")
                    .required(),
            )
            .add_password(
                PasswordFormField::new("api_token", "请输入您的 GitHub Personal Access Token")
                    .default(current_token)
                    .result_title("您的 GitHub Personal Access Token")
                    .required(),
            )
    }

    fn create_account_from_form(&self, form_result: &FormResult) -> Result<GitHubAccount, String> {
        let (name, email, api_token) = self.extract_basic_fields(form_result);

        if api_token.trim().is_empty() {
            return Err("添加新账户需要 GitHub API 令牌。".to_string());
        }

        let account_name = if name.trim().is_empty() {
            "default".to_string()
        } else {
            name.trim().to_string()
        };

        Ok(GitHubAccount {
            name: account_name,
            email: email.trim().to_string(),
            api_token,
        })
    }

    fn update_account_from_form(
        &self,
        account: &mut GitHubAccount,
        form_result: &FormResult,
        old_name: &str,
    ) -> Result<String, String> {
        let (new_name, email, api_token) = self.extract_basic_fields(form_result);

        let new_name_trimmed = new_name.trim().to_string();
        let updated_name = if !new_name_trimmed.is_empty() && new_name_trimmed != old_name {
            account.set_name(new_name_trimmed.clone());
            new_name_trimmed
        } else {
            old_name.to_string()
        };

        if !email.trim().is_empty() {
            account.set_email(email.trim().to_string());
        }

        if !api_token.trim().is_empty() {
            account.set_api_token(api_token);
        }

        Ok(updated_name)
    }
}

impl GlobalConfigAccessor<GitHubSettings> for GlobalConfig {
    fn get_settings_mut(&mut self) -> &mut GitHubSettings {
        &mut self.github
    }

    fn get_settings(&self) -> &GitHubSettings {
        &self.github
    }
}

impl PlatformAccount for GitHubAccount {
    fn name(&self) -> &str {
        &self.name
    }
    fn email(&self) -> &str {
        &self.email
    }
    fn api_token(&self) -> &str {
        &self.api_token
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn set_email(&mut self, email: String) {
        self.email = email;
    }
    fn set_api_token(&mut self, token: String) {
        self.api_token = token;
    }
}

impl PlatformSettings for GitHubSettings {
    type Account = GitHubAccount;

    fn accounts_mut(&mut self) -> &mut Vec<Self::Account> {
        &mut self.accounts
    }
    fn accounts(&self) -> &Vec<Self::Account> {
        &self.accounts
    }
    fn current(&self) -> &str {
        &self.current
    }
    fn set_current(&mut self, name: String) {
        self.current = name;
    }
}

// =================================================================================
// 账户操作
// =================================================================================

fn add_new_github_account(
    context: &mut WorkflowContext,
    set_mode: AccountSetMode,
) -> Result<String, String> {
    let configurator = GitHubConfigurator;
    add_account_generic::<GitHubSettings, _, _>(
        context,
        || {
            let builder = configurator.build_add_form();
            let builder = configurator.build_account_form_fields(builder);
            let form_result = builder.run().map_err(|e: PromptError| e.to_string())?;
            configurator.create_account_from_form(&form_result)
        },
        set_mode,
        "GitHub",
        None,
    )
}

fn update_github_account(context: &mut WorkflowContext) -> Result<(), String> {
    let configurator = GitHubConfigurator;
    let settings: &GitHubSettings = context.settings().get_settings();

    if !settings.has_accounts() {
        return Err("没有可更新的 GitHub 账户".to_string());
    }

    br!();
    info!("正在更新 GitHub 账户信息...");
    br!();

    let account_options: Vec<String> = settings
        .accounts()
        .iter()
        .map(|acc| acc.display_with_marker(settings.current() == acc.name()))
        .collect();

    let default_index = settings
        .accounts()
        .iter()
        .position(|acc| acc.name() == settings.current())
        .unwrap_or(0);

    let selected_account = SelectBuilder::new("请选择要更新的 GitHub 账户", account_options)
        .default(default_index)
        .result_title("要更新的账户")
        .prompt()
        .map_err(|e: PromptError| e.to_string())?;

    let account_name = selected_account
        .split(' ')
        .next()
        .ok_or_else(|| "解析账户名称失败".to_string())?
        .to_string();

    let settings: &GitHubSettings = context.settings().get_settings();
    let account = settings
        .find_account(&account_name)
        .ok_or_else(|| format!("未找到账户 '{}'", account_name))?;

    let old_name = account.name().to_string();
    let current_name = account.name().to_string();
    let current_email = account.email().to_string();
    let current_token = account.api_token().to_string();
    let was_current = settings.current() == old_name;

    br!();
    info!("正在更新账户: {}", account_name);
    info!("留空字段以保留当前值。");
    br!();

    let builder = configurator.build_update_form(&account_name);
    let builder =
        configurator.build_update_form_fields(builder, current_name, current_email, current_token);
    let form_result = builder.run().map_err(|e: PromptError| e.to_string())?;

    let (new_name, _, _) = configurator.extract_basic_fields(&form_result);
    let new_name_trimmed = new_name.trim().to_string();
    if !new_name_trimmed.is_empty() && new_name_trimmed != old_name {
        let settings: &GitHubSettings = context.settings().get_settings();
        if settings.account_exists(&new_name_trimmed) {
            return Err(format!(
                "账户名称 '{}' 已存在。请选择不同的名称。",
                new_name_trimmed
            ));
        }
    }

    let settings: &mut GitHubSettings = context.settings_mut().get_settings_mut();
    let account = settings
        .find_account_mut(&account_name)
        .ok_or_else(|| format!("未找到账户 '{}'", account_name))?;

    let updated_name = configurator.update_account_from_form(account, &form_result, &old_name)?;

    if was_current && updated_name != old_name {
        settings.set_current(updated_name.clone());
    }

    if context.mode() == WorkflowMode::Command {
        context.save().map_err(|e| format!("保存配置失败: {}", e))?;

        br!();
        success!("GitHub 账户 '{}' 更新成功。", updated_name);

        if configurator.auto_verify_in_command_setup() {
            br!();
            if let Err(err) = configurator.verify() {
                warning!("验证 GitHub 账户失败: {}", err);
            }
        }
    } else {
        br!();
        success!("GitHub 账户 '{}' 更新成功。", updated_name);
    }

    Ok(())
}
