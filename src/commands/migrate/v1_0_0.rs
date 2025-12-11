//! v1.0.0 迁移实现
//!
//! 从 branch.toml 迁移到 repositories.toml

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::base::settings::paths::Paths;
use crate::branch::config::{BranchConfig, RepositoryConfig};
use crate::jira::config::ConfigManager;
use crate::{log_info, log_success};

/// v1.0.0 迁移：从 branch.toml 迁移到 repositories.toml
pub fn migrate_v1_0_0(dry_run: bool, cleanup: bool) -> Result<()> {
    // 1. 检测旧配置文件
    let old_config_path = Paths::config_dir()?.join("branch.toml");
    if !old_config_path.exists() {
        log_info!("No old configuration file found. Nothing to migrate.");
        return Ok(());
    }

    // 2. 读取旧配置
    let old_config = load_old_config(&old_config_path)?;

    // 3. 读取新配置（如果已存在）
    let mut new_config = BranchConfig::load().unwrap_or_default();

    // 4. 执行迁移
    let migration_result = perform_migration(&old_config, &mut new_config)?;

    // 5. 预览或执行
    if dry_run {
        preview_migration(&migration_result)?;
    } else {
        execute_migration(
            &migration_result,
            &mut new_config,
            cleanup,
            &old_config_path,
        )?;
    }

    Ok(())
}

/// 读取旧配置
fn load_old_config(path: &Path) -> Result<BranchConfig> {
    let manager = ConfigManager::<BranchConfig>::new(path.to_path_buf());
    manager.read().context("Failed to load old configuration")
}

/// 执行迁移
fn perform_migration(
    old_config: &BranchConfig,
    new_config: &mut BranchConfig,
) -> Result<MigrationResult> {
    let mut result = MigrationResult::default();

    for (repo_name, old_repo_config) in &old_config.repositories {
        // 检查新配置中是否已存在该仓库
        if let Some(new_repo_config) = new_config.repositories.get(repo_name) {
            // 合并配置：保留新配置中的 branch_prefix，合并 branch_ignore
            let merged_config = merge_repo_config(old_repo_config, new_repo_config);
            new_config.repositories.insert(repo_name.clone(), merged_config);
            result.merged_repos.push(repo_name.clone());
        } else {
            // 直接迁移：转换为新格式
            let migrated_config = convert_repo_config(old_repo_config);
            new_config.repositories.insert(repo_name.clone(), migrated_config);
            result.migrated_repos.push(repo_name.clone());
        }
    }

    Ok(result)
}

/// 合并仓库配置
fn merge_repo_config(
    old_config: &RepositoryConfig,
    new_config: &RepositoryConfig,
) -> RepositoryConfig {
    let mut merged = new_config.clone();

    // 合并 branch_ignore：合并两个列表，去重
    let mut ignore_set = std::collections::HashSet::new();
    for branch in &old_config.branch_ignore {
        ignore_set.insert(branch.clone());
    }
    for branch in &new_config.branch_ignore {
        ignore_set.insert(branch.clone());
    }
    merged.branch_ignore = ignore_set.into_iter().collect();
    merged.branch_ignore.sort();

    merged
}

/// 转换仓库配置（从旧格式到新格式）
fn convert_repo_config(old_config: &RepositoryConfig) -> RepositoryConfig {
    // 旧配置中的 ignore 字段已经通过 serde alias 自动映射到 branch_ignore
    // 所以直接克隆即可
    old_config.clone()
}

/// 预览迁移结果
fn preview_migration(result: &MigrationResult) -> Result<()> {
    log_info!("Migration preview (dry-run mode):");
    log_info!("");

    if !result.migrated_repos.is_empty() {
        log_info!("Repositories to migrate ({}):", result.migrated_repos.len());
        for repo in &result.migrated_repos {
            log_info!("  - {}", repo);
        }
        log_info!("");
    }

    if !result.merged_repos.is_empty() {
        log_info!("Repositories to merge ({}):", result.merged_repos.len());
        for repo in &result.merged_repos {
            log_info!("  - {}", repo);
        }
        log_info!("");
    }

    if result.migrated_repos.is_empty() && result.merged_repos.is_empty() {
        log_info!("No changes needed.");
    } else {
        log_info!("To execute migration, run: workflow migrate");
        log_info!("To keep old file after migration, run: workflow migrate --keep-old");
    }

    Ok(())
}

/// 执行迁移
fn execute_migration(
    result: &MigrationResult,
    new_config: &mut BranchConfig,
    cleanup: bool,
    old_config_path: &Path,
) -> Result<()> {
    // 保存新配置
    new_config.save().context("Failed to save new configuration")?;

    log_success!("Migration completed successfully!");
    log_info!("");
    log_info!("Migrated repositories: {}", result.migrated_repos.len());
    log_info!("Merged repositories: {}", result.merged_repos.len());

    // 清理旧配置文件（默认删除，除非用户指定 --keep-old）
    if cleanup {
        if old_config_path.exists() {
            fs::remove_file(old_config_path).context("Failed to remove old configuration file")?;
            log_success!(
                "Old configuration file removed: {}",
                old_config_path.display()
            );
        }
    } else {
        log_info!(
            "Old configuration file preserved: {}",
            old_config_path.display()
        );
        log_info!("Use 'workflow migrate' (without --keep-old) to remove it.");
    }

    Ok(())
}

/// 迁移结果
#[derive(Default)]
struct MigrationResult {
    /// 迁移的仓库列表（新仓库）
    migrated_repos: Vec<String>,
    /// 合并的仓库列表（已存在的仓库）
    merged_repos: Vec<String>,
}
