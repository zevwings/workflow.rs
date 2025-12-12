//! Settings 模块测试
//!
//! 测试配置加载、初始化和路径管理功能。

use clap_complete::shells::Shell;
use std::fs;
use workflow::base::settings::paths::Paths;
use workflow::base::settings::settings::Settings;

// ==================== Settings 测试 ====================

#[test]
fn test_settings_initialization() {
    // 测试初始化（使用默认值）
    let settings = Settings::load();
    // 注意：这些测试会加载实际的配置文件，所以只测试结构是否正确加载
    assert_eq!(settings.log.output_folder_name, "logs");
    // LLM provider 可能是 openai 或用户配置的其他值
    assert!(!settings.llm.provider.is_empty());
}

#[test]
fn test_llm_provider() {
    // 测试 LLM provider 是否被正确加载
    let settings = Settings::load();
    // 可能是 openai (默认) 或用户配置的其他值
    assert!(!settings.llm.provider.is_empty());
}

// ==================== Paths 测试 ====================

#[test]
fn test_config_persistence() {
    // 获取配置目录
    let config_dir = Paths::config_dir().unwrap();

    // 验证目录存在
    assert!(config_dir.exists());
    assert!(config_dir.is_dir());

    // 创建测试文件
    let test_file = config_dir.join("test_integration.txt");
    fs::write(&test_file, "test content").unwrap();

    // 验证文件存在
    assert!(test_file.exists());

    // 读取文件
    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "test content");

    // 清理
    fs::remove_file(&test_file).ok();
}

