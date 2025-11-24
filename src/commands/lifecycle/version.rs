//! 版本命令
//! 显示 Workflow CLI 的版本信息

use crate::{log_info, log_success};
use anyhow::{Context, Result};
use std::process::Command;

/// 版本命令
pub struct VersionCommand;

impl VersionCommand {
    /// 显示当前版本信息
    ///
    /// 尝试多种方法获取版本号：
    /// 1. 从环境变量 CARGO_PKG_VERSION（编译时注入）
    /// 2. 运行 `workflow --version` 命令获取版本
    /// 3. 从 Cargo.toml 读取（开发环境）
    pub fn show() -> Result<()> {
        // 方法 1: 尝试从环境变量获取（编译时注入）
        if let Ok(version) = std::env::var("CARGO_PKG_VERSION") {
            log_success!("workflow v{}", version);
            return Ok(());
        }

        // 方法 2: 尝试运行 workflow --version 命令
        if let Ok(output) = Command::new("workflow").arg("--version").output() {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version = version_str.trim();
                log_success!("{}", version);
                return Ok(());
            }
        }

        // 方法 3: 尝试从 Cargo.toml 读取（开发环境）
        let cargo_toml_path = std::env::current_dir()
            .ok()
            .and_then(|dir| {
                // 尝试多个可能的路径
                let paths = [
                    dir.join("Cargo.toml"),
                    dir.join("../Cargo.toml"),
                    dir.join("../../Cargo.toml"),
                ];
                paths.iter().find(|p| p.exists()).cloned()
            })
            .or_else(|| {
                // 如果当前目录找不到，尝试从可执行文件位置推断
                std::env::current_exe()
                    .ok()
                    .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                    .and_then(|mut path| {
                        // 向上查找 Cargo.toml
                        for _ in 0..5 {
                            let cargo_toml = path.join("Cargo.toml");
                            if cargo_toml.exists() {
                                return Some(cargo_toml);
                            }
                            path = path.parent()?.to_path_buf();
                        }
                        None
                    })
            });

        if let Some(cargo_toml) = cargo_toml_path {
            let content = std::fs::read_to_string(&cargo_toml)
                .with_context(|| format!("Failed to read Cargo.toml: {}", cargo_toml.display()))?;

            for line in content.lines() {
                if line.trim().starts_with("version =") {
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line[start + 1..].find('"') {
                            let version = &line[start + 1..start + 1 + end];
                            log_success!("workflow v{}", version);
                            return Ok(());
                        }
                    }
                }
            }
        }

        // 如果都找不到，显示错误
        log_info!("Unable to determine version");
        Ok(())
    }
}

