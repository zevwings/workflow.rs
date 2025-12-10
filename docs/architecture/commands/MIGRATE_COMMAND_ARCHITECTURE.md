# 迁移系统架构说明

## 📁 文件组织

迁移系统采用**版本化文件组织**，每个需要迁移的版本都有独立的文件：

```
src/commands/migrate/
├── mod.rs              # 迁移命令主入口
├── migrations.rs       # 迁移注册和路由（版本列表）
├── history.rs          # 迁移历史管理
├── v1_0_0.rs          # v1.0.0 迁移实现
├── v1_1_0.rs          # v1.1.0 迁移实现（未来）
├── v2_0_0.rs          # v2.0.0 迁移实现（未来）
└── README.md          # 本文件
```

## 🔢 迁移版本 vs 软件版本

**重要**：迁移版本号**独立于**软件版本号！

- **软件版本**（如 `1.4.8`）：表示软件本身的版本，在 `Cargo.toml` 中定义
- **迁移版本**（如 `v1.0.0`）：表示**配置格式的版本**，只有当配置格式发生变化时才需要迁移

### 为什么独立？

1. **不是每个软件版本都需要迁移**
   - 软件可能从 1.0.0 升级到 1.4.8，但配置格式没有变化
   - 只有当配置格式发生变化时，才需要创建迁移

2. **迁移版本反映配置格式变化**
   - `v1.0.0`：第一个需要迁移的配置格式版本（如 branch.toml → repositories.toml）
   - `v1.1.0`：如果未来有新的配置格式变化
   - `v2.0.0`：如果有重大配置格式变化（如完全重构配置结构）

3. **示例场景**
   - 软件版本 1.4.8 时，实现了第一个配置迁移 → 迁移版本 `v1.0.0`
   - 软件版本 2.0.0 时，配置格式没有变化 → 不需要新迁移
   - 软件版本 2.1.0 时，配置格式有变化 → 迁移版本 `v1.1.0`（或 `v2.0.0`，取决于变化程度）

### 命名规范

迁移版本使用语义化版本（Semantic Versioning）：
- **Major**（v1.0.0 → v2.0.0）：重大配置格式变化，不向后兼容
- **Minor**（v1.0.0 → v1.1.0）：新增配置项或格式变化，向后兼容
- **Patch**（v1.0.0 → v1.0.1）：通常不使用，因为配置格式变化通常需要 minor 或 major

## 🔄 添加新迁移版本的步骤

当需要添加新的迁移版本时（例如 v1.1.0），按以下步骤操作：

### 1. 创建迁移实现文件

创建 `src/commands/migrate/v1_1_0.rs`：

```rust
//! v1.1.0 迁移实现
//!
//! 迁移描述：例如，迁移某个配置格式

use anyhow::{Context, Result};
use crate::{log_info, log_success};

/// v1.1.0 迁移实现
pub fn migrate_v1_1_0(dry_run: bool, cleanup: bool) -> Result<()> {
    // 1. 检测需要迁移的内容
    // 2. 执行迁移逻辑
    // 3. 可选：清理旧文件

    if dry_run {
        log_info!("Migration preview for v1.1.0...");
    } else {
        log_info!("Migrating to v1.1.0...");
        // 执行迁移
        log_success!("Migration to v1.1.0 completed!");
    }

    Ok(())
}
```

### 2. 在 mod.rs 中导出新模块

```rust
// src/commands/migrate/mod.rs
pub mod v1_1_0;  // 添加这一行
```

### 3. 在 mod.rs 中添加版本路由

```rust
// src/commands/migrate/mod.rs
fn migrate_version(version: &str, dry_run: bool, cleanup: bool) -> Result<()> {
    match version {
        "v1.0.0" => {
            v1_0_0::migrate_v1_0_0(dry_run, cleanup)?;
        }
        "v1.1.0" => {  // 添加这个分支
            v1_1_0::migrate_v1_1_0(dry_run, cleanup)?;
        }
        _ => {
            anyhow::bail!("Unknown migration version: {}", version);
        }
    }
    // ...
}
```

### 4. 在 migrations.rs 中注册新版本

```rust
// src/commands/migrate/migrations.rs

/// 所有可用的迁移版本
const ALL_MIGRATIONS: &[&str] = &["v1.0.0", "v1.1.0"];  // 添加 "v1.1.0"

/// 检查特定版本是否需要迁移
fn needs_migration(version: &str) -> Result<bool> {
    match version {
        "v1.0.0" => {
            // 检查是否存在 branch.toml
            let old_config_path = Paths::config_dir()?.join("branch.toml");
            Ok(old_config_path.exists())
        }
        "v1.1.0" => {  // 添加这个分支
            // 检查是否需要迁移的条件
            // 例如：检查某个旧配置是否存在
            Ok(true)  // 或根据实际情况返回
        }
        _ => Ok(false),
    }
}
```

## 📝 设计原则

1. **版本独立**：每个版本的迁移逻辑独立，互不干扰
2. **向后兼容**：新版本迁移不应该破坏已迁移的配置
3. **幂等性**：迁移应该可以安全地重复执行（通过历史记录避免）
4. **可测试**：每个迁移版本都可以独立测试

## ⚠️ 注意事项

1. **版本顺序**：迁移会按照 `ALL_MIGRATIONS` 数组的顺序执行
2. **历史记录**：已执行的迁移会记录在 `migration-history.toml` 中，避免重复执行
3. **清理操作**：`cleanup` 参数控制是否删除旧配置文件，谨慎使用
4. **错误处理**：迁移失败时应该提供清晰的错误信息，不影响其他迁移

## 🔍 示例：v1.0.0 迁移

参考 `v1_0_0.rs` 的实现，了解完整的迁移实现模式：
- 检测旧配置
- 读取和转换配置
- 合并到新配置
- 保存新配置
- 可选清理旧文件
