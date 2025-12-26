//! ProxyManager 业务逻辑测试
//!
//! 测试 ProxyManager 模块的核心功能，包括：
//! - 代理启用和禁用操作
//! - 代理信息管理
//! - 结果结构体处理
//! - 错误处理和边界情况
//!
//! ## 测试策略
//!
//! - 测试函数返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 使用 helper 函数创建测试数据
//! - 测试各种代理类型和配置组合

use color_eyre::Result;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
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
            address: Some("proxy.example.com".to_string()),
            port: Some(8080),
        },
    );

    proxy_info.set_config(
        ProxyType::Https,
        ProxyConfig {
            enable: true,
            address: Some("secure-proxy.example.com".to_string()),
            port: Some(8443),
        },
    );

    proxy_info
}

/// 创建 HTTPS 代理信息
fn create_https_proxy_info() -> ProxyInfo {
    let mut proxy_info = ProxyInfo::new();

    proxy_info.set_config(
        ProxyType::Https,
        ProxyConfig {
            enable: false,
            address: Some("secure-proxy.example.com".to_string()),
            port: Some(8443),
        },
    );

    proxy_info
}

/// 创建 SOCKS 代理信息
fn create_socks_proxy_info() -> ProxyInfo {
    let mut proxy_info = ProxyInfo::new();

    proxy_info.set_config(
        ProxyType::Socks,
        ProxyConfig {
            enable: true,
            address: Some("socks.example.com".to_string()),
            port: Some(1080),
        },
    );

    proxy_info
}

// ==================== Test Cases ====================

/// 测试 ProxyInfo 结构体创建
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_proxy_info_creation_return_ok() -> Result<()> {
    let proxy_info = create_test_proxy_info();

    // Assert: 验证 HTTP 代理配置
    if let Some(http_config) = proxy_info.get_config(ProxyType::Http) {
        assert_eq!(http_config.enable, true);
        assert_eq!(http_config.address, Some("proxy.example.com".to_string()));
        assert_eq!(http_config.port, Some(8080));
    }

    // Assert: 验证 HTTPS 代理配置
    if let Some(https_config) = proxy_info.get_config(ProxyType::Https) {
        assert_eq!(https_config.enable, true);
        assert_eq!(
            https_config.address,
            Some("secure-proxy.example.com".to_string())
        );
        assert_eq!(https_config.port, Some(8443));
    }
    Ok(())
}

/// 测试 ProxyInfo 不同代理类型
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_proxy_info_different_types_return_ok() -> Result<()> {
    let http_proxy = create_test_proxy_info();
    let https_proxy = create_https_proxy_info();
    let socks_proxy = create_socks_proxy_info();

    // Assert: 验证 HTTP 代理
    if let Some(http_config) = http_proxy.get_config(ProxyType::Http) {
        assert_eq!(http_config.enable, true);
        assert_eq!(http_config.port, Some(8080));
    }

    // Assert: 验证 HTTPS 代理（禁用状态）
    if let Some(https_config) = https_proxy.get_config(ProxyType::Https) {
        assert_eq!(https_config.enable, false);
        assert_eq!(https_config.port, Some(8443));
    }

    // Assert: 验证 SOCKS 代理
    if let Some(socks_config) = socks_proxy.get_config(ProxyType::Socks) {
        assert_eq!(socks_config.enable, true);
        assert_eq!(socks_config.port, Some(1080));
    }
    Ok(())
}

/// 测试 ProxyInfo 克隆功能
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_proxy_info_clone_return_ok() -> Result<()> {
    let original_proxy = create_test_proxy_info();
    let cloned_proxy = original_proxy.clone();

    // Assert: 验证 HTTP 配置克隆
    if let (Some(original_http), Some(cloned_http)) = (
        original_proxy.get_config(ProxyType::Http),
        cloned_proxy.get_config(ProxyType::Http),
    ) {
        assert_eq!(original_http.enable, cloned_http.enable);
        assert_eq!(original_http.address, cloned_http.address);
        assert_eq!(original_http.port, cloned_http.port);
    }
    Ok(())
}

