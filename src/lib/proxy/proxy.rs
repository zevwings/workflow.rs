//! 代理检测与管理工具
//!
//! 本模块提供了 macOS 系统代理的检测和管理功能，包括：
//! - 从系统设置读取代理配置
//! - 检查环境变量中的代理设置
//! - 生成代理命令字符串
//! - 生成环境变量 HashMap
//! - 开启和关闭代理（管理 shell 配置文件中的环境变量）

use std::collections::HashMap;

/// 代理类型枚举
///
/// 表示不同的代理类型：HTTP、HTTPS 和 SOCKS。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProxyType {
    /// HTTP 代理
    Http,
    /// HTTPS 代理
    Https,
    /// SOCKS 代理
    Socks,
}

impl ProxyType {
    /// 返回所有代理类型的迭代器
    pub fn all() -> impl Iterator<Item = Self> {
        [Self::Http, Self::Https, Self::Socks].iter().copied()
    }

    /// 返回对应的环境变量键名
    pub fn env_key(&self) -> &'static str {
        match self {
            Self::Http => "http_proxy",
            Self::Https => "https_proxy",
            Self::Socks => "all_proxy",
        }
    }

    /// 返回对应的 URL 协议方案
    pub fn url_scheme(&self) -> &'static str {
        match self {
            Self::Http | Self::Https => "http",
            Self::Socks => "socks5",
        }
    }

    /// 返回所有代理类型的环境变量键名
    ///
    /// 用于批量操作，如批量移除环境变量。
    pub fn all_env_keys() -> Vec<&'static str> {
        Self::all().map(|pt| pt.env_key()).collect()
    }
}

/// 单个代理类型的配置信息
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// 代理是否启用
    pub enable: bool,
    /// 代理地址
    pub address: Option<String>,
    /// 代理端口
    pub port: Option<u16>,
}

/// macOS 代理信息
///
/// 包含 HTTP、HTTPS 和 SOCKS 代理的配置信息。
#[derive(Debug, Clone, Default)]
pub struct ProxyInfo {
    proxies: HashMap<ProxyType, ProxyConfig>,
}

impl ProxyInfo {
    /// 创建新的 ProxyInfo 实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取指定代理类型的配置
    pub fn get_config(&self, proxy_type: ProxyType) -> Option<&ProxyConfig> {
        self.proxies.get(&proxy_type)
    }

    /// 获取指定代理类型的配置（可变引用）
    pub fn get_config_mut(&mut self, proxy_type: ProxyType) -> &mut ProxyConfig {
        self.proxies
            .entry(proxy_type)
            .or_insert_with(|| ProxyConfig {
                enable: false,
                address: None,
                port: None,
            })
    }

    /// 设置指定代理类型的配置
    pub fn set_config(&mut self, proxy_type: ProxyType, config: ProxyConfig) {
        self.proxies.insert(proxy_type, config);
    }

    /// 根据代理类型获取代理 URL
    ///
    /// # 参数
    ///
    /// * `proxy_type` - 代理类型
    ///
    /// # 返回
    ///
    /// 如果该类型的代理已启用且配置完整，返回代理 URL（如 `http://proxy:8080`），否则返回 `None`。
    pub fn get_proxy_url(&self, proxy_type: ProxyType) -> Option<String> {
        self.get_config(proxy_type)
            .filter(|config| config.enable)
            .and_then(|config| {
                config
                    .address
                    .as_ref()
                    .zip(config.port.as_ref())
                    .map(|(addr, port)| format!("{}://{}:{}", proxy_type.url_scheme(), addr, port))
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
