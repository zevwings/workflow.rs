use anyhow::{Context, Result};
use duct::cmd;
use std::collections::HashMap;

/// macOS 代理信息
#[derive(Debug, Clone)]
pub struct ProxyInfo {
    pub http_enable: bool,
    pub http_proxy: Option<String>,
    pub http_port: Option<u16>,
    pub https_enable: bool,
    pub https_proxy: Option<String>,
    pub https_port: Option<u16>,
    pub socks_enable: bool,
    pub socks_proxy: Option<String>,
    pub socks_port: Option<u16>,
}

/// 代理检查模块
pub struct Proxy;

impl Proxy {
    /// 从 macOS 系统设置读取代理配置
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
}
