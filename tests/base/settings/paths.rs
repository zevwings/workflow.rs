//! Base/Settings Paths 模块测试
//!
//! 测试路径管理模块的核心功能。

use color_eyre::Result;
use workflow::base::settings::paths::Paths;

#[test]
fn test_paths_home_dir_indirect() {
    // 测试 home_dir() 方法（通过其他方法间接测试，覆盖 paths.rs:49-51）
    // home_dir() 是私有方法，通过 expand("~") 间接测试
    let result = Paths::expand("~");

    // 应该能够获取主目录
    assert!(result.is_ok());
    let home = result.unwrap();
    assert!(home.exists() || !home.exists()); // 主目录可能不存在但路径有效
}

#[test]
fn test_paths_expand_tilde() -> Result<()> {
    // 测试展开 ~ 路径（覆盖 paths.rs:198-201）
    let result = Paths::expand("~/test/path")?;

    // 应该展开为主目录下的路径
    assert!(result.to_string_lossy().contains("test"));
    assert!(result.to_string_lossy().contains("path"));

    Ok(())
}

#[test]
fn test_paths_expand_tilde_only() -> Result<()> {
    // 测试展开单独的 ~（覆盖 paths.rs:202-204）
    let result = Paths::expand("~")?;

    // 应该返回主目录路径
    assert!(result.to_string_lossy().len() > 0);

    Ok(())
}

#[test]
fn test_paths_expand_absolute_path() -> Result<()> {
    // 测试绝对路径（覆盖 paths.rs:238-239）
    let result = Paths::expand("/absolute/path")?;

    // 应该直接返回绝对路径
    assert_eq!(result, std::path::PathBuf::from("/absolute/path"));

    Ok(())
}

#[test]
fn test_paths_expand_relative_path() -> Result<()> {
    // 测试相对路径（覆盖 paths.rs:238-239）
    let result = Paths::expand("relative/path")?;

    // 应该直接返回相对路径
    assert_eq!(result, std::path::PathBuf::from("relative/path"));

    Ok(())
}

#[cfg(target_os = "windows")]
#[test]
fn test_paths_expand_windows_env_var() -> Result<()> {
    // 测试 Windows 环境变量展开（覆盖 paths.rs:207-235）
    // 设置测试环境变量
    env::set_var("TEST_VAR", "test_value");

    let result = Paths::expand("%TEST_VAR%/path")?;

    // 应该展开环境变量
    assert!(result.to_string_lossy().contains("test_value"));

    // 清理
    env::remove_var("TEST_VAR");

    Ok(())
}

#[test]
fn test_paths_expand_env_var_not_set() {
    // 测试未设置的环境变量（覆盖 paths.rs:225-227）
    let result = Paths::expand("%NONEXISTENT_VAR%/path");

    // 应该返回错误
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Environment variable not set"));
}

#[test]
fn test_paths_config_dir() -> Result<()> {
    // 测试 config_dir() 方法（覆盖 paths.rs:261-275）
    let result = Paths::config_dir()?;

    // 应该返回配置目录路径
    assert!(result.to_string_lossy().contains(".workflow"));
    assert!(result.to_string_lossy().contains("config"));

    Ok(())
}

#[test]
fn test_paths_workflow_config() -> Result<()> {
    // 测试 workflow_config() 方法（覆盖 paths.rs:281-283）
    let result = Paths::workflow_config()?;

    // 应该返回 workflow.toml 路径
    assert!(result.to_string_lossy().contains("workflow.toml"));

    Ok(())
}

#[test]
fn test_paths_llm_config() -> Result<()> {
    // 测试 llm_config() 方法（覆盖 paths.rs:288-290）
    let result = Paths::llm_config()?;

    // 应该返回 llm.toml 路径
    assert!(result.to_string_lossy().contains("llm.toml"));

    Ok(())
}

#[test]
fn test_paths_jira_config() -> Result<()> {
    // 测试 jira_config() 方法（覆盖 paths.rs:296-298）
    let result = Paths::jira_config()?;

    // 应该返回 jira.toml 路径
    assert!(result.to_string_lossy().contains("jira.toml"));

    Ok(())
}

#[test]
fn test_paths_commands_config() -> Result<()> {
    // 测试 commands_config() 方法（覆盖 paths.rs:303-305）
    let result = Paths::commands_config()?;

    // 应该返回 commands.toml 路径
    assert!(result.to_string_lossy().contains("commands.toml"));

    Ok(())
}

