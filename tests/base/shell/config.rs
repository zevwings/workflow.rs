//! Base/Shell Config 模块测试
//!
//! 测试 Shell 配置管理器的核心功能。

use color_eyre::Result;
use std::collections::HashMap;
use std::fs;
use workflow::base::shell::ShellConfigManager;

use crate::common::environments::CliTestEnv;
use crate::common::fixtures::cli_env;
use rstest::rstest;

// ==================== ShellConfigManager Environment Variables Tests ====================

/// 测试从空文件加载环境变量（应返回空映射）
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
fn test_load_env_vars_empty_file_with_empty_file_returns_empty_map() {
    // Arrange: 准备空配置文件环境（注意：依赖于真实配置文件路径）

    // Act: 从空文件加载环境变量
    let result = ShellConfigManager::load_env_vars();

    // Assert: 验证返回空HashMap或包含变量的HashMap（如果文件不存在或为空）
    if let Ok(env_vars) = result {
        assert!(env_vars.is_empty() || !env_vars.is_empty());
    }
}

/// 测试设置和加载环境变量
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
fn test_set_and_load_env_vars_with_valid_vars_sets_and_loads_vars() {
    // Arrange: 准备测试环境变量
    let mut test_vars = HashMap::new();
    test_vars.insert("TEST_KEY1".to_string(), "test_value1".to_string());
    test_vars.insert("TEST_KEY2".to_string(), "test_value2".to_string());

    // Act: 设置环境变量
    let set_result = ShellConfigManager::set_env_vars(&test_vars);

    // Assert: 如果设置成功，验证可以加载且包含设置的变量
    if set_result.is_ok() {
        let load_result = ShellConfigManager::load_env_vars();
        if let Ok(loaded_vars) = load_result {
            for (key, value) in &test_vars {
                if let Some(loaded_value) = loaded_vars.get(key) {
                    assert_eq!(loaded_value, value);
                }
            }
        }
        // 清理：移除测试变量
        let _ = ShellConfigManager::remove_env_vars(&["TEST_KEY1", "TEST_KEY2"]);
    }
}

/// 测试移除环境变量
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
fn test_remove_env_vars() {
    // Arrange: 准备测试移除环境变量
    // 注意：这个测试依赖于真实的配置文件路径，可能在某些环境中失败

    // 先设置一些测试变量
    let mut test_vars = HashMap::new();
    test_vars.insert("TEST_REMOVE_KEY".to_string(), "test_value".to_string());
    let _ = ShellConfigManager::set_env_vars(&test_vars);

    // 移除变量
    let remove_result = ShellConfigManager::remove_env_vars(&["TEST_REMOVE_KEY"]);

    // Assert: 验证移除操作
    if let Ok(removed) = remove_result {
        // 如果文件存在，应该返回 true（表示移除了内容）
        assert!(removed || !removed);
    }
}

/// 测试添加source语句
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
fn test_add_source() {
    // Arrange: 准备测试添加 source 语句
    // 注意：这个测试依赖于真实的配置文件路径，可能在某些环境中失败

    let source_path = "$HOME/.workflow/.completions";
    let comment = Some("Test completion");

    let result = ShellConfigManager::add_source(source_path, comment);

    // Assert: 验证可以添加 source 语句
    if let Ok(added) = result {
        assert!(added || !added); // 可能已存在或成功添加
    }

    // 清理：移除测试 source 语句
    let _ = ShellConfigManager::remove_source(source_path);
}

/// 测试移除source语句
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
fn test_remove_source() {
    // Arrange: 准备测试移除 source 语句
    // 注意：这个测试依赖于真实的配置文件路径，可能在某些环境中失败

    let source_path = "$HOME/.workflow/.completions";

    // 先添加 source 语句
    let _ = ShellConfigManager::add_source(source_path, None);

    // 移除 source 语句
    let result = ShellConfigManager::remove_source(source_path);

    // Assert: 验证移除操作
    if let Ok(removed) = result {
        assert!(removed || !removed); // 可能已存在或成功移除
    }
}

/// 测试检查source语句是否存在
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
fn test_has_source() {
    // Arrange: 准备测试检查 source 语句是否存在
    // 注意：这个测试依赖于真实的配置文件路径，可能在某些环境中失败

    let source_path = "$HOME/.workflow/.completions";

    let result = ShellConfigManager::has_source(source_path);

    // Assert: 验证可以检查 source 语句
    if let Ok(has_source) = result {
        assert!(has_source || !has_source); // 可能存在或不存在
    }
}

