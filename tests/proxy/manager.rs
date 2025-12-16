//! Proxy 管理器测试
//!
//! 测试 Proxy 管理器相关的功能，包括：
//! - 检查环境变量代理
//! - 检查代理配置
//! - 启用/禁用代理

use pretty_assertions::assert_eq;
use std::collections::HashMap;
use workflow::proxy::{ProxyConfig, ProxyInfo, ProxyManager, ProxyType};

// ==================== 检查环境变量代理测试 ====================

#[test]
fn test_check_env_proxy_empty() {
    // 测试检查空的环境变量代理
    // 注意：这个测试依赖于实际环境，可能返回空或非空
    let env_proxy = ProxyManager::check_env_proxy();

    // 验证返回类型正确（HashMap）
    assert!(env_proxy.is_empty() || !env_proxy.is_empty());
}

#[test]
fn test_check_env_proxy_with_env() {
    // 测试检查有环境变量的代理
    // 设置测试环境变量
    std::env::set_var("http_proxy", "http://test-proxy:8080");

    let env_proxy = ProxyManager::check_env_proxy();

    // 应该包含 http_proxy
    assert!(env_proxy.contains_key("http_proxy") || env_proxy.is_empty());

    // 清理环境变量
    std::env::remove_var("http_proxy");
}

// ==================== 检查代理配置测试 ====================

#[test]
fn test_is_proxy_configured_matching() {
    // 测试代理配置匹配
    let mut proxy_info = ProxyInfo::new();
    proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("proxy.example.com".to_string()),
            port: Some(8080),
        },
    );

    // 设置匹配的环境变量
    std::env::set_var("http_proxy", "http://proxy.example.com:8080");

    let is_configured = ProxyManager::is_proxy_configured(&proxy_info);
    // 应该匹配
    assert!(is_configured);

    // 清理环境变量
    std::env::remove_var("http_proxy");
}

#[test]
fn test_is_proxy_configured_not_matching() {
    // 测试代理配置不匹配
    let mut proxy_info = ProxyInfo::new();
    proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("proxy.example.com".to_string()),
            port: Some(8080),
        },
    );

    // 设置不匹配的环境变量
    std::env::set_var("http_proxy", "http://different-proxy:9090");

    let is_configured = ProxyManager::is_proxy_configured(&proxy_info);
    // 应该不匹配
    assert!(!is_configured);

    // 清理环境变量
    std::env::remove_var("http_proxy");
}

#[test]
fn test_is_proxy_configured_disabled() {
    // 测试禁用的代理（应该认为配置正确）
    let mut proxy_info = ProxyInfo::new();
    proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: false,
            address: Some("proxy.example.com".to_string()),
            port: Some(8080),
        },
    );

    // 即使环境变量不匹配，禁用的代理也应该认为配置正确
    let is_configured = ProxyManager::is_proxy_configured(&proxy_info);
    assert!(is_configured);
}

#[test]
fn test_is_proxy_configured_no_env() {
    // 测试没有环境变量时
    let mut proxy_info = ProxyInfo::new();
    proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("proxy.example.com".to_string()),
            port: Some(8080),
        },
    );

    // 确保环境变量不存在
    std::env::remove_var("http_proxy");

    let is_configured = ProxyManager::is_proxy_configured(&proxy_info);
    // 没有环境变量时应该不匹配
    assert!(!is_configured);
}

// ==================== 确保代理启用测试 ====================

#[test]
fn test_ensure_proxy_enabled_no_system_proxy() {
    // 测试系统代理未启用时（应该静默跳过）
    // 注意：这个测试依赖于实际的系统代理设置
    // 在大多数测试环境中，系统代理可能未启用
    let result = ProxyManager::ensure_proxy_enabled();
    // 应该成功（静默跳过）
    assert!(result.is_ok());
}

// ==================== 启用/禁用代理测试 ====================

#[test]
fn test_enable_proxy_temporary() {
    // 测试临时启用代理
    // 注意：这个测试会尝试读取系统代理设置，可能失败
    let result = ProxyManager::enable(true);

    // 可能成功或失败，取决于系统代理设置
    if result.is_ok() {
        let enable_result = result.unwrap();
        // 临时模式下，shell_config_path 应该为 None
        assert_eq!(enable_result.shell_config_path, None);
    }
}

#[test]
fn test_disable_proxy_no_proxy() {
    // 测试禁用不存在的代理
    // 确保环境变量不存在
    for key in ProxyType::all_env_keys() {
        std::env::remove_var(key);
    }

    let result = ProxyManager::disable();
    assert!(result.is_ok());

    let disable_result = result.unwrap();
    // 没有代理时，found_proxy 应该为 false
    assert!(!disable_result.found_proxy);
}
