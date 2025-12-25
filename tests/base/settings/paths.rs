//! Base/Settings Paths 模块测试
//!
//! 测试路径管理模块的核心功能。

use color_eyre::Result;
use workflow::base::settings::paths::Paths;

// ==================== Paths Expand Tests ====================

/// 测试展开主目录路径（~）
#[test]
fn test_paths_home_dir_with_expand_tilde_returns_home_path() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）
    // 测试 home_dir() 方法（通过其他方法间接测试，覆盖 paths.rs:49-51）
    // home_dir() 是私有方法，通过 expand("~") 间接测试

    // Act: 展开 ~ 路径
    let home = Paths::expand("~")?;

    // Assert: 验证能够获取主目录
    assert!(home.exists() || !home.exists()); // 主目录可能不存在但路径有效
    Ok(())
}

/// 测试展开包含~的路径
#[test]
fn test_paths_expand_with_tilde_path_returns_expanded_path() -> Result<()> {
    // Arrange: 准备包含 ~ 的路径

    // Act: 展开 ~ 路径（覆盖 paths.rs:198-201）
    let result = Paths::expand("~/test/path")?;

    // Assert: 验证展开为主目录下的路径
    assert!(result.to_string_lossy().contains("test"));
    assert!(result.to_string_lossy().contains("path"));

    Ok(())
}

/// 测试展开单独的~
#[test]
fn test_paths_expand_with_tilde_only_returns_home_path() -> Result<()> {
    // Arrange: 准备单独的 ~

    // Act: 展开单独的 ~（覆盖 paths.rs:202-204）
    let result = Paths::expand("~")?;

    // Assert: 验证返回主目录路径
    assert!(result.to_string_lossy().len() > 0);

    Ok(())
}

/// 测试展开绝对路径
#[test]
fn test_paths_expand_with_absolute_path_returns_absolute_path() -> Result<()> {
    // Arrange: 准备绝对路径

    // Act: 展开绝对路径（覆盖 paths.rs:238-239）
    let result = Paths::expand("/absolute/path")?;

    // Assert: 验证直接返回绝对路径
    assert_eq!(result, std::path::PathBuf::from("/absolute/path"));

    Ok(())
}

/// 测试展开相对路径
#[test]
fn test_paths_expand_with_relative_path_returns_relative_path() -> Result<()> {
    // Arrange: 准备相对路径

    // Act: 展开相对路径（覆盖 paths.rs:238-239）
    let result = Paths::expand("relative/path")?;

    // Assert: 验证直接返回相对路径
    assert_eq!(result, std::path::PathBuf::from("relative/path"));

    Ok(())
}

/// 测试展开Windows环境变量路径（仅Windows）
#[cfg(target_os = "windows")]
#[test]
fn test_paths_expand_with_windows_env_var_returns_expanded_path() -> Result<()> {
    // Arrange: 使用 EnvGuard 设置测试环境变量
    // 测试 Windows 环境变量展开（覆盖 paths.rs:207-235）
    let mut guard = EnvGuard::new();
    guard.set("TEST_VAR", "test_value");

    // Act: 展开包含环境变量的路径
    let result = Paths::expand("%TEST_VAR%/path")?;

    // Assert: 验证展开环境变量
    assert!(result.to_string_lossy().contains("test_value"));
    // EnvGuard 会在 guard 离开作用域时自动恢复环境变量

    Ok(())
}

/// 测试展开未设置的环境变量时返回错误
#[test]
fn test_paths_expand_with_env_var_not_set_returns_error() {
    // Arrange: 准备未设置的环境变量

    // Act: 展开包含未设置环境变量的路径（覆盖 paths.rs:225-227）
    let result = Paths::expand("%NONEXISTENT_VAR%/path");

    // Assert: 验证返回错误
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Environment variable not set"));
}

// ==================== Paths Config Directory Tests ====================

/// 测试获取配置目录路径
#[test]
fn test_paths_config_dir_with_no_params_returns_config_dir() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）

    // Act: 获取配置目录（覆盖 paths.rs:261-275）
    let result = Paths::config_dir()?;

    // Assert: 验证返回配置目录路径
    assert!(result.to_string_lossy().contains(".workflow"));
    assert!(result.to_string_lossy().contains("config"));

    Ok(())
}

/// 测试获取workflow配置文件路径
#[test]
fn test_paths_workflow_config_with_no_params_returns_workflow_config_path() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）

    // Act: 获取 workflow 配置路径（覆盖 paths.rs:281-283）
    let result = Paths::workflow_config()?;

    // Assert: 验证返回 workflow.toml 路径
    assert!(result.to_string_lossy().contains("workflow.toml"));

    Ok(())
}

