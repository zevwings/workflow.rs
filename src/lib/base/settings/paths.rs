//! 路径管理
//!
//! 统一管理所有路径信息，包括：
//! - 配置文件路径（存储在 `~/.workflow/config/` 目录下）
//! - 安装路径（二进制文件和补全脚本的安装路径和名称）
//! - Shell 相关路径（shell 配置文件和 completion 目录）

use crate::base::util::directory::DirectoryWalker;

// 配置文件和目录名称常量
pub const WORKFLOW_DIR: &str = ".workflow";
pub const CONFIG_DIR: &str = "config";
pub const WORKFLOW_CONFIG_FILE: &str = "workflow.toml";
pub const JIRA_CONFIG_FILE: &str = "jira.toml";
pub const LLM_CONFIG_FILE: &str = "llm.toml";
pub const COMPLETIONS_FILE: &str = ".completions";
use clap_complete::shells::Shell;
use color_eyre::{
    eyre::{ContextCompat, WrapErr},
    Result,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// 路径管理器
///
/// 统一管理所有路径信息，包括配置路径、安装路径和 Shell 路径。
pub struct Paths;

impl Paths {
    // ==================== 私有辅助方法 ====================

    /// 获取用户主目录
    ///
    /// 优先使用 `HOME` 环境变量（Unix）或 `USERPROFILE` 环境变量（Windows），
    /// 如果环境变量未设置，则回退到 `dirs::home_dir()`。
    /// 这是一个统一的入口点，所有需要主目录的地方都应该调用此方法。
    ///
    /// # 返回
    ///
    /// 返回用户主目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法确定主目录，返回错误信息。
    pub(crate) fn home_dir() -> Result<PathBuf> {
        // 优先检查环境变量（确保测试环境中的 HOME 被正确使用）
        #[cfg(unix)]
        {
            if let Ok(home) = env::var("HOME") {
                let home_path = PathBuf::from(home);
                if home_path.is_absolute() {
                    return Ok(home_path);
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(home) = env::var("USERPROFILE") {
                let home_path = PathBuf::from(home);
                if home_path.is_absolute() {
                    return Ok(home_path);
                }
            }
        }

        // 回退到 dirs::home_dir()
        dirs::home_dir().wrap_err("Cannot determine home directory")
    }

    /// 尝试获取 iCloud 基础目录（指定主目录，仅 macOS）
    ///
    /// 检查 iCloud Drive 是否可用，如果可用则返回 .workflow 目录路径。
    ///
    /// # 参数
    ///
    /// * `home` - 用户主目录路径
    ///
    /// # 返回
    ///
    /// - `Some(PathBuf)` - iCloud Drive 可用且成功创建目录
    /// - `None` - iCloud Drive 不可用或创建目录失败
    ///
    /// # iCloud 路径
    ///
    /// macOS: `~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/`
    #[cfg(target_os = "macos")]
    fn try_icloud_base_dir_in(home: impl AsRef<Path>) -> Option<PathBuf> {
        // 构建 iCloud Drive 基础路径
        // ~/Library/Mobile Documents/com~apple~CloudDocs
        let icloud_base = home
            .as_ref()
            .join("Library")
            .join("Mobile Documents")
            .join("com~apple~CloudDocs");

        // 检查 iCloud Drive 是否可用
        if !icloud_base.exists() || !icloud_base.is_dir() {
            return None;
        }

        // 尝试创建 .workflow 目录
        let workflow_dir = icloud_base.join(WORKFLOW_DIR);
        if DirectoryWalker::new(&workflow_dir).ensure_exists().is_err() {
            return None;
        }

        // 设置目录权限为 700（仅用户可访问）
        #[cfg(unix)]
        {
            let _ = fs::set_permissions(&workflow_dir, fs::Permissions::from_mode(0o700));
        }

        Some(workflow_dir)
    }

    /// 非 macOS 平台：总是返回 None（指定主目录版本）
    #[cfg(not(target_os = "macos"))]
    #[allow(dead_code)]
    fn try_icloud_base_dir_in(_home: impl AsRef<Path>) -> Option<PathBuf> {
        None
    }

    /// 获取本地基础目录（总是可用）
    ///
    /// 返回 `~/.workflow/` 目录（Unix）。
    /// 此方法作为回退方案，确保在任何情况下都能获取到有效路径。
    ///
    /// # 返回
    ///
    /// 返回本地工作流目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法创建目录，返回相应的错误信息。
    pub fn local_base_dir() -> Result<PathBuf> {
        Self::local_base_dir_in(Self::home_dir()?)
    }

    /// 获取本地基础目录（指定主目录）
    ///
    /// 返回指定主目录下的 `.workflow/` 目录。
    /// 此方法允许指定主目录路径，避免依赖全局环境变量。
    ///
    /// # 参数
    ///
    /// * `home` - 用户主目录路径
    ///
    /// # 返回
    ///
    /// 返回本地工作流目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法创建目录，返回相应的错误信息。
    pub fn local_base_dir_in(home: impl AsRef<Path>) -> Result<PathBuf> {
        let workflow_dir = home.as_ref().join(WORKFLOW_DIR);

        // 确保目录存在
        DirectoryWalker::new(&workflow_dir).ensure_exists()?;

        // 设置目录权限为 700（仅用户可访问）
        #[cfg(unix)]
        {
            fs::set_permissions(&workflow_dir, fs::Permissions::from_mode(0o700))
                .wrap_err("Failed to set workflow directory permissions")?;
        }

        Ok(workflow_dir)
    }

    /// 获取配置基础目录（支持 iCloud）
    ///
    /// 决策逻辑：
    /// 1. 检查环境变量 `WORKFLOW_DISABLE_ICLOUD`，如果设置则强制使用本地
    /// 2. 在 macOS 上，如果 iCloud Drive 可用，优先使用 iCloud
    /// 3. 如果 iCloud 不可用，回退到本地目录
    /// 4. 在其他平台上，直接使用本地目录
    ///
    /// **注意**：如果用户已有本地配置，需要手动迁移到 iCloud：
    /// ```bash
    /// cp -r ~/.workflow/config/* \
    ///    ~/Library/Mobile\ Documents/com~apple~CloudDocs/.workflow/config/
    /// ```
    ///
    /// # 返回
    ///
    /// 返回配置基础目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法创建目录，返回相应的错误信息。
    /// 获取配置基础目录（指定主目录，支持 iCloud）
    ///
    /// 决策逻辑：
    /// 1. 如果 `disable_icloud` 为 `true`，强制使用本地目录
    /// 2. 在 macOS 上，如果 iCloud Drive 可用，优先使用 iCloud
    /// 3. 如果 iCloud 不可用，回退到本地目录
    /// 4. 在其他平台上，直接使用本地目录
    ///
    /// # 参数
    ///
    /// * `home` - 用户主目录路径
    /// * `disable_icloud` - 是否禁用 iCloud（如果为 `true`，强制使用本地目录）
    ///
    /// # 返回
    ///
    /// 返回配置基础目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法创建目录，返回相应的错误信息。
    pub(crate) fn config_base_dir_in(
        home: impl AsRef<Path>,
        disable_icloud: bool,
    ) -> Result<PathBuf> {
        // 如果明确禁用 iCloud，直接使用本地目录
        if disable_icloud {
            return Self::local_base_dir_in(home);
        }

        // macOS 上尝试 iCloud
        #[cfg(target_os = "macos")]
        {
            if let Some(icloud_dir) = Self::try_icloud_base_dir_in(home.as_ref()) {
                return Ok(icloud_dir);
            }
        }

        // 回退到本地
        Self::local_base_dir_in(home)
    }

    // ==================== 路径工具方法 ====================

    /// 展开路径字符串
    ///
    /// 支持的路径格式：
    /// - Unix: `~` 和 `~/path` - 展开为用户主目录
    /// - Windows: `%VAR%` 和 `%VAR%\path` - 展开环境变量
    /// - 绝对路径: 直接使用
    ///
    /// # 示例
    ///
    /// ```text
    /// // Unix
    /// expand("~/Documents/Workflow") -> "/home/user/Documents/Workflow"
    /// expand("~") -> "/home/user"
    ///
    /// // Windows
    /// expand("%USERPROFILE%\\Documents\\Workflow") -> "C:\\Users\\User\\Documents\\Workflow"
    /// expand("%APPDATA%\\workflow") -> "C:\\Users\\User\\AppData\\Roaming\\workflow"
    ///
    /// // 绝对路径
    /// expand("/absolute/path") -> "/absolute/path"
    /// expand("C:\\absolute\\path") -> "C:\\absolute\\path"
    /// ```
    pub fn expand(path_str: &str) -> Result<PathBuf> {
        let home = Self::home_dir()?;
        Self::expand_in(path_str, home)
    }

    /// 展开路径字符串（指定主目录）
    ///
    /// 支持的路径格式：
    /// - Unix: `~` 和 `~/path` - 展开为指定的主目录
    /// - Windows: `%VAR%` 和 `%VAR%\path` - 展开环境变量
    /// - 绝对路径: 直接使用
    ///
    /// # 参数
    ///
    /// * `path_str` - 要展开的路径字符串
    /// * `home` - 用户主目录路径（用于展开 `~`）
    ///
    /// # 示例
    ///
    /// ```text
    /// // Unix
    /// expand_in("~/Documents/Workflow", "/tmp/test") -> "/tmp/test/Documents/Workflow"
    /// expand_in("~", "/tmp/test") -> "/tmp/test"
    ///
    /// // Windows
    /// expand_in("%USERPROFILE%\\Documents\\Workflow", "C:\\Users\\User") -> "C:\\Users\\User\\Documents\\Workflow"
    ///
    /// // 绝对路径
    /// expand_in("/absolute/path", "/tmp/test") -> "/absolute/path"
    /// ```
    pub fn expand_in(path_str: &str, home: impl AsRef<Path>) -> Result<PathBuf> {
        let home = home.as_ref();

        // 处理 Unix 风格的 ~ 展开
        if let Some(rest) = path_str.strip_prefix("~/") {
            return Ok(home.join(rest));
        }
        if path_str == "~" {
            return Ok(home.to_path_buf());
        }

        // 处理 Windows 风格的环境变量展开 %VAR%
        if path_str.contains('%') {
            let mut result = String::new();
            let mut chars = path_str.chars().peekable();

            while let Some(ch) = chars.next() {
                if ch == '%' {
                    // 提取环境变量名
                    let mut var_name = String::new();
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch == '%' {
                            chars.next(); // 跳过结束的 %
                            break;
                        }
                        var_name.push(chars.next().unwrap());
                    }

                    // 展开环境变量
                    if !var_name.is_empty() {
                        let var_value = env::var(&var_name).wrap_err_with(|| {
                            format!("Environment variable not set: {}", var_name)
                        })?;
                        result.push_str(&var_value);
                    }
                } else {
                    result.push(ch);
                }
            }

            return Ok(PathBuf::from(result));
        }

        // 其他情况：直接使用路径（可能是绝对路径或相对路径）
        Ok(PathBuf::from(path_str))
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
    pub fn config_dir() -> Result<PathBuf> {
        let home = Self::home_dir()?;
        let disable_icloud = std::env::var("WORKFLOW_DISABLE_ICLOUD").is_ok();
        Self::config_dir_in(home, disable_icloud)
    }

    /// 获取配置目录（指定主目录）
    ///
    /// 返回指定主目录下的 `.workflow/config/` 目录路径。
    /// 支持 iCloud 同步（在 macOS 上，如果 iCloud 可用且未禁用）。
    ///
    /// # 参数
    ///
    /// * `home` - 用户主目录路径
    /// * `disable_icloud` - 是否禁用 iCloud（如果为 `true`，强制使用本地目录）
    ///
    /// # 路径示例
    ///
    /// - macOS + iCloud：`{home}/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config/`
    /// - macOS 无 iCloud / 其他系统：`{home}/.workflow/config/`
    ///
    /// # 返回
    ///
    /// 返回配置目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法创建目录，返回相应的错误信息。
    pub fn config_dir_in(home: impl AsRef<Path>, disable_icloud: bool) -> Result<PathBuf> {
        // 使用支持 iCloud 的配置基础目录
        let config_dir = Self::config_base_dir_in(home, disable_icloud)?.join(CONFIG_DIR);

        // 确保配置目录存在
        DirectoryWalker::new(&config_dir).ensure_exists()?;

        // 设置目录权限为 700（仅用户可访问，仅 Unix）
        #[cfg(unix)]
        {
            fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o700))
                .wrap_err("Failed to set config directory permissions")?;
        }

        Ok(config_dir)
    }

    /// 获取主配置文件路径
    ///
    /// 返回 `~/.workflow/config/workflow.toml` 的路径。
    pub fn workflow_config() -> Result<PathBuf> {
        let home = Self::home_dir()?;
        let disable_icloud = std::env::var("WORKFLOW_DISABLE_ICLOUD").is_ok();
        Self::workflow_config_in(home, disable_icloud)
    }

    /// 获取主配置文件路径（指定主目录）
    ///
    /// 返回指定主目录下的 `.workflow/config/workflow.toml` 路径。
    ///
    /// # 参数
    ///
    /// * `home` - 用户主目录路径
    /// * `disable_icloud` - 是否禁用 iCloud（如果为 `true`，强制使用本地目录）
    ///
    /// # 返回
    ///
    /// 返回主配置文件的 `PathBuf`。
    pub fn workflow_config_in(home: impl AsRef<Path>, disable_icloud: bool) -> Result<PathBuf> {
        Ok(Self::config_dir_in(home, disable_icloud)?.join(WORKFLOW_CONFIG_FILE))
    }

    /// 获取 LLM 配置文件路径
    ///
    /// 返回 `~/.workflow/config/llm.toml` 的路径。
    pub fn llm_config() -> Result<PathBuf> {
        let home = Self::home_dir()?;
        let disable_icloud = std::env::var("WORKFLOW_DISABLE_ICLOUD").is_ok();
        Self::llm_config_in(home, disable_icloud)
    }

    /// 获取 LLM 配置文件路径（指定主目录）
    ///
    /// 返回指定主目录下的 `.workflow/config/llm.toml` 路径。
    ///
    /// # 参数
    ///
    /// * `home` - 用户主目录路径
    /// * `disable_icloud` - 是否禁用 iCloud（如果为 `true`，强制使用本地目录）
    ///
    /// # 返回
    ///
    /// 返回 LLM 配置文件的 `PathBuf`。
    pub fn llm_config_in(home: impl AsRef<Path>, disable_icloud: bool) -> Result<PathBuf> {
        Ok(Self::config_dir_in(home, disable_icloud)?.join(LLM_CONFIG_FILE))
    }

    /// 获取 Jira 配置文件路径
    ///
    /// 返回 `~/.workflow/config/jira.toml` 的路径。
    /// 这是合并后的 Jira 配置文件，包含用户和状态配置。
    pub fn jira_config() -> Result<PathBuf> {
        let home = Self::home_dir()?;
        let disable_icloud = std::env::var("WORKFLOW_DISABLE_ICLOUD").is_ok();
        Self::jira_config_in(home, disable_icloud)
    }

    /// 获取 Jira 配置文件路径（指定主目录）
    ///
    /// 返回指定主目录下的 `.workflow/config/jira.toml` 路径。
    ///
    /// # 参数
    ///
    /// * `home` - 用户主目录路径
    /// * `disable_icloud` - 是否禁用 iCloud（如果为 `true`，强制使用本地目录）
    ///
    /// # 返回
    ///
    /// 返回 Jira 配置文件的 `PathBuf`。
    pub fn jira_config_in(home: impl AsRef<Path>, disable_icloud: bool) -> Result<PathBuf> {
        Ok(Self::config_dir_in(home, disable_icloud)?.join(JIRA_CONFIG_FILE))
    }

    /// 获取常用命令配置文件路径
    ///
    /// 返回 `~/.workflow/config/commands.toml` 的路径。
    pub fn commands_config() -> Result<PathBuf> {
        let home = Self::home_dir()?;
        let disable_icloud = std::env::var("WORKFLOW_DISABLE_ICLOUD").is_ok();
        Self::commands_config_in(home, disable_icloud)
    }

    /// 获取常用命令配置文件路径（指定主目录）
    ///
    /// 返回指定主目录下的 `.workflow/config/commands.toml` 路径。
    ///
    /// # 参数
    ///
    /// * `home` - 用户主目录路径
    /// * `disable_icloud` - 是否禁用 iCloud（如果为 `true`，强制使用本地目录）
    ///
    /// # 返回
    ///
    /// 返回常用命令配置文件的 `PathBuf`。
    pub fn commands_config_in(home: impl AsRef<Path>, disable_icloud: bool) -> Result<PathBuf> {
        Ok(Self::config_dir_in(home, disable_icloud)?.join("commands.toml"))
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
    pub fn project_config() -> Result<PathBuf> {
        Self::project_config_in(
            std::env::current_dir().wrap_err("Failed to get current directory")?,
        )
    }

    /// 获取项目级配置文件路径（指定仓库路径）
    ///
    /// 返回指定仓库路径下的 `.workflow/config.toml` 路径。
    /// 这是项目级别的配置文件，用于存储仓库特定的配置（如分支前缀、提交模板等）。
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库根目录路径
    ///
    /// # 路径示例
    ///
    /// - 相对于指定路径：`{repo_path}/.workflow/config.toml`
    ///
    /// # 返回
    ///
    /// 返回项目级配置文件的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法规范化路径，返回相应的错误信息。
    pub fn project_config_in(repo_path: impl AsRef<std::path::Path>) -> Result<PathBuf> {
        Ok(repo_path
            .as_ref()
            .canonicalize()
            .wrap_err("Failed to canonicalize repository path")?
            .join(WORKFLOW_DIR)
            .join("config.toml"))
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
    pub fn repository_config() -> Result<PathBuf> {
        let home = Self::home_dir()?;
        let disable_icloud = std::env::var("WORKFLOW_DISABLE_ICLOUD").is_ok();
        Self::repository_config_in(home, disable_icloud)
    }

    /// 获取个人偏好配置文件路径（指定主目录）
    ///
    /// 返回指定主目录下的 `.workflow/config/repository.toml` 路径。
    /// 支持 iCloud 同步（在 macOS 上，如果 iCloud 可用且未禁用）。
    ///
    /// # 参数
    ///
    /// * `home` - 用户主目录路径
    /// * `disable_icloud` - 是否禁用 iCloud（如果为 `true`，强制使用本地目录）
    ///
    /// # 路径示例
    ///
    /// - macOS + iCloud：`{home}/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config/repository.toml`
    /// - macOS 无 iCloud / 其他系统：`{home}/.workflow/config/repository.toml`
    ///
    /// # 返回
    ///
    /// 返回个人偏好配置文件的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法创建配置目录，返回相应的错误信息。
    pub fn repository_config_in(home: impl AsRef<Path>, disable_icloud: bool) -> Result<PathBuf> {
        Ok(Self::config_dir_in(home, disable_icloud)?.join("repository.toml"))
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
    pub fn workflow_dir() -> Result<PathBuf> {
        let home = Self::home_dir()?;
        let disable_icloud = std::env::var("WORKFLOW_DISABLE_ICLOUD").is_ok();
        Self::workflow_dir_in(home, disable_icloud)
    }

    /// 获取工作流目录路径（指定主目录，支持 iCloud）
    ///
    /// 返回指定主目录下的工作流基础目录。如果 iCloud 可用且未禁用，返回 iCloud 路径。
    ///
    /// # 参数
    ///
    /// * `home` - 用户主目录路径
    /// * `disable_icloud` - 是否禁用 iCloud（如果为 `true`，强制使用本地目录）
    ///
    /// # 返回
    ///
    /// 返回工作流目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法创建目录，返回相应的错误信息。
    pub fn workflow_dir_in(home: impl AsRef<Path>, disable_icloud: bool) -> Result<PathBuf> {
        // 直接返回配置基础目录
        Self::config_base_dir_in(home, disable_icloud)
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
    pub fn work_history_dir() -> Result<PathBuf> {
        let home = Self::home_dir()?;
        Self::work_history_dir_in(home)
    }

    /// 获取工作历史目录路径（指定主目录，强制本地）
    ///
    /// 返回指定主目录下的 `.workflow/work-history/` 路径（总是本地路径）。
    ///
    /// # 参数
    ///
    /// * `home` - 用户主目录路径
    ///
    /// # 返回
    ///
    /// 返回工作历史目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法创建目录，返回相应的错误信息。
    pub fn work_history_dir_in(home: impl AsRef<Path>) -> Result<PathBuf> {
        // 强制使用本地路径，不使用 iCloud
        let history_dir = Self::local_base_dir_in(home.as_ref())?.join("work-history");

        // 确保目录存在
        DirectoryWalker::new(&history_dir).ensure_exists()?;

        // 设置目录权限为 700（仅用户可访问，仅 Unix）
        #[cfg(unix)]
        {
            fs::set_permissions(&history_dir, fs::Permissions::from_mode(0o700))
                .wrap_err("Failed to set work-history directory permissions")?;
        }

        Ok(history_dir)
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
    pub fn logs_dir() -> Result<PathBuf> {
        let home = Self::home_dir()?;
        Self::logs_dir_in(home)
    }

    /// 获取日志目录路径（指定主目录，强制本地）
    ///
    /// 返回指定主目录下的 `.workflow/logs/` 路径（总是本地路径）。
    ///
    /// # 参数
    ///
    /// * `home` - 用户主目录路径
    ///
    /// # 返回
    ///
    /// 返回日志目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法创建目录，返回相应的错误信息。
    pub fn logs_dir_in(home: impl AsRef<Path>) -> Result<PathBuf> {
        // 强制使用本地路径，不使用 iCloud
        let logs_dir = Self::local_base_dir_in(home.as_ref())?.join("logs");

        // 确保目录存在
        DirectoryWalker::new(&logs_dir).ensure_exists()?;

        // 设置目录权限为 700（仅用户可访问，仅 Unix）
        #[cfg(unix)]
        {
            fs::set_permissions(&logs_dir, fs::Permissions::from_mode(0o700))
                .wrap_err("Failed to set logs directory permissions")?;
        }

        Ok(logs_dir)
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
    /// use workflow::base::settings::paths::Paths;
    ///
    /// let names = Paths::command_names();
    /// assert_eq!(names, ["workflow"]);
    /// ```
    pub fn command_names() -> &'static [&'static str] {
        &["workflow"]
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
    /// use workflow::base::settings::paths::Paths;
    ///
    /// let dir = Paths::binary_install_dir();
    /// // Unix: "/usr/local/bin"
    /// // Windows: "%LOCALAPPDATA%\\Programs\\workflow\\bin"
    /// ```
    pub fn binary_install_dir() -> String {
        if cfg!(target_os = "windows") {
            // Windows: 使用 dirs::data_local_dir() 获取 %LOCALAPPDATA%
            dirs::data_local_dir()
                .map(|d| d.join("Programs").join("workflow").join("bin"))
                .or_else(|| dirs::home_dir().map(|h| h.join(".local").join("bin")))
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "C:\\Users\\User\\Programs\\workflow\\bin".to_string())
        } else {
            // Unix-like: 使用 /usr/local/bin
            "/usr/local/bin".to_string()
        }
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
    /// use workflow::base::settings::paths::Paths;
    ///
    /// let paths = Paths::binary_paths();
    /// assert_eq!(paths.len(), 1);
    /// if cfg!(target_os = "windows") {
    ///     assert!(paths[0].ends_with("workflow.exe"));
    /// } else {
    ///     assert_eq!(paths[0], "/usr/local/bin/workflow");
    /// }
    /// ```
    pub fn binary_paths() -> Vec<String> {
        let install_dir = Self::binary_install_dir();
        let install_path = PathBuf::from(&install_dir);
        Self::command_names()
            .iter()
            .map(|name| {
                let binary_name = Self::binary_name(name);
                install_path.join(&binary_name).to_string_lossy().to_string()
            })
            .collect()
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
    /// use workflow::base::settings::paths::Paths;
    ///
    /// let name = Paths::binary_name("workflow");
    /// // Windows: "workflow.exe"
    /// // Unix: "workflow"
    /// ```
    pub fn binary_name(name: &str) -> String {
        if cfg!(target_os = "windows") {
            format!("{}.exe", name)
        } else {
            name.to_string()
        }
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
    pub fn completion_dir() -> Result<PathBuf> {
        let home = Self::home_dir()?;
        Self::completion_dir_in(home)
    }

    /// 获取补全脚本目录路径（指定主目录，强制本地）
    ///
    /// 返回指定主目录下的 `.workflow/completions/` 路径（总是本地路径）。
    ///
    /// # 参数
    ///
    /// * `home` - 用户主目录路径
    ///
    /// # 返回
    ///
    /// 返回补全脚本目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法创建目录，返回相应的错误信息。
    pub fn completion_dir_in(home: impl AsRef<Path>) -> Result<PathBuf> {
        // 确保使用本地路径
        let completion_dir = Self::local_base_dir_in(home.as_ref())?.join("completions");

        // 确保目录存在
        DirectoryWalker::new(&completion_dir).ensure_exists()?;

        Ok(completion_dir)
    }

    // ==================== 信息查询 API ====================

    /// 检查配置是否存储在 iCloud
    ///
    /// # 返回
    ///
    /// - `true` - 配置当前存储在 iCloud Drive
    /// - `false` - 配置存储在本地
    pub fn is_config_in_icloud() -> bool {
        #[cfg(target_os = "macos")]
        {
            // 直接调用 _in 版本，避免依赖未使用的私有函数
            if let Ok(home) = Self::home_dir() {
                Self::try_icloud_base_dir_in(home).is_some()
            } else {
                false
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    /// 获取存储位置的用户友好描述
    ///
    /// # 返回
    ///
    /// - "iCloud Drive (synced across devices)" - 使用 iCloud
    /// - "Local storage" - 使用本地存储
    pub fn storage_location() -> &'static str {
        if Self::is_config_in_icloud() {
            "iCloud Drive (synced across devices)"
        } else {
            "Local storage"
        }
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
    pub fn storage_info() -> Result<String> {
        let config_dir = Self::config_dir()?;
        let work_history_dir = Self::work_history_dir()?;

        let info = if Self::is_config_in_icloud() {
            format!(
                "Storage Type: iCloud Drive (synced across devices)\n\
                 \n\
                 Configuration (synced):\n\
                 {}\n\
                 \n\
                 Work History (local only, not synced):\n\
                 {}",
                config_dir.display(),
                work_history_dir.display()
            )
        } else {
            format!(
                "Storage Type: Local storage\n\
                 \n\
                 Configuration:\n\
                 {}\n\
                 \n\
                 Work History:\n\
                 {}",
                config_dir.display(),
                work_history_dir.display()
            )
        };

        Ok(info)
    }

    // ==================== Shell 路径相关方法 ====================
    /// 获取 shell 配置文件路径
    ///
    /// 支持的 shell 类型及其配置文件路径：
    /// - zsh → `~/.zshrc`
    /// - bash → `~/.bash_profile`（如果不存在则使用 `~/.bashrc`）
    /// - fish → `~/.config/fish/config.fish`
    /// - powershell → `~/.config/powershell/Microsoft.PowerShell_profile.ps1`
    /// - elvish → `~/.elvish/rc.elv`
    ///
    /// 注意：对于 bash，macOS 通常使用 `.bash_profile`，Linux 使用 `.bashrc`。
    /// 此方法会优先使用 `.bash_profile`，如果不存在则使用 `.bashrc`。
    ///
    /// # 参数
    ///
    /// * `shell` - Shell 枚举类型
    ///
    /// # 返回
    ///
    /// 返回 shell 配置文件的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果 `HOME` 环境变量未设置或 shell 类型不支持，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```
    /// use clap_complete::shells::Shell;
    /// ```
    /// use std::path::PathBuf;
    /// use clap_complete::shells::Shell;
    /// use workflow::base::settings::paths::Paths;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let zsh_path = Paths::config_file(&Shell::Zsh)?;
    /// assert_eq!(zsh_path, PathBuf::from("~/.zshrc"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn config_file(shell: &Shell) -> Result<PathBuf> {
        // 使用新的 home_dir() 方法
        let home = Self::home_dir()?;

        let config_file = match shell {
            #[cfg(target_os = "windows")]
            Shell::PowerShell => {
                // Windows PowerShell 配置文件路径
                // 优先使用 PowerShell Core 路径
                let pwsh_profile = home
                    .join("Documents")
                    .join("PowerShell")
                    .join("Microsoft.PowerShell_profile.ps1");
                let ps_profile = home
                    .join("Documents")
                    .join("WindowsPowerShell")
                    .join("Microsoft.PowerShell_profile.ps1");

                // 如果 PowerShell Core 配置文件存在，使用它；否则使用 Windows PowerShell 路径
                if pwsh_profile.exists() {
                    pwsh_profile
                } else {
                    ps_profile
                }
            }

            #[cfg(not(target_os = "windows"))]
            Shell::Zsh => home.join(".zshrc"),

            #[cfg(not(target_os = "windows"))]
            Shell::Bash => {
                let bash_profile = home.join(".bash_profile");
                let bashrc = home.join(".bashrc");
                if !bash_profile.exists() && bashrc.exists() {
                    bashrc
                } else {
                    bash_profile
                }
            }

            #[cfg(not(target_os = "windows"))]
            Shell::Fish => home.join(".config/fish/config.fish"),

            #[cfg(not(target_os = "windows"))]
            Shell::PowerShell => home.join(".config/powershell/Microsoft.PowerShell_profile.ps1"),

            #[cfg(not(target_os = "windows"))]
            Shell::Elvish => home.join(".elvish/rc.elv"),

            _ => color_eyre::eyre::bail!("Unsupported shell type"),
        };

        Ok(config_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 home_dir() 方法（这是唯一需要测试私有 API 的测试）
    ///
    /// 其他所有测试都已移动到 tests/paths_integration.rs，
    /// 因为它们只使用公开 API。
    #[test]
    fn test_home_dir() {
        let home = Paths::home_dir().unwrap();
        assert!(home.exists());
        assert!(home.is_dir());
    }
}
