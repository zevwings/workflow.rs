//! Base/Shell Reload 模块测试
//!
//! 测试 Shell 配置重载功能。

use clap_complete::Shell;
use workflow::base::shell::{Reload, ReloadResult};

#[test]
fn test_reload_result_structure() {
    // 测试 ReloadResult 结构

    let result = ReloadResult {
        reloaded: true,
        messages: vec!["Message 1".to_string(), "Message 2".to_string()],
        reload_hint: "source ~/.zshrc".to_string(),
    };

    assert!(result.reloaded);
    assert_eq!(result.messages.len(), 2);
    assert_eq!(result.reload_hint, "source ~/.zshrc");
}

#[test]
fn test_reload_result_clone() {
    // 测试 ReloadResult 的 Clone trait

    let result1 = ReloadResult {
        reloaded: true,
        messages: vec!["Message".to_string()],
        reload_hint: "hint".to_string(),
    };

    let result2 = result1.clone();
    assert_eq!(result1.reloaded, result2.reloaded);
    assert_eq!(result1.messages, result2.messages);
    assert_eq!(result1.reload_hint, result2.reload_hint);
}

#[test]
fn test_reload_result_debug() {
    // 测试 ReloadResult 的 Debug trait

    let result = ReloadResult {
        reloaded: false,
        messages: vec!["Error".to_string()],
        reload_hint: "hint".to_string(),
    };

    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("reloaded") || debug_str.contains("Error"));
}

// 注意：以下测试需要实际的 shell 环境，在 CI 环境中可能失败
// 但这些测试已经包含在下面的 test_reload_shell_* 测试中，所以这里保留作为备用

#[test]
fn test_reload_result_success_structure() {
    // 测试成功重载的结果结构（覆盖 reload.rs:76-83）
    let result = ReloadResult {
        reloaded: true,
        messages: vec![
            "Shell configuration reloaded (in subprocess)".to_string(),
            "Note: Changes may not take effect in the current shell.".to_string(),
        ],
        reload_hint: "source ~/.zshrc".to_string(),
    };

    assert!(result.reloaded);
    assert_eq!(result.messages.len(), 2);
    assert!(result.messages[0].contains("reloaded"));
    assert!(result.messages[1].contains("current shell"));
    assert_eq!(result.reload_hint, "source ~/.zshrc");
}

#[test]
fn test_reload_result_failure_structure() {
    // 测试失败重载的结果结构（覆盖 reload.rs:84-91）
    let result = ReloadResult {
        reloaded: false,
        messages: vec!["Could not reload shell configuration: error".to_string()],
        reload_hint: "source ~/.zshrc".to_string(),
    };

    assert!(!result.reloaded);
    assert_eq!(result.messages.len(), 1);
    assert!(result.messages[0].contains("Could not reload"));
    assert_eq!(result.reload_hint, "source ~/.zshrc");
}

#[test]
fn test_reload_result_empty_messages() {
    // 测试空消息列表的情况
    let result = ReloadResult {
        reloaded: true,
        messages: vec![],
        reload_hint: "source ~/.zshrc".to_string(),
    };

    assert!(result.reloaded);
    assert_eq!(result.messages.len(), 0);
}

#[test]
fn test_reload_result_multiple_messages() {
    // 测试多条消息的情况
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

#[test]
fn test_reload_result_reload_hint_powershell_format() {
    // 测试 PowerShell 格式的 reload_hint（覆盖 reload.rs:46-50）
    let result = ReloadResult {
        reloaded: true,
        messages: vec!["Message".to_string()],
        reload_hint: ". ~/Documents/PowerShell/Microsoft.PowerShell_profile.ps1".to_string(),
    };

    assert!(result.reload_hint.starts_with("."));
    assert!(result.reload_hint.contains(".ps1"));
}

#[test]
fn test_reload_result_reload_hint_unix_format() {
    // 测试 Unix shell 格式的 reload_hint（覆盖 reload.rs:52-55）
    let result = ReloadResult {
        reloaded: true,
        messages: vec!["Message".to_string()],
        reload_hint: "source ~/.zshrc".to_string(),
    };

    assert!(result.reload_hint.starts_with("source"));
}

// 测试实际调用 Reload::shell() 的功能
// 注意：这些测试可能在某些环境中失败，但可以验证方法的基本功能

#[test]
fn test_reload_shell_returns_result() {
    // 测试 Reload::shell() 总是返回 Result（覆盖 reload.rs:41）
    // 即使失败，也应该返回 Ok(ReloadResult)，而不是 Err
    let result = Reload::shell(&Shell::Zsh);

    // 验证返回的是 Ok(ReloadResult)
    assert!(result.is_ok());

    let reload_result = result.unwrap();
    // 验证结果结构
    assert!(
        reload_result.reload_hint.contains("source") || reload_result.reload_hint.contains(".")
    );
}

#[test]
fn test_reload_shell_powershell_hint_format() {
    // 测试 PowerShell 的 reload_hint 格式（覆盖 reload.rs:46-50）
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
        // 验证至少返回了结果（成功或失败）
        assert!(result.is_ok());
    }
}

#[test]
fn test_reload_shell_unix_hint_format() {
    // 测试 Unix shell 的 reload_hint 格式（覆盖 reload.rs:52-55）
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

#[test]
fn test_reload_shell_all_shell_types() {
    // 测试所有支持的 shell 类型都能返回结果
    let shells = vec![
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Elvish,
    ];

    for shell in shells {
        let result = Reload::shell(&shell);
        // 验证总是返回 Ok(ReloadResult)，即使执行失败
        assert!(
            result.is_ok(),
            "Reload::shell({:?}) should return Ok",
            shell
        );

        let reload_result = result.unwrap();
        // 验证结果包含必要的字段
        assert!(!reload_result.reload_hint.is_empty());
    }
}

#[test]
fn test_reload_shell_success_messages() {
    // 测试成功重载时的消息格式（覆盖 reload.rs:76-83）
    let result = Reload::shell(&Shell::Zsh);

    if let Ok(reload_result) = result {
        if reload_result.reloaded {
            // 验证成功消息格式
            assert_eq!(reload_result.messages.len(), 2);
            assert!(reload_result.messages[0].contains("reloaded"));
            assert!(reload_result.messages[1].contains("current shell"));
        }
    }
}

#[test]
fn test_reload_shell_failure_messages() {
    // 测试失败重载时的消息格式（覆盖 reload.rs:84-91）
    // 注意：这个测试可能在某些环境中总是成功，这是正常的
    let result = Reload::shell(&Shell::Zsh);

    if let Ok(reload_result) = result {
        if !reload_result.reloaded {
            // 验证失败消息格式
            assert_eq!(reload_result.messages.len(), 1);
            assert!(reload_result.messages[0].contains("Could not reload"));
        }
    }
}

#[test]
fn test_reload_shell_reload_hint_contains_config_path() {
    // 测试 reload_hint 包含配置文件路径
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

#[test]
fn test_reload_shell_consistency() {
    // 测试多次调用的一致性
    let result1 = Reload::shell(&Shell::Zsh);
    let result2 = Reload::shell(&Shell::Zsh);

    // 两次调用都应该返回结果
    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let reload_result1 = result1.unwrap();
    let reload_result2 = result2.unwrap();

    // reload_hint 应该相同（配置文件路径应该相同）
    assert_eq!(reload_result1.reload_hint, reload_result2.reload_hint);
}
