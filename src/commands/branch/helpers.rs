//! 分支命令辅助函数
//!
//! 提供分支清理忽略列表的配置文件管理功能。

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::base::settings::paths::Paths;
use crate::jira::config::ConfigManager;

/// 分支配置结构体
///
/// 用于管理分支清理时的忽略列表，按仓库名分组。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BranchConfig {
    /// 按仓库名分组的忽略分支列表
    /// Key: 仓库名 (如 "owner/repo")
    /// Value: 仓库的忽略分支配置
    #[serde(flatten)]
    pub repositories: HashMap<String, RepositoryIgnore>,
}

/// 仓库忽略配置
///
/// 每个仓库的忽略分支列表。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepositoryIgnore {
    /// 需要忽略的分支列表
    pub ignore: Vec<String>,
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
        .map(|repo| repo.ignore.clone())
        .unwrap_or_default()
}

/// 添加分支到忽略列表
pub fn add_ignore_branch(
    config: &mut BranchConfig,
    repo_name: String,
    branch_name: String,
) -> Result<bool> {
    let repo = config.repositories.entry(repo_name).or_default();

    if repo.ignore.contains(&branch_name) {
        return Ok(false); // 已存在
    }

    repo.ignore.push(branch_name);
    Ok(true) // 新添加
}

/// 从忽略列表移除分支
pub fn remove_ignore_branch(
    config: &mut BranchConfig,
    repo_name: &str,
    branch_name: &str,
) -> Result<bool> {
    if let Some(repo) = config.repositories.get_mut(repo_name) {
        if let Some(pos) = repo.ignore.iter().position(|b| b == branch_name) {
            repo.ignore.remove(pos);
            // 如果列表为空，移除整个仓库配置
            if repo.ignore.is_empty() {
                config.repositories.remove(repo_name);
            }
            return Ok(true); // 已移除
        }
    }
    Ok(false) // 不存在
}
