//! Proxy 代理类型和配置测试
//!
//! 测试 Proxy 相关的类型和配置功能，包括：
//! - 代理类型枚举
//! - 代理配置
//! - 代理信息

use pretty_assertions::assert_eq;
use workflow::proxy::{ProxyConfig, ProxyInfo, ProxyType};

// ==================== 代理类型测试 ====================

#[test]
fn test_proxy_type_all() {
    // 测试获取所有代理类型
    let all_types: Vec<ProxyType> = ProxyType::all().collect();
    assert_eq!(all_types.len(), 3);
    assert!(all_types.contains(&ProxyType::Http));
    assert!(all_types.contains(&ProxyType::Https));
    assert!(all_types.contains(&ProxyType::Socks));
}

#[test]
fn test_proxy_type_env_key() {
    // 测试获取环境变量键名
    assert_eq!(ProxyType::Http.env_key(), "http_proxy");
    assert_eq!(ProxyType::Https.env_key(), "https_proxy");
    assert_eq!(ProxyType::Socks.env_key(), "all_proxy");
}

#[test]
fn test_proxy_type_url_scheme() {
    // 测试获取 URL 协议方案
    assert_eq!(ProxyType::Http.url_scheme(), "http");
    assert_eq!(ProxyType::Https.url_scheme(), "http");
    assert_eq!(ProxyType::Socks.url_scheme(), "socks5");
}

#[test]
fn test_proxy_type_all_env_keys() {
    // 测试获取所有环境变量键名
    let keys = ProxyType::all_env_keys();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"http_proxy"));
    assert!(keys.contains(&"https_proxy"));
    assert!(keys.contains(&"all_proxy"));
}

// ==================== 代理配置测试 ====================

#[test]
fn test_proxy_config() {
    // 测试代理配置结构
    let config = ProxyConfig {
        enable: true,
        address: Some("proxy.example.com".to_string()),
        port: Some(8080),
    };

    assert!(config.enable);
    assert_eq!(config.address, Some("proxy.example.com".to_string()));
    assert_eq!(config.port, Some(8080));
}

#[test]
fn test_proxy_config_disabled() {
    // 测试禁用的代理配置
    let config = ProxyConfig {
        enable: false,
        address: Some("proxy.example.com".to_string()),
        port: Some(8080),
    };

    assert!(!config.enable);
}

#[test]
fn test_proxy_config_partial() {
    // 测试部分配置（只有地址或只有端口）
    let config = ProxyConfig {
        enable: true,
        address: Some("proxy.example.com".to_string()),
        port: None,
    };

    assert!(config.enable);
    assert!(config.address.is_some());
    assert!(config.port.is_none());
}

// ==================== 代理信息测试 ====================

#[test]
fn test_proxy_info_new() {
    // 测试创建新的代理信息
    let proxy_info = ProxyInfo::new();
    assert!(proxy_info.get_config(ProxyType::Http).is_none());
    assert!(proxy_info.get_config(ProxyType::Https).is_none());
    assert!(proxy_info.get_config(ProxyType::Socks).is_none());
}

#[test]
fn test_proxy_info_default() {
    // 测试默认代理信息
    let proxy_info = ProxyInfo::default();
    assert!(proxy_info.get_config(ProxyType::Http).is_none());
}

#[test]
fn test_proxy_info_set_and_get_config() {
    // 测试设置和获取配置
    let mut proxy_info = ProxyInfo::new();
    let config = ProxyConfig {
        enable: true,
        address: Some("proxy.example.com".to_string()),
        port: Some(8080),
    };

    proxy_info.set_config(ProxyType::Http, config.clone());

    let retrieved = proxy_info.get_config(ProxyType::Http);
    assert!(retrieved.is_some());
    let retrieved_config = retrieved.unwrap();
    assert_eq!(retrieved_config.enable, config.enable);
    assert_eq!(retrieved_config.address, config.address);
    assert_eq!(retrieved_config.port, config.port);
}