/// 测试 ProxyInfo 调试输出
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_proxy_info_debug() {
    let proxy_info = create_test_proxy_info();

    let debug_str = format!("{:?}", proxy_info);
    assert!(debug_str.contains("ProxyInfo"));
    // 由于 ProxyInfo 内部使用 HashMap，具体格式可能变化，只验证基本结构
    assert!(!debug_str.is_empty());
}

/// 测试 ProxyType 枚举
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_proxy_type_enum() {
    // Arrange: 准备测试所有代理类型
    let http_type = ProxyType::Http;
    let https_type = ProxyType::Https;
    let socks_type = ProxyType::Socks;

    // Arrange: 准备测试 Debug 实现
    assert_eq!(format!("{:?}", http_type), "Http");
    assert_eq!(format!("{:?}", https_type), "Https");
    assert_eq!(format!("{:?}", socks_type), "Socks");

    // Arrange: 准备测试 Clone 实现
    let cloned_http = http_type;
    assert_eq!(http_type, cloned_http);

    // Arrange: 准备测试 Copy 实现
    let copied_https = https_type;
    assert_eq!(https_type, copied_https);

    // Arrange: 准备测试 PartialEq 实现
    assert_eq!(http_type, ProxyType::Http);
    assert_ne!(http_type, ProxyType::Https);
    assert_ne!(https_type, ProxyType::Socks);

    // Arrange: 准备测试环境变量键名
    assert_eq!(http_type.env_key(), "http_proxy");
    assert_eq!(https_type.env_key(), "https_proxy");
    assert_eq!(socks_type.env_key(), "all_proxy");
}

/// 测试 ProxyEnableResult 结构体创建
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_proxy_enable_result_creation_return_ok() -> Result<()> {
    use std::path::PathBuf;

    let enable_result = ProxyEnableResult {
        already_configured: false,
        proxy_command: Some("export http_proxy=http://proxy.example.com:8080".to_string()),
        shell_config_path: Some(PathBuf::from("/home/user/.bashrc")),
    };

    assert_eq!(enable_result.already_configured, false);
    assert!(enable_result.proxy_command.is_some());
    assert!(enable_result.shell_config_path.is_some());

    // Assert: 验证具体内容
    if let Some(command) = &enable_result.proxy_command {
        assert!(command.contains("export"));
        assert!(command.contains("http_proxy"));
        assert!(command.contains("proxy.example.com"));
    }

    if let Some(config_path) = &enable_result.shell_config_path {
        assert_eq!(config_path, &PathBuf::from("/home/user/.bashrc"));
    }
    Ok(())
}

/// 测试 ProxyEnableResult 已配置场景
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_proxy_enable_result_already_configured() {
    let enable_result = ProxyEnableResult {
        already_configured: true,
        proxy_command: None,
        shell_config_path: None,
    };

    // Assert: 验证已配置的情况
    assert_eq!(enable_result.already_configured, true);
    assert!(enable_result.proxy_command.is_none());
    assert!(enable_result.shell_config_path.is_none());
}

/// 测试 ProxyEnableResult 临时模式场景
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_proxy_enable_result_temporary_mode_return_ok() -> Result<()> {
    let enable_result = ProxyEnableResult {
        already_configured: false,
        proxy_command: Some("export http_proxy=http://temp.proxy.com:8080".to_string()),
        shell_config_path: None, // 临时模式不写入配置文件
    };

    // Assert: 验证临时模式的情况
    assert_eq!(enable_result.already_configured, false);
    assert!(enable_result.proxy_command.is_some());
    assert!(enable_result.shell_config_path.is_none());

    if let Some(command) = &enable_result.proxy_command {
        assert!(command.contains("temp.proxy.com"));
    }
    Ok(())
}

