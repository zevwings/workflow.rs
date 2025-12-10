//! 迁移注册和路由
//!
//! 管理所有可用的迁移版本，检测待迁移的版本。

use anyhow::Result;

use crate::base::settings::paths::Paths;
use crate::commands::migrate::history;

/// 所有可用的迁移版本
const ALL_MIGRATIONS: &[&str] = &["v1.0.0"];

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
fn needs_migration(version: &str) -> Result<bool> {
    match version {
        "v1.0.0" => {
            // 检查是否存在 branch.toml
            let old_config_path = Paths::config_dir()?.join("branch.toml");
            Ok(old_config_path.exists())
        }
        _ => Ok(false),
    }
}
