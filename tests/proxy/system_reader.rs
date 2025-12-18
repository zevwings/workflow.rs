//! SystemProxyReader 和 ProxyConfigGenerator 业务逻辑测试
//!
//! 测试 Proxy 模块的核心功能，包括：
//! - 系统代理信息读取和解析
//! - 代理配置生成和格式化
//! - 代理类型和 URL 处理
//! - 环境变量生成

use pretty_assertions::assert_eq;
use std::collections::HashMap;
use workflow::proxy::{ProxyConfig, ProxyConfigGenerator, ProxyInfo, ProxyType};

// ==================== Helper Functions ====================

/// 创建测试用的 ProxyInfo
fn create_test_proxy_info() -> ProxyInfo {
    let mut proxy_info = ProxyInfo::new();

    // HTTP 代理配置
    proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("proxy.example.com".to_string()),
            port: Some(8080),
        },
    );

    // HTTPS 代理配置
    proxy_info.set_config(
        ProxyType::Https,
        ProxyConfig {
            enable: true,
            address: Some("proxy.example.com".to_string()),
            port: Some(8080),
        },
    );

    // SOCKS 代理配置（未启用）
    proxy_info.set_config(
        ProxyType::Socks,
        ProxyConfig {
            enable: false,
            address: Some("socks.example.com".to_string()),
            port: Some(1080),
        },
    );

    proxy_info
}

/// 创建部分启用的 ProxyInfo
fn create_partial_proxy_info() -> ProxyInfo {
    let mut proxy_info = ProxyInfo::new();

    // 只启用 HTTP 代理
    proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("http-proxy.test.com".to_string()),
            port: Some(3128),
        },
    );

    // HTTPS 代理未启用
    proxy_info.set_config(
        ProxyType::Https,
        ProxyConfig {
            enable: false,
            address: Some("https-proxy.test.com".to_string()),
            port: Some(3129),
        },
    );

    proxy_info
}

// ==================== ProxyType 测试 ====================

/// 测试 ProxyType 枚举基本功能
#[test]
fn test_proxy_type_basic_functionality() {
    // 测试环境变量键名
    assert_eq!(ProxyType::Http.env_key(), "http_proxy");
    assert_eq!(ProxyType::Https.env_key(), "https_proxy");
    assert_eq!(ProxyType::Socks.env_key(), "all_proxy");

    // 测试 URL 协议方案
    assert_eq!(ProxyType::Http.url_scheme(), "http");
    assert_eq!(ProxyType::Https.url_scheme(), "http");
    assert_eq!(ProxyType::Socks.url_scheme(), "socks5");

    // 测试所有环境变量键名
    let all_keys = ProxyType::all_env_keys();
    assert_eq!(all_keys.len(), 3);
    assert!(all_keys.contains(&"http_proxy"));
    assert!(all_keys.contains(&"https_proxy"));
    assert!(all_keys.contains(&"all_proxy"));
}

/// 测试 ProxyType 迭代器
#[test]
fn test_proxy_type_iterator() {
    let all_types: Vec<ProxyType> = ProxyType::all().collect();
    assert_eq!(all_types.len(), 3);
    assert!(all_types.contains(&ProxyType::Http));
    assert!(all_types.contains(&ProxyType::Https));
    assert!(all_types.contains(&ProxyType::Socks));

    // 测试迭代器的顺序
    let mut iter = ProxyType::all();
    assert_eq!(iter.next(), Some(ProxyType::Http));
    assert_eq!(iter.next(), Some(ProxyType::Https));
    assert_eq!(iter.next(), Some(ProxyType::Socks));
    assert_eq!(iter.next(), None);
}

/// 测试 ProxyType 比较和哈希
#[test]
fn test_proxy_type_equality_and_hash() {
    assert_eq!(ProxyType::Http, ProxyType::Http);
    assert_ne!(ProxyType::Http, ProxyType::Https);

    // 测试在 HashMap 中使用
    let mut map = HashMap::new();
    map.insert(ProxyType::Http, "http_value");
    map.insert(ProxyType::Https, "https_value");

    assert_eq!(map.get(&ProxyType::Http), Some(&"http_value"));
    assert_eq!(map.get(&ProxyType::Https), Some(&"https_value"));
    assert_eq!(map.get(&ProxyType::Socks), None);
}

// ==================== ProxyConfig 测试 ====================

/// 测试 ProxyConfig 结构体创建和字段访问
#[test]
fn test_proxy_config_creation() {
    let config = ProxyConfig {
        enable: true,
        address: Some("proxy.test.com".to_string()),
        port: Some(8080),
    };

    assert_eq!(config.enable, true);
    assert_eq!(config.address, Some("proxy.test.com".to_string()));
    assert_eq!(config.port, Some(8080));

    // 测试部分配置
    let partial_config = ProxyConfig {
        enable: false,
        address: None,
        port: None,
    };

    assert_eq!(partial_config.enable, false);
    assert_eq!(partial_config.address, None);
    assert_eq!(partial_config.port, None);
}

