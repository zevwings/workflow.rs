# 别名系统待办事项

## 📋 概述

本文档列出别名系统相关的待办功能，包括别名配置、别名展开和别名管理命令。

---

## ❌ 待实现功能

### 1. 别名系统

#### 1.1 别名配置
- ❌ 在配置文件中定义别名
- ❌ 支持命令参数传递
- ❌ 支持别名嵌套（别名引用别名）

**功能**：支持自定义命令别名，简化常用命令输入。

**配置格式**：
```toml
[aliases]
ci = "pr create"
cm = "pr merge"
js = "jira search"
ji = "jira info"
```

**使用示例**：
```bash
workflow ci                                        # 等同于 workflow pr create
workflow cm                                        # 等同于 workflow pr merge
workflow js "project = PROJ"                       # 等同于 workflow jira search "project = PROJ"
workflow ji PROJ-123                               # 等同于 workflow jira info PROJ-123
```

---

## 🏗️ 实现方案

### 1. 创建别名管理模块

#### 1.1 文件结构

```
src/lib/base/alias/
├── mod.rs          # 模块声明和导出
└── manager.rs      # AliasManager 实现
```

#### 1.2 核心数据结构

```rust
// src/lib/base/alias/mod.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AliasConfig {
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}

pub mod manager;
pub use manager::AliasManager;
```

#### 1.3 AliasManager 实现

```rust
// src/lib/base/alias/manager.rs
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use crate::base::settings::paths::Paths;
use crate::jira::config::ConfigManager;

use super::AliasConfig;

/// 别名管理器
///
/// 提供别名的加载、展开、添加、删除等功能。
pub struct AliasManager {
    config: AliasConfig,
    config_path: PathBuf,
}

impl AliasManager {
    /// 加载别名配置
    ///
    /// 从 `workflow.toml` 配置文件中加载别名配置。
    ///
    /// # 返回
    ///
    /// 返回 `AliasManager` 实例，如果配置文件不存在则返回默认配置。
    ///
    /// # 错误
    ///
    /// 如果配置文件存在但读取失败，返回相应的错误信息。
    pub fn load() -> Result<Self> {
        let config_path = Paths::workflow_config()?;
        let manager = ConfigManager::<AliasConfig>::new(config_path.clone());
        let config = manager.read().unwrap_or_default();

        Ok(Self {
            config,
            config_path,
        })
    }

    /// 展开别名（支持嵌套）
    ///
    /// 将别名展开为完整命令，支持别名嵌套和参数传递。
    ///
    /// # 参数
    ///
    /// * `command` - 包含别名的命令字符串
    ///
    /// # 返回
    ///
    /// 返回展开后的命令字符串。如果不是别名，返回原命令。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// let manager = AliasManager::load()?;
    /// let expanded = manager.expand_alias("ci")?;
    /// // 如果 "ci" 是 "pr create" 的别名，返回 "pr create"
    /// ```
    pub fn expand_alias(&self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(command.to_string());
        }

        let alias_name = parts[0];

        // 检查是否是别名
        if let Some(alias_value) = self.config.aliases.get(alias_name) {
            // 递归展开（防止无限循环）
            let mut expanded = alias_value.clone();
            let mut visited = HashSet::new();
            visited.insert(alias_name.to_string());

            // 处理嵌套别名
            while let Some(next_alias) = self.find_alias_in_command(&expanded, &mut visited) {
                if let Some(next_value) = self.config.aliases.get(&next_alias) {
                    expanded = expanded.replace(&next_alias, next_value);
                } else {
                    break;
                }
            }

            // 添加剩余参数
            if parts.len() > 1 {
                let args = parts[1..].join(" ");
                expanded = format!("{} {}", expanded, args);
            }

            Ok(expanded)
        } else {
            Ok(command.to_string())
        }
    }

    /// 查找命令中的别名
    ///
    /// 在命令字符串中查找第一个别名，用于嵌套别名展开。
    ///
    /// # 参数
    ///
    /// * `command` - 命令字符串
    /// * `visited` - 已访问的别名集合（防止循环）
    ///
    /// # 返回
    ///
    /// 如果找到别名，返回别名名称；否则返回 `None`。
    fn find_alias_in_command(
        &self,
        command: &str,
        visited: &mut HashSet<String>,
    ) -> Option<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(first) = parts.first() {
            if self.config.aliases.contains_key(*first) && !visited.contains(*first) {
                return Some(first.to_string());
            }
        }
        None
    }

    /// 添加别名
    ///
    /// # 参数
    ///
    /// * `name` - 别名名称
    /// * `value` - 别名值（完整命令）
    ///
    /// # 错误
    ///
    /// 如果保存配置失败，返回相应的错误信息。
    pub fn add_alias(&mut self, name: &str, value: &str) -> Result<()> {
        self.config.aliases.insert(name.to_string(), value.to_string());
        self.save()
    }

    /// 删除别名
    ///
    /// # 参数
    ///
    /// * `name` - 要删除的别名名称
    ///
    /// # 错误
    ///
    /// 如果保存配置失败，返回相应的错误信息。
    pub fn remove_alias(&mut self, name: &str) -> Result<()> {
        self.config.aliases.remove(name);
        self.save()
    }

    /// 列出所有别名
    ///
    /// # 返回
    ///
    /// 返回所有别名的 HashMap。
    pub fn list_aliases(&self) -> &HashMap<String, String> {
        &self.config.aliases
    }

    /// 检查别名是否存在
    ///
    /// # 参数
    ///
    /// * `name` - 别名名称
    ///
    /// # 返回
    ///
    /// 如果别名存在返回 `true`，否则返回 `false`。
    pub fn has_alias(&self, name: &str) -> bool {
        self.config.aliases.contains_key(name)
    }

    /// 保存配置
    ///
    /// 将别名配置保存到 `workflow.toml` 文件。
    ///
    /// # 错误
    ///
    /// 如果保存失败，返回相应的错误信息。
    fn save(&self) -> Result<()> {
        let manager = ConfigManager::<AliasConfig>::new(self.config_path.clone());
        manager.write(&self.config)
    }
}
```

### 2. 在主入口集成别名展开

#### 2.1 修改主入口文件

```rust
// src/bin/workflow.rs
use workflow::base::alias::AliasManager;
use anyhow::Result;
use clap::Parser;