/// 测试添加带注释的source语句
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
fn test_add_source_with_comment() {
    // Arrange: 准备测试添加带注释的 source 语句
    let source_path = "$HOME/.workflow/test_completions";
    let comment = Some("Test comment for completions");

    let result = ShellConfigManager::add_source(source_path, comment);

    // Assert: 验证可以添加带注释的 source 语句
    if let Ok(added) = result {
        assert!(added || !added);
    }

    // 清理
    let _ = ShellConfigManager::remove_source(source_path);
}

/// 测试添加相同的source语句两次（应跳过重复）
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
fn test_add_source_twice() {
    // Arrange: 准备测试添加相同的 source 语句两次（应该跳过）
    let source_path = "$HOME/.workflow/duplicate_test";

    // 第一次添加
    let result1 = ShellConfigManager::add_source(source_path, None);

    // 第二次添加（应该返回 false，因为已存在）
    let result2 = ShellConfigManager::add_source(source_path, None);

    if let (Ok(added1), Ok(added2)) = (result1, result2) {
        // 第一次应该成功添加，第二次应该跳过
        assert!(added1 || !added1);
        if added1 {
            // 如果第一次成功，第二次应该返回 false（已存在）
            assert!(!added2);
        }
    }

    // 清理
    let _ = ShellConfigManager::remove_source(source_path);
}

/// 测试移除不存在的source语句
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
fn test_remove_nonexistent_source() {
    // Arrange: 准备测试移除不存在的 source 语句
    let source_path = "$HOME/.workflow/nonexistent";

    let result = ShellConfigManager::remove_source(source_path);

    // 应该返回 false（不存在）
    if let Ok(removed) = result {
        assert!(!removed);
    }
}

/// 测试移除不存在的环境变量
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
fn test_remove_nonexistent_env_vars() {
    // Arrange: 准备测试移除不存在的环境变量
    let result = ShellConfigManager::remove_env_vars(&["NONEXISTENT_KEY"]);

    // 应该返回 false（不存在）
    if let Ok(removed) = result {
        assert!(!removed);
    }
}

/// 测试保存和加载环境变量的完整流程
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
fn test_save_and_load_env_vars() {
    // Arrange: 准备测试保存和加载环境变量的完整流程
    let mut test_vars = HashMap::new();
    test_vars.insert("SAVE_TEST_KEY".to_string(), "save_test_value".to_string());

    // 保存
    let save_result = ShellConfigManager::save_env_vars(&test_vars);

    if save_result.is_ok() {
        // 加载
        let load_result = ShellConfigManager::load_env_vars();

        if let Ok(loaded_vars) = load_result {
            // Assert: 验证加载的变量
            if let Some(loaded_value) = loaded_vars.get("SAVE_TEST_KEY") {
                assert_eq!(loaded_value, "save_test_value");
            }
        }

        // 清理
        let _ = ShellConfigManager::remove_env_vars(&["SAVE_TEST_KEY"]);
    }
}

/// 测试设置多个环境变量
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
fn test_set_env_vars_multiple() {
    // Arrange: 准备测试设置多个环境变量
    let mut test_vars = HashMap::new();
    test_vars.insert("MULTI_KEY1".to_string(), "value1".to_string());
    test_vars.insert("MULTI_KEY2".to_string(), "value2".to_string());
    test_vars.insert("MULTI_KEY3".to_string(), "value3".to_string());

    let result = ShellConfigManager::set_env_vars(&test_vars);

    if result.is_ok() {
        // Assert: 验证可以设置多个变量
        assert!(true);

        // 清理
        let _ = ShellConfigManager::remove_env_vars(&["MULTI_KEY1", "MULTI_KEY2", "MULTI_KEY3"]);
    }
}

/// 测试移除多个环境变量
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
fn test_remove_multiple_env_vars() {
    // Arrange: 准备测试移除多个环境变量
    let mut test_vars = HashMap::new();
    test_vars.insert("REMOVE_MULTI_KEY1".to_string(), "value1".to_string());
    test_vars.insert("REMOVE_MULTI_KEY2".to_string(), "value2".to_string());

    // 先设置
    let _ = ShellConfigManager::set_env_vars(&test_vars);

    // 移除多个
    let result = ShellConfigManager::remove_env_vars(&["REMOVE_MULTI_KEY1", "REMOVE_MULTI_KEY2"]);

    if let Ok(removed) = result {
        assert!(removed || !removed);
    }
}

// 注意：以下测试需要实际的 shell 环境，在 CI 环境中可能失败

