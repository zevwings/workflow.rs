use prompt::terminal_state;
use toolkit::{log_info, logger, register_spinner_handlers, LoggerConfig};

use crate::cli::Command;
use crate::registry::{get_global_config_repository, get_path_service};

pub fn setup_logger(command: &Command) -> Result<(), Box<dyn std::error::Error>> {
    let global_config_repository = get_global_config_repository();
    let global_config = global_config_repository.load()?;

    // RUST_LOG 优先于配置文件，便于调试时用 RUST_LOG=debug 看详细日志
    let level = std::env::var("RUST_LOG")
        .ok()
        .filter(|s| !s.is_empty())
        .or(global_config.log.level.clone());

    let path_service = get_path_service();
    let logger_config = LoggerConfig::new(
        level,
        global_config.log.format.clone(),
        global_config.log.enable_trace_console.unwrap_or(false),
        path_service.get_logs_dir()?,
    );

    // 初始化 logger（从配置文件读取 LogSettings 并转换为 LoggerConfig）
    // 注意：如果配置加载失败或 logger 初始化失败，我们继续执行（不阻塞应用启动）
    let command_name = get_command_name(command);

    // 忽略初始化错误（可能已经初始化过了，或者日志级别为 off）
    let _ = logger::init(command_name, &logger_config);

    // 注册终端处理器，让 tracing 输出时能协调 spinner/progress
    register_spinner_handlers(terminal_state::suspend, terminal_state::resume);

    log_info!(
        "Logger initialized (console={}, level={})",
        logger_config.enable_console,
        logger_config.level.as_deref().unwrap_or("off")
    );
    Ok(())
}

/// 获取命令名称字符串（用于日志文件名）
fn get_command_name(command: &Command) -> Option<&'static str> {
    match command {
        Command::Version => Some("version"),
        Command::Check => Some("check"),
        Command::Setup => Some("setup"),
        Command::Update(_) => Some("update"),
        Command::Uninstall(_) => Some("uninstall"),
        Command::Repo(_) => Some("repo"),
        Command::Log(_) => Some("log"),
        Command::Llm(_) => Some("llm"),
        Command::Github(_) => Some("github"),
        Command::Jira(_) => Some("jira"),
        Command::Branch(_) => Some("branch"),
        #[cfg(feature = "develop")]
        Command::Commit(_) => Some("commit"),
        Command::Stash(_) => Some("stash"),
        Command::Tag(_) => Some("tag"),
        Command::Pr(_) => Some("pr"),
        #[cfg(feature = "develop")]
        Command::Push => Some("push"),
        #[cfg(feature = "develop")]
        Command::Pull => Some("pull"),
        Command::Completion(_) => Some("completion"),
        Command::Alias(_) => Some("alias"),
        #[cfg(feature = "develop")]
        Command::Rollback(_) => Some("rollback"),
    }
}
