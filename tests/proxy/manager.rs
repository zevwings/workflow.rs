//! ProxyManager 业务逻辑测试
//!
//! 测试 ProxyManager 模块的核心功能，包括：
//! - 环境变量代理检查
//! - 代理配置匹配验证
//! - 代理启用和禁用结果结构
//! - 复杂代理管理场景

use pretty_assertions::assert_eq;
use serial_test::serial;
use std::collections::HashMap;
use std::path::PathBuf;
use workflow::proxy::{
    ProxyConfig, ProxyDisableResult, ProxyEnableResult, ProxyInfo, ProxyManager, ProxyType,
};

// ==================== Helper Functions ====================

/// 创建测试用的 ProxyInfo
fn create_test_proxy_info() -> ProxyInfo {
    let mut proxy_info = ProxyInfo::new();

    proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("test.proxy.com".to_string()),
            port: Some(8080),
        },
    );

    proxy_info.set_config(
        ProxyType::Https,
        ProxyConfig {
            enable: true,
            address: Some("test.proxy.com".to_string()),
            port: Some(8080),
        },
    );

    proxy_info
}

/// 设置测试环境变量
fn set_test_env_vars() {
    std::env::set_var("http_proxy", "http://test.proxy.com:8080");
    std::env::set_var("https_proxy", "http://test.proxy.com:8080");
}

/// 清理测试环境变量
fn cleanup_test_env_vars() {
    std::env::remove_var("http_proxy");
    std::env::remove_var("https_proxy");
    std::env::remove_var("all_proxy");
}

// ==================== ProxyEnableResult 测试 ====================

/// 测试 ProxyEnableResult 结构体创建
#[test]
fn test_proxy_enable_result_creation() {
    let result = ProxyEnableResult {
        already_configured: false,
        proxy_command: Some("export http_proxy=http://proxy.com:8080".to_string()),
        shell_config_path: Some(PathBuf::from("/home/user/.bashrc")),
    };

    assert_eq!(result.already_configured, false);
    assert!(result.proxy_command.is_some());
    assert!(result.shell_config_path.is_some());

    let command = result.proxy_command.unwrap();
    assert!(command.starts_with("export "));
    assert!(command.contains("http_proxy="));
}

/// 测试 ProxyEnableResult 已配置场景
#[test]
fn test_proxy_enable_result_already_configured() {
    let result = ProxyEnableResult {
        already_configured: true,
        proxy_command: None,
        shell_config_path: Some(PathBuf::from("/home/user/.zshrc")),
    };

    assert_eq!(result.already_configured, true);
    assert_eq!(result.proxy_command, None);
    assert!(result.shell_config_path.is_some());

    let shell_path = result.shell_config_path.unwrap();
    assert!(shell_path.to_string_lossy().contains(".zshrc"));
}

/// 测试 ProxyEnableResult 克隆和调试输出
#[test]
fn test_proxy_enable_result_clone_and_debug() {
    let original_result = ProxyEnableResult {
        already_configured: false,
        proxy_command: Some("export all_proxy=socks5://proxy:1080".to_string()),
        shell_config_path: Some(PathBuf::from("/test/.config/fish/config.fish")),
    };

    // 测试克隆
    let cloned_result = original_result.clone();
    assert_eq!(
        original_result.already_configured,
        cloned_result.already_configured
    );
    assert_eq!(original_result.proxy_command, cloned_result.proxy_command);
    assert_eq!(
        original_result.shell_config_path,
        cloned_result.shell_config_path
    );

    // 测试调试输出
    let debug_str = format!("{:?}", original_result);
    assert!(debug_str.contains("ProxyEnableResult"));
    assert!(debug_str.contains("already_configured"));
    assert!(debug_str.contains("proxy_command"));
    assert!(debug_str.contains("shell_config_path"));
}

// ==================== ProxyDisableResult 测试 ====================

