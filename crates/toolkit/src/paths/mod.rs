//! 路径管理
//!
//! 统一管理所有路径信息，包括：
//! - 配置文件路径（存储在 `~/.workflow/config/` 目录下）
//! - 安装路径（二进制文件和补全脚本的安装路径和名称）
//! - Completion 目录路径
//!

mod base;
mod config;
mod constants;
mod download;
mod error;
mod expand;
mod info;
mod install;

// 重新导出常量
pub use constants::*;

// 重新导出错误类型
pub use error::PathError;

// 重新导出路径展开功能
pub use expand::expand;

// 重新导出下载路径功能
pub use download::default_download_base_dir;

/// 路径管理器
///
/// 统一管理所有路径信息，包括配置路径、安装路径和 Shell 路径。
///
/// 所有方法都是静态方法，通过 `Paths::method_name()` 调用。
pub struct Paths;

impl Paths {
    // ==================== 路径工具方法 ====================

    /// 展开路径字符串
    ///
    /// 支持的路径格式：
    /// - Unix: `~` 和 `~/path` - 展开为用户主目录
    /// - Unix: `$VAR` 和 `${VAR}` - 展开环境变量
    /// - Windows: `%VAR%` 和 `%VAR%\path` - 展开环境变量
    /// - 绝对路径: 直接使用
    ///
    /// # 示例
    ///
    /// ```text
    /// // Unix
    /// expand("~/Documents/Workflow") -> "/home/user/Documents/Workflow"
    /// expand("~") -> "/home/user"
    /// expand("$HOME/Documents") -> "/home/user/Documents"
    /// expand("${HOME}/Documents") -> "/home/user/Documents"
    ///
    /// // Windows
    /// expand("%USERPROFILE%\\Documents\\Workflow") -> "C:\\Users\\User\\Documents\\Workflow"
    /// expand("%APPDATA%\\workflow") -> "C:\\Users\\User\\AppData\\Roaming\\workflow"
    ///
    /// // 绝对路径
    /// expand("/absolute/path") -> "/absolute/path"
    /// expand("C:\\absolute\\path") -> "C:\\absolute\\path"
    /// ```
    pub fn expand(path_str: &str) -> Result<std::path::PathBuf, PathError> {
        expand::expand(path_str)
    }

    // ==================== 配置路径相关方法 ====================

    /// 获取配置目录路径（支持 iCloud 同步）
    ///
    /// 返回配置文件存储目录。在 macOS 上，如果 iCloud Drive 可用，
    /// 配置将保存到 iCloud 并自动同步到其他设备。
    ///
    /// # 路径示例
    ///
    /// - macOS + iCloud：`~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config/`
    /// - macOS 无 iCloud / 其他系统：`~/.workflow/config/`
    ///
    /// # 返回
    ///
    /// 返回配置目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果环境变量未设置或无法创建目录，返回相应的错误信息。
    pub fn config_dir() -> Result<std::path::PathBuf, PathError> {
        config::config_dir()
    }

    /// 获取主配置文件路径
    ///
    /// 返回 `~/.workflow/config/workflow.toml` 的路径。
    pub fn workflow_config() -> Result<std::path::PathBuf, PathError> {
        config::workflow_config()
    }

    /// 获取 LLM 配置文件路径
    ///
    /// 返回 `~/.workflow/config/llm.toml` 的路径。
    pub fn llm_config() -> Result<std::path::PathBuf, PathError> {
        config::llm_config()
    }

    /// 获取 Jira 配置文件路径
    ///
    /// 返回 `~/.workflow/config/jira.toml` 的路径。
    /// 这是合并后的 Jira 配置文件，包含用户和状态配置。
    pub fn jira_config() -> Result<std::path::PathBuf, PathError> {
        config::jira_config()
    }

    /// 获取常用命令配置文件路径
    ///
    /// 返回 `~/.workflow/config/commands.toml` 的路径。
    pub fn commands_config() -> Result<std::path::PathBuf, PathError> {
        config::commands_config()
    }