use workflow::cli::Cli;

fn main() -> Result<()> {
    // ... 现有初始化代码 ...

    // 获取原始命令行参数
    let args: Vec<String> = std::env::args().collect();

    // 检查第一个参数是否是别名
    if args.len() > 1 {
        let first_arg = &args[1];

        // 尝试加载别名管理器
        if let Ok(mut alias_manager) = AliasManager::load() {
            // 展开别名
            if let Ok(expanded) = alias_manager.expand_alias(first_arg) {
                if expanded != *first_arg {
                    // 别名已展开，重新构建命令行参数
                    let expanded_parts: Vec<String> = expanded
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect();

                    // 重新构建参数：workflow + 展开的命令 + 剩余参数
                    let mut new_args = vec!["workflow".to_string()];
                    new_args.extend(expanded_parts);
                    new_args.extend(args.iter().skip(2).cloned());

                    // 重新解析
                    let cli = Cli::parse_from(new_args);
                    return handle_commands(cli);
                }
            }
        }
    }

    // 如果没有别名或展开失败，使用原始参数
    let cli = Cli::parse();
    handle_commands(cli)
}

fn handle_commands(cli: Cli) -> Result<()> {
    // ... 现有命令处理逻辑 ...
}
```

### 3. 添加别名管理命令

#### 3.1 扩展 CLI 定义

```rust
// src/lib/cli/commands.rs
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    // ... 现有命令 ...

    /// Manage command aliases
    ///
    /// Add, remove, and list command aliases for faster command input.
    Alias {
        #[command(subcommand)]
        subcommand: AliasSubcommand,
    },
}

#[derive(Subcommand)]
pub enum AliasSubcommand {
    /// List all defined aliases
    List,

    /// Add a new alias
    ///
    /// # Examples
    ///
    /// ```bash
    /// workflow alias add ci "pr create"
    /// workflow alias add cm "pr merge"
    /// ```
    Add {
        /// Alias name
        name: String,
        /// Command to alias (can include arguments)
        value: String,
    },

    /// Remove an alias
    ///
    /// # Examples
    ///
    /// ```bash
    /// workflow alias remove ci
    /// ```
    Remove {
        /// Alias name to remove
        name: String,
    },
}
```

#### 3.2 实现别名命令

```rust
// src/commands/alias/mod.rs
use crate::base::alias::AliasManager;
use anyhow::{Context, Result};
use crate::{log_info, log_success, log_warning};

pub mod list;
pub mod add;
pub mod remove;

pub use list::list;
pub use add::add;
pub use remove::remove;
```

```rust
// src/commands/alias/list.rs
use crate::base::alias::AliasManager;
use anyhow::Result;
use crate::{log_info, log_break};

pub fn list() -> Result<()> {
    let manager = AliasManager::load()?;
    let aliases = manager.list_aliases();

    if aliases.is_empty() {
        log_info!("No aliases defined");
        log_info!("Use 'workflow alias add <name> <command>' to create an alias");
        return Ok(());
    }

    log_break!();
    log_info!("Defined aliases:");
    log_break!();

    for (name, value) in aliases {
        log_info!("  {} = {}", name, value);
    }

    Ok(())
}
```

```rust
// src/commands/alias/add.rs
use crate::base::alias::AliasManager;
use anyhow::{Context, Result};
use crate::{log_success, log_warning};

pub fn add(name: String, value: String) -> Result<()> {
    let mut manager = AliasManager::load()?;

    // 检查别名是否已存在
    if manager.has_alias(&name) {
        log_warning!("Alias '{}' already exists", name);
        log_warning!("Use 'workflow alias remove {}' to remove it first", name);
        return Ok(());
    }

    manager
        .add_alias(&name, &value)
        .context("Failed to save alias")?;

    log_success!("Alias '{}' added: {}", name, value);
    log_info!("You can now use 'workflow {}' instead of 'workflow {}'", name, value);

    Ok(())
}
```

```rust
// src/commands/alias/remove.rs
use crate::base::alias::AliasManager;
use anyhow::{Context, Result};
use crate::{log_success, log_warning};

