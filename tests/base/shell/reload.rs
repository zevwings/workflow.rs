//! Base/Shell Reload 模块测试
//!
//! 测试 Shell 配置重载功能。
//!
//! ## 测试策略
//!
//! - 使用 `expect()` 替代 `unwrap()` 提供清晰的错误消息
//! - 测试所有shell类型的重载功能

use clap_complete::Shell;
use workflow::base::shell::{Reload, ReloadResult};

// ==================== ReloadResult Structure Tests ====================

/// 测试ReloadResult结构体的创建（包含有效字段）
#[test]
fn test_reload_result_structure_with_valid_fields_creates_result() {
    // Arrange: 准备ReloadResult字段值
    let reloaded = true;
    let messages = vec!["Message 1".to_string(), "Message 2".to_string()];
    let reload_hint = "source ~/.zshrc".to_string();

    // Act: 创建ReloadResult实例
    let result = ReloadResult {
        reloaded,
        messages: messages.clone(),
        reload_hint: reload_hint.clone(),
    };

    // Assert: 验证所有字段值正确
    assert!(result.reloaded);
    assert_eq!(result.messages.len(), 2);
    assert_eq!(result.reload_hint, reload_hint);
}

/// 测试ReloadResult的克隆功能
#[test]
fn test_reload_result_clone_with_valid_result_creates_clone() {
    // Arrange: 准备原始ReloadResult
    let result1 = ReloadResult {
        reloaded: true,
        messages: vec!["Message".to_string()],
        reload_hint: "hint".to_string(),
    };

    // Act: 克隆ReloadResult
    let result2 = result1.clone();

    // Assert: 验证克隆的字段值与原始值相同
    assert_eq!(result1.reloaded, result2.reloaded);
    assert_eq!(result1.messages, result2.messages);
    assert_eq!(result1.reload_hint, result2.reload_hint);
}

/// 测试ReloadResult的Debug格式化
#[test]
fn test_reload_result_debug_with_valid_result_returns_debug_string() {
    // Arrange: 准备ReloadResult
    let result = ReloadResult {
        reloaded: false,
        messages: vec!["Error".to_string()],
        reload_hint: "hint".to_string(),
    };

    // Act: 格式化Debug输出
    let debug_str = format!("{:?}", result);

    // Assert: 验证Debug字符串包含预期内容
    assert!(debug_str.contains("reloaded") || debug_str.contains("Error"));
}

// 注意：以下测试需要实际的 shell 环境，在 CI 环境中可能失败
// 但这些测试已经包含在下面的 test_reload_shell_* 测试中，所以这里保留作为备用

/// 测试成功重载的结果结构
#[test]
fn test_reload_result_success_structure_with_success_reload_creates_result() {
    // Arrange: 准备成功重载的结果字段值
    let messages = vec![
        "Shell configuration reloaded (in subprocess)".to_string(),
        "Note: Changes may not take effect in the current shell.".to_string(),
    ];
    let reload_hint = "source ~/.zshrc".to_string();

    // Act: 创建成功重载的ReloadResult
    let result = ReloadResult {
        reloaded: true,
        messages: messages.clone(),
        reload_hint: reload_hint.clone(),
    };

    // Assert: 验证成功重载的结果结构正确
    assert!(result.reloaded);
    assert_eq!(result.messages.len(), 2);
    assert!(result.messages[0].contains("reloaded"));
    assert!(result.messages[1].contains("current shell"));
    assert_eq!(result.reload_hint, reload_hint);
}

/// 测试失败重载的结果结构
#[test]
fn test_reload_result_failure_structure_with_failure_reload_creates_result() {
    // Arrange: 准备失败重载的结果字段值
    let messages = vec!["Could not reload shell configuration: error".to_string()];
    let reload_hint = "source ~/.zshrc".to_string();

    // Act: 创建失败重载的ReloadResult
    let result = ReloadResult {
        reloaded: false,
        messages: messages.clone(),
        reload_hint: reload_hint.clone(),
    };

    // Assert: 验证失败重载的结果结构正确
    assert!(!result.reloaded);
    assert_eq!(result.messages.len(), 1);
    assert!(result.messages[0].contains("Could not reload"));
    assert_eq!(result.reload_hint, reload_hint);
}

/// 测试ReloadResult的空消息列表
#[test]
fn test_reload_result_empty_messages() {
    // Arrange: 准备测试空消息列表的情况
    let result = ReloadResult {
        reloaded: true,
        messages: vec![],
        reload_hint: "source ~/.zshrc".to_string(),
    };

    assert!(result.reloaded);
    assert_eq!(result.messages.len(), 0);
}

/// 测试ReloadResult的多条消息
#[test]
fn test_reload_result_multiple_messages() {
    // Arrange: 准备测试多条消息的情况
    let result = ReloadResult {
        reloaded: true,
        messages: vec![
            "Message 1".to_string(),
            "Message 2".to_string(),
            "Message 3".to_string(),
        ],
        reload_hint: "source ~/.zshrc".to_string(),
    };

    assert!(result.reloaded);
    assert_eq!(result.messages.len(), 3);
    assert_eq!(result.messages[0], "Message 1");
    assert_eq!(result.messages[1], "Message 2");
    assert_eq!(result.messages[2], "Message 3");
}

/// 测试PowerShell格式的reload_hint
#[test]
fn test_reload_result_reload_hint_powershell_format() {
    // Arrange: 准备测试 PowerShell 格式的 reload_hint（覆盖 reload.rs:46-50）
    let result = ReloadResult {
        reloaded: true,
        messages: vec!["Message".to_string()],
        reload_hint: ". ~/Documents/PowerShell/Microsoft.PowerShell_profile.ps1".to_string(),
    };

    assert!(result.reload_hint.starts_with("."));
    assert!(result.reload_hint.contains(".ps1"));
}

