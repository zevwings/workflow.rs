//! 日志文件路径管理模块
//!
//! 负责生成和管理日志文件的路径。

use super::config::ConfigProvider;
use chrono::Local;
use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use std::fs;
use std::path::PathBuf;

/// 获取日志文件路径
///
/// 返回格式：`~/.workflow/logs/tracing/{command}-{timestamp}-{pid}.log`
///
/// # 参数
///
/// * `command_name` - 可选的命令名（如 "pr-create"、"jira-info"），如果为 None，使用 "workflow"
/// * `config` - 配置提供者
///
/// # 返回
///
/// 返回日志文件的完整路径
pub(crate) fn log_file_path(
    command_name: Option<&str>,
    config: &dyn ConfigProvider,
) -> Result<PathBuf> {
    // 获取日志目录（~/.workflow/logs/），强制本地存储
    let logs_dir = config.logs_dir().wrap_err("Failed to get logs directory")?;

    // 创建 tracing 子目录
    let tracing_dir = logs_dir.join("tracing");
    fs::create_dir_all(&tracing_dir)
        .wrap_err_with(|| format!("Failed to create tracing directory: {:?}", tracing_dir))?;

    // 生成时间戳（YYYYMMDDHHMMSS 格式）
    let timestamp = Local::now().format("%Y%m%d%H%M%S");
    let pid = std::process::id();

    // 确定命令名前缀
    let command_prefix = command_name.unwrap_or("workflow");

    // 文件命名：{command}-{timestamp}-{pid}.log
    let log_file = tracing_dir.join(format!("{}-{}-{}.log", command_prefix, timestamp, pid));

    Ok(log_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::logger::LogLevel;

    /// 测试用的配置提供者
    struct TestConfigProvider {
        logs_dir: PathBuf,
    }

    impl TestConfigProvider {
        fn new() -> Self {
            Self {
                logs_dir: std::env::temp_dir().join("workflow_test_logs"),
            }
        }
    }

    impl ConfigProvider for TestConfigProvider {
        fn log_level(&self) -> Option<LogLevel> {
            Some(LogLevel::Info)
        }
        fn log_format(&self) -> Option<String> {
            None
        }
        fn enable_console(&self) -> bool {
            false
        }
        fn logs_dir(&self) -> color_eyre::Result<PathBuf> {
            Ok(self.logs_dir.clone())
        }
    }

    /// 测试日志文件路径生成
    #[test]
    fn test_log_file_path_with_command() {
        let command_name = Some("pr-create");
        let config = TestConfigProvider::new();
        let result = log_file_path(command_name, &config);
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("pr-create"));
        assert!(path.to_string_lossy().ends_with(".log"));

        let filename = path.file_name().unwrap().to_string_lossy();
        let parts: Vec<&str> = filename.split('-').collect();
        assert_eq!(parts[0], "pr");
        assert_eq!(parts[1], "create");
        assert!(parts.len() >= 4);
    }

    /// 测试日志文件路径生成（无命令名）
    #[test]
    fn test_log_file_path_without_command() {
        let config = TestConfigProvider::new();
        let result = log_file_path(None, &config);
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("workflow"));
        assert!(path.to_string_lossy().ends_with(".log"));

        let filename = path.file_name().unwrap().to_string_lossy();
        let parts: Vec<&str> = filename.split('-').collect();
        assert_eq!(parts[0], "workflow");
        assert!(parts.len() >= 3);
    }

    /// 测试日志文件路径包含时间戳和 PID
    #[test]
    fn test_log_file_path_contains_timestamp_and_pid() {
        let config = TestConfigProvider::new();
        let result = log_file_path(Some("test-command"), &config);
        assert!(result.is_ok());

        let path = result.unwrap();
        let filename = path.file_name().unwrap().to_string_lossy();

        let has_timestamp = filename
            .split('-')
            .any(|part| part.len() == 14 && part.chars().all(|c| c.is_ascii_digit()));
        assert!(
            has_timestamp,
            "Filename should contain 14-digit timestamp: {}",
            filename
        );

        assert!(
            filename.ends_with(".log"),
            "Filename should end with .log: {}",
            filename
        );
        let pid_part = filename.strip_suffix(".log").unwrap().split('-').last().unwrap();
        assert!(
            pid_part.chars().all(|c| c.is_ascii_digit()),
            "Last part before .log should be PID (digits): {}",
            filename
        );
    }

    /// 测试日志目录创建
    #[test]
    fn test_log_directory_creation() {
        let config = TestConfigProvider::new();
        let result = log_file_path(Some("test"), &config);
        assert!(result.is_ok());

        let path = result.unwrap();
        let parent = path.parent().unwrap();

        assert!(parent.exists(), "Log directory should exist: {:?}", parent);

        let expected_dir = config.logs_dir().unwrap().join("tracing");
        assert_eq!(parent, expected_dir);
    }

    /// 测试不同命令名的路径生成
    #[test]
    fn test_different_command_names() {
        let commands = vec![
            ("pr-create", "pr-create"),
            ("jira-info", "jira-info"),
            ("jira-log-download", "jira-log-download"),
            ("branch-create", "branch-create"),
        ];

        let config = TestConfigProvider::new();
        for (command, expected_prefix) in commands {
            let result = log_file_path(Some(command), &config);
            assert!(
                result.is_ok(),
                "Should generate path for command: {}",
                command
            );

            let path = result.unwrap();
            let filename = path.file_name().unwrap().to_string_lossy();
            assert!(
                filename.starts_with(expected_prefix),
                "Filename should start with {}: {}",
                expected_prefix,
                filename
            );
        }
    }
}
