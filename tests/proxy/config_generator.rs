//! Proxy 配置生成器测试
//!
//! 测试 Proxy 配置生成器相关的功能，包括：
//! - 生成代理命令
//! - 生成环境变量

use pretty_assertions::assert_eq;
use std::collections::HashMap;
use workflow::proxy::{ProxyConfig, ProxyConfigGenerator, ProxyInfo, ProxyType};

// ==================== 生成代理命令测试 ====================

#[test]
fn test_generate_command_single_proxy() {
    // 测试生成单个代理的命令
    let mut proxy_info = ProxyInfo::new();
    proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("proxy.example.com".to_string()),
            port: Some(8080),
        },
    );

    let command = ProxyConfigGenerator::generate_command(&proxy_info);
    assert!(command.is_some());
    let cmd = command.unwrap();
    assert!(cmd.starts_with("export "));
    assert!(cmd.contains("http_proxy="));
    assert!(cmd.contains("http://proxy.example.com:8080"));
}

#[test]
fn test_generate_command_multiple_proxies() {
    // 测试生成多个代理的命令
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

    let command = ProxyConfigGenerator::generate_command(&proxy_info);
    assert!(command.is_some());
    let cmd = command.unwrap();
    assert!(cmd.contains("http_proxy="));
    assert!(cmd.contains("https_proxy="));
}

#[test]
fn test_generate_command_no_proxy() {
    // 测试没有代理时返回 None
    let proxy_info = ProxyInfo::new();

    let command = ProxyConfigGenerator::generate_command(&proxy_info);
    assert!(command.is_none());
}

#[test]
fn test_generate_command_disabled_proxy() {
    // 测试所有代理都禁用时返回 None
    let mut proxy_info = ProxyInfo::new();
    proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: false,
            address: Some("proxy.example.com".to_string()),
            port: Some(8080),
        },
    );

    let command = ProxyConfigGenerator::generate_command(&proxy_info);
    assert!(command.is_none());
}

// ==================== 生成环境变量测试 ====================

#[test]
fn test_generate_env_vars_single_proxy() {
    // 测试生成单个代理的环境变量
    let mut proxy_info = ProxyInfo::new();
    proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("proxy.example.com".to_string()),
            port: Some(8080),
        },
    );

    let env_vars = ProxyConfigGenerator::generate_env_vars(&proxy_info);
    assert_eq!(env_vars.len(), 1);
    assert_eq!(
        env_vars.get("http_proxy"),
        Some(&"http://proxy.example.com:8080".to_string())
    );
}

#[test]
fn test_generate_env_vars_multiple_proxies() {
    // 测试生成多个代理的环境变量
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
        ProxyType::Socks,
        ProxyConfig {
            enable: true,
            address: Some("socks-proxy.example.com".to_string()),
            port: Some(1080),
        },
    );

    let env_vars = ProxyConfigGenerator::generate_env_vars(&proxy_info);
    assert_eq!(env_vars.len(), 2);
    assert_eq!(
        env_vars.get("http_proxy"),
        Some(&"http://http-proxy.example.com:8080".to_string())
    );
    assert_eq!(
        env_vars.get("all_proxy"),
        Some(&"socks5://socks-proxy.example.com:1080".to_string())
    );
}

#[test]
fn test_generate_env_vars_empty() {
    // 测试没有代理时返回空 HashMap
    let proxy_info = ProxyInfo::new();

    let env_vars = ProxyConfigGenerator::generate_env_vars(&proxy_info);
    assert!(env_vars.is_empty());
}

#[test]
fn test_generate_env_vars_all_types() {
    // 测试生成所有代理类型的环境变量
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
    proxy_info.set_config(
        ProxyType::Socks,
        ProxyConfig {
            enable: true,
            address: Some("socks-proxy.example.com".to_string()),
            port: Some(1080),
        },
    );

    let env_vars = ProxyConfigGenerator::generate_env_vars(&proxy_info);
    assert_eq!(env_vars.len(), 3);
    assert!(env_vars.contains_key("http_proxy"));
    assert!(env_vars.contains_key("https_proxy"));
    assert!(env_vars.contains_key("all_proxy"));
}
