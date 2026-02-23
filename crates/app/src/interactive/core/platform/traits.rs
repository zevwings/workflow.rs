//! 平台相关 Trait 定义

use prompt::{FormBuilder, FormResult};
use toolkit::Sensitive;

/// 平台账号 Trait
pub trait PlatformAccount: Clone + Sized {
    fn name(&self) -> &str;
    fn email(&self) -> &str;
    fn api_token(&self) -> &str;
    fn set_name(&mut self, name: String);
    fn set_email(&mut self, email: String);
    fn set_api_token(&mut self, token: String);

    /// 获取账号的唯一标识符（用于显示）
    ///
    /// 默认返回 email，子类可覆盖
    fn identifier(&self) -> &str {
        self.email()
    }

    fn display_with_marker(&self, is_current: bool) -> String {
        let marker = if is_current { " (current)" } else { "" };
        format!("{} ({}){}", self.name(), self.identifier(), marker)
    }

    fn display(&self) -> String {
        format!("{} ({})", self.name(), self.identifier())
    }

    fn masked_token(&self) -> String
    where
        String: Sensitive,
    {
        self.api_token().to_string().mask()
    }
}

/// 平台设置 Trait
pub trait PlatformSettings {
    type Account: PlatformAccount;

    fn accounts_mut(&mut self) -> &mut Vec<Self::Account>;
    fn accounts(&self) -> &Vec<Self::Account>;
    fn current(&self) -> &str;
    fn set_current(&mut self, name: String);

    fn has_accounts(&self) -> bool {
        !self.accounts().is_empty()
    }

    fn find_account(&self, name: &str) -> Option<&Self::Account> {
        self.accounts().iter().find(|acc| acc.name() == name)
    }

    fn find_account_mut(&mut self, name: &str) -> Option<&mut Self::Account> {
        self.accounts_mut().iter_mut().find(|acc| acc.name() == name)
    }

    fn get_current_account(&self) -> Option<&Self::Account> {
        let current = self.current();
        if !current.is_empty() {
            self.find_account(current)
        } else {
            self.accounts().first()
        }
    }

    fn remove_account(&mut self, name: &str) -> bool {
        if let Some(index) = self.accounts().iter().position(|acc| acc.name() == name) {
            self.accounts_mut().remove(index);
            true
        } else {
            false
        }
    }

    fn account_exists(&self, name: &str) -> bool {
        self.accounts().iter().any(|acc| acc.name() == name)
    }
}

/// 平台配置器 Trait
pub trait PlatformConfigurator {
    fn platform_name(&self) -> &str;

    fn build_add_form(&self) -> FormBuilder {
        FormBuilder::new().with_title(format!("New {} Account", self.platform_name()))
    }

    fn build_update_form(&self, _account_name: &str) -> FormBuilder {
        FormBuilder::new().with_title(format!("Update {} Account", self.platform_name()))
    }

    fn extract_basic_fields(&self, form_result: &FormResult) -> (String, String, String) {
        let name = form_result.get_string("name");
        let email = form_result.get_string("email");
        let api_token = form_result.get_string("api_token");
        (name, email, api_token)
    }

    fn verify(&self) -> Result<(), String> {
        Ok(())
    }

    fn auto_verify_in_command_setup(&self) -> bool {
        true
    }
}

/// 全局配置访问器 Trait
pub trait GlobalConfigAccessor<S: PlatformSettings> {
    fn get_settings_mut(&mut self) -> &mut S;
    fn get_settings(&self) -> &S;
}
