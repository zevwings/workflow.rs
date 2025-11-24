//! 版本命令
//! 显示 Workflow CLI 的版本信息

use crate::{log_info, log_success};
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::process::Command;

/// 版本命令
pub struct VersionCommand;

impl VersionCommand {
    /// 显示当前版本信息
    ///
    /// 尝试多种方法获取版本号：
    /// 1. 从编译时嵌入的版本号（使用 env! 宏）
    /// 2. 运行当前二进制文件的 --version 命令获取版本
    /// 3. 从 Cargo.toml 读取（开发环境）
    pub fn show() -> Result<()> {
        // 方法 1: 尝试从编译时嵌入的版本号获取（使用 env! 宏）
        // 注意：env! 宏在编译时展开，所以这个值在运行时总是可用的
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        if !VERSION.is_empty() {
            log_success!("workflow v{}", VERSION);
            return Ok(());
        }

        // 方法 2: 尝试运行当前二进制文件的 --version 命令
        // 使用 env::current_exe() 获取当前二进制文件的路径，而不是依赖 PATH
        if let Ok(current_exe) = env::current_exe() {
            if let Ok(output) = Command::new(&current_exe).arg("--version").output() {
                if output.status.success() {
                    let version_str = String::from_utf8_lossy(&output.stdout);
                    let version = version_str.trim();
                    log_success!("{}", version);
                    return Ok(());
                }
            }
        }

        // 方法 2 的备选：如果 current_exe 失败，尝试使用 "workflow" 命令名
        if let Ok(output) = Command::new("workflow").arg("--version").output() {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version = version_str.trim();
                log_success!("{}", version);
                return Ok(());
            }
        }

        // 方法 3: 尝试从 Cargo.toml 读取（开发环境）
        let cargo_toml_path = env::current_dir()
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
                env::current_exe()
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
            let content = fs::read_to_string(&cargo_toml)
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

