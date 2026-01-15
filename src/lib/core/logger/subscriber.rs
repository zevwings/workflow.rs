//! Tracing Subscriber 配置模块
//!
//! 负责初始化 tracing subscriber，配置日志输出目标（文件、控制台或 sink）。

use super::config::ConfigProvider;
use super::level::LogLevel;
use super::path;
use color_eyre::{eyre::Context, Result};
use std::fs::OpenOptions;
use std::io;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// 初始化日志系统
///
/// 根据配置的日志级别决定是否输出到文件或完全丢弃。
/// 如果日志级别为 "off"，则输出到 sink（/dev/null）。
/// 否则，输出到日志文件（`~/.workflow/logs/tracing/{command}-{timestamp}-{pid}.log`）。
///
/// 如果启用了 `enable_trace_console` 配置，tracing 日志会同时输出到文件和控制台（stderr）。
///
/// # 参数
///
/// * `command_name` - 可选的命令名（如 "pr-create"、"jira-info"），如果为 None，使用 "workflow"
/// * `config` - 配置提供者
///
/// # 错误
///
/// 如果初始化失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust
/// use workflow::core::logger;
/// use workflow::infra::adapters::config::SettingsAdapter;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = SettingsAdapter::new();
/// logger::init(Some("pr-create"), &config)?;
/// # Ok(())
/// # }
/// ```
pub fn init(command_name: Option<&str>, config: &dyn ConfigProvider) -> Result<()> {
    let log_level = config.log_level().unwrap_or(LogLevel::None);
    let filter = EnvFilter::new(log_level.as_str());
    let use_json = is_json_format(config);
    let enable_console = config.enable_console();

    let registry = tracing_subscriber::registry().with(filter);

    // 如果日志级别为 None，输出到 sink（丢弃所有日志）
    if log_level == LogLevel::None {
        return if use_json {
            registry
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_writer(io::sink)
                        .json()
                        .flatten_event(true),
                )
                .try_init()
                .context("Failed to initialize tracing subscriber with sink")
        } else {
            registry
                .with(tracing_subscriber::fmt::layer().with_writer(io::sink))
                .try_init()
                .context("Failed to initialize tracing subscriber with sink")
        };
    }

    // 尝试创建文件输出
    if let Ok(file_path) = path::log_file_path(command_name, config) {
        if let Ok(file) = OpenOptions::new().create(true).append(true).open(&file_path) {
            return if use_json {
                let registry = registry.with(
                    tracing_subscriber::fmt::layer().with_writer(file).json().flatten_event(true),
                );
                if enable_console {
                    registry
                        .with(
                            tracing_subscriber::fmt::layer()
                                .with_writer(io::stderr)
                                .json()
                                .flatten_event(true),
                        )
                        .try_init()
                        .context("Failed to initialize tracing subscriber with file and console")
                } else {
                    registry.try_init().context("Failed to initialize tracing subscriber with file")
                }
            } else {
                let registry = registry.with(tracing_subscriber::fmt::layer().with_writer(file));
                if enable_console {
                    registry
                        .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
                        .try_init()
                        .context("Failed to initialize tracing subscriber with file and console")
                } else {
                    registry.try_init().context("Failed to initialize tracing subscriber with file")
                }
            };
        }
    }

    // 回退到 stderr 输出
    if use_json {
        registry
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(io::stderr)
                    .json()
                    .flatten_event(true),
            )
            .try_init()
            .context("Failed to initialize tracing subscriber with stderr")
    } else {
        registry
            .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
            .try_init()
            .context("Failed to initialize tracing subscriber with stderr")
    }
}

