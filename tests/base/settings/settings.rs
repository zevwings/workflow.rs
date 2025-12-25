//! Settings 配置管理测试
//!
//! 测试 Settings 模块的核心功能，包括：
//! - 配置结构体创建和字段访问
//! - 配置加载和默认值处理
//! - 配置验证和序列化
//! - 表格显示结构测试

use pretty_assertions::assert_eq;
use std::collections::HashMap;
use workflow::base::settings::settings::{
    default_download_base_dir, GitHubAccount, GitHubSettings, JiraSettings, LLMProviderSettings,
    LogSettings,
};
use workflow::base::settings::{
    GitHubAccountListRow, GitHubAccountRow, JiraConfigRow, LLMConfigRow, LLMSettings, Settings,
};

// ==================== Helper Functions ====================

/// 创建测试用的 JiraSettings
fn create_test_jira_settings() -> JiraSettings {
    JiraSettings {
        email: Some("test@example.com".to_string()),
        api_token: Some("test_token_123".to_string()),
        service_address: Some("https://company.atlassian.net".to_string()),
    }
}

/// 创建测试用的 GitHubSettings
fn create_test_github_settings() -> GitHubSettings {
    GitHubSettings {
        accounts: vec![
            GitHubAccount {
                name: "personal".to_string(),
                email: "personal@example.com".to_string(),
                api_token: "ghp_personal_token".to_string(),
            },
            GitHubAccount {
                name: "work".to_string(),
                email: "work@company.com".to_string(),
                api_token: "ghp_work_token".to_string(),
            },
        ],
        current: Some("personal".to_string()),
    }
}

/// 创建测试用的 LLMSettings
fn create_test_llm_settings() -> LLMSettings {
    LLMSettings {
        provider: "openai".to_string(),
        language: "English".to_string(),
        openai: LLMProviderSettings {
            url: None,
            key: Some("sk-test_openai_key".to_string()),
            model: Some("gpt-4".to_string()),
        },
        deepseek: LLMProviderSettings {
            url: None,
            key: Some("sk-test_deepseek_key".to_string()),
            model: Some("deepseek-chat".to_string()),
        },
        proxy: LLMProviderSettings {
            url: Some("https://api.proxy.com".to_string()),
            key: Some("proxy_key".to_string()),
            model: Some("proxy-model".to_string()),
        },
    }
}

// ==================== JiraSettings 测试 ====================

/// 测试 JiraSettings 创建和字段访问
#[test]
fn test_jira_settings_creation() {
    let jira_settings = create_test_jira_settings();

    assert_eq!(jira_settings.email, Some("test@example.com".to_string()));
    assert_eq!(jira_settings.api_token, Some("test_token_123".to_string()));
    assert_eq!(
        jira_settings.service_address,
        Some("https://company.atlassian.net".to_string())
    );
}

/// 测试 JiraSettings 默认实现
#[test]
fn test_jira_settings_default() {
    let default_jira = JiraSettings::default();

    assert_eq!(default_jira.email, None);
    assert_eq!(default_jira.api_token, None);
    assert_eq!(default_jira.service_address, None);
}

/// 测试 JiraSettings 克隆和调试输出
#[test]
fn test_jira_settings_clone_and_debug() {
    let original_jira = create_test_jira_settings();
    let cloned_jira = original_jira.clone();

    assert_eq!(original_jira.email, cloned_jira.email);
    assert_eq!(original_jira.api_token, cloned_jira.api_token);
    assert_eq!(original_jira.service_address, cloned_jira.service_address);

    // 测试调试输出
    let debug_str = format!("{:?}", original_jira);
    assert!(debug_str.contains("JiraSettings"));
    assert!(debug_str.contains("test@example.com"));
}

// ==================== GitHubSettings Tests ====================

#[test]
fn test_github_settings_creation_with_valid_accounts_creates_settings() {
    // Arrange: 准备测试用的 GitHubSettings
    let github_settings = create_test_github_settings();

    // Act: 验证设置创建
    // (验证在 Assert 中完成)

    // Assert: 验证账号数量和当前账号设置正确
    assert_eq!(github_settings.accounts.len(), 2);
    assert_eq!(github_settings.current, Some("personal".to_string()));
    let personal_account = &github_settings.accounts[0];
    assert_eq!(personal_account.name, "personal");
    assert_eq!(personal_account.email, "personal@example.com");
    assert_eq!(personal_account.api_token, "ghp_personal_token");
}

