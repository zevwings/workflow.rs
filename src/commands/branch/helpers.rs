//! 分支命令辅助函数
//!
//! 提供分支清理忽略列表和分支前缀的配置文件管理功能。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, IsTerminal};
use std::path::PathBuf;

use crate::base::dialog::InputDialog;
use crate::base::settings::paths::Paths;
use crate::git::GitRepo;
use crate::jira::config::ConfigManager;
use crate::{log_info, log_success, log_warning};

/// 分支配置结构体
///
/// 用于管理分支相关配置，包括忽略列表和分支前缀，按仓库名分组。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BranchConfig {
    /// 按仓库名分组的配置
    /// Key: 仓库名 (如 "owner/repo")
    /// Value: 仓库的分支配置
    #[serde(flatten)]
    pub repositories: HashMap<String, RepositoryConfig>,
}

/// 仓库配置
///
/// 每个仓库的分支相关配置，包括忽略分支列表和分支前缀。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// 分支前缀（可选）
    ///
    /// 用于生成分支名时添加前缀，例如 "feature"、"fix" 等。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_prefix: Option<String>,
    /// 需要忽略的分支列表
    ///
    /// 在分支清理时，这些分支不会被删除。
    /// 支持向后兼容：可以读取旧的 "ignore" 字段名。
    #[serde(default, alias = "ignore")]
    pub branch_ignore: Vec<String>,
    /// 是否已提示过设置分支前缀（用于避免重复提示）
    ///
    /// 这是一个内部字段，用于记录是否已经提示过用户设置分支前缀。
    /// 当值为 false 时，不会序列化到配置文件中。
    #[serde(default, skip_serializing_if = "is_false")]
    pub branch_prefix_prompted: bool,
}

/// 辅助函数：用于 skip_serializing_if，当值为 false 时跳过序列化
fn is_false(b: &bool) -> bool {
    !b
}

/// 验证仓库名格式
///
/// 验证仓库名是否符合 `owner/repo` 格式。
///
/// # 参数
/// * `repo_name` - 仓库名
///
/// # 返回
/// 如果格式正确返回 `Ok(())`，否则返回错误。
///
/// # 示例
/// ```
/// use workflow::commands::branch::validate_repo_name_format;
/// assert!(validate_repo_name_format("owner/repo").is_ok());
/// assert!(validate_repo_name_format("invalid").is_err());
/// ```
pub fn validate_repo_name_format(repo_name: &str) -> Result<()> {
    let parts: Vec<&str> = repo_name.split('/').collect();
    if parts.len() != 2 {
        anyhow::bail!(
            "Invalid repository name format: '{}'. Expected format: 'owner/repo'",
            repo_name
        );
    }

    let owner = parts[0];
    let repo = parts[1];

    if owner.is_empty() {
        anyhow::bail!(
            "Invalid repository name format: '{}'. Owner cannot be empty",
            repo_name
        );
    }

    if repo.is_empty() {
        anyhow::bail!(
            "Invalid repository name format: '{}'. Repository name cannot be empty",
            repo_name
        );
    }

    Ok(())
}

impl BranchConfig {
    /// 获取配置文件路径
    pub fn config_path() -> Result<PathBuf> {
        Paths::branch_config()
    }

    /// 读取配置文件
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        let manager = ConfigManager::<Self>::new(path);
        manager.read()
    }

    /// 获取指定仓库的分支前缀
    ///
    /// # 参数
    /// * `repo_name` - 仓库名（格式：`owner/repo`）
    ///
    /// # 返回
    /// 如果仓库配置了 `branch_prefix`，返回前缀字符串的引用；否则返回 `None`。
    pub fn get_branch_prefix_for_repo(&self, repo_name: &str) -> Option<&str> {
        self.repositories
            .get(repo_name)
            .and_then(|repo_config| repo_config.branch_prefix.as_deref())
    }

    /// 获取当前仓库的分支前缀
    ///
    /// 从当前 Git 仓库获取 owner/repo，然后查找对应的配置。
    /// 如果当前仓库未配置 prefix，返回 None。
    ///
    /// # 返回
    /// 如果当前仓库配置了 `branch_prefix`，返回前缀字符串的引用；否则返回 `None`。
    ///
    /// # 错误
    /// 如果无法获取当前仓库信息，返回相应的错误。
    pub fn get_current_repo_branch_prefix(&self) -> Result<Option<&str>> {
        let repo_name = GitRepo::extract_repo_name().context("Failed to extract repo name")?;
        Ok(self.get_branch_prefix_for_repo(&repo_name))
    }
}

/// 保存配置文件
pub fn save(config: &BranchConfig) -> Result<()> {
    let path = BranchConfig::config_path()?;
    let manager = ConfigManager::<BranchConfig>::new(path);
    manager.write(config)
}

/// 获取指定仓库的忽略分支列表
pub fn get_ignore_branches(config: &BranchConfig, repo_name: &str) -> Vec<String> {
    config
        .repositories
        .get(repo_name)
        .map(|repo| repo.branch_ignore.clone())
        .unwrap_or_default()
}

/// 添加分支到忽略列表
pub fn add_ignore_branch(
    config: &mut BranchConfig,
    repo_name: String,
    branch_name: String,
) -> Result<bool> {
    let repo = config.repositories.entry(repo_name).or_default();

    if repo.branch_ignore.contains(&branch_name) {
        return Ok(false); // 已存在
    }

    repo.branch_ignore.push(branch_name);
    Ok(true) // 新添加
}