/// 测试 ProxyDisableResult 结构体创建
#[test]
fn test_proxy_disable_result_creation() {
    let mut current_env = HashMap::new();
    current_env.insert(
        "http_proxy".to_string(),
        "http://old.proxy.com:8080".to_string(),
    );
    current_env.insert(
        "https_proxy".to_string(),
        "http://old.proxy.com:8080".to_string(),
    );

    let result = ProxyDisableResult {
        found_proxy: true,
        shell_config_path: Some(PathBuf::from("/home/user/.bashrc")),
        unset_command: Some("unset http_proxy https_proxy all_proxy".to_string()),
        current_env_proxy: current_env.clone(),
    };

    assert_eq!(result.found_proxy, true);
    assert!(result.shell_config_path.is_some());
    assert!(result.unset_command.is_some());
    assert_eq!(result.current_env_proxy.len(), 2);

    let unset_cmd = result.unset_command.unwrap();
    assert!(unset_cmd.starts_with("unset "));
    assert!(unset_cmd.contains("http_proxy"));
}

/// 测试 ProxyDisableResult 未找到代理场景
#[test]
fn test_proxy_disable_result_no_proxy_found() {
    let result = ProxyDisableResult {
        found_proxy: false,
        shell_config_path: None,
        unset_command: None,
        current_env_proxy: HashMap::new(),
    };

    assert_eq!(result.found_proxy, false);
    assert_eq!(result.shell_config_path, None);
    assert_eq!(result.unset_command, None);
    assert!(result.current_env_proxy.is_empty());
}

/// 测试 ProxyDisableResult 克隆和调试输出
#[test]
fn test_proxy_disable_result_clone_and_debug() {
    let mut env_proxy = HashMap::new();
    env_proxy.insert(
        "all_proxy".to_string(),
        "socks5://socks.proxy.com:1080".to_string(),
    );

    let original_result = ProxyDisableResult {
        found_proxy: true,
        shell_config_path: Some(PathBuf::from("/test/.profile")),
        unset_command: Some("unset all_proxy".to_string()),
        current_env_proxy: env_proxy.clone(),
    };

    // 测试克隆
    let cloned_result = original_result.clone();
    assert_eq!(original_result.found_proxy, cloned_result.found_proxy);
    assert_eq!(
        original_result.shell_config_path,
        cloned_result.shell_config_path
    );
    assert_eq!(original_result.unset_command, cloned_result.unset_command);
    assert_eq!(
        original_result.current_env_proxy,
        cloned_result.current_env_proxy
    );

    // 测试调试输出
    let debug_str = format!("{:?}", original_result);
    assert!(debug_str.contains("ProxyDisableResult"));
    assert!(debug_str.contains("found_proxy"));
    assert!(debug_str.contains("current_env_proxy"));
}

// ==================== ProxyManager 测试 ====================

/// 测试环境变量代理检查
#[test]
#[serial]
fn test_proxy_manager_check_env_proxy() {
    // 清理环境变量
    cleanup_test_env_vars();

    // 测试空环境变量
    let empty_env = ProxyManager::check_env_proxy();
    assert!(empty_env.is_empty());

    // 设置测试环境变量
    set_test_env_vars();

    // 检查环境变量
    let env_proxy = ProxyManager::check_env_proxy();
    assert_eq!(env_proxy.len(), 2);
    assert_eq!(
        env_proxy.get("http_proxy"),
        Some(&"http://test.proxy.com:8080".to_string())
    );
    assert_eq!(
        env_proxy.get("https_proxy"),
        Some(&"http://test.proxy.com:8080".to_string())
    );

    // 清理
    cleanup_test_env_vars();
}

/// 测试代理配置匹配检查
#[test]
#[serial]
fn test_proxy_manager_is_proxy_configured() {
    cleanup_test_env_vars();

    let proxy_info = create_test_proxy_info();

    // 测试未配置环境变量的情况
    let is_configured_empty = ProxyManager::is_proxy_configured(&proxy_info);
    assert_eq!(is_configured_empty, false);

    // 设置匹配的环境变量
    set_test_env_vars();

    let is_configured_match = ProxyManager::is_proxy_configured(&proxy_info);
    assert_eq!(is_configured_match, true);

    // 设置不匹配的环境变量
    std::env::set_var("http_proxy", "http://different.proxy.com:9090");

    let is_configured_mismatch = ProxyManager::is_proxy_configured(&proxy_info);
    assert_eq!(is_configured_mismatch, false);

    cleanup_test_env_vars();
}