#[test]
fn test_github_settings_current_account_with_valid_settings_returns_account() {
    // Arrange: 准备测试用的 GitHubSettings
    let github_settings = create_test_github_settings();

    // Act: 获取当前账号和 token
    let current_account = github_settings.get_current_account();
    let current_token = github_settings.get_current_token();

    // Assert: 验证当前账号和 token 正确
    assert!(current_account.is_some());
    let account = current_account.expect("current account should exist");
    assert_eq!(account.name, "personal");
    assert_eq!(account.email, "personal@example.com");
    assert_eq!(current_token, Some("ghp_personal_token"));
}

#[test]
fn test_github_settings_no_current_account_with_none_current_returns_first_account() {
    // Arrange: 准备 GitHubSettings（current 为 None）
    let mut github_settings = create_test_github_settings();
    github_settings.current = None;

    // Act: 获取当前账号
    let current_account = github_settings.get_current_account();

    // Assert: 验证返回第一个账号
    assert!(current_account.is_some());
    let account = current_account.expect("should return first account");
    assert_eq!(account.name, "personal");
}

#[test]
fn test_github_settings_empty_accounts_with_no_accounts_returns_none() {
    // Arrange: 准备空的 GitHubSettings
    let empty_github = GitHubSettings {
        accounts: vec![],
        current: None,
    };

    // Act: 获取当前账号和 token
    let current_account = empty_github.get_current_account();
    let current_token = empty_github.get_current_token();

    // Assert: 验证返回 None
    assert!(current_account.is_none());
    assert!(current_token.is_none());
}

#[test]
fn test_github_settings_default_with_no_parameters_creates_empty_settings() {
    // Arrange: 准备创建默认设置

    // Act: 创建默认的 GitHubSettings
    let default_github = GitHubSettings::default();

    // Assert: 验证账号列表为空且当前账号为 None
    assert!(default_github.accounts.is_empty());
    assert_eq!(default_github.current, None);
}

// ==================== LLMSettings Tests ====================

#[test]
fn test_llm_settings_creation_with_valid_providers_creates_settings() {
    // Arrange: 准备测试用的 LLMSettings
    let llm_settings = create_test_llm_settings();

    // Act: 验证设置创建
    // (验证在 Assert 中完成)

    // Assert: 验证提供商和语言设置正确，以及各个提供商配置正确
    assert_eq!(llm_settings.provider, "openai");
    assert_eq!(llm_settings.language, "English");
    assert_eq!(
        llm_settings.openai.key,
        Some("sk-test_openai_key".to_string())
    );
    assert_eq!(llm_settings.openai.model, Some("gpt-4".to_string()));
    assert_eq!(
        llm_settings.deepseek.key,
        Some("sk-test_deepseek_key".to_string())
    );
    assert_eq!(
        llm_settings.proxy.url,
        Some("https://api.proxy.com".to_string())
    );
}

#[test]
fn test_llm_settings_current_provider_with_valid_settings_returns_provider() {
    // Arrange: 准备测试用的 LLMSettings
    let llm_settings = create_test_llm_settings();

    // Act: 获取当前提供商
    let current_provider = llm_settings.current_provider();

    // Assert: 验证当前提供商配置正确
    assert_eq!(current_provider.key, Some("sk-test_openai_key".to_string()));
    assert_eq!(current_provider.model, Some("gpt-4".to_string()));
}

#[test]
fn test_llm_settings_defaults_with_no_parameters_returns_default_values() {
    // Arrange: 准备检查默认值

    // Act & Assert: 验证各个默认值方法返回正确的值
    assert_eq!(LLMSettings::default_provider(), "openai");
    assert_eq!(LLMSettings::default_language(), "en");
    assert_eq!(LLMSettings::default_model("openai"), "gpt-4.0");
    assert_eq!(LLMSettings::default_model("deepseek"), "deepseek-chat");
    assert_eq!(LLMSettings::default_model("unknown"), ""); // proxy 必须输入，没有默认值
}

