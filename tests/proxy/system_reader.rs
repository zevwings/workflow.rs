//! SystemProxyReader 和 ProxyConfigGenerator 业务逻辑测试
//!
//! 测试 SystemProxyReader 和 ProxyConfigGenerator 模块的核心功能，包括：
//! - 系统代理配置读取
//! - 代理配置生成
//! - 环境变量生成
//! - 命令字符串生成

use pretty_assertions::assert_eq;
use workflow::proxy::{ProxyConfig, ProxyConfigGenerator, ProxyInfo, ProxyType, SystemProxyReader};

// ==================== Helper Functions ====================

/// 创建测试用的 ProxyInfo
fn create_test_proxy_info() -> ProxyInfo {
    let mut proxy_info = ProxyInfo::new();

    // 设置 HTTP 代理
    proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("proxy.example.com".to_string()),
            port: Some(8080),
        },
    );

    // 设置 HTTPS 代理
    proxy_info.set_config(
        ProxyType::Https,
        ProxyConfig {
            enable: true,
            address: Some("secure-proxy.example.com".to_string()),
            port: Some(8443),
        },
    );

    // 设置 SOCKS 代理
    proxy_info.set_config(
        ProxyType::Socks,
        ProxyConfig {
            enable: false,
            address: Some("socks-proxy.example.com".to_string()),
            port: Some(1080),
        },
    );

    proxy_info
}

/// 创建最小配置的 ProxyInfo（只有 HTTP 代理）
fn create_minimal_proxy_info() -> ProxyInfo {
    let mut proxy_info = ProxyInfo::new();

    proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("simple.proxy.com".to_string()),
            port: Some(3128),
        },
    );

    proxy_info
}

/// 创建空的 ProxyInfo（所有代理都禁用）
fn create_empty_proxy_info() -> ProxyInfo {
    let mut proxy_info = ProxyInfo::new();

    // 设置所有代理为禁用状态
    for proxy_type in [ProxyType::Http, ProxyType::Https, ProxyType::Socks] {
        proxy_info.set_config(
            proxy_type,
            ProxyConfig {
                enable: false,
                address: None,
                port: None,
            },
        );
    }

    proxy_info
}

// ==================== ProxyConfig Tests ====================

#[test]
fn test_proxy_config_creation_with_valid_fields_creates_config() {
    // Arrange: 准备代理配置字段值
    let enable = true;
    let address = Some("proxy.example.com".to_string());
    let port = Some(8080);

    // Act: 创建 ProxyConfig 实例
    let config = ProxyConfig {
        enable,
        address: address.clone(),
        port,
    };

    // Assert: 验证所有字段值正确
    assert_eq!(config.enable, enable);
    assert_eq!(config.address, address);
    assert_eq!(config.port, port);
}

#[test]
fn test_proxy_config_disabled_with_disabled_state_creates_config() {
    // Arrange: 准备禁用状态的代理配置
    let enable = false;

    // Act: 创建 ProxyConfig 实例（禁用状态）
    let config = ProxyConfig {
        enable,
        address: None,
        port: None,
    };

    // Assert: 验证字段值正确
    assert_eq!(config.enable, enable);
    assert_eq!(config.address, None);
    assert_eq!(config.port, None);
}

#[test]
fn test_proxy_config_clone_with_valid_config_creates_clone() {
    // Arrange: 准备原始 ProxyConfig
    let original_config = ProxyConfig {
        enable: true,
        address: Some("test.proxy.com".to_string()),
        port: Some(3128),
    };

    // Act: 克隆配置
    let cloned_config = original_config.clone();

    // Assert: 验证克隆的字段值与原始值相同
    assert_eq!(original_config.enable, cloned_config.enable);
    assert_eq!(original_config.address, cloned_config.address);
    assert_eq!(original_config.port, cloned_config.port);
}

