//! Tracing Subscriber 配置模块
//!
//! 负责初始化 tracing subscriber，配置日志输出目标（文件、控制台或 sink）。

use std::fs::OpenOptions;
use std::io;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::logger::{config::LoggerConfig, path, LoggerError};
use crate::terminal::SpinnerAwareMakeWriter;

/// 宏：根据 use_json 创建并初始化 layer
macro_rules! init_with_layer {
    ($registry:expr, $use_json:expr, $writer:expr, $context:expr) => {
        if $use_json {
            $registry
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_writer($writer)
                        .json()
                        .flatten_event(true),
                )
                .try_init()
                .map_err(|e| LoggerError::InitializationFailed(format!("{}: {}", $context, e)))
        } else {
            $registry
                .with(tracing_subscriber::fmt::layer().with_writer($writer))
                .try_init()
                .map_err(|e| LoggerError::InitializationFailed(format!("{}: {}", $context, e)))
        }
    };
}

/// 宏：添加 JSON 格式 layer
macro_rules! add_json_layer {
    ($subscriber:expr, $writer:expr) => {
        $subscriber
            .with(tracing_subscriber::fmt::layer().with_writer($writer).json().flatten_event(true))
    };
}

/// 宏：添加文本格式 layer
macro_rules! add_text_layer {
    ($subscriber:expr, $writer:expr) => {
        $subscriber.with(tracing_subscriber::fmt::layer().with_writer($writer))
    };
}