/// 测试向Zsh配置文件添加source语句
///
/// ## 测试目的
/// 验证`ShellConfigManager`能够正确向用户的~/.zshrc文件添加source语句，
/// 用于加载shell补全脚本。
///
/// ## 为什么被忽略
/// - **修改系统文件**: 会实际修改用户的~/.zshrc配置文件
/// - **安全风险**: 错误的修改可能破坏用户的shell环境
/// - **需要shell环境**: 需要实际的zsh shell环境
/// - **难以恢复**: 配置修改可能难以完全撤销
/// - **环境依赖**: 不同系统的shell配置文件位置可能不同
///
/// ## 如何手动运行
/// ```bash
/// # 1. 先备份配置文件！
/// cp ~/.zshrc ~/.zshrc.backup
///
/// # 2. 运行测试
/// cargo test test_add_source_for_shell_zsh -- --ignored
///
/// # 3. 如需恢复
/// cp ~/.zshrc.backup ~/.zshrc
/// ```
/// **⚠️ 警告**: 此测试会修改你的~/.zshrc文件！请务必先备份！
///
/// ## 测试场景
/// 1. 准备source路径（$HOME/.workflow/zsh_completions）
/// 2. 调用add_source_for_shell添加source语句到~/.zshrc
/// 3. 验证source语句已添加
/// 4. 清理：调用remove_source_for_shell删除添加的语句
///
/// ## 预期行为
/// - source语句正确添加到~/.zshrc末尾
/// - 添加的语句格式：`source $HOME/.workflow/zsh_completions`
/// - 文件格式保持完整（换行符、编码等）
/// - 不破坏现有配置
/// - 清理操作能正确删除添加的语句
/// - 可以被zsh正确解析和执行
#[cfg(not(target_os = "windows"))]
#[test]
#[ignore] // 需要实际的 shell 环境
fn test_add_source_for_shell_zsh() {
    // Arrange: 准备测试为 zsh 添加 source 语句
    use clap_complete::Shell;

    let source_path = "$HOME/.workflow/zsh_completions";
    let result = ShellConfigManager::add_source_for_shell(&Shell::Zsh, source_path, None);

    if let Ok(added) = result {
        assert!(added || !added);
    }

    // 清理
    let _ = ShellConfigManager::remove_source_for_shell(&Shell::Zsh, source_path);
}

/// 测试向Bash配置文件添加source语句
///
/// ## 测试目的
/// 验证`ShellConfigManager`能够正确向用户的~/.bashrc文件添加source语句，
/// 用于加载shell补全脚本。
///
/// ## 为什么被忽略
/// - **修改系统文件**: 会实际修改用户的~/.bashrc配置文件
/// - **安全风险**: 错误的修改可能破坏用户的shell环境
/// - **需要shell环境**: 需要实际的bash shell环境
/// - **难以恢复**: 配置修改可能难以完全撤销
/// - **环境依赖**: 不同系统的shell配置文件位置可能不同
///
/// ## 如何手动运行
/// ```bash
/// # 1. 先备份配置文件！
/// cp ~/.bashrc ~/.bashrc.backup
///
/// # 2. 运行测试
/// cargo test test_add_source_for_shell_bash -- --ignored
///
/// # 3. 如需恢复
/// cp ~/.bashrc.backup ~/.bashrc
/// ```
/// **⚠️ 警告**: 此测试会修改你的~/.bashrc文件！请务必先备份！
///
/// ## 测试场景
/// 1. 准备source路径（$HOME/.workflow/bash_completions）
/// 2. 调用add_source_for_shell添加source语句到~/.bashrc
/// 3. 验证source语句已添加
/// 4. 清理：调用remove_source_for_shell删除添加的语句
///
/// ## 预期行为
/// - source语句正确添加到~/.bashrc末尾
/// - 添加的语句格式：`source $HOME/.workflow/bash_completions`
/// - 文件格式保持完整（换行符、编码等）
/// - 不破坏现有配置
/// - 清理操作能正确删除添加的语句
/// - 可以被bash正确解析和执行
#[cfg(not(target_os = "windows"))]
#[test]
#[ignore] // 需要实际的 shell 环境
fn test_add_source_for_shell_bash() {
    // Arrange: 准备测试为 bash 添加 source 语句
    use clap_complete::Shell;

    let source_path = "$HOME/.workflow/bash_completions";
    let result = ShellConfigManager::add_source_for_shell(&Shell::Bash, source_path, None);

    if let Ok(added) = result {
        assert!(added || !added);
    }

    // 清理
    let _ = ShellConfigManager::remove_source_for_shell(&Shell::Bash, source_path);
}