    /// 获取仓库根目录路径
    ///
    /// 返回当前工作目录（仓库根目录）的路径。
    /// 这是仓库的根目录，用于存储仓库特定的配置（如 `.workflow/config.toml`）。
    ///
    /// # 路径示例
    ///
    /// - 当前工作目录：`/path/to/repo`
    ///
    /// # 返回
    ///
    /// 返回仓库根目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法获取当前工作目录，返回相应的错误信息。
    pub fn repo_root() -> Result<std::path::PathBuf, PathError> {
        config::repo_root()
    }

    /// 获取项目级配置文件路径
    ///
    /// 返回当前工作目录下的 `.workflow/config.toml` 路径。
    /// 这是项目级别的配置文件，用于存储仓库特定的配置（如分支前缀、提交模板等）。
    ///
    /// # 路径示例
    ///
    /// - 相对于当前工作目录：`.workflow/config.toml`
    ///
    /// # 返回
    ///
    /// 返回项目级配置文件的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法获取当前工作目录，返回相应的错误信息。
    pub fn project_config() -> Result<std::path::PathBuf, PathError> {
        config::project_config()
    }

    /// 获取个人偏好配置文件路径
    ///
    /// 返回 `~/.workflow/config/repository.toml` 的路径。
    /// 支持 iCloud 同步（在 macOS 上，如果 iCloud 可用）。
    ///
    /// # 路径示例
    ///
    /// - macOS + iCloud：`~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config/repository.toml`
    /// - macOS 无 iCloud / 其他系统：`~/.workflow/config/repository.toml`
    ///
    /// # 返回
    ///
    /// 返回个人偏好配置文件的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法创建配置目录，返回相应的错误信息。
    pub fn repository_config() -> Result<std::path::PathBuf, PathError> {
        config::repository_config()
    }

    /// 获取工作流目录路径（支持 iCloud）
    ///
    /// 返回工作流基础目录。如果配置在 iCloud，此方法返回 iCloud 路径。
    ///
    /// # 返回
    ///
    /// 返回工作流目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法创建目录，返回相应的错误信息。
    pub fn workflow_dir() -> Result<std::path::PathBuf, PathError> {
        config::workflow_dir()
    }

    /// 获取工作历史目录路径（强制本地，不同步）
    ///
    /// 返回 `~/.workflow/work-history/`（总是本地路径）。
    ///
    /// **重要**：工作历史是设备本地的，不应该跨设备同步，因为：
    /// - 每个设备的工作历史是独立的
    /// - 避免多设备冲突（不同设备可能处理不同的 PR）
    /// - 防止历史记录混乱（PR ID 可能在不同仓库中重复）
    /// - 性能考虑（本地读写更快，不需要等待 iCloud 同步）
    ///
    /// # 路径示例
    ///
    /// - 所有平台：`~/.workflow/work-history/`
    ///
    /// # 返回
    ///
    /// 返回工作历史目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果环境变量未设置或无法创建目录，返回相应的错误信息。
    pub fn work_history_dir() -> Result<std::path::PathBuf, PathError> {
        config::work_history_dir()
    }

    /// 获取日志目录路径（强制本地，不同步）
    ///
    /// 返回 `~/.workflow/logs/`（总是本地路径）。
    ///
    /// **重要**：日志文件是设备本地的，不应该跨设备同步，因为：
    /// - 每个设备的日志是独立的
    /// - 避免 iCloud 同步延迟影响性能
    /// - 日志文件可能较大，不适合同步
    ///
    /// # 路径示例
    ///
    /// - 所有平台：`~/.workflow/logs/`
    ///
    /// # 返回
    ///
    /// 返回日志目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法创建目录，返回相应的错误信息。
    pub fn logs_dir() -> Result<std::path::PathBuf, PathError> {
        config::logs_dir()
    }

    // ==================== 安装路径相关方法 ====================

