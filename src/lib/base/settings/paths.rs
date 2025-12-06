//! 路径管理
//!
//! 统一管理所有路径信息，包括：
//! - 配置文件路径（存储在 `~/.workflow/config/` 目录下）
//! - 安装路径（二进制文件和补全脚本的安装路径和名称）
//! - Shell 相关路径（shell 配置文件和 completion 目录）

use anyhow::{Context, Result};
use clap_complete::shells::Shell;
use std::fs;
use std::path::PathBuf;

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
    /// 使用 dirs crate 提供的跨平台主目录获取功能。
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
        dirs::home_dir().context("Cannot determine home directory")
    }

    /// 尝试获取 iCloud 基础目录（仅 macOS）
    ///
    /// 检查 iCloud Drive 是否可用，如果可用则返回 .workflow 目录路径。
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
    fn try_icloud_base_dir() -> Option<PathBuf> {
        // 获取主目录
        let home = Self::home_dir().ok()?;

        // 构建 iCloud Drive 基础路径
        // ~/Library/Mobile Documents/com~apple~CloudDocs
        let icloud_base = home
            .join("Library")
            .join("Mobile Documents")
            .join("com~apple~CloudDocs");

        // 检查 iCloud Drive 是否可用
        if !icloud_base.exists() || !icloud_base.is_dir() {
            return None;
        }

        // 尝试创建 .workflow 目录
        let workflow_dir = icloud_base.join(".workflow");
        if fs::create_dir_all(&workflow_dir).is_err() {
            return None;
        }

        // 设置目录权限为 700（仅用户可访问）
        #[cfg(unix)]
        {
            let _ = fs::set_permissions(&workflow_dir, fs::Permissions::from_mode(0o700));
        }

        Some(workflow_dir)
    }

    /// 非 macOS 平台：总是返回 None
    #[cfg(not(target_os = "macos"))]
    fn try_icloud_base_dir() -> Option<PathBuf> {
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
    fn local_base_dir() -> Result<PathBuf> {
        let home = Self::home_dir()?;
        let workflow_dir = home.join(".workflow");

        // 确保目录存在
        fs::create_dir_all(&workflow_dir).context("Failed to create local .workflow directory")?;

        // 设置目录权限为 700（仅用户可访问）
        #[cfg(unix)]
        {
            fs::set_permissions(&workflow_dir, fs::Permissions::from_mode(0o700))
                .context("Failed to set workflow directory permissions")?;
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
    fn config_base_dir() -> Result<PathBuf> {
        // 检查用户是否明确禁用 iCloud
        if std::env::var("WORKFLOW_DISABLE_ICLOUD").is_ok() {
            return Self::local_base_dir();
        }

        // macOS 上尝试 iCloud
        #[cfg(target_os = "macos")]
        {
            if let Some(icloud_dir) = Self::try_icloud_base_dir() {
                return Ok(icloud_dir);
            }
        }

        // 回退到本地
        Self::local_base_dir()
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
        // 使用支持 iCloud 的配置基础目录
        let config_dir = Self::config_base_dir()?.join("config");

        // 确保配置目录存在
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

        // 设置目录权限为 700（仅用户可访问，仅 Unix）
        #[cfg(unix)]
        {
            fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o700))
                .context("Failed to set config directory permissions")?;
        }

        Ok(config_dir)
    }

    /// 获取主配置文件路径
    ///
    /// 返回 `~/.workflow/config/workflow.toml` 的路径。
    pub fn workflow_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("workflow.toml"))
    }

    /// 获取 LLM 配置文件路径
    ///
    /// 返回 `~/.workflow/config/llm.toml` 的路径。
    pub fn llm_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("llm.toml"))
    }

    /// 获取 Jira 状态配置文件路径
    ///
    /// 返回 `~/.workflow/config/jira-status.toml` 的路径。
    pub fn jira_status_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("jira-status.toml"))
    }

    /// 获取 Jira 用户配置文件路径
    ///
    /// 返回 `~/.workflow/config/jira-users.toml` 的路径。
    pub fn jira_users_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("jira-users.toml"))
    }

    /// 获取分支配置文件路径
    ///
    /// 返回 `~/.workflow/config/branch.toml` 的路径。
    pub fn branch_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("branch.toml"))
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
        // 直接返回配置基础目录
        Self::config_base_dir()
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
        // 强制使用本地路径，不使用 iCloud
        let history_dir = Self::local_base_dir()?.join("work-history");

        // 确保目录存在
        fs::create_dir_all(&history_dir)
            .context("Failed to create .workflow/work-history directory")?;

        // 设置目录权限为 700（仅用户可访问，仅 Unix）
        #[cfg(unix)]
        {
            fs::set_permissions(&history_dir, fs::Permissions::from_mode(0o700))
                .context("Failed to set work-history directory permissions")?;
        }

        Ok(history_dir)
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
    /// use workflow::settings::paths::Paths;
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
    /// use workflow::settings::paths::Paths;
    ///
    /// let dir = Paths::binary_install_dir();
    /// // Unix: "/usr/local/bin"
    /// // Windows: "%LOCALAPPDATA%\\Programs\\workflow\\bin"
    /// ```
    pub fn binary_install_dir() -> String {
        if cfg!(target_os = "windows") {
            // Windows: 使用 %LOCALAPPDATA%\Programs\workflow\bin
            let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
                std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\User".to_string())
            });
            format!("{}\\Programs\\workflow\\bin", local_app_data)
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
    /// use workflow::settings::paths::Paths;
    ///
    /// let paths = Paths::binary_paths();
    /// assert_eq!(paths, vec![
    ///     "/usr/local/bin/workflow".to_string(),
    /// ]);
    /// ```
    pub fn binary_paths() -> Vec<String> {
        let install_dir = Self::binary_install_dir();
        let install_path = PathBuf::from(&install_dir);
        Self::command_names()
            .iter()
            .map(|name| {
                let binary_name = Self::binary_name(name);
                install_path
                    .join(&binary_name)
                    .to_string_lossy()
                    .to_string()
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
    /// use workflow::settings::paths::Paths;
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
        // 确保使用本地路径
        Ok(Self::local_base_dir()?.join("completions"))
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
            Self::try_icloud_base_dir().is_some()
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
    /// use workflow::settings::paths::Paths;
    ///
    /// let zsh_path = Paths::config_file(&Shell::Zsh)?;
    /// assert_eq!(zsh_path, PathBuf::from("~/.zshrc"));
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

            _ => anyhow::bail!("Unsupported shell type"),
        };

        Ok(config_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap_complete::shells::Shell;

    #[test]
    fn test_home_dir() {
        let home = Paths::home_dir().unwrap();
        assert!(home.exists());
        assert!(home.is_dir());
    }

    #[test]
    fn test_config_dir() {
        let config_dir = Paths::config_dir().unwrap();
        assert!(config_dir.exists());
        assert!(config_dir.is_dir());
        assert!(
            config_dir.ends_with(".workflow/config")
                || config_dir.to_string_lossy().contains("workflow")
        );
    }

    #[test]
    fn test_work_history_dir() {
        let history_dir = Paths::work_history_dir().unwrap();
        assert!(history_dir.exists());
        assert!(history_dir.is_dir());
        // work_history 应该总是在本地路径下
        let path_str = history_dir.to_string_lossy();
        assert!(path_str.contains("work-history"));
    }

    #[test]
    fn test_completion_dir() {
        let completion_dir = Paths::completion_dir().unwrap();
        let path_str = completion_dir.to_string_lossy();
        assert!(path_str.contains("completions"));
    }

    #[test]
    fn test_workflow_dir() {
        let workflow_dir = Paths::workflow_dir().unwrap();
        assert!(workflow_dir.exists());
        assert!(workflow_dir.is_dir());
    }

    #[test]
    fn test_config_file_paths() {
        // 测试所有支持的 shell 配置文件路径
        let shells = vec![
            Shell::Zsh,
            Shell::Bash,
            Shell::Fish,
            Shell::PowerShell,
            Shell::Elvish,
        ];

        for shell in shells {
            let config_file = Paths::config_file(&shell);
            match config_file {
                Ok(path) => {
                    // 验证路径格式正确
                    assert!(!path.to_string_lossy().is_empty());
                }
                Err(_) => {
                    // Windows 上某些 shell 可能不支持，这是正常的
                    #[cfg(target_os = "windows")]
                    {
                        // Windows 上只有 PowerShell 应该成功
                        if matches!(shell, Shell::PowerShell) {
                            panic!("PowerShell config file should be available on Windows");
                        }
                    }
                }
            }
        }
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_shell_config_paths_unix() {
        let zsh_config = Paths::config_file(&Shell::Zsh).unwrap();
        assert!(zsh_config.to_string_lossy().ends_with(".zshrc"));

        let bash_config = Paths::config_file(&Shell::Bash).unwrap();
        let bash_path = bash_config.to_string_lossy();
        assert!(
            bash_path.ends_with(".bash_profile") || bash_path.ends_with(".bashrc"),
            "Bash config should be .bash_profile or .bashrc"
        );
    }

    #[test]
    fn test_work_history_always_local() {
        let work_history = Paths::work_history_dir().unwrap();
        let home = Paths::home_dir().unwrap();
        let local_base = home.join(".workflow");

        // work_history 应该总是在本地路径下
        assert!(work_history.starts_with(&local_base));

        // 确保不在 iCloud 路径下（如果 iCloud 可用）
        #[cfg(target_os = "macos")]
        {
            let icloud_base = home
                .join("Library")
                .join("Mobile Documents")
                .join("com~apple~CloudDocs")
                .join(".workflow");
            if icloud_base.exists() {
                assert!(!work_history.starts_with(&icloud_base));
            }
        }
    }

    #[test]
    fn test_completion_dir_is_local() {
        let completion_dir = Paths::completion_dir().unwrap();
        let home = Paths::home_dir().unwrap();
        let local_base = home.join(".workflow");

        // completion 应该总是在本地路径下
        assert!(completion_dir.starts_with(&local_base));
    }

    #[test]
    fn test_storage_location() {
        let location = Paths::storage_location();
        assert!(!location.is_empty());
        // 应该是 "iCloud Drive (synced across devices)" 或 "Local storage"
        assert!(location == "iCloud Drive (synced across devices)" || location == "Local storage");
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn test_non_macos_always_local() {
        assert!(!Paths::is_config_in_icloud());
        assert_eq!(Paths::storage_location(), "Local storage");
    }
}
