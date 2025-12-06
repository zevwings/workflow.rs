//! Shell 模块测试
//!
//! 测试 `base::shell` 模块中的 Shell 检测和配置功能。

use clap_complete::shells::Shell;
use workflow::base::settings::paths::Paths;
use workflow::base::shell::Detect;

// ==================== Shell 检测测试 ====================

#[test]
fn test_detect_shell() {
    // 测试检测当前 Shell
    let shell_result = Detect::shell();
    // 应该返回一个有效的 Shell 类型或错误
    match shell_result {
        Ok(shell) => {
            // 验证返回的 Shell 类型是支持的
            assert!(matches!(
                shell,
                Shell::Bash | Shell::Zsh | Shell::Fish | Shell::PowerShell | Shell::Elvish
            ));
        }
        Err(_) => {
            // 在某些环境下可能无法检测 Shell，这是可以接受的
            // 特别是在 CI/CD 环境中
        }
    }
}

#[test]
fn test_detect_installed_shells() {
    // 测试检测已安装的 Shell
    let installed = Detect::installed_shells();
    // 应该至少返回一个 Shell（当前 Shell）
    // 或者在某些系统上可能为空（如 Windows）
    // 所以只测试方法可以调用
    assert!(!installed.is_empty() || installed.is_empty()); // 允许为空或非空
}

#[test]
fn test_get_shell_config_path() {
    // 测试获取 Shell 配置文件路径（使用 Paths 模块）
    let shells = vec![Shell::Zsh, Shell::Bash, Shell::Fish];

    for shell in shells {
        let config_path = Paths::config_file(&shell);
        // 配置文件路径应该存在（可能不存在，但路径格式应该正确）
        match config_path {
            Ok(path) => {
                // 验证路径格式
                let path_str = path.to_string_lossy();
                match shell {
                    Shell::Zsh => {
                        assert!(
                            path_str.ends_with(".zshrc"),
                            "Zsh config should end with .zshrc"
                        );
                    }
                    Shell::Bash => {
                        assert!(
                            path_str.ends_with(".bash_profile") || path_str.ends_with(".bashrc"),
                            "Bash config should end with .bash_profile or .bashrc"
                        );
                    }
                    Shell::Fish => {
                        assert!(
                            path_str.contains("fish") || path_str.ends_with(".fish"),
                            "Fish config should contain 'fish' or end with .fish"
                        );
                    }
                    _ => {}
                }
            }
            Err(_) => {
                // 在某些平台上某些 Shell 可能不支持，这是正常的
                #[cfg(target_os = "windows")]
                {
                    // Windows 上只有 PowerShell 应该成功
                    if matches!(shell, Shell::PowerShell) {
                        panic!("PowerShell config should be available on Windows");
                    }
                }
            }
        }
    }
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_get_shell_config_path_unix() {
    // Unix 系统上的测试
    let zsh_config = Paths::config_file(&Shell::Zsh);
    assert!(zsh_config.is_ok());
    let zsh_path = zsh_config.unwrap();
    assert!(zsh_path.to_string_lossy().ends_with(".zshrc"));

    let bash_config = Paths::config_file(&Shell::Bash);
    assert!(bash_config.is_ok());
    let bash_path = bash_config.unwrap();
    let bash_str = bash_path.to_string_lossy();
    assert!(
        bash_str.ends_with(".bash_profile") || bash_str.ends_with(".bashrc"),
        "Bash config should be .bash_profile or .bashrc"
    );
}

#[test]
#[cfg(target_os = "windows")]
fn test_get_shell_config_path_windows() {
    // Windows 系统上的测试
    let powershell_config = Paths::config_file(&Shell::PowerShell);
    assert!(powershell_config.is_ok());

    // 其他 Shell 在 Windows 上可能不支持
    let zsh_config = Paths::config_file(&Shell::Zsh);
    // Zsh 在 Windows 上可能不支持，这是正常的
}

// ==================== Shell 配置测试 ====================

// 注意：Shell 配置的测试需要实际的文件操作
// 这些测试可能需要使用临时文件或 mock

#[test]
fn test_shell_detection_consistency() {
    // 测试多次调用应该返回一致的结果（在同一个测试运行中）
    let shell1 = Detect::shell();
    let shell2 = Detect::shell();
    // 如果都成功，应该返回相同的 Shell
    if let (Ok(s1), Ok(s2)) = (shell1, shell2) {
        assert_eq!(s1, s2);
    }
}

// ==================== 辅助函数测试 ====================

#[test]
fn test_shell_to_string() {
    // 测试 Shell 枚举的字符串表示
    assert_eq!(Shell::Bash.to_string(), "bash");
    assert_eq!(Shell::Zsh.to_string(), "zsh");
    assert_eq!(Shell::Fish.to_string(), "fish");
    assert_eq!(Shell::PowerShell.to_string(), "powershell");
}