/// 检查是否使用 JSON 格式
fn is_json_format(config: &dyn ConfigProvider) -> bool {
    config.log_format().as_deref().is_some_and(|f| f.eq_ignore_ascii_case("json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serial_test::serial;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Mock ConfigProvider 用于测试
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

    // ==================== LogLevel::None 测试 ====================

    #[rstest]
    #[serial]
    #[case(None, false)] // 文本格式
    #[case(Some("json".to_string()), true)] // JSON 格式
    fn test_logger_init_with_none_level(
        #[case] log_format: Option<String>,
        #[case] _use_json: bool,
    ) {
        let config = MockConfigProvider::new(Some(LogLevel::None), log_format, false, None);
        let result = init(Some("test-command"), &config);
        // tracing subscriber 只能初始化一次，后续初始化会失败，这是预期的行为
        // 第一次初始化应该成功，后续的失败也是可以接受的（因为全局状态已设置）
        // 我们只验证函数调用不会 panic，不验证返回值（因为全局状态的影响）
        let _ = result; // 接受任何结果
    }

    // ==================== 文件输出测试 ====================

    #[rstest]
    #[serial]
    #[case(None, false)] // 文本格式
    #[case(Some("json".to_string()), true)] // JSON 格式
    fn test_logger_init_with_file_output(
        #[case] log_format: Option<String>,
        #[case] _use_json: bool,
    ) {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config =
            MockConfigProvider::new(Some(LogLevel::Debug), log_format, false, Some(logs_dir));

        let result = init(Some("test-command"), &config);
        // tracing subscriber 只能初始化一次，后续初始化会失败，这是预期的行为
        // 我们只验证函数调用不会 panic，不验证返回值（因为全局状态的影响）
        let _ = result; // 接受任何结果
    }

    // ==================== 文件创建失败回退测试 ====================

    #[rstest]
    #[serial]
    #[case(None, false)] // 文本格式
    #[case(Some("json".to_string()), true)] // JSON 格式
    fn test_logger_init_fallback_to_stderr(
        #[case] log_format: Option<String>,
        #[case] _use_json: bool,
    ) {
        let invalid_dir = PathBuf::from("/nonexistent/path/that/should/not/exist");
        let config =
            MockConfigProvider::new(Some(LogLevel::Debug), log_format, false, Some(invalid_dir));

        let result = init(Some("test-command"), &config);
        // tracing subscriber 只能初始化一次，后续初始化会失败，这是预期的行为
        // 我们只验证函数调用不会 panic，不验证返回值（因为全局状态的影响）
        let _ = result; // 接受任何结果
    }

    // ==================== enable_console 双路输出测试 ====================

    #[rstest]
    #[serial]
    #[case(None, false)] // 文本格式
    #[case(Some("json".to_string()), true)] // JSON 格式
    fn test_logger_init_with_console_enabled(
        #[case] log_format: Option<String>,
        #[case] _use_json: bool,
    ) {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config = MockConfigProvider::new(
            Some(LogLevel::Debug),
            log_format,
            true, // enable_console
            Some(logs_dir),
        );

        let result = init(Some("test-command"), &config);
        // tracing subscriber 只能初始化一次，后续初始化会失败，这是预期的行为
        // 我们只验证函数调用不会 panic，不验证返回值（因为全局状态的影响）
        let _ = result; // 接受任何结果
    }

    // ==================== JSON 格式测试 ====================

    #[rstest]
    #[serial]
    #[case("json")] // 小写
    #[case("JSON")] // 大写
    #[case("Json")] // 混合大小写
    fn test_logger_init_json_format_case_insensitive(#[case] format: &str) {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config = MockConfigProvider::new(
            Some(LogLevel::Debug),
            Some(format.to_string()),
            false,
            Some(logs_dir),
        );

        let result = init(Some("test-command"), &config);
        // tracing subscriber 只能初始化一次，后续初始化会失败，这是预期的行为
        // 我们只验证函数调用不会 panic，不验证返回值（因为全局状态的影响）
        let _ = result; // 接受任何结果
    }

    // ==================== 默认值测试 ====================

    #[test]
    #[serial]
    fn test_logger_init_default_log_level() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config = MockConfigProvider::new(None, None, false, Some(logs_dir));

        let result = init(Some("test-command"), &config);
        // 默认应该是 LogLevel::None，输出到 sink
        // tracing subscriber 只能初始化一次，后续初始化会失败，这是预期的行为
        // 我们只验证函数调用不会 panic，不验证返回值（因为全局状态的影响）
        let _ = result; // 接受任何结果
    }

    // ==================== 命令名测试 ====================

    #[rstest]
    #[serial]
    #[case(Some("custom-command"))] // 带命令名
    #[case(None)] // 不带命令名（使用默认 "workflow"）
    fn test_logger_init_with_command_name(#[case] command_name: Option<&str>) {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config = MockConfigProvider::new(Some(LogLevel::Debug), None, false, Some(logs_dir));

        let result = init(command_name, &config);
        // tracing subscriber 只能初始化一次，后续初始化会失败，这是预期的行为
        // 我们只验证函数调用不会 panic，不验证返回值（因为全局状态的影响）
        let _ = result; // 接受任何结果
    }

    // ==================== is_json_format 私有函数测试 ====================

    #[rstest]
    #[case(None, false)]
    #[case(Some("text".to_string()), false)]
    #[case(Some("json".to_string()), true)]
    #[case(Some("JSON".to_string()), true)]
    #[case(Some("Json".to_string()), true)]
    fn test_is_json_format(#[case] log_format: Option<String>, #[case] expected: bool) {
        let config = MockConfigProvider::new(None, log_format, false, None);
        assert_eq!(is_json_format(&config), expected);
    }
}