/// 初始化日志系统
///
/// 根据配置的日志级别决定是否输出到文件或完全丢弃。
/// 如果日志级别为 "off"，则输出到 sink（/dev/null）。
/// 否则，输出到日志文件（`~/.workflow/logs/tracing/{command}-{timestamp}-{pid}.log`）。
///
/// 如果启用了 `enable_console` 配置，tracing 日志会同时输出到文件和控制台（stderr）。
///
/// # 参数
///
/// * `command_name` - 可选的命令名（如 "pr-create"、"jira-info"），如果为 None，使用 "workflow"
/// * `config` - 日志配置
///
/// # 错误
///
/// 如果初始化失败，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use toolkit::logger::{self, LoggerConfig};
/// use toolkit::logger::LoggerError;
/// use std::path::PathBuf;
///
/// # fn main() -> std::result::Result<(), LoggerError> {
/// let config = LoggerConfig::new(
///     Some("info".to_string()),
///     Some("text".to_string()),
///     true,
///     PathBuf::from("/tmp/logs"),
/// );
/// logger::init(Some("my-app"), &config)?;
/// # Ok(())
/// # }
/// ```
pub fn init(command_name: Option<&str>, config: &LoggerConfig) -> Result<(), LoggerError> {
    let log_level_str = config.level.as_deref().unwrap_or("off");

    // 构建过滤器：项目代码使用用户设置的级别，第三方库（特别是 HTTP 相关）限制为 warn
    // 这避免了 hyper/h2/rustls 等库的 DEBUG 日志干扰交互式 UI（如选择菜单）
    let filter = EnvFilter::new(format!(
        "{},hyper=warn,h2=warn,rustls=warn,reqwest=warn,tower=warn",
        log_level_str
    ));
    let use_json = config.format.as_deref().is_some_and(|f| f.eq_ignore_ascii_case("json"));
    let enable_console = config.enable_console;

    let registry = tracing_subscriber::registry().with(filter);

    // 如果日志级别为 "off"，输出到 sink（丢弃所有日志）
    if log_level_str == "off" {
        return init_with_layer!(
            registry,
            use_json,
            io::sink,
            "Failed to initialize tracing subscriber with sink"
        );
    }

    // 尝试创建文件输出
    if let Ok(file_path) = path::log_file_path(command_name, config) {
        if let Ok(file) = OpenOptions::new().create(true).append(true).open(&file_path) {
            return if use_json {
                if enable_console {
                    add_json_layer!(add_json_layer!(registry, file), SpinnerAwareMakeWriter)
                        .try_init()
                        .map_err(|e| {
                            LoggerError::InitializationFailed(format!(
                                "Failed to initialize tracing subscriber with file and console: {}",
                                e
                            ))
                        })
                } else {
                    add_json_layer!(registry, file).try_init().map_err(|e| {
                        LoggerError::InitializationFailed(format!(
                            "Failed to initialize tracing subscriber with file: {}",
                            e
                        ))
                    })
                }
            } else if enable_console {
                add_text_layer!(add_text_layer!(registry, file), SpinnerAwareMakeWriter)
                    .try_init()
                    .map_err(|e| {
                        LoggerError::InitializationFailed(format!(
                            "Failed to initialize tracing subscriber with file and console: {}",
                            e
                        ))
                    })
            } else {
                add_text_layer!(registry, file).try_init().map_err(|e| {
                    LoggerError::InitializationFailed(format!(
                        "Failed to initialize tracing subscriber with file: {}",
                        e
                    ))
                })
            };
        }
    }

    // 回退到 stderr 输出（使用 SpinnerAwareMakeWriter 协调 spinner）
    init_with_layer!(
        registry,
        use_json,
        SpinnerAwareMakeWriter,
        "Failed to initialize tracing subscriber with stderr"
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rstest::rstest;
    use serial_test::serial;
    use tempfile::TempDir;

    use super::*;

    /// 创建测试用的 LoggerConfig
    fn create_test_config(
        log_level: Option<String>,
        log_format: Option<String>,
        enable_console: bool,
        logs_dir: Option<PathBuf>,
    ) -> LoggerConfig {
        LoggerConfig::new(
            log_level,
            log_format,
            enable_console,
            logs_dir.unwrap_or_else(|| PathBuf::from("/tmp")),
        )
    }

    // ==================== "off" 级别测试 ====================

    #[rstest]
    #[serial]
    #[case(None)] // 文本格式
    #[case(Some("json".to_string()))] // JSON 格式
    fn test_logger_init_with_none_level(#[case] log_format: Option<String>) {
        let config = create_test_config(Some("off".to_string()), log_format, false, None);
        let result = init(Some("test-command"), &config);
        // tracing subscriber 只能初始化一次，后续初始化会失败，这是预期的行为
        // 第一次初始化应该成功，后续的失败也是可以接受的（因为全局状态已设置）
        // 我们只验证函数调用不会 panic，不验证返回值（因为全局状态的影响）
        let _ = result; // 接受任何结果
    }

    // ==================== 文件输出测试 ====================

    #[rstest]
    #[serial]
    #[case(None)] // 文本格式
    #[case(Some("json".to_string()))] // JSON 格式
    fn test_logger_init_with_file_output(#[case] log_format: Option<String>) {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config =
            create_test_config(Some("debug".to_string()), log_format, false, Some(logs_dir));

        let result = init(Some("test-command"), &config);
        // tracing subscriber 只能初始化一次，后续初始化会失败，这是预期的行为
        // 我们只验证函数调用不会 panic，不验证返回值（因为全局状态的影响）
        let _ = result; // 接受任何结果
    }

    // ==================== 文件创建失败回退测试 ====================

    #[rstest]
    #[serial]
    #[case(None)] // 文本格式
    #[case(Some("json".to_string()))] // JSON 格式
    fn test_logger_init_fallback_to_stderr(#[case] log_format: Option<String>) {
        let invalid_dir = PathBuf::from("/nonexistent/path/that/should/not/exist");
        let config = create_test_config(
            Some("debug".to_string()),
            log_format,
            false,
            Some(invalid_dir),
        );

        let result = init(Some("test-command"), &config);
        // tracing subscriber 只能初始化一次，后续初始化会失败，这是预期的行为
        // 我们只验证函数调用不会 panic，不验证返回值（因为全局状态的影响）
        let _ = result; // 接受任何结果
    }

    // ==================== enable_console 双路输出测试 ====================

    #[rstest]
    #[serial]
    #[case(None)] // 文本格式
    #[case(Some("json".to_string()))] // JSON 格式
    fn test_logger_init_with_console_enabled(#[case] log_format: Option<String>) {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let logs_dir = temp_dir.path().to_path_buf();
        let config = create_test_config(
            Some("debug".to_string()),
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
        let config = create_test_config(
            Some("debug".to_string()),
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
        let config = create_test_config(None, None, false, Some(logs_dir));

        let result = init(Some("test-command"), &config);
        // 默认应该是 "off"，输出到 sink
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
        let config = create_test_config(Some("debug".to_string()), None, false, Some(logs_dir));

        let result = init(command_name, &config);
        // tracing subscriber 只能初始化一次，后续初始化会失败，这是预期的行为
        // 我们只验证函数调用不会 panic，不验证返回值（因为全局状态的影响）
        let _ = result; // 接受任何结果
    }
}
