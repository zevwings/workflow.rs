//! 配置查看命令
//! 显示当前的环境变量配置

use crate::{log_break, log_info, log_warning, mask_sensitive_value, EnvFile};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// 配置查看命令
pub struct ConfigCommand;

impl ConfigCommand {
    /// 显示当前配置（从环境变量读取）
    pub fn show() -> Result<()> {
        log_break!('=', 40, "Current Configuration");
        log_break!();

        // 显示配置文件路径
        let shell_config_path = EnvFile::get_shell_config_path()
            .map_err(|_| anyhow::anyhow!("Failed to get shell config path"))?;
        log_info!("Shell config: {:?}\n", shell_config_path);

        // 从多个来源加载环境变量：当前环境变量 > shell 配置文件
        let env_var_keys = EnvFile::get_workflow_env_keys();
        let env_vars = EnvFile::load_merged(&env_var_keys);

        // 检查是否有配置
        if env_vars.is_empty() {
            log_warning!("  No configuration found!");
            log_info!("   Run 'workflow setup' to initialize configuration.");
            return Ok(());
        }

        // 显示所有配置（统一显示为环境变量）
        log_break!();
        log_break!('-', 100, "Environment Variables");
        Self::print_all_config(&env_vars)?;

        Ok(())
    }

    /// 打印所有配置（统一显示为环境变量）
    fn print_all_config(env_vars: &HashMap<String, String>) -> Result<()> {
        // 定义敏感键（需要隐藏）
        let sensitive_keys: HashSet<&str> = [
            "JIRA_API_TOKEN",
            "LLM_OPENAI_KEY",
            "LLM_PROXY_KEY",
            "LLM_DEEPSEEK_KEY",
            "CODEUP_CSRF_TOKEN",
            "CODEUP_COOKIE",
        ]
        .iter()
        .copied()
        .collect();

        // 定义需要显示的键（按逻辑分组和顺序）
        let display_order = vec![
            // 用户配置
            "EMAIL",
            // Jira 配置
            "JIRA_SERVICE_ADDRESS",
            "JIRA_API_TOKEN",
            // GitHub 配置
            "GITHUB_BRANCH_PREFIX",
            // 日志配置
            "LOG_OUTPUT_FOLDER_NAME",
            "LOG_DELETE_WHEN_OPERATION_COMPLETED",
            // 代理配置
            "DISABLE_CHECK_PROXY",
            // LLM 配置
            "LLM_PROVIDER",
            "LLM_OPENAI_KEY",
            "LLM_DEEPSEEK_KEY",
            "LLM_PROXY_URL",
            "LLM_PROXY_KEY",
            // Codeup 配置
            "CODEUP_PROJECT_ID",
            "CODEUP_CSRF_TOKEN",
            "CODEUP_COOKIE",
        ];

        // 按顺序显示
        for key in &display_order {
            if let Some(value) = env_vars.get(*key) {
                let display_value = if sensitive_keys.contains(key) {
                    mask_sensitive_value(value)
                } else {
                    // 布尔值转换为可读格式
                    match key {
                        &"LOG_DELETE_WHEN_OPERATION_COMPLETED" | &"DISABLE_CHECK_PROXY" => {
                            if value == "1" {
                                "Yes".to_string()
                            } else {
                                "No".to_string()
                            }
                        }
                        _ => value.clone(),
                    }
                };
                log_info!("  {}: {}", key, display_value);
            }
        }

        // 显示其他未列出的键（如果有）
        let display_order_set: HashSet<&str> = display_order.iter().copied().collect();
        let mut other_keys: Vec<&String> = env_vars
            .keys()
            .filter(|k| !display_order_set.contains(k.as_str()))
            .collect();
        if !other_keys.is_empty() {
            other_keys.sort();
            log_break!();
            log_info!("  Other variables:");
            for key in &other_keys {
                let value = env_vars.get(*key).unwrap();
                let display_value = if sensitive_keys.contains(key.as_str()) {
                    mask_sensitive_value(value)
                } else {
                    value.clone()
                };
                log_info!("    {}: {}", key, display_value);
            }
        }

        Ok(())
    }
}