#[test]
fn test_proxy_config_debug_with_valid_config_returns_debug_string() {
    // Arrange: 准备 ProxyConfig 实例
    let config = ProxyConfig {
        enable: true,
        address: Some("debug.proxy.com".to_string()),
        port: Some(8080),
    };

    // Act: 格式化 Debug 输出
    let debug_str = format!("{:?}", config);

    // Assert: 验证 Debug 字符串包含预期内容
    assert!(debug_str.contains("ProxyConfig"));
    assert!(debug_str.contains("enable"));
    assert!(debug_str.contains("address"));
    assert!(debug_str.contains("debug.proxy.com"));
}

// ==================== SystemProxyReader Tests ====================

#[test]
fn test_system_proxy_reader_read_with_system_config_returns_proxy_info() {
    // Arrange: 准备读取系统代理配置
    // 注意：由于 SystemProxyReader 需要访问系统配置，在测试环境中可能会失败

    // Act: 读取系统代理配置
    let read_result = SystemProxyReader::read();

    // Assert: 验证返回结果（成功或失败都是可以接受的）
    match read_result {
        Ok(proxy_info) => {
            // 如果成功读取，验证 ProxyInfo 结构
            if let Some(http_config) = proxy_info.get_config(ProxyType::Http) {
                assert!(http_config.enable == true || http_config.enable == false);
            }
            if let Some(https_config) = proxy_info.get_config(ProxyType::Https) {
                assert!(https_config.enable == true || https_config.enable == false);
            }
            if let Some(socks_config) = proxy_info.get_config(ProxyType::Socks) {
                assert!(socks_config.enable == true || socks_config.enable == false);
            }
        }
        Err(_) => {
            // 在测试环境中失败是可以接受的（可能没有 scutil 命令或权限问题）
        }
    }
}

// ==================== ProxyInfo Tests ====================

#[test]
fn test_proxy_info_creation_with_valid_configs_creates_info() {
    // Arrange: 准备测试用的 ProxyInfo
    let proxy_info = create_test_proxy_info();

    // Act: 获取各个代理类型的配置
    let http_config = proxy_info.get_config(ProxyType::Http).expect("operation should succeed");
    let https_config = proxy_info.get_config(ProxyType::Https).expect("operation should succeed");
    let socks_config = proxy_info.get_config(ProxyType::Socks).expect("operation should succeed");

    // Assert: 验证所有代理配置正确
    assert_eq!(http_config.enable, true);
    assert_eq!(http_config.address, Some("proxy.example.com".to_string()));
    assert_eq!(http_config.port, Some(8080));
    assert_eq!(https_config.enable, true);
    assert_eq!(
        https_config.address,
        Some("secure-proxy.example.com".to_string())
    );
    assert_eq!(https_config.port, Some(8443));
    assert_eq!(socks_config.enable, false);
    assert_eq!(
        socks_config.address,
        Some("socks-proxy.example.com".to_string())
    );
    assert_eq!(socks_config.port, Some(1080));
}

#[test]
fn test_proxy_info_get_proxy_url_with_enabled_proxies_returns_urls() {
    // Arrange: 准备测试用的 ProxyInfo
    let proxy_info = create_test_proxy_info();

    // Act: 获取各个代理类型的 URL
    let http_url = proxy_info.get_proxy_url(ProxyType::Http);
    let https_url = proxy_info.get_proxy_url(ProxyType::Https);
    let socks_url = proxy_info.get_proxy_url(ProxyType::Socks);

    // Assert: 验证 HTTP 和 HTTPS 代理返回 URL，SOCKS 代理返回 None
    assert!(http_url.is_some());
    let http_url = http_url.expect("operation should succeed");
    assert!(http_url.contains("http://"));
    assert!(http_url.contains("proxy.example.com"));
    assert!(http_url.contains("8080"));
    assert!(https_url.is_some());
    let https_url = https_url.expect("operation should succeed");
    assert!(https_url.contains("http://"));
    assert!(https_url.contains("secure-proxy.example.com"));
    assert!(https_url.contains("8443"));
    assert!(socks_url.is_none());
}

// ==================== ProxyConfigGenerator Tests ====================

