//! 代理检测与管理工具
//!
//! 本模块提供了 macOS 系统代理的检测和管理功能，包括：
//! - 从系统设置读取代理配置
//! - 检查环境变量中的代理设置
//! - 生成代理命令字符串
//! - 生成环境变量 HashMap
//! - 开启和关闭代理（管理 shell 配置文件中的环境变量）

use crate::utils::env::EnvFile;
use anyhow::{Context, Result};
use duct::cmd;
use std::collections::HashMap;

/// macOS 代理信息
///
/// 包含 HTTP、HTTPS 和 SOCKS 代理的配置信息。
#[derive(Debug, Clone)]
pub struct ProxyInfo {
    /// HTTP 代理是否启用
    pub http_enable: bool,
    /// HTTP 代理地址
    pub http_proxy: Option<String>,
    /// HTTP 代理端口
    pub http_port: Option<u16>,
    /// HTTPS 代理是否启用
    pub https_enable: bool,
    /// HTTPS 代理地址
    pub https_proxy: Option<String>,
    /// HTTPS 代理端口
    pub https_port: Option<u16>,
    /// SOCKS 代理是否启用
    pub socks_enable: bool,
    /// SOCKS 代理地址
    pub socks_proxy: Option<String>,
    /// SOCKS 代理端口
    pub socks_port: Option<u16>,
}

/// 代理检查模块
///
/// 提供代理检测和管理功能。
pub struct Proxy;

impl Proxy {
    /// 从 macOS 系统设置读取代理配置
    ///
    /// 使用 `scutil --proxy` 命令从 macOS 系统设置中读取代理配置。
    ///
    /// # 返回
    ///
    /// 返回 `ProxyInfo` 结构体，包含所有代理配置信息。
    ///
    /// # 错误
    ///
    /// 如果命令执行失败，返回相应的错误信息。
    pub fn get_system_proxy() -> Result<ProxyInfo> {
        // 使用 scutil --proxy 获取系统代理设置
        let output = cmd("scutil", &["--proxy"])
            .read()
            .context("Failed to get system proxy settings")?;

        // 解析输出
        let mut proxy_info = ProxyInfo {
            http_enable: false,
            http_proxy: None,
            http_port: None,
            https_enable: false,
            https_proxy: None,
            https_port: None,
            socks_enable: false,
            socks_proxy: None,
            socks_port: None,
        };

        // 解析每行（格式：key : value）
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // 分割键值对（格式：key : value）
            let parts: Vec<&str> = trimmed.split(':').collect();
            if parts.len() < 2 {
                continue;
            }

            let key = parts[0].trim();
            let value = parts[1..].join(":").trim().to_string();

            match key {
                "HTTPEnable" => {
                    proxy_info.http_enable = value == "1";
                }
                "HTTPProxy" => {
                    proxy_info.http_proxy = Some(value);
                }
                "HTTPPort" => {
                    if let Ok(port) = value.parse::<u16>() {
                        proxy_info.http_port = Some(port);
                    }
                }
                "HTTPSEnable" => {
                    proxy_info.https_enable = value == "1";
                }
                "HTTPSProxy" => {
                    proxy_info.https_proxy = Some(value);
                }
                "HTTPSPort" => {
                    if let Ok(port) = value.parse::<u16>() {
                        proxy_info.https_port = Some(port);
                    }
                }
                "SOCKSEnable" => {
                    proxy_info.socks_enable = value == "1";
                }
                "SOCKSProxy" => {
                    proxy_info.socks_proxy = Some(value);
                }
                "SOCKSPort" => {
                    if let Ok(port) = value.parse::<u16>() {
                        proxy_info.socks_port = Some(port);
                    }
                }
                _ => {}
            }
        }

