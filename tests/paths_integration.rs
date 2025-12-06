//! 路径管理集成测试
//!
//! 测试路径相关的集成功能，包括目录创建、路径解析等。

use workflow::base::settings::paths::Paths;
use std::fs;

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
    let branch_config = Paths::branch_config().unwrap();
    
    // 验证所有路径都在同一个目录下
    let config_dir = workflow_config.parent().unwrap();
    assert_eq!(config_dir, jira_status.parent().unwrap());
    assert_eq!(config_dir, jira_users.parent().unwrap());
    assert_eq!(config_dir, branch_config.parent().unwrap());
    
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