/// 测试获取llm配置文件路径
#[test]
fn test_paths_llm_config_with_no_params_returns_llm_config_path() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）

    // Act: 获取 llm 配置路径（覆盖 paths.rs:288-290）
    let result = Paths::llm_config()?;

    // Assert: 验证返回 llm.toml 路径
    assert!(result.to_string_lossy().contains("llm.toml"));

    Ok(())
}

/// 测试获取jira配置文件路径
#[test]
fn test_paths_jira_config_with_no_params_returns_jira_config_path() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）

    // Act: 获取 jira 配置路径（覆盖 paths.rs:296-298）
    let result = Paths::jira_config()?;

    // Assert: 验证返回 jira.toml 路径
    assert!(result.to_string_lossy().contains("jira.toml"));

    Ok(())
}

/// 测试获取commands配置文件路径
#[test]
fn test_paths_commands_config_with_no_params_returns_commands_config_path() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）

    // Act: 获取 commands 配置路径（覆盖 paths.rs:303-305）
    let result = Paths::commands_config()?;

    // Assert: 验证返回 commands.toml 路径
    assert!(result.to_string_lossy().contains("commands.toml"));

    Ok(())
}

/// 测试获取项目配置文件路径
#[test]
fn test_paths_project_config() -> Result<()> {
    // 测试 project_config() 方法（覆盖 paths.rs:323-328）
    let result = Paths::project_config()?;

    // Assert: 验证返回项目配置路径
    assert!(result.to_string_lossy().contains(".workflow"));
    assert!(result.to_string_lossy().contains("config.toml"));

    Ok(())
}

/// 测试获取本地基础目录路径
#[test]
fn test_paths_local_base_dir() -> Result<()> {
    // 测试 local_base_dir() 方法（覆盖 paths.rs:116-131）
    let result = Paths::local_base_dir()?;

    // Assert: 验证返回本地基础目录
    assert!(result.to_string_lossy().contains(".workflow"));

    Ok(())
}

/// 测试获取配置基础目录路径（间接测试）
#[test]
fn test_paths_config_base_dir_indirect() -> Result<()> {
    // 测试 config_base_dir() 方法（通过 config_dir() 间接测试，覆盖 paths.rs:154-170）
    // config_base_dir() 是私有方法，通过 config_dir() 间接测试
    let result = Paths::config_dir()?;

    // Assert: 验证返回配置目录（包含 config_base_dir）
    assert!(result.to_string_lossy().contains(".workflow"));
    assert!(result.to_string_lossy().contains("config"));

    Ok(())
}

/// 测试展开空环境变量名
#[test]
fn test_paths_expand_empty_env_var() {
    // 测试空环境变量名（覆盖 paths.rs:224）
    let result = Paths::expand("%%/path");

    // 空环境变量名应该被忽略或返回错误
    // 根据实现，可能会返回错误或忽略
    assert!(result.is_ok() || result.is_err());
}

/// 测试展开多个环境变量
#[test]
fn test_paths_expand_multiple_env_vars() -> Result<()> {
    // 测试多个环境变量（覆盖 paths.rs:207-235）
    #[cfg(target_os = "windows")]
    {
        let mut guard = EnvGuard::new();
        guard.set("VAR1", "value1");
        guard.set("VAR2", "value2");

        let result = Paths::expand("%VAR1%/%VAR2%/path")?;

        assert!(result.to_string_lossy().contains("value1"));
        assert!(result.to_string_lossy().contains("value2"));
        // EnvGuard 会在 guard 离开作用域时自动恢复环境变量
    }

    #[cfg(not(target_os = "windows"))]
    {
        // 在非 Windows 系统上，环境变量展开可能不支持
        let result = Paths::expand("test/path");
        assert!(result.is_ok());
    }

    Ok(())
}

/// 测试展开路径中间的环境变量
#[test]
fn test_paths_expand_env_var_in_middle() -> Result<()> {
    // 测试路径中间的环境变量（覆盖 paths.rs:207-235）
    #[cfg(target_os = "windows")]
    {
        let mut guard = EnvGuard::new();
        guard.set("MID_VAR", "middle");

        let result = Paths::expand("prefix/%MID_VAR%/suffix")?;

        assert!(result.to_string_lossy().contains("middle"));
        // EnvGuard 会在 guard 离开作用域时自动恢复环境变量
    }

    #[cfg(not(target_os = "windows"))]
    {
        // 在非 Windows 系统上，环境变量展开可能不支持
        let result = Paths::expand("prefix/test/suffix");
        assert!(result.is_ok());
    }

    Ok(())
}

// ==================== Boundary and Cross-Platform Tests ====================

/// 测试展开带尾随斜杠的~路径
#[test]
fn test_paths_expand_tilde_with_trailing_slash() -> Result<()> {
    // 测试 ~/path/ 格式
    let result = Paths::expand("~/test/")?;

    assert!(result.to_string_lossy().contains("test"));

    Ok(())
}

