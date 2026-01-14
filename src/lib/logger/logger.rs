//! Tracing 封装模块
//!
//! 本模块提供了对 tracing 库的封装，用于 lib 层的结构化日志记录。
//! 通过封装，如果未来需要替换为其他日志库，只需要修改本模块即可。
//!
//! ## 设计原则
//!
//! 1. **职责分离**：
//!    - Lib 层使用 `log_*!` 宏进行结构化日志记录（不直接输出到控制台）
//!    - Commands 层使用 `log_*!` 宏进行用户友好的控制台输出
//!
//! 2. **默认行为**：
//!    - 默认情况下，tracing 不输出到控制台（通过配置控制）
//!    - 可以通过环境变量 `RUST_LOG` 启用调试输出到 stderr
//!
//! 3. **可替换性**：
//!    - 所有 lib 层代码使用 `log_*!` 宏，而不是直接使用 `tracing::*`
//!    - 如果未来需要替换日志库，只需要修改本模块的实现
//!
//! 4. **自动模块识别**：
//!    - 所有 `log_*!` 宏自动包含模块信息作为日志字段（`module={module_name}`）
//!    - 模块名从 `module_path!()` 自动提取
//!
//! 5. **每次操作独立日志文件**：
//!    - 每次命令执行创建独立的日志文件（`{command}-{timestamp}-{pid}.log`）
//!    - 避免日志文件无限增长，便于追踪单次操作的完整日志
//!
//! ## 使用示例
//!
//! ### 基础日志记录
//!
//! ```rust
//! use workflow::{log_debug, log_info, log_warn, log_error};
//!
//! let data = "test data";
//! log_debug!("Processing data: {}", data);
//! log_info!("Operation completed");
//! log_warn!("Retrying operation");
//! let error = "connection failed";
//! log_error!("Operation failed: {}", error);
//! ```
//!
//! ### 带结构化字段的日志记录
//!
//! ```rust
//! use workflow::log_info_with_fields;
//!
//! log_info_with_fields!(
//!     user_id = 123,
//!     request_id = "abc-123",
//!     duration_ms = 45,
//!     "Request processed"
//! );
//! ```
//!
//! ### 带错误的日志记录
//!
//! ```rust
//! use workflow::log_error_with_fields;
//!
//! let error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
//! log_error_with_fields!(
//!     file_path = "/path/to/file",
//!     error = ?error,
//!     "Failed to open file"
//! );
//! ```
//!
//! ## 初始化
//!
//! ```rust
//! use workflow::Logger;
//!
//! // 从配置文件读取日志级别并初始化（无命令名）
//! Logger::init();
//!
//! // 使用命令名初始化（推荐，每次操作创建独立日志文件）
//! // 需要提供配置提供者（由调用方创建适配器）
//! use workflow::infra::adapters::config::SettingsAdapter;
//! let config = SettingsAdapter::new();
//! Logger::init_with_command(Some("pr-create"), &config);
//! ```

use crate::logger::LogConfigProvider;
use crate::util::directory::DirectoryWalker;
use crate::LogLevel;
use chrono::Local;
use color_eyre::eyre::WrapErr;
use std::fs::OpenOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Logger 封装结构体
///
/// 提供统一的日志接口，内部使用 tracing crate。
/// 如果未来需要替换为其他日志库，只需要修改本结构体的实现。
pub struct Logger;

impl Logger {
    /// 初始化 tracing subscriber（从配置读取日志级别）
    ///
    /// **注意**：此方法已废弃，请使用 `init_with_command` 并提供配置提供者。
    /// 此方法会使用默认配置（日志级别为 None，不输出日志）。
    #[deprecated(note = "请使用 init_with_command 并提供配置提供者")]
    pub fn init() {
        // 使用默认配置（不输出日志）
        struct DefaultConfig;
        impl LogConfigProvider for DefaultConfig {
            fn get_log_level(&self) -> Option<crate::LogLevel> {
                Some(crate::LogLevel::None)
            }
            fn get_log_format(&self) -> Option<String> {
                None
            }
            fn get_enable_console(&self) -> bool {
                false
            }
            fn get_logs_dir(&self) -> color_eyre::Result<std::path::PathBuf> {
                // 这里不应该被调用，因为日志级别为 None
                Err(color_eyre::eyre::eyre!(
                    "Logs directory not needed when log level is None"
                ))
            }
        }
        let default_config = DefaultConfig;
        Self::init_with_command(None, &default_config);
    }