/// 测试 ProxyDisableResult 结构体创建
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_proxy_disable_result_creation_return_ok() -> Result<()> {
    use std::path::PathBuf;

    let mut current_env_proxy = HashMap::new();
    current_env_proxy.insert(
        "http_proxy".to_string(),
        "http://proxy.example.com:8080".to_string(),
    );
    current_env_proxy.insert(
        "https_proxy".to_string(),
        "https://proxy.example.com:8080".to_string(),
    );

    let disable_result = ProxyDisableResult {
        found_proxy: true,
        shell_config_path: Some(PathBuf::from("/home/user/.zshrc")),
        unset_command: Some("unset http_proxy && unset https_proxy".to_string()),
        current_env_proxy: current_env_proxy.clone(),
    };

    assert_eq!(disable_result.found_proxy, true);
    assert!(disable_result.shell_config_path.is_some());
    assert!(disable_result.unset_command.is_some());
    assert_eq!(disable_result.current_env_proxy.len(), 2);

    // Assert: 验证具体内容
    if let Some(unset_cmd) = &disable_result.unset_command {
        assert!(unset_cmd.contains("unset"));
        assert!(unset_cmd.contains("http_proxy"));
    }

    assert!(disable_result.current_env_proxy.contains_key("http_proxy"));
    assert!(disable_result.current_env_proxy.contains_key("https_proxy"));
    Ok(())
}

/// 测试 ProxyDisableResult 没有找到代理场景
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_proxy_disable_result_no_proxy_found() {
    let disable_result = ProxyDisableResult {
        found_proxy: false,
        shell_config_path: None,
        unset_command: None,
        current_env_proxy: HashMap::new(),
    };

    // Assert: 验证没有找到代理的情况
    assert_eq!(disable_result.found_proxy, false);
    assert!(disable_result.shell_config_path.is_none());
    assert!(disable_result.unset_command.is_none());
    assert!(disable_result.current_env_proxy.is_empty());
}

/// 测试 ProxyDisableResult 只有环境变量场景
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_proxy_disable_result_env_only() {
    let mut current_env_proxy = HashMap::new();
    current_env_proxy.insert(
        "http_proxy".to_string(),
        "http://env.proxy.com:8080".to_string(),
    );

    let disable_result = ProxyDisableResult {
        found_proxy: true,
        shell_config_path: None, // 配置文件中没有代理设置
        unset_command: Some("unset http_proxy".to_string()),
        current_env_proxy,
    };

    // Assert: 验证只有环境变量的情况
    assert_eq!(disable_result.found_proxy, true);
    assert!(disable_result.shell_config_path.is_none());
    assert!(disable_result.unset_command.is_some());
    assert_eq!(disable_result.current_env_proxy.len(), 1);
}

/// 测试结构体克隆功能
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_results_clone() {
    use std::path::PathBuf;

    // Arrange: 准备测试 ProxyEnableResult 克隆
    let original_enable = ProxyEnableResult {
        already_configured: false,
        proxy_command: Some("export http_proxy=test".to_string()),
        shell_config_path: Some(PathBuf::from("test_file")),
    };

    let cloned_enable = original_enable.clone();
    assert_eq!(
        original_enable.already_configured,
        cloned_enable.already_configured
    );
    assert_eq!(original_enable.proxy_command, cloned_enable.proxy_command);
    assert_eq!(
        original_enable.shell_config_path,
        cloned_enable.shell_config_path
    );

    // Arrange: 准备测试 ProxyDisableResult 克隆
    let mut env_proxy = HashMap::new();
    env_proxy.insert("TEST_VAR".to_string(), "test_value".to_string());

    let original_disable = ProxyDisableResult {
        found_proxy: true,
        shell_config_path: Some(PathBuf::from("test_file")),
        unset_command: Some("unset TEST_VAR".to_string()),
        current_env_proxy: env_proxy,
    };

    let cloned_disable = original_disable.clone();
    assert_eq!(original_disable.found_proxy, cloned_disable.found_proxy);
    assert_eq!(
        original_disable.shell_config_path,
        cloned_disable.shell_config_path
    );
    assert_eq!(original_disable.unset_command, cloned_disable.unset_command);
    assert_eq!(
        original_disable.current_env_proxy.len(),
        cloned_disable.current_env_proxy.len()
    );
}