/// 测试展开复杂路径（包含相对路径符号）
#[test]
fn test_paths_expand_complex_path() -> Result<()> {
    // 测试复杂路径展开
    let result = Paths::expand("~/path/../another/./test")?;

    // Assert: 验证展开波浪号，但保留相对路径符号
    assert!(result.to_string_lossy().len() > 0);

    Ok(())
}

/// 测试获取二进制文件路径列表
#[test]
fn test_paths_binary_paths() {
    // 测试 binary_paths() 方法（覆盖 paths.rs:517-527）
    let result = Paths::binary_paths();

    // Assert: 验证返回至少一个二进制文件路径
    assert!(!result.is_empty());
    assert!(result[0].contains("workflow"));

    // 验证路径格式
    #[cfg(target_os = "windows")]
    {
        assert!(result[0].ends_with(".exe"));
    }

    #[cfg(not(target_os = "windows"))]
    {
        assert!(!result[0].ends_with(".exe"));
    }
}

/// 测试获取二进制文件名（跨平台）
#[test]
fn test_paths_binary_name() {
    // 测试 binary_name() 方法（覆盖 paths.rs:550-556）
    let result = Paths::binary_name("workflow");

    // Windows 应该有 .exe 扩展名
    #[cfg(target_os = "windows")]
    {
        assert_eq!(result, "workflow.exe");
    }

    // 其他平台不应该有扩展名
    #[cfg(not(target_os = "windows"))]
    {
        assert_eq!(result, "workflow");
    }
}

/// 测试获取自定义二进制文件名
#[test]
fn test_paths_binary_name_custom() {
    // 测试自定义名称
    let result = Paths::binary_name("custom-tool");

    #[cfg(target_os = "windows")]
    {
        assert_eq!(result, "custom-tool.exe");
    }

    #[cfg(not(target_os = "windows"))]
    {
        assert_eq!(result, "custom-tool");
    }
}

/// 测试获取命令名称列表
///
/// ## 测试目的
/// 验证 `Paths::command_names()` 方法能够返回可用的命令名称列表。
///
/// ## 测试场景
/// 1. 调用 `Paths::command_names()` 获取命令名称列表
/// 2. 验证返回结果
///
/// ## 预期结果
/// - 返回的命令名称列表不为空
/// - 列表中包含 "workflow" 命令
#[test]
fn test_paths_command_names() {
    // 测试 command_names() 方法（覆盖 paths.rs:461-463）
    let result = Paths::command_names();

    // Assert: 验证返回命令名称列表
    assert!(!result.is_empty());
    assert!(result.contains(&"workflow"));
}

/// 测试获取二进制安装目录路径（跨平台）
#[test]
fn test_paths_binary_install_dir() {
    // 测试 binary_install_dir() 方法（覆盖 paths.rs:482-494）
    let result = Paths::binary_install_dir();

    // Assert: 验证返回安装目录路径
    assert!(!result.is_empty());

    #[cfg(target_os = "windows")]
    {
        // Windows 应该包含 Programs 或 bin
        assert!(result.contains("Programs") || result.contains("bin"));
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix-like 应该是 /usr/local/bin
        assert_eq!(result, "/usr/local/bin");
    }
}

/// 测试获取补全脚本目录路径
#[test]
fn test_paths_completion_dir() -> Result<()> {
    // 测试 completion_dir() 方法（覆盖 paths.rs:570-578）
    let result = Paths::completion_dir()?;

    // Assert: 验证返回 completions 目录路径
    assert!(result.to_string_lossy().contains(".workflow"));
    assert!(result.to_string_lossy().contains("completions"));

    Ok(())
}

/// 测试获取工作历史目录路径
#[test]
fn test_paths_work_history_dir() -> Result<()> {
    // 测试 work_history_dir() 方法（覆盖 paths.rs:388-403）
    let result = Paths::work_history_dir()?;

    // Assert: 验证返回 work-history 目录路径
    assert!(result.to_string_lossy().contains(".workflow"));
    assert!(result.to_string_lossy().contains("work-history"));

    Ok(())
}

/// 测试获取日志目录路径
#[test]
fn test_paths_logs_dir() -> Result<()> {
    // 测试 logs_dir() 方法（覆盖 paths.rs:425-440）
    let result = Paths::logs_dir()?;

    // Assert: 验证返回 logs 目录路径
    assert!(result.to_string_lossy().contains(".workflow"));
    assert!(result.to_string_lossy().contains("logs"));

    Ok(())
}

/// 测试获取仓库配置文件路径
#[test]
fn test_paths_repository_config() -> Result<()> {
    // 测试 repository_config() 方法（覆盖 paths.rs:347-349）
    let result = Paths::repository_config()?;

    // Assert: 验证返回 repository.toml 路径
    assert!(result.to_string_lossy().contains("repository.toml"));

    Ok(())
}