        Ok(proxy_info)
    }

    /// 生成代理命令字符串
    ///
    /// 根据代理配置生成 `export` 命令字符串，用于设置环境变量。
    ///
    /// # 参数
    ///
    /// * `proxy_info` - 代理信息结构体
    ///
    /// # 返回
    ///
    /// 返回 `export` 命令字符串（如 `export http_proxy=... https_proxy=...`）。
    /// 如果没有启用的代理，返回 `None`。
    pub fn generate_proxy_command(proxy_info: &ProxyInfo) -> Option<String> {
        let mut parts = Vec::new();

        if proxy_info.http_enable {
            if let (Some(addr), Some(port)) = (&proxy_info.http_proxy, &proxy_info.http_port) {
                parts.push(format!("http_proxy=http://{}:{}", addr, port));
            }
        }

        if proxy_info.https_enable {
            if let (Some(addr), Some(port)) = (&proxy_info.https_proxy, &proxy_info.https_port) {
                parts.push(format!("https_proxy=http://{}:{}", addr, port));
            }
        }

        if proxy_info.socks_enable {
            if let (Some(addr), Some(port)) = (&proxy_info.socks_proxy, &proxy_info.socks_port) {
                parts.push(format!("all_proxy=socks5://{}:{}", addr, port));
            }
        }

        if parts.is_empty() {
            None
        } else {
            Some(format!("export {}", parts.join(" ")))
        }
    }

    /// 检查环境变量中的代理设置
    ///
    /// 从当前环境变量中读取代理配置（`http_proxy`、`https_proxy`、`all_proxy`）。
    ///
    /// # 返回
    ///
    /// 返回包含代理环境变量的 HashMap。
    pub fn check_env_proxy() -> HashMap<String, String> {
        let mut env_proxy = HashMap::new();

        if let Ok(http_proxy) = std::env::var("http_proxy") {
            env_proxy.insert("http_proxy".to_string(), http_proxy);
        }

        if let Ok(https_proxy) = std::env::var("https_proxy") {
            env_proxy.insert("https_proxy".to_string(), https_proxy);
        }

        if let Ok(all_proxy) = std::env::var("all_proxy") {
            env_proxy.insert("all_proxy".to_string(), all_proxy);
        }

        env_proxy
    }

    /// 检查代理设置是否匹配
    ///
    /// 检查环境变量中的代理设置是否与系统代理配置匹配。
    ///
    /// # 参数
    ///
    /// * `proxy_info` - 代理信息结构体
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果环境变量中的代理设置与系统配置匹配，否则返回 `false`。
    pub fn is_proxy_configured(proxy_info: &ProxyInfo) -> bool {
        let env_proxy = Self::check_env_proxy();

        if proxy_info.http_enable {
            if let (Some(addr), Some(port)) = (&proxy_info.http_proxy, &proxy_info.http_port) {
                let expected = format!("http://{}:{}", addr, port);
                if env_proxy.get("http_proxy").map(|v| v.as_str()) != Some(&expected) {
                    return false;
                }
            }
        }

        if proxy_info.https_enable {
            if let (Some(addr), Some(port)) = (&proxy_info.https_proxy, &proxy_info.https_port) {
                let expected = format!("http://{}:{}", addr, port);
                if env_proxy.get("https_proxy").map(|v| v.as_str()) != Some(&expected) {
                    return false;
                }
            }
        }

        if proxy_info.socks_enable {
            if let (Some(addr), Some(port)) = (&proxy_info.socks_proxy, &proxy_info.socks_port) {
                let expected = format!("socks5://{}:{}", addr, port);
                if env_proxy.get("all_proxy").map(|v| v.as_str()) != Some(&expected) {
                    return false;
                }
            }
        }

        true
    }

    /// 生成环境变量 HashMap（用于写入到 .zshrc）
    ///
    /// 根据代理配置生成环境变量 HashMap，用于保存到 shell 配置文件。
    ///
    /// # 参数
    ///
    /// * `proxy_info` - 代理信息结构体
    ///
    /// # 返回
    ///
    /// 返回包含代理环境变量的 HashMap（键：`http_proxy`、`https_proxy`、`all_proxy`）。
    pub fn generate_env_vars(proxy_info: &ProxyInfo) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();

        if proxy_info.http_enable {
            if let (Some(addr), Some(port)) = (&proxy_info.http_proxy, &proxy_info.http_port) {
                env_vars.insert(
                    "http_proxy".to_string(),
                    format!("http://{}:{}", addr, port),
                );
            }
        }

        if proxy_info.https_enable {
            if let (Some(addr), Some(port)) = (&proxy_info.https_proxy, &proxy_info.https_port) {
                env_vars.insert(
                    "https_proxy".to_string(),
                    format!("http://{}:{}", addr, port),
                );
            }
        }

        if proxy_info.socks_enable {
            if let (Some(addr), Some(port)) = (&proxy_info.socks_proxy, &proxy_info.socks_port) {
                env_vars.insert(
                    "all_proxy".to_string(),
                    format!("socks5://{}:{}", addr, port),
                );
            }
        }

        env_vars
    }

    /// 开启代理（设置环境变量到 shell 配置文件）
    ///
    /// 从系统设置读取代理配置，并保存到 shell 配置文件中。
    ///
    /// # 返回
    ///
    /// 返回 `ProxyEnableResult`，包含代理命令字符串和 shell 配置文件路径。
    ///
    /// # 错误
    ///
    /// 如果读取系统代理设置失败或保存配置失败，返回相应的错误信息。
    pub fn enable_proxy() -> Result<ProxyEnableResult> {
        // 1. 获取系统代理设置
        let proxy_info = Self::get_system_proxy()?;

        // 2. 检查代理是否已配置
        if Self::is_proxy_configured(&proxy_info) {
            return Ok(ProxyEnableResult {
                already_configured: true,
                proxy_command: None,
                shell_config_path: None,
            });
        }

        // 3. 生成代理命令
        let proxy_cmd = Self::generate_proxy_command(&proxy_info);

        // 4. 生成环境变量 HashMap 并写入到 shell 配置文件
        let env_vars = Self::generate_env_vars(&proxy_info);
        let shell_config_path = if !env_vars.is_empty() {
            crate::utils::env::EnvFile::set_multiple(&env_vars)
                .context("Failed to save proxy settings to shell config")?;

            Some(
                crate::utils::env::EnvFile::get_shell_config_path()
                    .context("Failed to get shell config path")?,
            )
        } else {
            None
        };

        Ok(ProxyEnableResult {
            already_configured: false,
            proxy_command: proxy_cmd,
            shell_config_path,
        })
    }

    /// 关闭代理（从 shell 配置文件移除环境变量）
    ///
    /// 从 shell 配置文件中移除所有代理相关的环境变量。
    ///
    /// # 返回
    ///
    /// 返回 `ProxyDisableResult`，包含是否找到代理设置、shell 配置文件路径和 unset 命令。
    ///
    /// # 错误
    ///
    /// 如果读取或写入配置文件失败，返回相应的错误信息。
    pub fn disable_proxy() -> Result<ProxyDisableResult> {
        // 检查当前代理环境变量（包括环境变量和配置文件）
        let env_proxy = Self::check_env_proxy();
        let shell_config_env = EnvFile::load().unwrap_or_default();

        // 检查配置文件中的代理设置
        let has_http_proxy = shell_config_env.contains_key("http_proxy");
        let has_https_proxy = shell_config_env.contains_key("https_proxy");
        let has_all_proxy = shell_config_env.contains_key("all_proxy");
        let has_env_http_proxy = env_proxy.contains_key("http_proxy");
        let has_env_https_proxy = env_proxy.contains_key("https_proxy");
        let has_env_all_proxy = env_proxy.contains_key("all_proxy");

        if !has_http_proxy
            && !has_https_proxy
            && !has_all_proxy
            && !has_env_http_proxy
            && !has_env_https_proxy
            && !has_env_all_proxy
        {
            return Ok(ProxyDisableResult {
                found_proxy: false,
                shell_config_path: None,
                unset_command: None,
                current_env_proxy: env_proxy,
            });
        }

        // 从 shell 配置文件中移除代理配置（包括配置块内外）
        // 1. 从配置块内移除（通过 EnvFile API）
        let mut env_vars = shell_config_env;
        let mut removed_from_block = false;

        if env_vars.remove("http_proxy").is_some() {
            removed_from_block = true;
        }
        if env_vars.remove("https_proxy").is_some() {
            removed_from_block = true;
        }
        if env_vars.remove("all_proxy").is_some() {
            removed_from_block = true;
        }

        if removed_from_block {
            // 保存更新后的配置（移除了代理配置）
            EnvFile::save(&env_vars)
                .context("Failed to remove proxy settings from shell config")?;
        }

        // 2. 从整个文件中移除所有代理相关的 export 语句（包括配置块外）
        let proxy_keys = ["http_proxy", "https_proxy", "all_proxy"];
        EnvFile::remove_from_file(&proxy_keys)
            .context("Failed to remove proxy settings from shell config")?;

        let shell_config_path =
            EnvFile::get_shell_config_path().context("Failed to get shell config path")?;

        // 生成 unset 命令（用于当前 shell 会话）
        let mut unset_cmds = Vec::new();
        if has_env_http_proxy {
            unset_cmds.push("unset http_proxy".to_string());
        }
        if has_env_https_proxy {
            unset_cmds.push("unset https_proxy".to_string());
        }
        if has_env_all_proxy {
            unset_cmds.push("unset all_proxy".to_string());
        }

        let unset_command = if !unset_cmds.is_empty() {
            Some(unset_cmds.join(" && "))
        } else {
            None
        };

        Ok(ProxyDisableResult {
            found_proxy: true,
            shell_config_path: Some(shell_config_path),
            unset_command,
            current_env_proxy: env_proxy,
        })
    }
}

/// 开启代理的结果
#[derive(Debug, Clone)]
pub struct ProxyEnableResult {
    /// 代理是否已经配置
    pub already_configured: bool,
    /// 代理命令字符串（用于当前 shell 会话）
    pub proxy_command: Option<String>,
    /// Shell 配置文件路径
    pub shell_config_path: Option<std::path::PathBuf>,
}

/// 关闭代理的结果
#[derive(Debug, Clone)]
pub struct ProxyDisableResult {
    /// 是否找到代理设置
    pub found_proxy: bool,
    /// Shell 配置文件路径
    pub shell_config_path: Option<std::path::PathBuf>,
    /// Unset 命令（用于当前 shell 会话）
    pub unset_command: Option<String>,
    /// 当前环境变量中的代理设置
    pub current_env_proxy: HashMap<String, String>,
}
