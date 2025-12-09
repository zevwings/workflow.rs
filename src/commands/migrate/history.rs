//! 迁移历史管理
//!
//! 记录已执行的迁移，避免重复执行。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::base::settings::paths::Paths;
use crate::jira::config::ConfigManager;

/// 迁移历史记录
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MigrationHistory {
    pub migrations: Vec<MigrationEntry>,
}

/// 迁移记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationEntry {
    pub version: String,
    pub timestamp: String,
}

/// 获取迁移历史文件路径
fn get_history_path() -> Result<PathBuf> {
    Ok(Paths::config_dir()?.join("migration-history.toml"))
}

/// 加载迁移历史
pub fn load_migration_history() -> Result<Vec<MigrationEntry>> {
    let path = get_history_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let manager = ConfigManager::<MigrationHistory>::new(path);
    let history = manager.read().unwrap_or_default();
    Ok(history.migrations)
}

/// 记录迁移
pub fn record_migration(version: &str) -> Result<()> {
    let path = get_history_path()?;
    let mut history = if path.exists() {
        let manager = ConfigManager::<MigrationHistory>::new(path.clone());
        manager.read().unwrap_or_default()
    } else {
        MigrationHistory::default()
    };

    // 检查是否已记录
    if history.migrations.iter().any(|e| e.version == version) {
        return Ok(()); // 已记录，跳过
    }

    // 添加新记录
    let entry = MigrationEntry {
        version: version.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    history.migrations.push(entry);

    // 保存
    let manager = ConfigManager::<MigrationHistory>::new(path);
    manager.write(&history).context("Failed to save migration history")?;

    Ok(())
}

/// 检查特定版本是否已迁移
pub fn is_migration_completed(version: &str) -> Result<bool> {
    let history = load_migration_history()?;
    Ok(history.iter().any(|e| e.version == version))
}
