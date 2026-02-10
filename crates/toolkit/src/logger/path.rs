//! 日志文件路径管理模块
//!
//! 负责生成和管理日志文件的路径。

use crate::logger::{config::LoggerConfig, LoggerError};
use chrono::Local;
use std::fs;
use std::path::PathBuf;

/// 获取日志文件路径
///
/// 返回格式：`~/.workflow/logs/tracing/{command}-{timestamp}-{pid}.log`
///
/// # 参数
///
/// * `command_name` - 可选的命令名（如 "pr-create"、"jira-info"），如果为 None，使用 "workflow"
/// * `config` - 日志配置
///
/// # 返回
///
/// 返回日志文件的完整路径
pub(crate) fn log_file_path(
    command_name: Option<&str>,
    config: &LoggerConfig,
) -> Result<PathBuf, LoggerError> {
    // 获取日志目录（~/.workflow/logs/），强制本地存储
    let logs_dir = &config.logs_dir;

    // 创建 tracing 子目录
    fs::create_dir_all(logs_dir).map_err(|e| {
        LoggerError::CreateDirectoryFailed(format!(
            "Failed to create tracing directory {:?}: {}",
            logs_dir, e
        ))
    })?;

    // 生成时间戳（YYYYMMDDHHMMSS 格式）
    let timestamp = Local::now().format("%Y%m%d%H%M%S");
    let pid = std::process::id();

    // 确定命令名前缀
    let command_prefix = command_name.unwrap_or("workflow");

    // 文件命名：{command}-{timestamp}-{pid}.log
    let log_file = logs_dir.join(format!("{}-{}-{}.log", command_prefix, timestamp, pid));

    Ok(log_file)
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use super::*;

    /// 创建测试用的 LoggerConfig
    fn create_test_config(logs_dir: PathBuf) -> LoggerConfig {
        LoggerConfig::new(Some("info".to_string()), None, false, logs_dir)
    }

    /// Fixture: 测试用的日志配置
    #[fixture]
    fn test_config() -> LoggerConfig {
        create_test_config(std::env::temp_dir().join("workflow_test_logs"))
    }

    /// 测试日志文件路径生成
    #[rstest]
    fn test_log_file_path_with_command(test_config: LoggerConfig) {
        let command_name = Some("pr-create");
        let result = log_file_path(command_name, &test_config);
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
    #[rstest]
    fn test_log_file_path_without_command(test_config: LoggerConfig) {
        let result = log_file_path(None, &test_config);
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
    #[rstest]
    fn test_log_file_path_contains_timestamp_and_pid(test_config: LoggerConfig) {
        let result = log_file_path(Some("test-command"), &test_config);
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
        let pid_part = filename.strip_suffix(".log").unwrap().split('-').next_back().unwrap();
        assert!(
            pid_part.chars().all(|c| c.is_ascii_digit()),
            "Last part before .log should be PID (digits): {}",
            filename
        );
    }

    /// 测试日志目录创建
    #[rstest]
    fn test_log_directory_creation(test_config: LoggerConfig) {
        let result = log_file_path(Some("test"), &test_config);
        assert!(result.is_ok());

        let path = result.unwrap();
        let parent = path.parent().unwrap();

        assert!(parent.exists(), "Log directory should exist: {:?}", parent);

        // 实现将日志文件直接放在 logs_dir，无 tracing 子目录
        assert_eq!(parent, test_config.logs_dir);
    }

    /// 测试不同命令名的路径生成
    #[rstest]
    #[case("pr-create", "pr-create")]
    #[case("jira-info", "jira-info")]
    #[case("jira-log-download", "jira-log-download")]
    #[case("branch-create", "branch-create")]
    fn test_different_command_names(
        test_config: LoggerConfig,
        #[case] command: &str,
        #[case] expected_prefix: &str,
    ) {
        let result = log_file_path(Some(command), &test_config);
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