#[test]
fn test_proxy_info_get_config_mut() {
    // 测试获取可变配置
    let mut proxy_info = ProxyInfo::new();
    let config = proxy_info.get_config_mut(ProxyType::Http);

    config.enable = true;
    config.address = Some("proxy.example.com".to_string());
    config.port = Some(8080);

    let retrieved = proxy_info.get_config(ProxyType::Http);
    assert!(retrieved.is_some());
    assert!(retrieved.unwrap().enable);
}

#[test]
fn test_proxy_info_get_proxy_url_enabled() {
    // 测试获取启用的代理 URL
    let mut proxy_info = ProxyInfo::new();
    let config = ProxyConfig {
        enable: true,
        address: Some("proxy.example.com".to_string()),
        port: Some(8080),
    };

    proxy_info.set_config(ProxyType::Http, config);

    let url = proxy_info.get_proxy_url(ProxyType::Http);
    assert!(url.is_some());
    assert_eq!(url.unwrap(), "http://proxy.example.com:8080");
}

#[test]
fn test_proxy_info_get_proxy_url_disabled() {
    // 测试获取禁用的代理 URL（应该返回 None）
    let mut proxy_info = ProxyInfo::new();
    let config = ProxyConfig {
        enable: false,
        address: Some("proxy.example.com".to_string()),
        port: Some(8080),
    };

    proxy_info.set_config(ProxyType::Http, config);

    let url = proxy_info.get_proxy_url(ProxyType::Http);
    assert!(url.is_none());
}

#[test]
fn test_proxy_info_get_proxy_url_incomplete() {
    // 测试获取不完整配置的代理 URL（应该返回 None）
    let mut proxy_info = ProxyInfo::new();
    let config = ProxyConfig {
        enable: true,
        address: Some("proxy.example.com".to_string()),
        port: None, // 缺少端口
    };

    proxy_info.set_config(ProxyType::Http, config);

    let url = proxy_info.get_proxy_url(ProxyType::Http);
    assert!(url.is_none());
}

#[test]
fn test_proxy_info_get_proxy_url_https() {
    // 测试 HTTPS 代理 URL
    let mut proxy_info = ProxyInfo::new();
    let config = ProxyConfig {
        enable: true,
        address: Some("proxy.example.com".to_string()),
        port: Some(443),
    };

    proxy_info.set_config(ProxyType::Https, config);

    let url = proxy_info.get_proxy_url(ProxyType::Https);
    assert!(url.is_some());
    assert_eq!(url.unwrap(), "http://proxy.example.com:443");
}

#[test]
fn test_proxy_info_get_proxy_url_socks() {
    // 测试 SOCKS 代理 URL
    let mut proxy_info = ProxyInfo::new();
    let config = ProxyConfig {
        enable: true,
        address: Some("socks.example.com".to_string()),
        port: Some(1080),
    };

    proxy_info.set_config(ProxyType::Socks, config);

    let url = proxy_info.get_proxy_url(ProxyType::Socks);
    assert!(url.is_some());
    assert_eq!(url.unwrap(), "socks5://socks.example.com:1080");
}

#[test]
fn test_proxy_info_multiple_proxies() {
    // 测试多个代理配置
    let mut proxy_info = ProxyInfo::new();

    proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("http-proxy.example.com".to_string()),
            port: Some(8080),
        },
    );

    proxy_info.set_config(
        ProxyType::Https,
        ProxyConfig {
            enable: true,
            address: Some("https-proxy.example.com".to_string()),
            port: Some(8443),
        },
    );

    let http_url = proxy_info.get_proxy_url(ProxyType::Http);
    let https_url = proxy_info.get_proxy_url(ProxyType::Https);

    assert_eq!(http_url.unwrap(), "http://http-proxy.example.com:8080");
    assert_eq!(https_url.unwrap(), "http://https-proxy.example.com:8443");
}