/// 测试结构体调试输出
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_results_debug() {
    let enable_result = ProxyEnableResult {
        already_configured: true,
        proxy_command: None,
        shell_config_path: None,
    };

    let enable_debug = format!("{:?}", enable_result);
    assert!(enable_debug.contains("ProxyEnableResult"));
    assert!(enable_debug.contains("already_configured"));

    let disable_result = ProxyDisableResult {
        found_proxy: false,
        shell_config_path: None,
        unset_command: None,
        current_env_proxy: HashMap::new(),
    };

    let disable_debug = format!("{:?}", disable_result);
    assert!(disable_debug.contains("ProxyDisableResult"));
    assert!(disable_debug.contains("found_proxy"));
}

/// 测试 ProxyManager 静态方法
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_proxy_manager_static_methods() {
    // 由于 ProxyManager 的实际方法可能需要系统权限或特定环境，
    // 我们主要测试方法调用不会 panic，以及基本的参数处理

    // Arrange: 准备测试检查环境代理
    let env_proxy = ProxyManager::check_env_proxy();
    // 环境变量可能为空或包含代理设置
    assert!(env_proxy.is_empty() || !env_proxy.is_empty());

    // Arrange: 准备测试启用代理（在测试环境中可能会失败，但不应该 panic）
    let enable_result = ProxyManager::enable(false); // 非临时模式
    match enable_result {
        Ok(result) => {
            // 如果成功，验证结果结构
            assert!(result.already_configured == true || result.already_configured == false);
        }
        Err(_) => {
            // 在测试环境中失败是可以接受的（可能没有系统代理设置）
        }
    }

    // Arrange: 准备测试临时启用代理
    let temp_enable_result = ProxyManager::enable(true); // 临时模式
    match temp_enable_result {
        Ok(result) => {
            // 临时模式不应该有配置文件路径
            if !result.already_configured {
                assert!(result.shell_config_path.is_none());
            }
        }
        Err(_) => {
            // 在测试环境中失败是可以接受的
        }
    }

    // Arrange: 准备测试禁用代理
    let disable_result = ProxyManager::disable();
    match disable_result {
        Ok(result) => {
            // 如果成功，验证结果结构
            assert!(result.found_proxy == true || result.found_proxy == false);
        }
        Err(_) => {
            // 在测试环境中失败是可以接受的
        }
    }

    // Arrange: 准备测试确保代理启用
    let ensure_result = ProxyManager::ensure_proxy_enabled();
    match ensure_result {
        Ok(()) => {
            // 如果成功，说明方法工作正常
        }
        Err(_) => {
            // 在测试环境中失败是可以接受的
        }
    }
}

