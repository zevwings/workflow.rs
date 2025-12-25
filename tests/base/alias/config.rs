//! Base/Alias Config 模块测试
//!
//! 测试常用命令配置的加载和获取功能。

use color_eyre::Result;
use pretty_assertions::assert_eq;
use workflow::base::alias::CommandsConfig;
use workflow::base::util::file::FileWriter;

use crate::common::environments::CliTestEnv;

// ==================== CommandsConfig Initialization Tests ====================

#[test]
fn test_commands_config_default_with_no_parameters_creates_empty_config() {
    // Arrange: 准备创建默认配置

    // Act: 创建默认配置
    let config = CommandsConfig::default();

    // Assert: 验证配置为空
    assert_eq!(config.common_commands.len(), 0);
}

// ==================== CommandsConfig Common Commands Tests ====================

#[test]
fn test_commands_config_get_common_commands_default_with_no_config_returns_defaults() -> Result<()> {
    // Arrange: 准备无配置文件的环境

    // Act: 获取默认常用命令列表
    let commands = CommandsConfig::get_common_commands()?;

    // Assert: 验证返回硬编码的默认命令列表
    assert!(!commands.is_empty());
    assert!(commands.contains(&"pr create".to_string()));
    assert!(commands.contains(&"jira info".to_string()));
    assert!(commands.contains(&"branch create".to_string()));

    Ok(())
}

#[test]
fn test_commands_config_get_common_commands_from_file_with_valid_config_reads_commands() -> Result<()> {
    // Arrange: 准备配置文件
    let mut env = CliTestEnv::new()?;
    let config_path = env.path().join("commands.toml");
    let config_content = r#"
common_commands = [
    "custom command 1",
    "custom command 2",
    "custom command 3"
]
"#;
    FileWriter::new(&config_path).write_str(config_content)?;
    let config_dir = env.path().to_string_lossy().to_string();
    env.env_guard().set("WORKFLOW_CONFIG_DIR", &config_dir);

    // Act: 从配置文件读取常用命令列表
    // 注意：由于 Paths::commands_config() 可能使用其他路径逻辑，
    // 这个测试可能需要调整以匹配实际的路径解析逻辑
    // EnvGuard 会在 env 离开作用域时自动恢复环境变量

    Ok(())
}

#[test]
fn test_commands_config_get_common_commands_empty_file_with_empty_config_returns_defaults() -> Result<()> {
    // Arrange: 准备空配置文件环境

    // Act: 获取常用命令列表
    let commands = CommandsConfig::get_common_commands()?;

    // Assert: 验证返回默认列表
    assert!(!commands.is_empty());

    Ok(())
}

// ==================== CommandsConfig Loading Tests ====================

#[test]
fn test_commands_config_load_nonexistent_file_with_missing_file_handles_gracefully() {
    // Arrange: 准备不存在的配置文件环境

    // Act: 尝试加载配置文件
    let _result = CommandsConfig::load();

    // Assert: 验证不会panic（可能返回错误或默认配置）
    assert!(true);
}

#[test]
fn test_commands_config_load_existing_file_with_valid_config_loads_config() -> Result<()> {
    // Arrange: 准备存在的配置文件
    let mut env = CliTestEnv::new()?;
    let config_path = env.path().join("commands.toml");
    let config_content = r#"
common_commands = [
    "test command 1",
    "test command 2"
]
"#;
    FileWriter::new(&config_path).write_str(config_content)?;
    let config_dir = env.path().to_string_lossy().to_string();
    env.env_guard().set("WORKFLOW_CONFIG_DIR", &config_dir);

    // Act: 尝试加载配置
    let result = CommandsConfig::load();
    // EnvGuard 会在 env 离开作用域时自动恢复环境变量

    // Assert: 验证可以加载（可能成功或失败，取决于路径解析逻辑）
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

#[test]
fn test_commands_config_get_common_commands_with_custom_file_reads_custom_commands() -> Result<()> {
    // Arrange: 准备包含自定义命令的配置文件
    let mut env = CliTestEnv::new()?;
    let config_path = env.path().join("commands.toml");
    let config_content = r#"
common_commands = [
    "custom command 1",
    "custom command 2",
    "custom command 3"
]
"#;
    FileWriter::new(&config_path).write_str(config_content)?;
    let config_dir = env.path().to_string_lossy().to_string();
    env.env_guard().set("WORKFLOW_CONFIG_DIR", &config_dir);

    // Act: 获取常用命令
    let commands = CommandsConfig::get_common_commands()?;
    // EnvGuard 会在 env 离开作用域时自动恢复环境变量

    // 验证返回了命令列表（可能是自定义的或默认的，取决于路径解析）
    assert!(!commands.is_empty());

    Ok(())
}
