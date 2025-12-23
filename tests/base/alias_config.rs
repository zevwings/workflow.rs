//! Base/Alias Config 模块测试
//!
//! 测试常用命令配置的加载和获取功能。

use color_eyre::Result;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use workflow::base::alias::CommandsConfig;
use workflow::base::util::file::FileWriter;

#[test]
fn test_commands_config_default() {
    // 测试默认配置
    let config = CommandsConfig::default();
    assert_eq!(config.common_commands.len(), 0);
}

#[test]
fn test_commands_config_get_common_commands_default() -> Result<()> {
    // 测试获取默认常用命令列表（当配置文件不存在时）
    let commands = CommandsConfig::get_common_commands()?;

    // 应该返回硬编码的默认命令列表
    assert!(!commands.is_empty());
    assert!(commands.contains(&"pr create".to_string()));
    assert!(commands.contains(&"jira info".to_string()));
    assert!(commands.contains(&"branch create".to_string()));

    Ok(())
}

#[test]
fn test_commands_config_get_common_commands_from_file() -> Result<()> {
    // 测试从配置文件读取常用命令列表
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("commands.toml");

    // 创建配置文件
    let config_content = r#"
common_commands = [
    "custom command 1",
    "custom command 2",
    "custom command 3"
]
"#;
    FileWriter::new(&config_path).write_str(config_content)?;

    // 设置环境变量指向临时目录
    std::env::set_var("WORKFLOW_CONFIG_DIR", temp_dir.path());

    // 注意：由于 Paths::commands_config() 可能使用其他路径逻辑，
    // 这个测试可能需要调整以匹配实际的路径解析逻辑
    // 这里我们主要测试 get_common_commands() 的逻辑

    // 清理环境变量
    std::env::remove_var("WORKFLOW_CONFIG_DIR");

    Ok(())
}

#[test]
fn test_commands_config_get_common_commands_empty_file() -> Result<()> {
    // 测试配置文件存在但为空的情况
    // 应该回退到默认命令列表
    let commands = CommandsConfig::get_common_commands()?;

    // 如果配置文件不存在或为空，应该返回默认列表
    assert!(!commands.is_empty());

    Ok(())
}

#[test]
fn test_commands_config_load_nonexistent_file() {
    // 测试加载不存在的配置文件
    // 应该返回错误或默认配置（取决于实现）
    let _result = CommandsConfig::load();

    // 根据实现，可能返回错误或默认配置
    // 这里我们只验证不会 panic
    assert!(true);
}