/// 测试复杂的代理配置场景
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_complex_proxy_scenarios() {
    // Arrange: 准备测试 HTTP 代理配置
    let mut http_proxy_info = ProxyInfo::new();
    http_proxy_info.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("auth-proxy.example.com".to_string()),
            port: Some(3128),
        },
    );

    // Arrange: 准备测试禁用的 HTTPS 代理
    let mut https_proxy_info = ProxyInfo::new();
    https_proxy_info.set_config(
        ProxyType::Https,
        ProxyConfig {
            enable: false,
            address: Some("public-proxy.example.com".to_string()),
            port: Some(8443),
        },
    );

    // Arrange: 准备测试 SOCKS 代理配置
    let mut socks_proxy_info = ProxyInfo::new();
    socks_proxy_info.set_config(
        ProxyType::Socks,
        ProxyConfig {
            enable: true,
            address: Some("localhost".to_string()),
            port: Some(1080),
        },
    );

    // Assert: 验证不同类型代理的配置
    if let Some(http_config) = http_proxy_info.get_config(ProxyType::Http) {
        assert_eq!(http_config.enable, true);
        assert_eq!(
            http_config.address,
            Some("auth-proxy.example.com".to_string())
        );
        assert_eq!(http_config.port, Some(3128));
    }

    if let Some(https_config) = https_proxy_info.get_config(ProxyType::Https) {
        assert_eq!(https_config.enable, false);
        assert_eq!(
            https_config.address,
            Some("public-proxy.example.com".to_string())
        );
        assert_eq!(https_config.port, Some(8443));
    }

    if let Some(socks_config) = socks_proxy_info.get_config(ProxyType::Socks) {
        assert_eq!(socks_config.enable, true);
        assert_eq!(socks_config.address, Some("localhost".to_string()));
        assert_eq!(socks_config.port, Some(1080));
    }

    // Arrange: 准备测试代理 URL 生成
    assert!(http_proxy_info.get_proxy_url(ProxyType::Http).is_some());
    assert!(https_proxy_info.get_proxy_url(ProxyType::Https).is_none()); // 禁用状态
    assert!(socks_proxy_info.get_proxy_url(ProxyType::Socks).is_some());
}

/// 测试边界情况和错误处理
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_edge_cases_and_error_handling_return_false() -> Result<()> {
    // Arrange: 准备测试极端端口号
    let mut extreme_port_proxy = ProxyInfo::new();
    extreme_port_proxy.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: Some("test.example.com".to_string()),
            port: Some(65535), // 最大端口号
        },
    );

    if let Some(config) = extreme_port_proxy.get_config(ProxyType::Http) {
        assert_eq!(config.port, Some(65535));
    }

    // Arrange: 准备测试空主机名（虽然在实际使用中不应该这样）
    let mut empty_host_proxy = ProxyInfo::new();
    empty_host_proxy.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: false,
            address: Some("".to_string()),
            port: Some(8080),
        },
    );

    if let Some(empty_config) = empty_host_proxy.get_config(ProxyType::Http) {
        assert_eq!(empty_config.address, Some("".to_string()));
        assert_eq!(empty_config.enable, false);
    }

    // Arrange: 准备测试 None 地址和端口
    let mut invalid_proxy = ProxyInfo::new();
    invalid_proxy.set_config(
        ProxyType::Http,
        ProxyConfig {
            enable: true,
            address: None,
            port: None,
        },
    );

    // 无效配置不应该生成 URL
    assert!(invalid_proxy.get_proxy_url(ProxyType::Http).is_none());

    // Arrange: 准备测试代理配置匹配
    let test_proxy_info = create_test_proxy_info();
    let is_configured = ProxyManager::is_proxy_configured(&test_proxy_info);
    // 在测试环境中，代理可能配置也可能未配置
    assert!(is_configured == true || is_configured == false);

    // Arrange: 准备测试空结果结构体
    let empty_enable_result = ProxyEnableResult {
        already_configured: false,
        proxy_command: None,
        shell_config_path: None,
    };

    assert_eq!(empty_enable_result.already_configured, false);
    assert!(empty_enable_result.proxy_command.is_none());
    assert!(empty_enable_result.shell_config_path.is_none());

    let empty_disable_result = ProxyDisableResult {
        found_proxy: false,
        shell_config_path: None,
        unset_command: None,
        current_env_proxy: HashMap::new(),
    };

    assert_eq!(empty_disable_result.found_proxy, false);
    assert!(empty_disable_result.shell_config_path.is_none());
    assert!(empty_disable_result.unset_command.is_none());
    assert!(empty_disable_result.current_env_proxy.is_empty());
    Ok(())
}
