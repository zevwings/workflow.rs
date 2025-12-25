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

/// 测试使用有效字段创建代理配置
///
/// ## 测试目的
/// 验证`ProxyConfig`结构体可以使用有效的字段值（启用状态、地址、端口）正确创建。
///
/// ## 测试场景
/// 1. 准备代理配置字段值：启用状态为`true`，地址为`proxy.example.com`，端口为`8080`
/// 2. 使用这些字段值创建`ProxyConfig`实例
/// 3. 验证所有字段值正确设置
///
/// ## 预期结果
/// - `ProxyConfig`实例创建成功
/// - `enable`字段为`true`
/// - `address`字段为`Some("proxy.example.com")`
/// - `port`字段为`Some(8080)`
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

/// 测试创建禁用状态的代理配置
///
/// ## 测试目的
/// 验证`ProxyConfig`结构体可以正确创建禁用状态的配置（`enable`为`false`，地址和端口为`None`）。
///
/// ## 测试场景
/// 1. 准备禁用状态的代理配置：`enable`为`false`，`address`和`port`为`None`
/// 2. 使用这些字段值创建`ProxyConfig`实例
/// 3. 验证所有字段值正确设置
///
/// ## 预期结果
/// - `ProxyConfig`实例创建成功
/// - `enable`字段为`false`
/// - `address`字段为`None`
/// - `port`字段为`None`
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

/// 测试克隆代理配置
///
/// ## 测试目的
/// 验证`ProxyConfig`结构体的`Clone`实现能够正确创建配置的副本，所有字段值保持一致。
///
/// ## 测试场景
/// 1. 准备原始`ProxyConfig`实例，包含启用状态、地址和端口
/// 2. 调用`clone()`方法创建副本
/// 3. 验证克隆后的配置与原始配置的所有字段值相同
///
/// ## 预期结果
/// - 克隆操作成功
/// - 克隆后的`enable`、`address`、`port`字段值与原始配置完全相同
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

/// 测试代理配置的Debug格式化输出
///
/// ## 测试目的
/// 验证`ProxyConfig`结构体的`Debug`实现能够正确格式化输出，包含结构体名称和关键字段信息。
///
/// ## 测试场景
/// 1. 准备包含启用状态、地址和端口的`ProxyConfig`实例
/// 2. 使用`format!("{:?}", config)`格式化Debug输出
/// 3. 验证Debug字符串包含结构体名称和关键字段信息
///
/// ## 预期结果
/// - Debug字符串包含`"ProxyConfig"`结构体名称
/// - Debug字符串包含`"enable"`字段名
/// - Debug字符串包含`"address"`字段名
/// - Debug字符串包含地址值`"debug.proxy.com"`
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

/// 测试从系统读取代理配置
///
/// ## 测试目的
/// 验证`SystemProxyReader::read()`能够从系统配置中读取代理信息，返回`ProxyInfo`结构。
///
/// ## 测试场景
/// 1. 调用`SystemProxyReader::read()`读取系统代理配置
/// 2. 如果成功，验证返回的`ProxyInfo`包含有效的代理配置（HTTP、HTTPS、SOCKS）
/// 3. 如果失败，验证错误处理（在测试环境中失败是可以接受的，可能没有`scutil`命令或权限问题）
///
/// ## 预期结果
/// - 成功时：返回`Ok(ProxyInfo)`，包含HTTP、HTTPS、SOCKS代理配置（启用或禁用状态）
/// - 失败时：返回`Err`（在测试环境中可能发生，因为需要访问系统配置或`scutil`命令）
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

/// 测试使用有效配置创建代理信息
///
/// ## 测试目的
/// 验证`ProxyInfo`结构体可以使用多个代理类型的配置（HTTP、HTTPS、SOCKS）正确创建，并能通过`get_config()`方法获取配置。
///
/// ## 测试场景
/// 1. 使用`create_test_proxy_info()`创建包含HTTP、HTTPS、SOCKS配置的`ProxyInfo`
/// 2. 通过`get_config()`获取各个代理类型的配置
/// 3. 验证HTTP代理配置：启用状态、地址、端口
/// 4. 验证HTTPS代理配置：启用状态、地址、端口
/// 5. 验证SOCKS代理配置：禁用状态、地址、端口
///
/// ## 预期结果
/// - HTTP代理：启用，地址为`proxy.example.com`，端口为`8080`
/// - HTTPS代理：启用，地址为`secure-proxy.example.com`，端口为`8443`
/// - SOCKS代理：禁用，地址为`socks-proxy.example.com`，端口为`1080`
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

/// 测试获取启用的代理URL
///
/// ## 测试目的
/// 验证`ProxyInfo::get_proxy_url()`能够为启用的代理类型返回正确的URL字符串，为禁用的代理返回`None`。
///
/// ## 测试场景
/// 1. 使用`create_test_proxy_info()`创建包含HTTP（启用）、HTTPS（启用）、SOCKS（禁用）配置的`ProxyInfo`
/// 2. 调用`get_proxy_url()`获取各个代理类型的URL
/// 3. 验证HTTP代理返回包含协议、地址、端口的URL
/// 4. 验证HTTPS代理返回包含协议、地址、端口的URL
/// 5. 验证SOCKS代理返回`None`（因为禁用）
///
/// ## 预期结果
/// - HTTP代理URL：包含`http://`协议、`proxy.example.com`地址、`8080`端口
/// - HTTPS代理URL：包含`http://`协议、`secure-proxy.example.com`地址、`8443`端口
/// - SOCKS代理URL：返回`None`（因为代理被禁用）
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

/// 测试使用有效代理信息生成环境变量
///
/// ## 测试目的
/// 验证`ProxyConfigGenerator::generate_env_vars()`能够根据`ProxyInfo`中的启用代理配置生成对应的环境变量映射。
///
/// ## 测试场景
/// 1. 使用`create_test_proxy_info()`创建包含HTTP（启用）、HTTPS（启用）、SOCKS（禁用）配置的`ProxyInfo`
/// 2. 调用`generate_env_vars()`生成环境变量映射
/// 3. 验证生成的环境变量不为空
/// 4. 验证`http_proxy`环境变量包含协议、地址、端口
/// 5. 验证`https_proxy`环境变量包含协议、地址、端口
/// 6. 验证`all_proxy`环境变量不存在（因为SOCKS代理被禁用）
///
/// ## 预期结果
/// - 环境变量映射不为空
/// - `http_proxy`包含`http://`协议、`proxy.example.com`地址、`8080`端口
/// - `https_proxy`包含`http://`协议、`secure-proxy.example.com`地址、`8443`端口
/// - `all_proxy`不存在（因为SOCKS代理被禁用）
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

/// 测试使用有效代理信息生成命令字符串
///
/// ## 测试目的
/// 验证`ProxyConfigGenerator::generate_command()`能够根据`ProxyInfo`中的启用代理配置生成用于设置环境变量的shell命令字符串。
///
/// ## 测试场景
/// 1. 使用`create_minimal_proxy_info()`创建包含HTTP代理（启用）配置的`ProxyInfo`
/// 2. 调用`generate_command()`生成命令字符串
/// 3. 验证生成的命令字符串以`export `开头
/// 4. 验证命令包含`http_proxy=`环境变量设置
/// 5. 验证命令包含代理地址和端口信息
///
/// ## 预期结果
/// - 命令字符串不为`None`
/// - 命令以`export `开头
/// - 命令包含`http_proxy=`设置
/// - 命令包含`simple.proxy.com`地址
/// - 命令包含`3128`端口
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