    /// 初始化 tracing subscriber（从配置读取日志级别，支持命令名）
    ///
    /// 根据配置的日志级别决定是否输出到文件或完全丢弃。
    /// 如果日志级别为 "off"，则输出到 sink（/dev/null）。
    /// 否则，输出到日志文件（`~/.workflow/logs/tracing/{command}-{timestamp}-{pid}.log`）。
    ///
    /// 如果启用了 `enable_trace_console` 配置（为 `true`），tracing 日志会同时输出到文件和控制台（stderr）。
    /// 如果配置文件中不存在此字段（为 `None`），默认为 `false`（只输出到文件）。
    ///
    /// 日志级别从配置提供者读取。如果配置文件中未设置，则默认使用 "off"（不输出）。
    ///
    /// # 参数
    ///
    /// * `command_name` - 可选的命令名（如 "pr-create"、"jira-info"），如果为 None，使用 "workflow"
    /// * `config_provider` - 配置提供者，必须提供（由调用方创建适配器实例）
    pub fn init_with_command(command_name: Option<&str>, config_provider: &dyn LogConfigProvider) {
        let config = config_provider;

        let log_level = config.get_log_level().unwrap_or(LogLevel::None);

        let tracing_filter = log_level.as_str();
        let use_json = config
            .get_log_format()
            .as_deref()
            .map(|f| f.to_lowercase() == "json")
            .unwrap_or(false);

        if log_level != LogLevel::None {
            let enable_console = config.get_enable_console();

            if let Ok(file_path) = Self::get_log_file_path_internal(command_name, config) {
                if let Ok(file) = OpenOptions::new().create(true).append(true).open(&file_path) {
                    let registry =
                        tracing_subscriber::registry().with(EnvFilter::new(tracing_filter));

                    if use_json {
                        let registry = registry.with(
                            tracing_subscriber::fmt::layer()
                                .with_writer(file)
                                .json()
                                .flatten_event(true),
                        );
                        let _ = if enable_console {
                            registry
                                .with(
                                    tracing_subscriber::fmt::layer()
                                        .with_writer(std::io::stderr)
                                        .json()
                                        .flatten_event(true),
                                )
                                .try_init()
                        } else {
                            registry.try_init()
                        };
                    } else {
                        let registry =
                            registry.with(tracing_subscriber::fmt::layer().with_writer(file));
                        let _ = if enable_console {
                            registry
                                .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
                                .try_init()
                        } else {
                            registry.try_init()
                        };
                    }
                    return;
                }
            }

            let registry = tracing_subscriber::registry().with(EnvFilter::new(tracing_filter));
            if use_json {
                let _ = registry
                    .with(
                        tracing_subscriber::fmt::layer()
                            .with_writer(std::io::stderr)
                            .json()
                            .flatten_event(true),
                    )
                    .try_init();
            } else {
                let _ = registry
                    .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
                    .try_init();
            }
        } else {
            let _ = tracing_subscriber::registry()
                .with(EnvFilter::new(tracing_filter))
                .with(tracing_subscriber::fmt::layer().with_writer(std::io::sink))
                .try_init();
        }
    }

    /// 获取日志文件路径（测试用）
    ///
    /// 返回格式：`~/.workflow/logs/tracing/{command}-{timestamp}-{pid}.log`
    #[cfg(test)]
    pub fn get_log_file_path(
        command_name: Option<&str>,
        config: &dyn LogConfigProvider,
    ) -> color_eyre::Result<std::path::PathBuf> {
        Self::get_log_file_path_internal(command_name, config)
    }