/// 测试部分代理配置匹配
#[test]
#[serial]
fn test_proxy_manager_partial_proxy_configuration() {
    cleanup_test_env_vars();

    let mut partial_info = ProxyInfo::new();
    partial_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("partial.proxy.com".to_string()),
            port: Some(3128),
        },
    );
    // HTTPS 和 SOCKS 未启用

    // 只设置 HTTP 代理环境变量
    std::env::set_var("http_proxy", "http://partial.proxy.com:3128");

    let is_configured = ProxyManager::is_proxy_configured(&partial_info);
    assert_eq!(is_configured, true);

    // 设置不匹配的 HTTP 代理
    std::env::set_var("http_proxy", "http://wrong.proxy.com:3128");

    let is_not_configured = ProxyManager::is_proxy_configured(&partial_info);
    assert_eq!(is_not_configured, false);

    cleanup_test_env_vars();
}

/// 测试 SOCKS 代理环境变量检查
#[test]
#[serial]
fn test_proxy_manager_socks_proxy_check() {
    cleanup_test_env_vars();

    let mut socks_info = ProxyInfo::new();
    socks_info.set_config(
        ProxyType::Socks,
        ProxyConfig {
            enable: true,
            address: Some("socks.test.com".to_string()),
            port: Some(1080),
        },
    );

    // 设置 SOCKS 代理环境变量
    std::env::set_var("all_proxy", "socks5://socks.test.com:1080");

    let env_proxy = ProxyManager::check_env_proxy();
    // 检查是否包含我们设置的 SOCKS 代理
    assert!(env_proxy.contains_key("all_proxy"));
    assert_eq!(
        env_proxy.get("all_proxy"),
        Some(&"socks5://socks.test.com:1080".to_string())
    );

    let is_configured = ProxyManager::is_proxy_configured(&socks_info);
    assert_eq!(is_configured, true);

    cleanup_test_env_vars();
}

/// 测试复杂代理配置场景
#[test]
#[serial]
fn test_complex_proxy_configuration_scenario() {
    cleanup_test_env_vars();

    // 创建复杂的代理配置
    let mut complex_info = ProxyInfo::new();

    complex_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("http.complex.com".to_string()),
            port: Some(8080),
        },
    );

    complex_info.set_config(
        ProxyType::Https,
        ProxyConfig {
            enable: false, // HTTPS 代理未启用
            address: Some("https.complex.com".to_string()),
            port: Some(8443),
        },
    );

    complex_info.set_config(
        ProxyType::Socks,
        ProxyConfig {
            enable: true,
            address: Some("socks.complex.com".to_string()),
            port: Some(1080),
        },
    );

    // 设置对应的环境变量（只设置启用的代理）
    std::env::set_var("http_proxy", "http://http.complex.com:8080");
    std::env::set_var("all_proxy", "socks5://socks.complex.com:1080");
    // 不设置 https_proxy，因为 HTTPS 代理未启用

    let env_proxy = ProxyManager::check_env_proxy();
    // 检查是否包含我们设置的代理（可能有其他测试设置的环境变量）
    assert!(env_proxy.len() >= 2);

    let is_configured = ProxyManager::is_proxy_configured(&complex_info);
    assert_eq!(is_configured, true);

    // 测试额外设置 HTTPS 代理的情况
    std::env::set_var("https_proxy", "http://wrong.https.com:8443");

    let is_still_configured = ProxyManager::is_proxy_configured(&complex_info);
    assert_eq!(is_still_configured, true); // 因为 HTTPS 代理未启用，所以仍然匹配

    cleanup_test_env_vars();
}