/// 测试 ProxyConfig 克隆和调试输出
#[test]
fn test_proxy_config_clone_and_debug() {
    let original_config = ProxyConfig {
        enable: true,
        address: Some("clone.test.com".to_string()),
        port: Some(9090),
    };

    // 测试克隆
    let cloned_config = original_config.clone();
    assert_eq!(original_config.enable, cloned_config.enable);
    assert_eq!(original_config.address, cloned_config.address);
    assert_eq!(original_config.port, cloned_config.port);

    // 测试调试输出
    let debug_str = format!("{:?}", original_config);
    assert!(debug_str.contains("ProxyConfig"));
    assert!(debug_str.contains("enable: true"));
    assert!(debug_str.contains("clone.test.com"));
    assert!(debug_str.contains("9090"));
}

// ==================== ProxyInfo 测试 ====================

/// 测试 ProxyInfo 创建和基本操作
#[test]
fn test_proxy_info_creation_and_basic_operations() {
    let mut proxy_info = ProxyInfo::new();

    // 测试初始状态
    assert!(proxy_info.get_config(ProxyType::Http).is_none());
    assert!(proxy_info.get_config(ProxyType::Https).is_none());
    assert!(proxy_info.get_config(ProxyType::Socks).is_none());

    // 测试设置配置
    let http_config = ProxyConfig {
        enable: true,
        address: Some("test.proxy.com".to_string()),
        port: Some(8080),
    };

    proxy_info.set_config(ProxyType::Http, http_config.clone());

    // 验证配置已设置
    let retrieved_config = proxy_info.get_config(ProxyType::Http).unwrap();
    assert_eq!(retrieved_config.enable, true);
    assert_eq!(retrieved_config.address, Some("test.proxy.com".to_string()));
    assert_eq!(retrieved_config.port, Some(8080));
}

/// 测试 ProxyInfo 可变引用获取
#[test]
fn test_proxy_info_mutable_config() {
    let mut proxy_info = ProxyInfo::new();

    // 获取可变引用（会自动创建默认配置）
    let http_config = proxy_info.get_config_mut(ProxyType::Http);
    assert_eq!(http_config.enable, false);
    assert_eq!(http_config.address, None);
    assert_eq!(http_config.port, None);

    // 修改配置
    http_config.enable = true;
    http_config.address = Some("mutable.proxy.com".to_string());
    http_config.port = Some(7070);

    // 验证修改生效
    let retrieved_config = proxy_info.get_config(ProxyType::Http).unwrap();
    assert_eq!(retrieved_config.enable, true);
    assert_eq!(
        retrieved_config.address,
        Some("mutable.proxy.com".to_string())
    );
    assert_eq!(retrieved_config.port, Some(7070));
}

/// 测试 ProxyInfo 代理 URL 生成
#[test]
fn test_proxy_info_url_generation() {
    let proxy_info = create_test_proxy_info();

    // 测试启用的代理 URL 生成
    assert_eq!(
        proxy_info.get_proxy_url(ProxyType::Http),
        Some("http://proxy.example.com:8080".to_string())
    );
    assert_eq!(
        proxy_info.get_proxy_url(ProxyType::Https),
        Some("http://proxy.example.com:8080".to_string())
    );

    // 测试未启用的代理
    assert_eq!(proxy_info.get_proxy_url(ProxyType::Socks), None);

    // 测试不完整配置
    let mut incomplete_info = ProxyInfo::new();
    incomplete_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("incomplete.com".to_string()),
            port: None, // 缺少端口
        },
    );

    assert_eq!(incomplete_info.get_proxy_url(ProxyType::Http), None);
}

/// 测试 ProxyInfo 默认实现和克隆
#[test]
fn test_proxy_info_default_and_clone() {
    let default_info = ProxyInfo::default();
    assert!(default_info.get_config(ProxyType::Http).is_none());

    let proxy_info = create_test_proxy_info();
    let cloned_info = proxy_info.clone();

    // 验证克隆的完整性
    assert_eq!(
        proxy_info.get_proxy_url(ProxyType::Http),
        cloned_info.get_proxy_url(ProxyType::Http)
    );
    assert_eq!(
        proxy_info.get_proxy_url(ProxyType::Https),
        cloned_info.get_proxy_url(ProxyType::Https)
    );
    assert_eq!(
        proxy_info.get_proxy_url(ProxyType::Socks),
        cloned_info.get_proxy_url(ProxyType::Socks)
    );
}

// ==================== ProxyConfigGenerator 测试 ====================