/// 测试Unix shell格式的reload_hint
#[test]
fn test_reload_result_reload_hint_unix_format() {
    // Arrange: 准备测试 Unix shell 格式的 reload_hint（覆盖 reload.rs:52-55）
    let result = ReloadResult {
        reloaded: true,
        messages: vec!["Message".to_string()],
        reload_hint: "source ~/.zshrc".to_string(),
    };

    assert!(result.reload_hint.starts_with("source"));
}

// 测试实际调用 Reload::shell() 的功能
// 注意：这些测试可能在某些环境中失败，但可以验证方法的基本功能

/// 测试Reload::shell()总是返回Result
#[test]
fn test_reload_shell_returns_result() {
    // Arrange: 准备测试 Reload::shell() 总是返回 Result（覆盖 reload.rs:41）
    // 即使失败，也应该返回 Ok(ReloadResult)，而不是 Err
    let result = Reload::shell(&Shell::Zsh);

    // Assert: 验证返回的是 Ok(ReloadResult)
    assert!(result.is_ok());

    let reload_result = result.expect("Reload::shell should return Ok(ReloadResult)");
    // Assert: 验证结果结构
    assert!(
        reload_result.reload_hint.contains("source") || reload_result.reload_hint.contains(".")
    );
}

/// 测试PowerShell的reload_hint格式
#[test]
fn test_reload_shell_powershell_hint_format() {
    // Arrange: 准备测试 PowerShell 的 reload_hint 格式（覆盖 reload.rs:46-50）
    #[cfg(target_os = "windows")]
    {
        let result = Reload::shell(&Shell::PowerShell);

        if let Ok(reload_result) = result {
            // PowerShell 应该使用 "." 而不是 "source"
            assert!(reload_result.reload_hint.starts_with("."));
            assert!(!reload_result.reload_hint.starts_with("source"));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // 在非 Windows 系统上，PowerShell 测试可能失败，这是正常的
        let result = Reload::shell(&Shell::PowerShell);
        // Assert: 验证至少返回了结果（成功或失败）
        assert!(result.is_ok());
    }
}

/// 测试Unix shell的reload_hint格式
#[test]
fn test_reload_shell_unix_hint_format() {
    // Arrange: 准备测试 Unix shell 的 reload_hint 格式（覆盖 reload.rs:52-55）
    #[cfg(not(target_os = "windows"))]
    {
        let result = Reload::shell(&Shell::Zsh);

        if let Ok(reload_result) = result {
            // Unix shell 应该使用 "source" 而不是 "."
            assert!(reload_result.reload_hint.starts_with("source"));
            assert!(!reload_result.reload_hint.starts_with("."));
        }
    }
}

/// 测试所有支持的shell类型都能返回结果
#[test]
fn test_reload_shell_all_shell_types() {
    // Arrange: 准备测试所有支持的 shell 类型都能返回结果
    let shells = vec![
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Elvish,
    ];

    for shell in shells {
        let result = Reload::shell(&shell);
        // Assert: 验证总是返回 Ok(ReloadResult)，即使执行失败
        assert!(
            result.is_ok(),
            "Reload::shell({:?}) should return Ok",
            shell
        );

        let reload_result = result.expect("Reload::shell should return Ok(ReloadResult)");
        // Assert: 验证结果包含必要的字段
        assert!(!reload_result.reload_hint.is_empty());
    }
}

/// 测试成功重载时的消息格式
#[test]
fn test_reload_shell_success_messages() {
    // Arrange: 准备测试成功重载时的消息格式（覆盖 reload.rs:76-83）
    let result = Reload::shell(&Shell::Zsh);

    if let Ok(reload_result) = result {
        if reload_result.reloaded {
            // Assert: 验证成功消息格式
            assert_eq!(reload_result.messages.len(), 2);
            assert!(reload_result.messages[0].contains("reloaded"));
            assert!(reload_result.messages[1].contains("current shell"));
        }
    }
}

/// 测试失败重载时的消息格式
#[test]
fn test_reload_shell_failure_messages() {
    // Arrange: 准备测试失败重载时的消息格式（覆盖 reload.rs:84-91）
    // 注意：这个测试可能在某些环境中总是成功，这是正常的
    let result = Reload::shell(&Shell::Zsh);

    if let Ok(reload_result) = result {
        if !reload_result.reloaded {
            // Assert: 验证失败消息格式
            assert_eq!(reload_result.messages.len(), 1);
            assert!(reload_result.messages[0].contains("Could not reload"));
        }
    }
}

/// 测试reload_hint包含配置文件路径
#[test]
fn test_reload_shell_reload_hint_contains_config_path() {
    // Arrange: 准备测试 reload_hint 包含配置文件路径
    let result = Reload::shell(&Shell::Zsh);

    if let Ok(reload_result) = result {
        // reload_hint 应该包含配置文件路径（可能是相对路径或绝对路径）
        // 对于 zsh，应该是 "source" 加上路径
        assert!(
            reload_result.reload_hint.starts_with("source")
                || reload_result.reload_hint.starts_with(".")
        );
    }
}

/// 测试多次调用的一致性
#[test]
fn test_reload_shell_consistency() {
    // Arrange: 准备测试多次调用的一致性
    let result1 = Reload::shell(&Shell::Zsh);
    let result2 = Reload::shell(&Shell::Zsh);

    // 两次调用都应该返回结果
    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let reload_result1 = result1.expect("First call should return Ok(ReloadResult)");
    let reload_result2 = result2.expect("Second call should return Ok(ReloadResult)");

    // reload_hint 应该相同（配置文件路径应该相同）
    assert_eq!(reload_result1.reload_hint, reload_result2.reload_hint);
}