pub fn remove(name: String) -> Result<()> {
    let mut manager = AliasManager::load()?;

    if !manager.has_alias(&name) {
        log_warning!("Alias '{}' does not exist", name);
        return Ok(());
    }

    manager
        .remove_alias(&name)
        .context("Failed to remove alias")?;

    log_success!("Alias '{}' removed", name);

    Ok(())
}
```

#### 3.3 在主入口注册命令

```rust
// src/bin/workflow.rs
use workflow::commands::alias::{add, list, remove};
use workflow::cli::{AliasSubcommand, Commands};

fn handle_commands(cli: Cli) -> Result<()> {
    match &cli.command {
        // ... 现有命令 ...

        Commands::Alias { subcommand } => match subcommand {
            AliasSubcommand::List => list()?,
            AliasSubcommand::Add { name, value } => add(name.clone(), value.clone())?,
            AliasSubcommand::Remove { name } => remove(name.clone())?,
        },
    }

    Ok(())
}
```

### 4. 更新 Settings 结构

#### 4.1 扩展 Settings

```rust
// src/lib/base/settings/settings.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    // ... 现有字段 ...

    #[serde(default)]
    pub aliases: HashMap<String, String>,
}
```

**注意**：如果别名配置已经通过 `AliasConfig` 管理，可以保持独立，不需要添加到 `Settings`。

---

## 📝 实现步骤

### 阶段 1：核心功能
1. [ ] 创建 `src/lib/base/alias/` 目录结构
2. [ ] 实现 `AliasConfig` 结构体
3. [ ] 实现 `AliasManager` 核心功能（load, expand_alias, save）
4. [ ] 添加单元测试

### 阶段 2：主入口集成
5. [ ] 修改 `src/bin/workflow.rs`，集成别名展开逻辑
6. [ ] 测试别名展开功能
7. [ ] 测试参数传递功能
8. [ ] 测试嵌套别名功能

### 阶段 3：管理命令
9. [ ] 扩展 `src/lib/cli/commands.rs`，添加 `Alias` 命令
10. [ ] 实现 `src/commands/alias/list.rs`
11. [ ] 实现 `src/commands/alias/add.rs`
12. [ ] 实现 `src/commands/alias/remove.rs`
13. [ ] 在主入口注册命令

### 阶段 4：测试和文档
14. [ ] 编写集成测试
15. [ ] 测试边界情况（循环别名、不存在的别名等）
16. [ ] 更新文档
17. [ ] 添加使用示例

---

## ✅ 验收标准

### 功能验收
- [ ] 能够在配置文件中定义别名
- [ ] 别名能够正确展开为完整命令
- [ ] 支持命令参数传递（`workflow ci --title "test"`）
- [ ] 支持别名嵌套（别名引用别名）
- [ ] 能够添加新别名（`workflow alias add <name> <command>`）
- [ ] 能够删除别名（`workflow alias remove <name>`）
- [ ] 能够列出所有别名（`workflow alias list`）

### 边界情况
- [ ] 处理循环别名（防止无限递归）
- [ ] 处理不存在的别名（返回原命令）
- [ ] 处理空别名配置
- [ ] 处理别名名称冲突（与现有命令冲突）

### 用户体验
- [ ] 别名展开对用户透明
- [ ] 错误信息清晰友好
- [ ] 命令帮助信息完整

---

## 🔍 技术细节

### 别名展开逻辑

1. **基本展开**：
   - 检查第一个参数是否是别名
   - 如果是，替换为别名值
   - 保留剩余参数

2. **嵌套别名处理**：
   - 使用 `HashSet` 跟踪已访问的别名（防止循环）
   - 递归展开嵌套别名
   - 最多展开深度限制（可选）

3. **参数传递**：
   - 别名展开后，将原始命令的剩余参数追加到展开后的命令
   - 例如：`workflow ci --title "test"` → `workflow pr create --title "test"`

### 配置文件格式

别名配置存储在 `workflow.toml` 中：

```toml
[aliases]
ci = "pr create"
cm = "pr merge"
js = "jira search"
ji = "jira info"

# 支持嵌套别名
prc = "ci"  # prc -> ci -> pr create
```

### 错误处理

- 配置文件不存在：使用默认空配置
- 配置文件格式错误：返回错误，提示用户修复
- 别名循环：检测并返回错误
- 别名不存在：返回原命令（不报错）

---

## 📚 相关文档

- [UX 需求文档](../requirements/UX_REQUIREMENTS.md) - 别名系统需求
- [配置架构文档](../architecture/lib/SETTINGS_ARCHITECTURE.md) - 配置文件管理
- [CLI 架构文档](../architecture/lib/CLI_ARCHITECTURE.md) - 命令解析

---

## 🎯 优先级

**优先级**: 中

**原因**：
- 提高命令输入效率
- 简化常用命令
- 提升用户体验

**依赖**：
- 配置文件管理系统（已实现）
- CLI 命令解析系统（已实现）

---

**最后更新**: 2025-12-09