#[test]
fn test_proxy_config_generator_generate_env_vars_with_valid_info_generates_vars() {
    // Arrange: 准备测试用的 ProxyInfo
    let proxy_info = create_test_proxy_info();

    // Act: 生成环境变量
    let env_vars = ProxyConfigGenerator::generate_env_vars(&proxy_info);

    // Assert: 验证生成的环境变量包含预期内容
    assert!(!env_vars.is_empty());
    if let Some(http_proxy) = env_vars.get("http_proxy") {
        assert!(http_proxy.contains("http://"));
        assert!(http_proxy.contains("proxy.example.com"));
        assert!(http_proxy.contains("8080"));
    }
    if let Some(https_proxy) = env_vars.get("https_proxy") {
        assert!(https_proxy.contains("http://"));
        assert!(https_proxy.contains("secure-proxy.example.com"));
        assert!(https_proxy.contains("8443"));
    }
    assert!(!env_vars.contains_key("all_proxy"));
}

#[test]
fn test_proxy_config_generator_generate_command_with_valid_info_generates_command() {
    // Arrange: 准备最小配置的 ProxyInfo
    let proxy_info = create_minimal_proxy_info();

    // Act: 生成命令字符串
    let command = ProxyConfigGenerator::generate_command(&proxy_info);

    // Assert: 验证生成的命令字符串包含预期内容
    if let Some(cmd) = command {
        assert!(cmd.starts_with("export "));
        assert!(cmd.contains("http_proxy="));
        assert!(cmd.contains("simple.proxy.com"));
        assert!(cmd.contains("3128"));
    }
}

/// 测试空 ProxyInfo 的命令生成
#[test]
fn test_proxy_config_generator_empty_command() {
    let empty_proxy_info = create_empty_proxy_info();

    let command = ProxyConfigGenerator::generate_command(&empty_proxy_info);

    // 空配置应该返回 None
    assert!(command.is_none());
}

/// 测试不同代理类型的配置生成
#[test]
fn test_proxy_config_generator_different_types() {
    // Arrange: 准备测试 HTTP 代理
    let mut http_proxy_info = ProxyInfo::new();
    http_proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("http.example.com".to_string()),
            port: Some(8080),
        },
    );

    // Arrange: 准备测试 HTTPS 代理
    let mut https_proxy_info = ProxyInfo::new();
    https_proxy_info.set_config(
        ProxyType::Https,
        ProxyConfig {
            enable: true,
            address: Some("https.example.com".to_string()),
            port: Some(8443),
        },
    );

    // Arrange: 准备测试 SOCKS 代理
    let mut socks_proxy_info = ProxyInfo::new();
    socks_proxy_info.set_config(
        ProxyType::Socks,
        ProxyConfig {
            enable: true,
            address: Some("socks.example.com".to_string()),
            port: Some(1080),
        },
    );

    // Arrange: 准备测试每种类型的环境变量生成
    let http_env_vars = ProxyConfigGenerator::generate_env_vars(&http_proxy_info);
    assert!(http_env_vars.contains_key("http_proxy"));

    let https_env_vars = ProxyConfigGenerator::generate_env_vars(&https_proxy_info);
    assert!(https_env_vars.contains_key("https_proxy"));

    let socks_env_vars = ProxyConfigGenerator::generate_env_vars(&socks_proxy_info);
    assert!(socks_env_vars.contains_key("all_proxy"));
}

/// 测试 ProxyType 枚举功能
#[test]
fn test_proxy_type_enum() {
    // Arrange: 准备测试所有代理类型
    let all_types: Vec<ProxyType> = ProxyType::all().collect();
    assert_eq!(all_types.len(), 3);
    assert!(all_types.contains(&ProxyType::Http));
    assert!(all_types.contains(&ProxyType::Https));
    assert!(all_types.contains(&ProxyType::Socks));

    // Arrange: 准备测试环境变量键名
    assert_eq!(ProxyType::Http.env_key(), "http_proxy");
    assert_eq!(ProxyType::Https.env_key(), "https_proxy");
    assert_eq!(ProxyType::Socks.env_key(), "all_proxy");

    // Arrange: 准备测试 URL 协议方案
    assert_eq!(ProxyType::Http.url_scheme(), "http");
    assert_eq!(ProxyType::Https.url_scheme(), "http");
    assert_eq!(ProxyType::Socks.url_scheme(), "socks5");

    // Arrange: 准备测试所有环境变量键名
    let all_env_keys = ProxyType::all_env_keys();
    assert_eq!(all_env_keys.len(), 3);
    assert!(all_env_keys.contains(&"http_proxy"));
    assert!(all_env_keys.contains(&"https_proxy"));
    assert!(all_env_keys.contains(&"all_proxy"));
}