/// 测试代理命令生成
#[test]
fn test_proxy_config_generator_command_generation() {
    let proxy_info = create_test_proxy_info();

    let command = ProxyConfigGenerator::generate_command(&proxy_info);
    assert!(command.is_some());

    let cmd_str = command.unwrap();
    assert!(cmd_str.starts_with("export "));
    assert!(cmd_str.contains("http_proxy=http://proxy.example.com:8080"));
    assert!(cmd_str.contains("https_proxy=http://proxy.example.com:8080"));
    assert!(!cmd_str.contains("all_proxy")); // SOCKS 未启用

    // 测试空代理信息
    let empty_info = ProxyInfo::new();
    let empty_command = ProxyConfigGenerator::generate_command(&empty_info);
    assert_eq!(empty_command, None);
}

/// 测试环境变量生成
#[test]
fn test_proxy_config_generator_env_vars() {
    let proxy_info = create_test_proxy_info();

    let env_vars = ProxyConfigGenerator::generate_env_vars(&proxy_info);
    assert_eq!(env_vars.len(), 2); // HTTP 和 HTTPS

    assert_eq!(
        env_vars.get("http_proxy"),
        Some(&"http://proxy.example.com:8080".to_string())
    );
    assert_eq!(
        env_vars.get("https_proxy"),
        Some(&"http://proxy.example.com:8080".to_string())
    );
    assert!(env_vars.get("all_proxy").is_none()); // SOCKS 未启用

    // 测试部分代理配置
    let partial_info = create_partial_proxy_info();
    let partial_env_vars = ProxyConfigGenerator::generate_env_vars(&partial_info);
    assert_eq!(partial_env_vars.len(), 1); // 只有 HTTP

    assert_eq!(
        partial_env_vars.get("http_proxy"),
        Some(&"http://http-proxy.test.com:3128".to_string())
    );
    assert!(partial_env_vars.get("https_proxy").is_none());
}

/// 测试 SOCKS 代理 URL 生成
#[test]
fn test_socks_proxy_url_generation() {
    let mut proxy_info = ProxyInfo::new();
    proxy_info.set_config(
        ProxyType::Socks,
        ProxyConfig {
            enable: true,
            address: Some("socks.proxy.com".to_string()),
            port: Some(1080),
        },
    );

    assert_eq!(
        proxy_info.get_proxy_url(ProxyType::Socks),
        Some("socks5://socks.proxy.com:1080".to_string())
    );

    let env_vars = ProxyConfigGenerator::generate_env_vars(&proxy_info);
    assert_eq!(env_vars.len(), 1);
    assert_eq!(
        env_vars.get("all_proxy"),
        Some(&"socks5://socks.proxy.com:1080".to_string())
    );
}

/// 测试复杂代理配置场景
#[test]
fn test_complex_proxy_configuration() {
    let mut complex_info = ProxyInfo::new();

    // 设置所有类型的代理
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
            enable: true,
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

    // 验证所有代理 URL
    assert_eq!(
        complex_info.get_proxy_url(ProxyType::Http),
        Some("http://http.complex.com:8080".to_string())
    );
    assert_eq!(
        complex_info.get_proxy_url(ProxyType::Https),
        Some("http://https.complex.com:8443".to_string())
    );
    assert_eq!(
        complex_info.get_proxy_url(ProxyType::Socks),
        Some("socks5://socks.complex.com:1080".to_string())
    );

    // 验证环境变量生成
    let env_vars = ProxyConfigGenerator::generate_env_vars(&complex_info);
    assert_eq!(env_vars.len(), 3);

    // 验证命令生成
    let command = ProxyConfigGenerator::generate_command(&complex_info);
    assert!(command.is_some());

    let cmd_str = command.unwrap();
    assert!(cmd_str.contains("http_proxy=http://http.complex.com:8080"));
    assert!(cmd_str.contains("https_proxy=http://https.complex.com:8443"));
    assert!(cmd_str.contains("all_proxy=socks5://socks.complex.com:1080"));
}

/// 测试边界情况和错误处理
#[test]
fn test_edge_cases_and_error_handling() {
    // 测试端口为 0 的情况
    let mut zero_port_info = ProxyInfo::new();
    zero_port_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("zero-port.com".to_string()),
            port: Some(0),
        },
    );

    assert_eq!(
        zero_port_info.get_proxy_url(ProxyType::Http),
        Some("http://zero-port.com:0".to_string())
    );

    // 测试地址为空字符串的情况
    let mut empty_addr_info = ProxyInfo::new();
    empty_addr_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("".to_string()),
            port: Some(8080),
        },
    );

    assert_eq!(
        empty_addr_info.get_proxy_url(ProxyType::Http),
        Some("http://:8080".to_string())
    );

    // 测试最大端口号
    let mut max_port_info = ProxyInfo::new();
    max_port_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("max-port.com".to_string()),
            port: Some(65535),
        },
    );

    assert_eq!(
        max_port_info.get_proxy_url(ProxyType::Http),
        Some("http://max-port.com:65535".to_string())
    );
}