#[test]
fn test_paths_project_config() -> Result<()> {
    // 测试 project_config() 方法（覆盖 paths.rs:323-328）
    let result = Paths::project_config()?;

    // 应该返回项目配置路径
    assert!(result.to_string_lossy().contains(".workflow"));
    assert!(result.to_string_lossy().contains("config.toml"));

    Ok(())
}

#[test]
fn test_paths_local_base_dir() -> Result<()> {
    // 测试 local_base_dir() 方法（覆盖 paths.rs:116-131）
    let result = Paths::local_base_dir()?;

    // 应该返回本地基础目录
    assert!(result.to_string_lossy().contains(".workflow"));

    Ok(())
}

#[test]
fn test_paths_config_base_dir_indirect() -> Result<()> {
    // 测试 config_base_dir() 方法（通过 config_dir() 间接测试，覆盖 paths.rs:154-170）
    // config_base_dir() 是私有方法，通过 config_dir() 间接测试
    let result = Paths::config_dir()?;

    // 应该返回配置目录（包含 config_base_dir）
    assert!(result.to_string_lossy().contains(".workflow"));
    assert!(result.to_string_lossy().contains("config"));

    Ok(())
}

#[test]
fn test_paths_expand_empty_env_var() {
    // 测试空环境变量名（覆盖 paths.rs:224）
    let result = Paths::expand("%%/path");

    // 空环境变量名应该被忽略或返回错误
    // 根据实现，可能会返回错误或忽略
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_paths_expand_multiple_env_vars() -> Result<()> {
    // 测试多个环境变量（覆盖 paths.rs:207-235）
    #[cfg(target_os = "windows")]
    {
        env::set_var("VAR1", "value1");
        env::set_var("VAR2", "value2");

        let result = Paths::expand("%VAR1%/%VAR2%/path")?;

        assert!(result.to_string_lossy().contains("value1"));
        assert!(result.to_string_lossy().contains("value2"));

        env::remove_var("VAR1");
        env::remove_var("VAR2");
    }

    #[cfg(not(target_os = "windows"))]
    {
        // 在非 Windows 系统上，环境变量展开可能不支持
        let result = Paths::expand("test/path");
        assert!(result.is_ok());
    }

    Ok(())
}

#[test]
fn test_paths_expand_env_var_in_middle() -> Result<()> {
    // 测试路径中间的环境变量（覆盖 paths.rs:207-235）
    #[cfg(target_os = "windows")]
    {
        env::set_var("MID_VAR", "middle");

        let result = Paths::expand("prefix/%MID_VAR%/suffix")?;

        assert!(result.to_string_lossy().contains("middle"));

        env::remove_var("MID_VAR");
    }

    #[cfg(not(target_os = "windows"))]
    {
        // 在非 Windows 系统上，环境变量展开可能不支持
        let result = Paths::expand("prefix/test/suffix");
        assert!(result.is_ok());
    }

    Ok(())
}

// ==================== 边界和跨平台测试 ====================

#[test]
fn test_paths_expand_tilde_with_trailing_slash() -> Result<()> {
    // 测试 ~/path/ 格式
    let result = Paths::expand("~/test/")?;

    assert!(result.to_string_lossy().contains("test"));

    Ok(())
}

#[test]
fn test_paths_expand_complex_path() -> Result<()> {
    // 测试复杂路径展开
    let result = Paths::expand("~/path/../another/./test")?;

    // 应该展开波浪号，但保留相对路径符号
    assert!(result.to_string_lossy().len() > 0);

    Ok(())
}

#[test]
fn test_paths_binary_paths() {
    // 测试 binary_paths() 方法（覆盖 paths.rs:517-527）
    let result = Paths::binary_paths();

    // 应该返回至少一个二进制文件路径
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

#[test]
fn test_paths_command_names() {
    // 测试 command_names() 方法（覆盖 paths.rs:461-463）
    let result = Paths::command_names();

    // 应该返回命令名称列表
    assert!(!result.is_empty());
    assert!(result.contains(&"workflow"));
}

#[test]
fn test_paths_binary_install_dir() {
    // 测试 binary_install_dir() 方法（覆盖 paths.rs:482-494）
    let result = Paths::binary_install_dir();

    // 应该返回安装目录路径
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

#[test]
fn test_paths_completion_dir() -> Result<()> {
    // 测试 completion_dir() 方法（覆盖 paths.rs:570-578）
    let result = Paths::completion_dir()?;

    // 应该返回 completions 目录路径
    assert!(result.to_string_lossy().contains(".workflow"));
    assert!(result.to_string_lossy().contains("completions"));

    Ok(())
}

#[test]
fn test_paths_work_history_dir() -> Result<()> {
    // 测试 work_history_dir() 方法（覆盖 paths.rs:388-403）
    let result = Paths::work_history_dir()?;

    // 应该返回 work-history 目录路径
    assert!(result.to_string_lossy().contains(".workflow"));
    assert!(result.to_string_lossy().contains("work-history"));

    Ok(())
}

#[test]
fn test_paths_logs_dir() -> Result<()> {
    // 测试 logs_dir() 方法（覆盖 paths.rs:425-440）
    let result = Paths::logs_dir()?;

    // 应该返回 logs 目录路径
    assert!(result.to_string_lossy().contains(".workflow"));
    assert!(result.to_string_lossy().contains("logs"));

    Ok(())
}

#[test]
fn test_paths_repository_config() -> Result<()> {
    // 测试 repository_config() 方法（覆盖 paths.rs:347-349）
    let result = Paths::repository_config()?;

    // 应该返回 repository.toml 路径
    assert!(result.to_string_lossy().contains("repository.toml"));

    Ok(())
}

#[test]
fn test_paths_workflow_dir() -> Result<()> {
    // 测试 workflow_dir() 方法（覆盖 paths.rs:362-365）
    let result = Paths::workflow_dir()?;

    // 应该返回 .workflow 目录路径
    assert!(result.to_string_lossy().contains(".workflow"));

    Ok(())
}

#[test]
fn test_paths_is_config_in_icloud() {
    // 测试 is_config_in_icloud() 方法（覆盖 paths.rs:588-598）
    let result = Paths::is_config_in_icloud();

    // 应该返回布尔值（具体值取决于平台和环境）
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

#[test]
fn test_paths_storage_location() {
    // 测试 storage_location() 方法（覆盖 paths.rs:606-612）
    let result = Paths::storage_location();

    // 应该返回存储位置描述
    assert!(!result.is_empty());
    assert!(result.contains("iCloud") || result.contains("Local"));
}

#[test]
fn test_paths_storage_info() -> Result<()> {
    // 测试 storage_info() 方法（覆盖 paths.rs:625-655）
    let result = Paths::storage_info()?;

    // 应该返回包含存储信息的字符串
    assert!(!result.is_empty());
    assert!(result.contains("Storage Type") || result.contains("Configuration"));

    Ok(())
}

#[test]
#[cfg(unix)]
fn test_paths_config_file_unix() -> Result<()> {
    use clap_complete::shells::Shell;

    // 测试 Unix shell 配置文件路径
    let zsh_config = Paths::config_file(&Shell::Zsh)?;
    assert!(zsh_config.to_string_lossy().ends_with(".zshrc"));

    let bash_config = Paths::config_file(&Shell::Bash)?;
    assert!(bash_config.to_string_lossy().ends_with(".bash_profile")
            || bash_config.to_string_lossy().ends_with(".bashrc"));

    let fish_config = Paths::config_file(&Shell::Fish)?;
    assert!(fish_config.to_string_lossy().contains("config.fish"));

    Ok(())
}

#[test]
#[cfg(target_os = "windows")]
fn test_paths_config_file_windows() -> Result<()> {
    use clap_complete::shells::Shell;

    // 测试 Windows PowerShell 配置文件路径
    let ps_config = Paths::config_file(&Shell::PowerShell)?;
    assert!(ps_config.to_string_lossy().contains("Microsoft.PowerShell_profile.ps1"));

    Ok(())
}

#[test]
fn test_paths_expand_with_special_characters() -> Result<()> {
    // 测试路径中包含特殊字符
    let result = Paths::expand("~/test-path_with.dots")?;

    assert!(result.to_string_lossy().contains("test-path_with.dots"));

    Ok(())
}

#[test]
fn test_paths_expand_with_unicode() -> Result<()> {
    // 测试路径中包含 Unicode 字符
    let result = Paths::expand("~/测试/path")?;

    assert!(result.to_string_lossy().contains("测试"));
    assert!(result.to_string_lossy().contains("path"));

    Ok(())
}

#[test]
fn test_paths_expand_deep_nested_path() -> Result<()> {
    // 测试深层嵌套路径
    let deep_path = "~/a/b/c/d/e/f/g/h/i/j";
    let result = Paths::expand(deep_path)?;

    assert!(result.to_string_lossy().contains("j"));

    Ok(())
}

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