    /// 获取日志文件路径（内部实现）
    fn get_log_file_path_internal(
        command_name: Option<&str>,
        config: &dyn LogConfigProvider,
    ) -> color_eyre::Result<std::path::PathBuf> {
        // 获取日志目录（~/.workflow/logs/），强制本地存储
        let logs_dir = config.get_logs_dir().wrap_err("Failed to get logs directory")?;

        // 创建 tracing 子目录
        let tracing_dir = logs_dir.join("tracing");
        DirectoryWalker::new(&tracing_dir).ensure_exists()?;

        // 生成时间戳（YYYYMMDDHHMMSS 格式）
        let timestamp = Local::now().format("%Y%m%d%H%M%S");
        let pid = std::process::id();

        // 确定命令名前缀
        let command_prefix = command_name.unwrap_or("workflow");

        // 文件命名：{command}-{timestamp}-{pid}.log
        let log_file = tracing_dir.join(format!("{}-{}-{}.log", command_prefix, timestamp, pid));

        Ok(log_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试用的配置提供者
    struct TestConfigProvider;
    impl LogConfigProvider for TestConfigProvider {
        fn get_log_level(&self) -> Option<crate::LogLevel> {
            Some(crate::LogLevel::Info)
        }
        fn get_log_format(&self) -> Option<String> {
            None
        }
        fn get_enable_console(&self) -> bool {
            false
        }
        fn get_logs_dir(&self) -> color_eyre::Result<std::path::PathBuf> {
            crate::settings::paths::Paths::logs_dir()
        }
    }

    /// 测试日志文件路径生成
    #[test]
    fn test_get_log_file_path_with_command() {
        // 测试带命令名的路径生成
        let command_name = Some("pr-create");
        let config = TestConfigProvider;
        let result = Logger::get_log_file_path(command_name, &config);
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("pr-create"));
        assert!(path.to_string_lossy().ends_with(".log"));

        // 验证文件路径格式：{command}-{timestamp}-{pid}.log
        let filename = path.file_name().unwrap().to_string_lossy();
        let parts: Vec<&str> = filename.split('-').collect();
        assert_eq!(parts[0], "pr");
        assert_eq!(parts[1], "create");
        assert!(parts.len() >= 4); // pr-create-timestamp-pid.log
    }

    /// 测试日志文件路径生成（无命令名）
    #[test]
    fn test_get_log_file_path_without_command() {
        // 测试无命令名的路径生成
        let config = TestConfigProvider;
        let result = Logger::get_log_file_path(None, &config);
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("workflow"));
        assert!(path.to_string_lossy().ends_with(".log"));

        // 验证文件路径格式：workflow-{timestamp}-{pid}.log
        let filename = path.file_name().unwrap().to_string_lossy();
        let parts: Vec<&str> = filename.split('-').collect();
        assert_eq!(parts[0], "workflow");
        assert!(parts.len() >= 3); // workflow-timestamp-pid.log
    }

    /// 测试日志文件路径包含时间戳和 PID
    #[test]
    fn test_log_file_path_contains_timestamp_and_pid() {
        let config = TestConfigProvider;
        let result = Logger::get_log_file_path(Some("test-command"), &config);
        assert!(result.is_ok());

        let path = result.unwrap();
        let filename = path.file_name().unwrap().to_string_lossy();

        // 验证文件名包含时间戳（14位数字：YYYYMMDDHHMMSS）
        // 使用简单的字符串检查，因为 regex 可能不在测试依赖中
        let has_timestamp = filename
            .split('-')
            .any(|part| part.len() == 14 && part.chars().all(|c| c.is_ascii_digit()));
        assert!(
            has_timestamp,
            "Filename should contain 14-digit timestamp: {}",
            filename
        );

        // 验证文件名以 .log 结尾，且前面有数字（PID）
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
        let config = TestConfigProvider;
        let result = Logger::get_log_file_path(Some("test"), &config);
        assert!(result.is_ok());

        let path = result.unwrap();
        let parent = path.parent().unwrap();

        // 验证目录存在
        assert!(parent.exists(), "Log directory should exist: {:?}", parent);

        // 验证目录路径正确（使用测试配置）
        struct TestConfig;
        impl LogConfigProvider for TestConfig {
            fn get_log_level(&self) -> Option<crate::LogLevel> {
                Some(crate::LogLevel::Info)
            }
            fn get_log_format(&self) -> Option<String> {
                None
            }
            fn get_enable_console(&self) -> bool {
                false
            }
            fn get_logs_dir(&self) -> color_eyre::Result<std::path::PathBuf> {
                crate::settings::paths::Paths::logs_dir()
            }
        }
        let test_config = TestConfig;
        let expected_dir =
            test_config.get_logs_dir().expect("Should get logs directory").join("tracing");
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

        let config = TestConfigProvider;
        for (command, expected_prefix) in commands {
            let result = Logger::get_log_file_path(Some(command), &config);
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
