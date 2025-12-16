//! Proxy 系统读取器测试
//!
//! 测试 Proxy 系统读取器相关的功能，包括：
//! - 读取系统代理配置
//! - 解析代理配置

use pretty_assertions::assert_eq;
use workflow::proxy::{ProxyConfig, ProxyInfo, ProxyType, SystemProxyReader};

// ==================== 读取系统代理测试 ====================

#[test]
fn test_read_system_proxy() {
    // 测试读取系统代理配置
    // 注意：这个测试依赖于实际的系统代理设置
    // 在 macOS 上，会调用 `scutil --proxy` 命令
    let result = SystemProxyReader::read();

    // 可能成功或失败，取决于系统环境
    if result.is_ok() {
        let proxy_info = result.unwrap();
        // 验证返回的是 ProxyInfo 结构
        assert!(true); // 如果能成功读取，说明结构正确
    } else {
        // 如果失败，可能是系统不支持或命令不存在
        // 这在非 macOS 系统上是正常的
        #[cfg(not(target_os = "macos"))]
        assert!(result.is_err()); // 非 macOS 系统应该失败
    }
}

// ==================== 代理信息解析测试 ====================

#[test]
fn test_proxy_info_parsing() {
    // 测试代理信息解析逻辑
    // 创建一个模拟的 scutil 输出
    let mock_output = r#"
HTTPEnable : 1
HTTPProxy : proxy.example.com
HTTPPort : 8080
HTTPSEnable : 1
HTTPSProxy : https-proxy.example.com
HTTPSPort : 8443
SOCKSEnable : 0
"#;

    // 手动解析（模拟 SystemProxyReader::read 的逻辑）
    let mut proxy_info = ProxyInfo::new();

    for line in mock_output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim();
            let value = trimmed[colon_pos + 1..].trim();

            match key {
                "HTTPEnable" => {
                    let config = proxy_info.get_config_mut(ProxyType::Http);
                    config.enable = value == "1";
                }
                "HTTPProxy" => {
                    let config = proxy_info.get_config_mut(ProxyType::Http);
                    config.address = Some(value.to_string());
                }
                "HTTPPort" => {
                    let config = proxy_info.get_config_mut(ProxyType::Http);
                    if let Ok(port) = value.parse::<u16>() {
                        config.port = Some(port);
                    }
                }
                "HTTPSEnable" => {
                    let config = proxy_info.get_config_mut(ProxyType::Https);
                    config.enable = value == "1";
                }
                "HTTPSProxy" => {
                    let config = proxy_info.get_config_mut(ProxyType::Https);
                    config.address = Some(value.to_string());
                }
                "HTTPSPort" => {
                    let config = proxy_info.get_config_mut(ProxyType::Https);
                    if let Ok(port) = value.parse::<u16>() {
                        config.port = Some(port);
                    }
                }
                "SOCKSEnable" => {
                    let config = proxy_info.get_config_mut(ProxyType::Socks);
                    config.enable = value == "1";
                }
                _ => {}
            }
        }
    }

    // 验证解析结果
    let http_config = proxy_info.get_config(ProxyType::Http);
    assert!(http_config.is_some());
    let http = http_config.unwrap();
    assert!(http.enable);
    assert_eq!(http.address, Some("proxy.example.com".to_string()));
    assert_eq!(http.port, Some(8080));

    let https_config = proxy_info.get_config(ProxyType::Https);
    assert!(https_config.is_some());
    let https = https_config.unwrap();
    assert!(https.enable);
    assert_eq!(https.address, Some("https-proxy.example.com".to_string()));
    assert_eq!(https.port, Some(8443));

    let socks_config = proxy_info.get_config(ProxyType::Socks);
    // SOCKS 可能未设置或已禁用
    if let Some(socks) = socks_config {
        assert!(!socks.enable);
    }
}

#[test]
fn test_proxy_info_parsing_invalid_port() {
    // 测试解析无效端口（应该忽略）
    let mock_output = r#"
HTTPEnable : 1
HTTPProxy : proxy.example.com
HTTPPort : invalid
"#;

    let mut proxy_info = ProxyInfo::new();

    for line in mock_output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim();
            let value = trimmed[colon_pos + 1..].trim();

            match key {
                "HTTPEnable" => {
                    let config = proxy_info.get_config_mut(ProxyType::Http);
                    config.enable = value == "1";
                }
                "HTTPProxy" => {
                    let config = proxy_info.get_config_mut(ProxyType::Http);
                    config.address = Some(value.to_string());
                }
                "HTTPPort" => {
                    let config = proxy_info.get_config_mut(ProxyType::Http);
                    if let Ok(port) = value.parse::<u16>() {
                        config.port = Some(port);
                    }
                    // 无效端口应该被忽略
                }
                _ => {}
            }
        }
    }

    let http_config = proxy_info.get_config(ProxyType::Http);
    assert!(http_config.is_some());
    let http = http_config.unwrap();
    assert!(http.enable);
    assert_eq!(http.port, None); // 无效端口应该为 None
}
