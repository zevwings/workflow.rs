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
