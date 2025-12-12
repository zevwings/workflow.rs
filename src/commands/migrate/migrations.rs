//! 迁移注册和路由
//!
//! 管理所有可用的迁移版本，检测待迁移的版本。

use color_eyre::Result;

use crate::commands::migrate::history;

/// 所有可用的迁移版本
const ALL_MIGRATIONS: &[&str] = &[];

/// 检测需要迁移的版本
pub fn detect_pending_migrations() -> Result<Vec<String>> {
    let completed = history::load_migration_history()?
        .iter()
        .map(|e| e.version.clone())
        .collect::<std::collections::HashSet<_>>();

    let pending: Vec<String> = ALL_MIGRATIONS
        .iter()
        .filter(|v| !completed.contains(**v))
        .map(|v| v.to_string())
        .collect();

    // 检查每个版本是否需要迁移
    let mut needed = Vec::new();
    for version in &pending {
        if needs_migration(version)? {
            needed.push(version.clone());
        }
    }

    Ok(needed)
}

/// 检查特定版本是否需要迁移
fn needs_migration(_version: &str) -> Result<bool> {
    Ok(false)
}
