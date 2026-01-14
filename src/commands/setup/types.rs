//! 配置收集相关的类型定义

use crate::config::settings::settings::GitHubAccount;

/// 收集的配置数据
#[derive(Debug, Clone)]
pub struct CollectedConfig {
    // Workflow 配置
    pub jira_email: Option<String>,
    pub jira_api_token: Option<String>,
    pub jira_service_address: Option<String>,
    pub github_accounts: Vec<GitHubAccount>,
    pub github_current: Option<String>,
    pub log_output_folder_name: Option<String>,
    pub log_download_base_dir: Option<String>,
    pub enable_trace_console: Option<bool>,
    // LLM 配置
    pub llm_provider: String,
    pub llm_language: String, // LLM 输出语言（所有 provider 共享）
    // 各 provider 的配置
    pub llm_openai_key: Option<String>,
    pub llm_openai_model: Option<String>,
    pub llm_deepseek_key: Option<String>,
    pub llm_deepseek_model: Option<String>,
    pub llm_proxy_url: Option<String>,
    pub llm_proxy_key: Option<String>,
    pub llm_proxy_model: Option<String>,
}

/// Jira 配置结果
#[derive(Debug, Clone)]
pub struct JiraConfig {
    pub email: Option<String>,
    pub api_token: Option<String>,
    pub service_address: Option<String>,
}

/// LLM 配置结果
#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub provider: String,
    pub language: String,
    pub openai_key: Option<String>,
    pub openai_model: Option<String>,
    pub deepseek_key: Option<String>,
    pub deepseek_model: Option<String>,
    pub proxy_url: Option<String>,
    pub proxy_key: Option<String>,
    pub proxy_model: Option<String>,
}

/// Log 配置结果
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub output_folder_name: Option<String>,
    pub download_base_dir: Option<String>,
    pub enable_trace_console: Option<bool>,
}
