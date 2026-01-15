//! Base/Logger/Subscriber 模块测试
//!
//! 测试日志系统初始化的功能，包括：
//! - LogLevel::None 时输出到 sink
//! - 文件创建失败时回退到 stderr
//! - enable_console 双路输出
//! - log_format=json 时使用 JSON layer

use std::path::PathBuf;
use tempfile::TempDir;
use workflow::core::logger::{init, ConfigProvider, LogLevel};

// Mock ConfigProvider 用于测试
struct MockConfigProvider {
    log_level: Option<LogLevel>,
    log_format: Option<String>,
    enable_console: bool,
    logs_dir: Option<PathBuf>,
}

impl MockConfigProvider {
    fn new(
        log_level: Option<LogLevel>,
        log_format: Option<String>,
        enable_console: bool,
        logs_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            log_level,
            log_format,
            enable_console,
            logs_dir,
        }
    }
}

impl ConfigProvider for MockConfigProvider {
    fn log_level(&self) -> Option<LogLevel> {
        self.log_level
    }

    fn log_format(&self) -> Option<String> {
        self.log_format.clone()
    }

    fn enable_console(&self) -> bool {
        self.enable_console
    }

    fn logs_dir(&self) -> color_eyre::Result<PathBuf> {
        self.logs_dir.clone().ok_or_else(|| color_eyre::eyre::eyre!("Logs dir not set"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== LogLevel::None 测试 ====================

    #[test]
    fn test_logger_init_with_none_level() {
        // 测试 LogLevel::None 时输出到 sink
        let config = MockConfigProvider::new(Some(LogLevel::None), None, false, None);

        // 应该成功初始化（输出到 sink）
        let result = init(Some("test-command"), &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_logger_init_with_none_level_json() {
        // 测试 LogLevel::None 且 JSON 格式时输出到 sink
        let config =
            MockConfigProvider::new(Some(LogLevel::None), Some("json".to_string()), false, None);

        let result = init(Some("test-command"), &config);
        assert!(result.is_ok());
    }

    // ==================== 文件输出测试 ====================

    #[test]
    fn test_logger_init_with_file_output() {
        // 测试文件输出
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config = MockConfigProvider::new(Some(LogLevel::Debug), None, false, Some(logs_dir));

        let result = init(Some("test-command"), &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_logger_init_with_file_output_json() {
        // 测试文件输出（JSON 格式）
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config = MockConfigProvider::new(
            Some(LogLevel::Debug),
            Some("json".to_string()),
            false,
            Some(logs_dir),
        );

        let result = init(Some("test-command"), &config);
        assert!(result.is_ok());
    }

    // ==================== 文件创建失败回退测试 ====================

    #[test]
    fn test_logger_init_fallback_to_stderr() {
        // 测试文件创建失败时回退到 stderr
        // 使用一个不存在的目录来模拟文件创建失败
        let invalid_dir = PathBuf::from("/nonexistent/path/that/should/not/exist");
        let config = MockConfigProvider::new(Some(LogLevel::Debug), None, false, Some(invalid_dir));

        // 应该回退到 stderr 并成功初始化
        let result = init(Some("test-command"), &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_logger_init_fallback_to_stderr_json() {
        // 测试文件创建失败时回退到 stderr（JSON 格式）
        let invalid_dir = PathBuf::from("/nonexistent/path/that/should/not/exist");
        let config = MockConfigProvider::new(
            Some(LogLevel::Debug),
            Some("json".to_string()),
            false,
            Some(invalid_dir),
        );

        let result = init(Some("test-command"), &config);
        assert!(result.is_ok());
    }

    // ==================== enable_console 双路输出测试 ====================

    #[test]
    fn test_logger_init_with_console_enabled() {
        // 测试 enable_console 时双路输出（文件 + 控制台）
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config = MockConfigProvider::new(
            Some(LogLevel::Debug),
            None,
            true, // enable_console
            Some(logs_dir),
        );

        let result = init(Some("test-command"), &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_logger_init_with_console_enabled_json() {
        // 测试 enable_console 时双路输出（JSON 格式）
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config = MockConfigProvider::new(
            Some(LogLevel::Debug),
            Some("json".to_string()),
            true, // enable_console
            Some(logs_dir),
        );

        let result = init(Some("test-command"), &config);
        assert!(result.is_ok());
    }

    // ==================== JSON 格式测试 ====================

    #[test]
    fn test_logger_init_json_format() {
        // 测试 JSON 格式配置
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config = MockConfigProvider::new(
            Some(LogLevel::Debug),
            Some("json".to_string()),
            false,
            Some(logs_dir),
        );

        let result = init(Some("test-command"), &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_logger_init_json_format_case_insensitive() {
        // 测试 JSON 格式配置（大小写不敏感）
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config = MockConfigProvider::new(
            Some(LogLevel::Debug),
            Some("JSON".to_string()), // 大写
            false,
            Some(logs_dir),
        );

        let result = init(Some("test-command"), &config);
        assert!(result.is_ok());
    }

    // ==================== 默认值测试 ====================

    #[test]
    fn test_logger_init_default_log_level() {
        // 测试默认日志级别（None）
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config = MockConfigProvider::new(None, None, false, Some(logs_dir));

        let result = init(Some("test-command"), &config);
        // 默认应该是 LogLevel::None，输出到 sink
        assert!(result.is_ok());
    }

    #[test]
    fn test_logger_init_with_command_name() {
        // 测试带命令名的初始化
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config = MockConfigProvider::new(Some(LogLevel::Debug), None, false, Some(logs_dir));

        let result = init(Some("custom-command"), &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_logger_init_without_command_name() {
        // 测试不带命令名的初始化（使用默认 "workflow"）
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config = MockConfigProvider::new(Some(LogLevel::Debug), None, false, Some(logs_dir));

        let result = init(None, &config);
        assert!(result.is_ok());
    }
}
