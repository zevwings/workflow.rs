//! 卸载工具函数
//! 提供删除 Workflow CLI 配置的功能

use crate::settings::paths::ConfigPaths;
use anyhow::{Context, Result};
use std::fs;

/// 卸载工具
pub struct Uninstall;

impl Uninstall {
    /// 删除所有 Workflow CLI TOML 配置文件
    /// 这会删除 workflow.toml 和 llm.toml
    pub fn remove_config_files() -> Result<()> {
        // 删除 workflow.toml
        if let Ok(workflow_config_path) = ConfigPaths::workflow_config() {
            if workflow_config_path.exists() {
                fs::remove_file(&workflow_config_path).context("Failed to remove workflow.toml")?;
            }
        }

        // 删除 llm.toml
        if let Ok(llm_config_path) = ConfigPaths::llm_config() {
            if llm_config_path.exists() {
                fs::remove_file(&llm_config_path).context("Failed to remove llm.toml")?;
            }
        }

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
    /// 这会删除所有 TOML 配置文件
    pub fn uninstall_all() -> Result<()> {
        Self::remove_config_files()?;
        Ok(())
    }

    /// 卸载所有 Workflow CLI 配置和二进制文件
    /// 这会删除所有 TOML 配置文件以及二进制文件
    pub fn uninstall_all_with_binaries() -> Result<(Vec<String>, Vec<String>)> {
        // 删除配置
        Self::uninstall_all()?;
        // 删除二进制文件
        Self::remove_binaries()
    }
}
