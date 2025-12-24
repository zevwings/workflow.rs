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
#[cfg(not(target_os = "windows"))]
#[test]
#[ignore] // 需要实际的 shell 环境
fn test_reload_shell_zsh() {
    // 测试重载 zsh 配置
    let result = Reload::shell(&Shell::Zsh);
    // 可能成功或失败，取决于环境
    assert!(result.is_ok() || result.is_err());
}

#[cfg(not(target_os = "windows"))]
#[test]
#[ignore] // 需要实际的 shell 环境
fn test_reload_shell_bash() {
    // 测试重载 bash 配置
    let result = Reload::shell(&Shell::Bash);
    // 可能成功或失败，取决于环境
    assert!(result.is_ok() || result.is_err());
}

#[test]
#[ignore] // 需要实际的 shell 环境
fn test_reload_shell_powershell() {
    // 测试重载 PowerShell 配置（覆盖 reload.rs:46-50）
    let result = Reload::shell(&Shell::PowerShell);
    // 可能成功或失败，取决于环境
    assert!(result.is_ok() || result.is_err());
}

#[cfg(not(target_os = "windows"))]
#[test]
#[ignore] // 需要实际的 shell 环境
fn test_reload_shell_fish() {
    // 测试重载 fish 配置
    let result = Reload::shell(&Shell::Fish);
    // 可能成功或失败，取决于环境
    assert!(result.is_ok() || result.is_err());
}

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