/// 测试边界情况和错误处理
#[test]
#[serial]
fn test_edge_cases_and_error_handling() {
    cleanup_test_env_vars();

    // 测试空的 ProxyInfo
    let empty_info = ProxyInfo::new();
    let empty_is_configured = ProxyManager::is_proxy_configured(&empty_info);

    // 注意：环境变量可能被其他测试设置，所以我们只检查逻辑
    assert_eq!(empty_is_configured, true); // 空配置认为是匹配的

    // 测试只有地址没有端口的配置
    let mut incomplete_info = ProxyInfo::new();
    incomplete_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("incomplete.com".to_string()),
            port: None,
        },
    );

    let incomplete_is_configured = ProxyManager::is_proxy_configured(&incomplete_info);
    assert_eq!(incomplete_is_configured, true); // 因为无法生成 URL，所以认为匹配

    // 测试环境变量值为空字符串的情况
    std::env::set_var("http_proxy", "");

    let empty_value_env = ProxyManager::check_env_proxy();
    assert_eq!(empty_value_env.len(), 1);
    assert_eq!(empty_value_env.get("http_proxy"), Some(&"".to_string()));

    cleanup_test_env_vars();
}

/// 测试多种代理类型同时存在的环境
#[test]
#[serial]
fn test_multiple_proxy_types_environment() {
    cleanup_test_env_vars();

    // 设置所有类型的代理环境变量
    std::env::set_var("http_proxy", "http://http.multi.com:8080");
    std::env::set_var("https_proxy", "http://https.multi.com:8443");
    std::env::set_var("all_proxy", "socks5://socks.multi.com:1080");

    let all_env_proxy = ProxyManager::check_env_proxy();
    // 检查是否包含我们设置的所有代理（可能有其他测试设置的环境变量）
    assert!(all_env_proxy.len() >= 3);

    // 验证所有代理类型都被检测到
    assert!(all_env_proxy.contains_key("http_proxy"));
    assert!(all_env_proxy.contains_key("https_proxy"));
    assert!(all_env_proxy.contains_key("all_proxy"));

    // 创建匹配的 ProxyInfo
    let mut multi_info = ProxyInfo::new();

    multi_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("http.multi.com".to_string()),
            port: Some(8080),
        },
    );

    multi_info.set_config(
        ProxyType::Https,
        ProxyConfig {
            enable: true,
            address: Some("https.multi.com".to_string()),
            port: Some(8443),
        },
    );

    multi_info.set_config(
        ProxyType::Socks,
        ProxyConfig {
            enable: true,
            address: Some("socks.multi.com".to_string()),
            port: Some(1080),
        },
    );

    let is_all_configured = ProxyManager::is_proxy_configured(&multi_info);
    assert_eq!(is_all_configured, true);

    cleanup_test_env_vars();
}

/// 测试代理结果结构的完整性
#[test]
fn test_proxy_result_structures_completeness() {
    // 测试 ProxyEnableResult 的所有字段组合
    let enable_results = vec![
        ProxyEnableResult {
            already_configured: true,
            proxy_command: None,
            shell_config_path: None,
        },
        ProxyEnableResult {
            already_configured: false,
            proxy_command: Some("export http_proxy=http://test:8080".to_string()),
            shell_config_path: Some(PathBuf::from("/home/.bashrc")),
        },
        ProxyEnableResult {
            already_configured: false,
            proxy_command: None,
            shell_config_path: Some(PathBuf::from("/home/.zshrc")),
        },
    ];

    for result in &enable_results {
        // 验证结构体的一致性
        if result.already_configured {
            assert!(result.proxy_command.is_none());
        }
        // 其他验证逻辑可以根据业务需求添加
    }

    // 测试 ProxyDisableResult 的所有字段组合
    let disable_results = vec![
        ProxyDisableResult {
            found_proxy: false,
            shell_config_path: None,
            unset_command: None,
            current_env_proxy: HashMap::new(),
        },
        ProxyDisableResult {
            found_proxy: true,
            shell_config_path: Some(PathBuf::from("/test/.profile")),
            unset_command: Some("unset http_proxy https_proxy".to_string()),
            current_env_proxy: {
                let mut map = HashMap::new();
                map.insert("http_proxy".to_string(), "http://test:8080".to_string());
                map
            },
        },
    ];

    for result in &disable_results {
        // 验证结构体的一致性
        if !result.found_proxy {
            assert!(result.unset_command.is_none());
            assert!(result.current_env_proxy.is_empty());
        }
    }
}