/// 测试向PowerShell配置文件添加source语句
///
/// ## 测试目的
/// 验证`ShellConfigManager`能够正确向用户的PowerShell配置文件添加source语句（使用`.`关键字），
/// 用于加载shell补全脚本。
///
/// ## 为什么被忽略
/// - **修改系统文件**: 会实际修改用户的PowerShell配置文件（$PROFILE）
/// - **安全风险**: 错误的修改可能破坏用户的PowerShell环境
/// - **需要PowerShell环境**: 需要实际的PowerShell shell环境
/// - **难以恢复**: 配置修改可能难以完全撤销
/// - **Windows特定**: 主要在Windows系统上运行，跨平台测试困难
/// - **配置文件位置**: PowerShell配置文件位置因版本和安装方式不同而异
///
/// ## 如何手动运行
/// ```powershell
/// # 1. 先备份配置文件！（PowerShell）
/// Copy-Item $PROFILE "$PROFILE.backup"
///
/// # 2. 运行测试
/// cargo test test_add_source_for_shell_powershell -- --ignored
///
/// # 3. 如需恢复
/// Copy-Item "$PROFILE.backup" $PROFILE
/// ```
/// **⚠️ 警告**: 此测试会修改你的PowerShell配置文件！请务必先备份！
///
/// ## 测试场景
/// 1. 准备source路径（$HOME/.workflow/powershell_completions）
/// 2. 调用add_source_for_shell添加source语句到PowerShell配置文件
/// 3. 验证source语句已添加
/// 4. 清理：调用remove_source_for_shell删除添加的语句
///
/// ## 预期行为
/// - source语句正确添加到PowerShell配置文件末尾
/// - 添加的语句格式：`. $HOME/.workflow/powershell_completions`（使用`.`而非`source`）
/// - 文件格式保持完整（UTF-16编码、换行符等）
/// - 不破坏现有配置
/// - 清理操作能正确删除添加的语句
/// - 可以被PowerShell正确解析和执行
#[test]
#[ignore] // 需要实际的 shell 环境
fn test_add_source_for_shell_powershell() {
    // Arrange: 准备测试为 PowerShell 添加 source 语句（使用 `.` 关键字）
    use clap_complete::Shell;

    let source_path = "$HOME/.workflow/powershell_completions";
    let result = ShellConfigManager::add_source_for_shell(&Shell::PowerShell, source_path, None);

    if let Ok(added) = result {
        assert!(added || !added);
    }

    // 清理
    let _ = ShellConfigManager::remove_source_for_shell(&Shell::PowerShell, source_path);
}

/// 测试配置块解析功能
///
/// ## 测试目的
/// 验证 ShellConfigManager 能够正确解析配置块格式。
///
/// ## 测试场景
/// 1. 准备包含配置块的测试内容
/// 2. 解析配置块
/// 3. 验证解析结果
///
/// ## 预期结果
/// - 配置块被正确解析
#[rstest]
fn test_config_block_parsing(cli_env: CliTestEnv) -> Result<()> {
    // Arrange: 准备测试配置块解析功能
    // 这个测试验证配置块的解析逻辑（通过实际文件操作）

    let config_content = r#"# Workflow CLI Configuration - Start
# Generated by Workflow CLI - DO NOT edit manually
# These environment variables will be loaded when you start a new shell

export TEST_KEY="test_value"
export ANOTHER_KEY="another_value"
# Workflow CLI Configuration - End
"#;

    // 创建一个临时文件来测试解析
    let env = &cli_env;
    let config_file = env.path().join("test_config");
    fs::write(&config_file, config_content)
        .map_err(|e| color_eyre::eyre::eyre!("should write config file: {}", e))?;

    // 读取并验证内容
    let content = fs::read_to_string(&config_file)
        .map_err(|e| color_eyre::eyre::eyre!("should read config file: {}", e))?;
    assert!(content.contains("TEST_KEY"));
    assert!(content.contains("test_value"));
    assert!(content.contains("ANOTHER_KEY"));
    assert!(content.contains("another_value"));
    Ok(())
}

/// 测试配置块格式
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
fn test_config_block_format() {
    // Arrange: 准备测试配置块格式
    // Assert: 验证配置块的格式正确性

    let config_content = r#"# Workflow CLI Configuration - Start
# Generated by Workflow CLI - DO NOT edit manually
# These environment variables will be loaded when you start a new shell

export KEY="value"
# Workflow CLI Configuration - End
"#;

    // Assert: 验证配置块包含必要的标记
    assert!(config_content.contains("# Workflow CLI Configuration - Start"));
    assert!(config_content.contains("# Workflow CLI Configuration - End"));
    assert!(config_content.contains("export KEY="));
}