/// 从忽略列表移除分支
pub fn remove_ignore_branch(
    config: &mut BranchConfig,
    repo_name: &str,
    branch_name: &str,
) -> Result<bool> {
    if let Some(repo) = config.repositories.get_mut(repo_name) {
        if let Some(pos) = repo.branch_ignore.iter().position(|b| b == branch_name) {
            repo.branch_ignore.remove(pos);
            // 如果列表为空且没有其他配置，移除整个仓库配置
            if repo.branch_ignore.is_empty()
                && repo.branch_prefix.is_none()
                && !repo.branch_prefix_prompted
            {
                config.repositories.remove(repo_name);
            }
            return Ok(true); // 已移除
        }
    }
    Ok(false) // 不存在
}

/// 为当前仓库设置分支前缀
///
/// # 参数
/// * `prefix` - 分支前缀，如果为 None 则通过交互式输入获取
///
/// # 返回
/// 成功返回 `Ok(())`，失败返回错误
pub fn set_branch_prefix(prefix: Option<String>) -> Result<()> {
    let repo_name = GitRepo::extract_repo_name().context("Failed to extract repository name")?;

    // 验证仓库名格式
    validate_repo_name_format(&repo_name).context("Invalid repository name format")?;

    let prefix = if let Some(p) = prefix {
        p
    } else {
        InputDialog::new("Enter branch prefix (e.g., 'feature', 'fix'), or press Enter to remove:")
            .allow_empty(true)
            .prompt()
            .context("Failed to get branch prefix")?
    };

    let mut config = BranchConfig::load().context("Failed to load branch config")?;
    let repo_config = config.repositories.entry(repo_name.clone()).or_default();

    if prefix.trim().is_empty() {
        // 移除 prefix
        repo_config.branch_prefix = None;
        log_success!("Branch prefix removed for repository: {}", repo_name);
    } else {
        repo_config.branch_prefix = Some(prefix.trim().to_string());
        log_success!(
            "Branch prefix set to '{}' for repository: {}",
            prefix.trim(),
            repo_name
        );
    }

    save(&config).context("Failed to save branch config")?;
    Ok(())
}

/// 获取当前仓库的分支前缀
///
/// # 返回
/// 如果当前仓库配置了分支前缀，返回前缀字符串；否则返回 None
pub fn get_branch_prefix() -> Option<String> {
    let repo_name = match GitRepo::extract_repo_name() {
        Ok(name) => name,
        Err(_) => return None,
    };

    let config = match BranchConfig::load() {
        Ok(c) => c,
        Err(_) => return None,
    };

    config
        .get_branch_prefix_for_repo(&repo_name)
        .map(|s| s.to_string())
}

/// 移除当前仓库的分支前缀
///
/// # 返回
/// 成功返回 `Ok(())`，失败返回错误
pub fn remove_branch_prefix() -> Result<()> {
    set_branch_prefix(Some(String::new()))
}

/// 检测并提示设置 branch_prefix（如果需要）
///
/// 第一次在某个仓库使用需要生成分支名的命令时，如果未配置 branch_prefix，
/// 直接提示用户输入 prefix。
///
/// # 注意事项
/// - 只在交互式环境中提示（检查是否为 TTY）
/// - 每个仓库只提示一次（使用配置文件中的 `branch_prefix_prompted` 字段持久化记录）
/// - 如果已配置或已提示过，直接返回
/// - **不会打断调用流程**：即使发生错误（如用户按 Ctrl+C），也会返回 Ok(())，不影响主流程
pub fn check_and_prompt_branch_prefix() -> Result<()> {
    // 检查是否为交互式环境
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Ok(()); // 非交互式环境，跳过提示
    }

    // 获取当前仓库名
    let repo_name = match GitRepo::extract_repo_name() {
        Ok(name) => name,
        Err(_) => return Ok(()), // 不在 Git 仓库中，跳过
    };

    // 检查是否已配置
    let mut config = match BranchConfig::load() {
        Ok(c) => c,
        Err(_) => return Ok(()), // 加载失败，跳过提示
    };
    if config.get_branch_prefix_for_repo(&repo_name).is_some() {
        return Ok(()); // 已配置，无需提示
    }

    // 检查是否已经提示过（从配置文件中读取）
    let repo_config = config.repositories.get(&repo_name);
    if let Some(repo) = repo_config {
        if repo.branch_prefix_prompted {
            return Ok(()); // 已提示过，跳过
        }
    }

    // 提示用户并直接进入输入
    log_warning!(
        "Branch prefix is not configured for repository: {}",
        repo_name
    );
    log_info!("Branch prefix helps organize branches (e.g., 'feature/', 'fix/').");

    // 直接引导用户输入，无需确认对话框
    let prefix = match InputDialog::new(
        "Branch prefix is not set. Enter branch prefix (e.g., 'feature'), or press Enter to skip:",
    )
    .allow_empty(true)
    .prompt()
    {
        Ok(value) => value,
        Err(_) => {
            // 用户取消输入，视为跳过，不输出任何提示信息
            // 标记为已提示，避免下次再提示
            let repo_config = config.repositories.entry(repo_name.clone()).or_default();
            repo_config.branch_prefix_prompted = true;
            let _ = save(&config); // 忽略保存错误
            return Ok(()); // 不中断流程
        }
    };

    if !prefix.trim().is_empty() {
        // 设置 prefix（如果出错，记录但不中断流程）
        if let Err(e) = set_branch_prefix(Some(prefix.trim().to_string())) {
            log_warning!(
                "Failed to save branch prefix: {}. PR creation will continue without prefix.",
                e
            );
        } else {
            log_success!("Branch prefix configured successfully!");
        }
    }

    // 无论用户是否输入了 prefix，都标记为已提示，避免下次再提示
    let repo_config = config.repositories.entry(repo_name).or_default();
    repo_config.branch_prefix_prompted = true;
    let _ = save(&config); // 忽略保存错误

    Ok(())
}