#[test]
fn test_llm_provider_settings_creation_with_valid_fields_creates_settings() {
    // Arrange: 准备提供商设置字段值
    let url = Some("https://api.example.com".to_string());
    let key = Some("test_key".to_string());
    let model = Some("test_model".to_string());

    // Act: 创建 LLMProviderSettings 实例
    let provider_settings = LLMProviderSettings {
        url: url.clone(),
        key: key.clone(),
        model: model.clone(),
    };

    // Assert: 验证字段值正确
    assert_eq!(provider_settings.url, url);
    assert_eq!(provider_settings.key, key);
    assert_eq!(provider_settings.model, model);

    // 测试默认值
    let default_provider = LLMProviderSettings::default();
    assert_eq!(default_provider.url, None);
    assert_eq!(default_provider.key, None);
    assert_eq!(default_provider.model, None);
}

// ==================== LogSettings 测试 ====================

/// 测试 LogSettings 创建和默认值
#[test]
fn test_log_settings_creation() {
    let log_settings = LogSettings {
        output_folder_name: Some("custom_logs".to_string()),
        download_base_dir: Some("/custom/path".to_string()),
        level: Some("debug".to_string()),
        enable_trace_console: Some(true),
    };

    assert_eq!(log_settings.get_output_folder_name(), "custom_logs");
    assert_eq!(
        log_settings.download_base_dir,
        Some("/custom/path".to_string())
    );
    assert_eq!(log_settings.level, Some("debug".to_string()));
    assert_eq!(log_settings.enable_trace_console, Some(true));
}

/// 测试 LogSettings 默认实现
#[test]
fn test_log_settings_default() {
    let default_log = LogSettings::default();

    assert_eq!(default_log.get_output_folder_name(), "logs");
    assert_eq!(default_log.output_folder_name, None);
    assert_eq!(default_log.download_base_dir, None);
    assert_eq!(default_log.level, None);
    assert_eq!(default_log.enable_trace_console, None);
}

/// 测试 LogSettings 默认方法
#[test]
fn test_log_settings_default_methods() {
    assert_eq!(LogSettings::default_log_folder(), "logs");

    // default_download_base_dir_option() returns None (to indicate using default without writing to config)
    let default_base_dir_option = LogSettings::default_download_base_dir_option();
    assert_eq!(default_base_dir_option, None);

    // Check the actual default path function
    let default_base_dir = default_download_base_dir();
    assert!(default_base_dir.contains("Workflow"));
}

// ==================== Settings 主结构测试 ====================

/// 测试 Settings 创建和默认实现
#[test]
fn test_settings_creation() {
    let settings = Settings {
        jira: create_test_jira_settings(),
        github: create_test_github_settings(),
        log: LogSettings::default(),
        llm: create_test_llm_settings(),
        aliases: {
            let mut aliases = HashMap::new();
            aliases.insert("st".to_string(), "status".to_string());
            aliases.insert("co".to_string(), "checkout".to_string());
            aliases
        },
    };

    assert!(settings.jira.email.is_some());
    assert_eq!(settings.github.accounts.len(), 2);
    assert_eq!(settings.aliases.len(), 2);
    assert_eq!(settings.aliases.get("st"), Some(&"status".to_string()));
}

/// 测试 Settings 默认实现
#[test]
fn test_settings_default() {
    let default_settings = Settings::default();

    assert_eq!(default_settings.jira.email, None);
    assert!(default_settings.github.accounts.is_empty());
    assert_eq!(default_settings.log.get_output_folder_name(), "logs");
    assert_eq!(default_settings.llm.provider, "openai");
    assert!(default_settings.aliases.is_empty());
}

// ==================== 表格显示结构测试 ====================