#[test]
fn test_work_history_independence() {
    let config_dir = Paths::config_dir().unwrap();
    let work_history_dir = Paths::work_history_dir().unwrap();

    // 验证两个目录存在且不同
    assert!(config_dir.exists());
    assert!(work_history_dir.exists());
    assert_ne!(config_dir, work_history_dir);

    // 在两个目录下创建同名文件
    let config_test = config_dir.join("test.json");
    let history_test = work_history_dir.join("test.json");

    fs::write(&config_test, r#"{"type": "config"}"#).unwrap();
    fs::write(&history_test, r#"{"type": "history"}"#).unwrap();

    // 验证两个文件独立存在
    assert!(config_test.exists());
    assert!(history_test.exists());

    // 验证内容不同
    let config_content = fs::read_to_string(&config_test).unwrap();
    let history_content = fs::read_to_string(&history_test).unwrap();
    assert_ne!(config_content, history_content);

    // 清理
    fs::remove_file(&config_test).ok();
    fs::remove_file(&history_test).ok();
}

#[test]
fn test_completion_dir_creation() {
    let completion_dir = Paths::completion_dir().unwrap();

    // 验证目录存在（或可以创建）
    assert!(completion_dir.parent().unwrap().exists());

    // 创建测试文件
    let test_file = completion_dir.join("test_completion.bash");
    fs::write(&test_file, "# test completion").unwrap();

    // 验证文件存在
    assert!(test_file.exists());

    // 清理
    fs::remove_file(&test_file).ok();
}

#[test]
fn test_all_config_paths() {
    // 测试所有配置路径方法都能正常工作
    let workflow_config = Paths::workflow_config().unwrap();
    let jira_status = Paths::jira_status_config().unwrap();
    let jira_users = Paths::jira_users_config().unwrap();

    // 验证所有路径都在同一个目录下
    let config_dir = workflow_config.parent().unwrap();
    assert_eq!(config_dir, jira_status.parent().unwrap());
    assert_eq!(config_dir, jira_users.parent().unwrap());

    // 验证目录存在
    assert!(config_dir.exists());
    assert!(config_dir.is_dir());
}

#[test]
#[cfg(target_os = "macos")]
fn test_icloud_detection_integration() {
    // 测试 iCloud 检测逻辑
    let is_icloud = Paths::is_config_in_icloud();
    let location = Paths::storage_location();
    let config_dir = Paths::config_dir().unwrap();

    if is_icloud {
        assert_eq!(location, "iCloud Drive (synced across devices)");

        // 验证配置目录在 iCloud 路径下
        let path_str = config_dir.to_string_lossy();
        assert!(path_str.contains("com~apple~CloudDocs"));
    } else {
        assert_eq!(location, "Local storage");

        // 验证配置目录在本地路径下
        let path_str = config_dir.to_string_lossy();
        assert!(path_str.contains(".workflow") || path_str.contains("workflow"));
    }
}

#[test]
fn test_storage_info_format() {
    let info = Paths::storage_info().unwrap();

    // 验证信息格式正确
    assert!(!info.is_empty());
    assert!(info.contains("Storage Type"));
    assert!(info.contains("Configuration"));
    assert!(info.contains("Work History"));

    // 验证包含路径信息
    let config_dir = Paths::config_dir().unwrap();
    let work_history_dir = Paths::work_history_dir().unwrap();

    assert!(info.contains(&*config_dir.to_string_lossy()));
    assert!(info.contains(&*work_history_dir.to_string_lossy()));
}

#[test]
fn test_path_consistency() {
    // 测试路径的一致性
    let config_dir1 = Paths::config_dir().unwrap();
    let config_dir2 = Paths::config_dir().unwrap();

    // 多次调用应该返回相同路径
    assert_eq!(config_dir1, config_dir2);

    // 测试工作历史目录
    let history_dir1 = Paths::work_history_dir().unwrap();
    let history_dir2 = Paths::work_history_dir().unwrap();
    assert_eq!(history_dir1, history_dir2);

    // 测试补全目录
    let completion_dir1 = Paths::completion_dir().unwrap();
    let completion_dir2 = Paths::completion_dir().unwrap();
    assert_eq!(completion_dir1, completion_dir2);
}

// ==================== 基础路径测试 ====================

#[test]
fn test_config_dir() {
    let config_dir = Paths::config_dir().unwrap();
    assert!(config_dir.exists());
    assert!(config_dir.is_dir());
    assert!(
        config_dir.ends_with(".workflow/config")
            || config_dir.to_string_lossy().contains("workflow")
    );
}

#[test]
fn test_work_history_dir() {
    let history_dir = Paths::work_history_dir().unwrap();
    assert!(history_dir.exists());
    assert!(history_dir.is_dir());
    let path_str = history_dir.to_string_lossy();
    assert!(path_str.contains("work-history"));
}

#[test]
fn test_completion_dir() {
    let completion_dir = Paths::completion_dir().unwrap();
    let path_str = completion_dir.to_string_lossy();
    assert!(path_str.contains("completions"));
}

#[test]
fn test_workflow_dir() {
    let workflow_dir = Paths::workflow_dir().unwrap();
    assert!(workflow_dir.exists());
    assert!(workflow_dir.is_dir());
}

#[test]
fn test_config_file_paths() {
    // 测试所有支持的 shell 配置文件路径
    let shells = vec![
        Shell::Zsh,
        Shell::Bash,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Elvish,
    ];

    for shell in shells {
        let config_file = Paths::config_file(&shell);
        match config_file {
            Ok(path) => {
                // 验证路径格式正确
                assert!(!path.to_string_lossy().is_empty());
            }
            Err(_) => {
                // Windows 上某些 shell 可能不支持，这是正常的
                #[cfg(target_os = "windows")]
                {
                    // Windows 上只有 PowerShell 应该成功
                    if matches!(shell, Shell::PowerShell) {
                        panic!("PowerShell config file should be available on Windows");
                    }
                }
            }
        }
    }
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_shell_config_paths_unix() {
    let zsh_config = Paths::config_file(&Shell::Zsh).unwrap();
    assert!(zsh_config.to_string_lossy().ends_with(".zshrc"));

    let bash_config = Paths::config_file(&Shell::Bash).unwrap();
    let bash_path = bash_config.to_string_lossy();
    assert!(
        bash_path.ends_with(".bash_profile") || bash_path.ends_with(".bashrc"),
        "Bash config should be .bash_profile or .bashrc"
    );
}

#[test]
fn test_work_history_always_local() {
    let work_history = Paths::work_history_dir().unwrap();
    // 通过路径字符串验证，不直接调用 home_dir()
    let path_str = work_history.to_string_lossy();

    // work_history 应该总是在本地路径下（包含 .workflow/work-history）
    assert!(path_str.contains(".workflow"));
    assert!(path_str.contains("work-history"));

    // 确保不在 iCloud 路径下
    assert!(!path_str.contains("com~apple~CloudDocs"));
}

#[test]
fn test_completion_dir_is_local() {
    let completion_dir = Paths::completion_dir().unwrap();
    // 通过路径字符串验证，不直接调用 home_dir()
    let path_str = completion_dir.to_string_lossy();

    // completion 应该总是在本地路径下
    assert!(path_str.contains(".workflow"));
    assert!(path_str.contains("completions"));

    // 确保不在 iCloud 路径下
    assert!(!path_str.contains("com~apple~CloudDocs"));
}

#[test]
fn test_storage_location() {
    let location = Paths::storage_location();
    assert!(!location.is_empty());
    // 应该是 "iCloud Drive (synced across devices)" 或 "Local storage"
    assert!(location == "iCloud Drive (synced across devices)" || location == "Local storage");
}

#[test]
#[cfg(not(target_os = "macos"))]
fn test_non_macos_always_local() {
    assert!(!Paths::is_config_in_icloud());
    assert_eq!(Paths::storage_location(), "Local storage");
}