    /// 获取所有命令名称
    ///
    /// 返回所有 Workflow CLI 命令的名称列表，这些名称同时用于：
    /// - 二进制文件名（workflow）
    /// - 补全脚本命令名（用于生成补全脚本）
    ///
    /// # 返回
    ///
    /// 返回包含所有命令名称的静态字符串切片数组。
    ///
    /// # 示例
    ///
    /// ```
    /// use toolkit::paths::Paths;
    ///
    /// let names = Paths::command_names();
    /// assert_eq!(names, ["workflow"]);
    /// ```
    pub fn command_names() -> &'static [&'static str] {
        install::command_names()
    }

    /// 获取二进制文件安装目录
    ///
    /// 返回二进制文件安装的系统目录路径。
    ///
    /// # 返回
    ///
    /// 返回安装目录路径的字符串。
    ///
    /// # 示例
    ///
    /// ```
    /// use toolkit::paths::Paths;
    ///
    /// let dir = Paths::binary_install_dir();
    /// // Unix: "/usr/local/bin"
    /// // Windows: "%LOCALAPPDATA%\\Programs\\workflow\\bin"
    /// ```
    pub fn binary_install_dir() -> String {
        install::binary_install_dir()
    }

    /// 获取所有二进制文件的完整路径
    ///
    /// 基于 `command_names()` 和 `binary_install_dir()` 构建完整路径。
    ///
    /// # 返回
    ///
    /// 返回包含所有二进制文件完整路径的字符串向量。
    ///
    /// # 示例
    ///
    /// ```
    /// use toolkit::paths::Paths;
    ///
    /// let paths = Paths::binary_paths();
    /// assert_eq!(paths, vec![
    ///     "/usr/local/bin/workflow".to_string(),
    /// ]);
    /// ```
    pub fn binary_paths() -> Vec<String> {
        install::binary_paths()
    }

    /// 获取平台特定的二进制文件名
    ///
    /// 在 Windows 上添加 .exe 扩展名，其他平台保持不变。
    ///
    /// # 参数
    ///
    /// * `name` - 二进制文件的基础名称（不含扩展名）
    ///
    /// # 返回
    ///
    /// 返回平台特定的二进制文件名。
    ///
    /// # 示例
    ///
    /// ```
    /// use toolkit::paths::Paths;
    ///
    /// let name = Paths::binary_name("workflow");
    /// // Windows: "workflow.exe"
    /// // Unix: "workflow"
    /// ```
    pub fn binary_name(name: &str) -> String {
        install::binary_name(name)
    }

    /// 获取补全脚本目录路径（强制本地）
    ///
    /// 返回 `~/.workflow/completions/`（总是本地路径）。
    /// Shell 补全脚本是本地安装的，不需要同步。
    ///
    /// # 返回
    ///
    /// 返回补全脚本目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法获取本地目录，返回相应的错误信息。
    pub fn completion_dir() -> Result<std::path::PathBuf, PathError> {
        install::completion_dir()
    }

    // ==================== 信息查询 API ====================

    /// 检查配置是否存储在 iCloud
    ///
    /// # 返回
    ///
    /// - `true` - 配置当前存储在 iCloud Drive
    /// - `false` - 配置存储在本地
    ///
    /// 保留此方法以备将来扩展使用（如 CLI 显示存储位置）。
    #[allow(dead_code)]
    pub fn is_config_in_icloud() -> bool {
        info::is_config_in_icloud()
    }

    /// 获取存储位置的用户友好描述
    ///
    /// # 返回
    ///
    /// - "iCloud Drive (synced across devices)" - 使用 iCloud
    /// - "Local storage" - 使用本地存储
    ///
    /// 保留此方法以备将来扩展使用（如 CLI 显示存储位置）。
    #[allow(dead_code)]
    pub fn storage_location() -> &'static str {
        info::storage_location()
    }

    /// 获取详细的存储信息
    ///
    /// 返回包含存储类型、配置路径和工作历史路径的详细信息。
    ///
    /// # 返回
    ///
    /// 返回格式化的存储信息字符串。
    ///
    /// # 错误
    ///
    /// 如果无法获取路径，返回相应的错误信息。
    ///
    /// 保留此方法以备将来扩展使用（如 CLI 显示存储信息）。
    #[allow(dead_code)]
    pub fn storage_info() -> Result<String, PathError> {
        info::storage_info()
    }
}
