//! 卸载工具函数
//! 提供删除 Workflow CLI 配置的功能

use super::EnvFile;
use anyhow::{Context, Result};
use std::fs;

/// 卸载工具
pub struct Uninstall;

impl Uninstall {
    /// 获取所有 Workflow CLI 相关的环境变量键
    pub fn get_all_env_keys() -> Vec<&'static str> {
        vec![
            "EMAIL",
            "JIRA_API_TOKEN",
            "JIRA_SERVICE_ADDRESS",
            "GH_BRANCH_PREFIX",
            "LOG_OUTPUT_FOLDER_NAME",
            "LOG_DELETE_WHEN_OPERATION_COMPLETED",
            "DISABLE_CHECK_PROXY",
            "LLM_PROVIDER",
            "LLM_OPENAI_KEY",
            "LLM_DEEPSEEK_KEY",
            "LLM_PROXY_URL",
            "LLM_PROXY_KEY",
            "CODEUP_CSRF_TOKEN",
            "CODEUP_COOKIE",
            "CODEUP_PROJECT_ID",
        ]
    }

    /// 删除所有 Workflow CLI 配置块
    /// 这会从 shell 配置文件中删除整个配置块（包括标记行）
    pub fn remove_config_block() -> Result<()> {
        let shell_config_path =
            EnvFile::get_shell_config_path().context("Failed to get shell config path")?;

        if !shell_config_path.exists() {
            return Ok(());
        }

        let content =
            fs::read_to_string(&shell_config_path).context("Failed to read shell config file")?;

        let marker_start = "# Workflow CLI Configuration - Start";
        let marker_end = "# Workflow CLI Configuration - End";

        // 查找配置块的开始和结束位置
        if let Some(start_pos) = content.find(marker_start) {
            if let Some(end_pos) = content[start_pos..].find(marker_end) {
                let end_pos = start_pos + end_pos + marker_end.len();

                // 提取配置块前后的内容
                let before = content[..start_pos].trim_end();
                let after = content[end_pos..].trim_start();

                // 重新组合内容（移除配置块）
                let new_content = if before.is_empty() && after.is_empty() {
                    // 如果配置块是文件的全部内容，保留空文件
                    String::new()
                } else if before.is_empty() {
                    // 如果配置块在文件开头
                    after.to_string()
                } else if after.is_empty() {
                    // 如果配置块在文件末尾
                    format!("{}\n", before)
                } else {
                    // 配置块在中间
                    format!("{}\n{}", before, after)
                };

                // 确保文件以换行符结尾（如果内容不为空）
                let final_content = if !new_content.is_empty() && !new_content.ends_with('\n') {
                    format!("{}\n", new_content)
                } else {
                    new_content
                };

                // 写入文件
                fs::write(&shell_config_path, final_content)
                    .context("Failed to write to shell config file")?;

                return Ok(());
            }
        }

        // 如果没有找到配置块，尝试删除所有相关的环境变量行
        Self::remove_all_env_vars()?;

        Ok(())
    }

    /// 删除所有 Workflow CLI 相关的环境变量
    /// 这会从配置文件中删除所有相关的 export 行
    pub fn remove_all_env_vars() -> Result<()> {
        let env_keys = Self::get_all_env_keys();
        EnvFile::remove_from_file(&env_keys)
            .context("Failed to remove environment variables from file")?;
        Ok(())
    }

    /// 获取所有 Workflow CLI 二进制文件路径
    pub fn get_binary_paths() -> Vec<&'static str> {
        vec![
            "/usr/local/bin/workflow",
            "/usr/local/bin/pr",
            "/usr/local/bin/qk",
        ]
    }

    /// 删除所有 Workflow CLI 二进制文件
    /// 这会删除 /usr/local/bin 目录下的二进制文件
    /// 返回成功删除的文件列表和需要 sudo 权限的文件列表
    pub fn remove_binaries() -> Result<(Vec<String>, Vec<String>)> {
        let binary_paths = Self::get_binary_paths();
        let mut removed = Vec::new();
        let mut need_sudo = Vec::new();

        for binary_path in binary_paths {
            let path = std::path::Path::new(binary_path);
            if path.exists() {
                match fs::remove_file(path) {
                    Ok(_) => {
                        removed.push(binary_path.to_string());
                    }
                    Err(e) => {
                        // 检查是否是权限错误
                        if e.kind() == std::io::ErrorKind::PermissionDenied {
                            need_sudo.push(binary_path.to_string());
                        } else {
                            return Err(anyhow::anyhow!(
                                "Failed to remove binary file: {}: {}",
                                binary_path,
                                e
                            ));
                        }
                    }
                }
            }
        }

        Ok((removed, need_sudo))
    }

    /// 卸载所有 Workflow CLI 配置
    /// 这会删除配置块和所有相关的环境变量
    pub fn uninstall_all() -> Result<()> {
        // 首先尝试删除配置块（更彻底）
        Self::remove_config_block()?;
        // 然后删除所有相关的环境变量（作为备用）
        Self::remove_all_env_vars()?;
        Ok(())
    }

    /// 卸载所有 Workflow CLI 配置和二进制文件
    /// 这会删除配置块、所有相关的环境变量以及二进制文件
    pub fn uninstall_all_with_binaries() -> Result<(Vec<String>, Vec<String>)> {
        // 删除配置
        Self::uninstall_all()?;
        // 删除二进制文件
        Self::remove_binaries()
    }
}