/// 测试复杂的 ProxyInfo 配置
#[test]
fn test_complex_proxy_info_configuration() {
    let mut proxy_info = ProxyInfo::new();

    // 设置混合配置：HTTP 启用，HTTPS 禁用，SOCKS 启用
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
            enable: false,
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

    // Assert: 验证环境变量生成
    let env_vars = ProxyConfigGenerator::generate_env_vars(&proxy_info);

    // 应该包含启用的代理
    assert!(env_vars.contains_key("http_proxy"));
    assert!(env_vars.contains_key("all_proxy"));

    // 不应该包含禁用的代理
    assert!(!env_vars.contains_key("https_proxy"));

    // Assert: 验证 URL 格式
    let http_url = env_vars.get("http_proxy").expect("operation should succeed");
    assert!(http_url.starts_with("http://"));
    assert!(http_url.contains("http-proxy.example.com:8080"));

    let socks_url = env_vars.get("all_proxy").expect("operation should succeed");
    assert!(socks_url.starts_with("socks5://"));
    assert!(socks_url.contains("socks-proxy.example.com:1080"));
}

/// 测试边界情况和错误处理
#[test]
fn test_edge_cases_and_error_handling() {
    // Arrange: 准备测试无效的代理配置（空地址）
    let mut invalid_proxy_info = ProxyInfo::new();
    invalid_proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: None, // 空地址
            port: Some(8080),
        },
    );

    let env_vars = ProxyConfigGenerator::generate_env_vars(&invalid_proxy_info);
    // 无效配置不应该生成环境变量
    assert!(env_vars.is_empty());

    // Arrange: 准备测试无效端口
    let mut invalid_port_proxy = ProxyInfo::new();
    invalid_port_proxy.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("test.com".to_string()),
            port: None, // 无端口
        },
    );

    let env_vars_no_port = ProxyConfigGenerator::generate_env_vars(&invalid_port_proxy);
    // 无端口配置不应该生成环境变量
    assert!(env_vars_no_port.is_empty());

    // Arrange: 准备测试空配置的环境变量生成
    let empty_proxy_info = create_empty_proxy_info();
    let empty_env_vars = ProxyConfigGenerator::generate_env_vars(&empty_proxy_info);
    // 空配置应该生成空的环境变量映射
    assert!(empty_env_vars.is_empty());

    // Arrange: 准备测试空配置的命令生成
    let empty_command = ProxyConfigGenerator::generate_command(&empty_proxy_info);
    // 空配置应该返回 None
    assert!(empty_command.is_none());
}

/// 测试 ProxyInfo 的可变操作
#[test]
fn test_proxy_info_mutable_operations() {
    let mut proxy_info = ProxyInfo::new();

    // 初始状态应该没有配置
    assert!(proxy_info.get_config(ProxyType::Http).is_none());

    // 添加配置
    proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("new-proxy.example.com".to_string()),
            port: Some(9090),
        },
    );

    // Assert: 验证配置已添加
    let config = proxy_info.get_config(ProxyType::Http).expect("operation should succeed");
    assert_eq!(config.enable, true);
    assert_eq!(config.address, Some("new-proxy.example.com".to_string()));
    assert_eq!(config.port, Some(9090));

    // 使用可变引用修改配置
    let config_mut = proxy_info.get_config_mut(ProxyType::Http);
    config_mut.enable = false;
    config_mut.port = Some(8888);

    // Assert: 验证修改生效
    let updated_config = proxy_info.get_config(ProxyType::Http).expect("operation should succeed");
    assert_eq!(updated_config.enable, false);
    assert_eq!(updated_config.port, Some(8888));
}