/// 测试表格行结构创建
#[test]
fn test_table_row_structures() {
    // 测试 LLMConfigRow
    let llm_row = LLMConfigRow {
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        key: "sk-****".to_string(),
        language: "English".to_string(),
    };

    assert_eq!(llm_row.provider, "openai");
    assert_eq!(llm_row.model, "gpt-4");

    // 测试 JiraConfigRow
    let jira_row = JiraConfigRow {
        email: "jira@example.com".to_string(),
        service_address: "https://jira.company.com".to_string(),
        api_token: "****".to_string(),
    };

    assert_eq!(jira_row.email, "jira@example.com");
    assert!(jira_row.service_address.contains("jira.company.com"));

    // 测试 GitHubAccountRow
    let github_row = GitHubAccountRow {
        name: "personal".to_string(),
        email: "github@example.com".to_string(),
        token: "ghp_****".to_string(),
        status: "Active".to_string(),
        verification: "Success".to_string(),
    };

    assert_eq!(github_row.name, "personal");
    assert_eq!(github_row.status, "Active");

    // 测试 GitHubAccountListRow
    let github_list_row = GitHubAccountListRow {
        index: "1".to_string(),
        name: "work".to_string(),
        email: "work@company.com".to_string(),
        token: "ghp_****".to_string(),
        status: "Inactive".to_string(),
    };

    assert_eq!(github_list_row.index, "1");
    assert_eq!(github_list_row.status, "Inactive");
}

/// 测试复杂配置场景
#[test]
fn test_complex_configuration_scenario() {
    // 创建包含所有配置的复杂设置
    let mut aliases = HashMap::new();
    aliases.insert("s".to_string(), "status".to_string());
    aliases.insert("c".to_string(), "commit".to_string());
    aliases.insert("p".to_string(), "push".to_string());

    let complex_settings = Settings {
        jira: JiraSettings {
            email: Some("complex@jira.com".to_string()),
            api_token: Some("complex_jira_token".to_string()),
            service_address: Some("https://complex.atlassian.net".to_string()),
        },
        github: GitHubSettings {
            accounts: vec![
                GitHubAccount {
                    name: "main".to_string(),
                    email: "main@github.com".to_string(),
                    api_token: "ghp_main_token".to_string(),
                },
                GitHubAccount {
                    name: "backup".to_string(),
                    email: "backup@github.com".to_string(),
                    api_token: "ghp_backup_token".to_string(),
                },
                GitHubAccount {
                    name: "test".to_string(),
                    email: "test@github.com".to_string(),
                    api_token: "ghp_test_token".to_string(),
                },
            ],
            current: Some("main".to_string()),
        },
        log: LogSettings {
            output_folder_name: Some("complex_logs".to_string()),
            download_base_dir: Some("/complex/logs/path".to_string()),
            level: Some("info".to_string()),
            enable_trace_console: Some(false),
        },
        llm: LLMSettings {
            provider: "proxy".to_string(),
            language: "Chinese".to_string(),
            openai: LLMProviderSettings {
                url: None,
                key: Some("sk-openai_complex".to_string()),
                model: Some("gpt-4-turbo".to_string()),
            },
            deepseek: LLMProviderSettings {
                url: None,
                key: Some("sk-deepseek_complex".to_string()),
                model: Some("deepseek-coder".to_string()),
            },
            proxy: LLMProviderSettings {
                url: Some("https://complex.proxy.api.com".to_string()),
                key: Some("proxy_complex_key".to_string()),
                model: Some("complex-model".to_string()),
            },
        },
        aliases,
    };

    // 验证复杂配置的各个方面
    assert!(complex_settings.jira.email.is_some());
    assert_eq!(complex_settings.github.accounts.len(), 3);
    assert_eq!(complex_settings.log.level, Some("info".to_string()));
    assert_eq!(complex_settings.llm.provider, "proxy");
    assert_eq!(complex_settings.aliases.len(), 3);

    // 验证 GitHub 当前账号功能
    let current_account = complex_settings.github.get_current_account();
    assert!(current_account.is_some());
    assert_eq!(
        current_account.expect("current account should exist").name,
        "main"
    );

    // 验证 LLM 当前提供商功能
    let current_llm = complex_settings.llm.current_provider();
    assert_eq!(
        current_llm.url,
        Some("https://complex.proxy.api.com".to_string())
    );

    // 验证别名功能
    assert_eq!(
        complex_settings.aliases.get("s"),
        Some(&"status".to_string())
    );
    assert_eq!(
        complex_settings.aliases.get("c"),
        Some(&"commit".to_string())
    );
    assert_eq!(complex_settings.aliases.get("p"), Some(&"push".to_string()));
}
