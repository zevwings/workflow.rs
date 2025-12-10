//! 系统代理读取器
//!
//! 负责从 macOS 系统设置读取代理配置。

use anyhow::{Context, Result};
use duct::cmd;

use crate::proxy::{ProxyInfo, ProxyType};

/// 系统代理读取器
///
/// 提供从 macOS 系统设置读取代理配置的功能。
pub struct SystemProxyReader;

impl SystemProxyReader {
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
    pub fn read() -> Result<ProxyInfo> {
        // 使用 scutil --proxy 获取系统代理设置
        let output = cmd("scutil", &["--proxy"])
            .read()
            .context("Failed to get system proxy settings")?;

        // 解析输出
        let mut proxy_info = ProxyInfo::new();

        // 定义映射关系：系统键名 -> (代理类型, 字段名)
        const PROXY_KEY_MAP: &[(&str, ProxyType, &str)] = &[
            ("HTTPEnable", ProxyType::Http, "enable"),
            ("HTTPProxy", ProxyType::Http, "address"),
            ("HTTPPort", ProxyType::Http, "port"),
            ("HTTPSEnable", ProxyType::Https, "enable"),
            ("HTTPSProxy", ProxyType::Https, "address"),
            ("HTTPSPort", ProxyType::Https, "port"),
            ("SOCKSEnable", ProxyType::Socks, "enable"),
            ("SOCKSProxy", ProxyType::Socks, "address"),
            ("SOCKSPort", ProxyType::Socks, "port"),
        ];

        // 解析每行（格式：key : value）
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // 分割键值对（格式：key : value）
            // 使用 splitn 限制分割次数，避免值中包含冒号时被错误分割
            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim();
                let value = trimmed[colon_pos + 1..].trim();

                // 查找匹配的键
                for (sys_key, proxy_type, field) in PROXY_KEY_MAP {
                    if key == *sys_key {
                        let config = proxy_info.get_config_mut(*proxy_type);
                        match *field {
                            "enable" => {
                                config.enable = value == "1";
                            }
                            "address" => {
                                config.address = Some(value.to_string());
                            }
                            "port" => {
                                if let Ok(port) = value.parse::<u16>() {
                                    config.port = Some(port);
                                }
                            }
                            _ => {}
                        }
                        break;
                    }
                }
            }
        }

        Ok(proxy_info)
    }
}