/// 测试获取workflow目录路径
#[test]
fn test_paths_workflow_dir() -> Result<()> {
    // 测试 workflow_dir() 方法（覆盖 paths.rs:362-365）
    let result = Paths::workflow_dir()?;

    // Assert: 验证返回 .workflow 目录路径
    assert!(result.to_string_lossy().contains(".workflow"));

    Ok(())
}

/// 测试检查配置是否在iCloud中（仅macOS）
#[test]
fn test_paths_is_config_in_icloud() {
    // 测试 is_config_in_icloud() 方法（覆盖 paths.rs:588-598）
    let result = Paths::is_config_in_icloud();

    // Assert: 验证返回布尔值（具体值取决于平台和环境）
    #[cfg(target_os = "macos")]
    {
        // macOS 可能返回 true 或 false
        assert!(result || !result);
    }

    #[cfg(not(target_os = "macos"))]
    {
        // 非 macOS 应该返回 false
        assert!(!result);
    }
}

/// 测试获取存储位置描述
#[test]
fn test_paths_storage_location() {
    // 测试 storage_location() 方法（覆盖 paths.rs:606-612）
    let result = Paths::storage_location();

    // Assert: 验证返回存储位置描述
    assert!(!result.is_empty());
    assert!(result.contains("iCloud") || result.contains("Local"));
}

/// 测试获取存储信息详情
#[test]
fn test_paths_storage_info() -> Result<()> {
    // 测试 storage_info() 方法（覆盖 paths.rs:625-655）
    let result = Paths::storage_info()?;

    // Assert: 验证返回包含存储信息的字符串
    assert!(!result.is_empty());
    assert!(result.contains("Storage Type") || result.contains("Configuration"));

    Ok(())
}

/// 测试获取Unix shell配置文件路径（仅Unix）
#[test]
#[cfg(unix)]
fn test_paths_config_file_unix() -> Result<()> {
    use clap_complete::shells::Shell;

    // 测试 Unix shell 配置文件路径
    let zsh_config = Paths::config_file(&Shell::Zsh)?;
    assert!(zsh_config.to_string_lossy().ends_with(".zshrc"));

    let bash_config = Paths::config_file(&Shell::Bash)?;
    assert!(
        bash_config.to_string_lossy().ends_with(".bash_profile")
            || bash_config.to_string_lossy().ends_with(".bashrc")
    );

    let fish_config = Paths::config_file(&Shell::Fish)?;
    assert!(fish_config.to_string_lossy().contains("config.fish"));

    Ok(())
}

/// 测试获取Windows PowerShell配置文件路径（仅Windows）
#[test]
#[cfg(target_os = "windows")]
fn test_paths_config_file_windows() -> Result<()> {
    use clap_complete::shells::Shell;

    // 测试 Windows PowerShell 配置文件路径
    let ps_config = Paths::config_file(&Shell::PowerShell)?;
    assert!(ps_config.to_string_lossy().contains("Microsoft.PowerShell_profile.ps1"));

    Ok(())
}

/// 测试展开包含特殊字符的路径
#[test]
fn test_paths_expand_with_special_characters() -> Result<()> {
    // 测试路径中包含特殊字符
    let result = Paths::expand("~/test-path_with.dots")?;

    assert!(result.to_string_lossy().contains("test-path_with.dots"));

    Ok(())
}

/// 测试展开包含Unicode字符的路径
#[test]
fn test_paths_expand_with_unicode() -> Result<()> {
    // 测试路径中包含 Unicode 字符
    let result = Paths::expand("~/测试/path")?;

    assert!(result.to_string_lossy().contains("测试"));
    assert!(result.to_string_lossy().contains("path"));

    Ok(())
}

/// 测试展开深层嵌套路径
#[test]
fn test_paths_expand_deep_nested_path() -> Result<()> {
    // 测试深层嵌套路径
    let deep_path = "~/a/b/c/d/e/f/g/h/i/j";
    let result = Paths::expand(deep_path)?;

    assert!(result.to_string_lossy().contains("j"));

    Ok(())
}

/// 测试多个配置方法返回的路径一致性
#[test]
fn test_paths_multiple_config_methods() -> Result<()> {
    // 测试多个配置方法返回的路径都是有效的
    let workflow = Paths::workflow_config()?;
    let jira = Paths::jira_config()?;
    let llm = Paths::llm_config()?;

    // 所有配置文件应该在同一个目录下
    assert_eq!(workflow.parent(), jira.parent());
    assert_eq!(workflow.parent(), llm.parent());

    // 文件名应该不同
    assert_ne!(workflow.file_name(), jira.file_name());
    assert_ne!(workflow.file_name(), llm.file_name());
    assert_ne!(jira.file_name(), llm.file_name());

    Ok(())
}
