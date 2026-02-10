# 需求：合并 cli 和 commands 模块

**优先级**: 🔴 高
**类型**: Refactoring
**影响范围**: `crates/app/src/cli/`, `crates/app/src/commands/`

## 问题描述

当前 CLI 参数定义（`cli/`）和命令实现（`commands/`）分散在两个独立目录中，形成了 1:1 映射关系，但物理上分离。

### 当前状态

```
crates/app/src/
├── cli/                    # CLI 参数定义（17 个文件）
│   ├── pr.rs              # PR 命令参数
│   ├── jira.rs            # Jira 命令参数
│   ├── branch.rs          # Branch 命令参数
│   └── ...
├── commands/               # 命令实现（19 个子模块）
│   ├── pr/                # PR 命令实现
│   ├── jira/              # Jira 命令实现
│   ├── branch/            # Branch 命令实现
│   └── ...
```

### 职责划分

| 模块 | 职责 | 文件数 |
|------|------|--------|
| `cli/` | 定义 `clap` 参数结构（`Args`, `Command`） | 17 |
| `commands/` | 实现命令业务逻辑（`execute` 函数） | 19 |

## 为什么需要修改

### 1. **违反内聚性原则**

同一个功能的代码分散在两个目录：

```rust
// cli/pr.rs - 参数定义
pub enum PrSubcommand {
    Create { ... },
    Merge { ... },
}

// commands/pr/mod.rs - 命令实现
pub mod create;
pub mod merge;
```

**问题**:
- 开发新命令需要同时修改两个目录
- 代码跳转不直观（参数 → 实现需要切换目录）
- 增加认知负担

### 2. **维护成本高**

添加新命令的步骤：

1. 在 `cli/` 中定义参数结构
2. 在 `commands/` 中实现业务逻辑
3. 在 `cli/mod.rs` 中导出参数
4. 在 `commands/mod.rs` 中导出实现
5. 在 `bin/workflow.rs` 中添加路由

**5 个位置** 需要同步修改，容易遗漏。

### 3. **目录结构冗余**

```
cli/                     commands/
├── pr.rs               ├── pr/
├── jira.rs             ├── jira/
├── branch.rs           ├── branch/
└── ...                 └── ...
```

两个目录的结构几乎完全一致，造成冗余。

### 4. **不符合 Rust 社区惯例**

参考主流 Rust CLI 项目（cargo, ripgrep, bat）：

```rust
// 标准做法：参数和实现在同一模块
commands/
├── pr/
│   ├── cli.rs      // 参数定义
│   ├── merge.rs    // merge 实现
│   ├── create.rs   // create 实现
│   └── mod.rs      // 模块导出
```

## 解决方案

### 目标结构

```
crates/app/src/
├── commands/
│   ├── pr/
│   │   ├── cli.rs          # 参数定义（原 cli/pr.rs）
│   │   ├── merge.rs        # merge 实现
│   │   ├── create/         # create 子命令
│   │   │   ├── cli.rs
│   │   │   ├── execute.rs
│   │   │   └── mod.rs
│   │   └── mod.rs          # 导出 PrSubcommand + 实现
│   ├── jira/
│   │   ├── cli.rs          # 参数定义（原 cli/jira.rs）
│   │   ├── info.rs
│   │   ├── clean.rs
│   │   └── mod.rs
│   └── ...
└── cli/
    └── main.rs             # 仅保留顶层 Cli 和 Command 枚举
```

### 迁移步骤

#### 第一阶段：迁移单个命令（以 pr 为例）

```bash
# 1. 移动参数定义到 commands/pr/
mv crates/app/src/cli/pr.rs crates/app/src/commands/pr/cli.rs

# 2. 更新 commands/pr/mod.rs
cat > crates/app/src/commands/pr/mod.rs << 'EOF'
mod cli;
mod merge;
mod create;
// ...

pub use cli::PrSubcommand;  // 重新导出
EOF

# 3. 更新导入路径
# cli/main.rs 改为从 commands 导入
use crate::commands::pr::PrSubcommand;
```

#### 第二阶段：批量迁移所有命令

```bash
# 迁移脚本
for cmd in pr jira branch commit tag stash; do
    mv crates/app/src/cli/$cmd.rs crates/app/src/commands/$cmd/cli.rs
done
```

#### 第三阶段：清理 cli 目录

```bash
# 只保留顶层命令定义
crates/app/src/cli/
├── main.rs      # Cli 和 Command 顶层枚举
└── args.rs      # 共享参数类型（DryRunArgs, ForceArgs 等）
```

### 代码示例

**迁移前**:
```rust
// cli/pr.rs
#[derive(Subcommand)]
pub enum PrSubcommand {
    Merge { ... },
}

// commands/pr/merge.rs
pub fn execute() { ... }
```

**迁移后**:
```rust
// commands/pr/cli.rs
#[derive(Subcommand)]
pub enum PrSubcommand {
    Merge { ... },
}

// commands/pr/merge.rs
pub fn execute() { ... }

// commands/pr/mod.rs
mod cli;
mod merge;

pub use cli::PrSubcommand;
pub use merge::execute as merge_execute;
```

## 影响评估

### 影响范围

| 类型 | 影响 |
|------|------|
| **文件移动** | 17 个 CLI 文件迁移到 commands 子目录 |
| **导入路径** | `bin/workflow.rs` 需要更新导入 |
| **模块导出** | `lib.rs` 简化，移除 `pub mod cli` |
| **破坏性** | 无（内部重构，不影响外部 API） |

### 优点

- ✅ **内聚性提升**：相关代码集中在一起
- ✅ **维护成本降低**：新增命令只需修改一个目录
- ✅ **代码导航更直观**：参数和实现在同一模块
- ✅ **符合社区惯例**：与主流 Rust CLI 项目一致

### 风险

- ⚠️ **迁移工作量**：需要移动 17 个文件并更新导入
- ⚠️ **测试需求**：需要全面测试所有命令

## 实施计划

### 阶段 1：准备（1 天）
- [ ] 创建迁移脚本
- [ ] 制定回滚方案
- [ ] 准备测试用例

### 阶段 2：迁移（2-3 天）
- [ ] 迁移 `pr` 命令（试点）
- [ ] 验证测试通过
- [ ] 批量迁移其他命令
- [ ] 清理旧的 `cli/` 目录

### 阶段 3：验证（1 天）
- [ ] 运行完整测试套件
- [ ] 手动测试所有命令
- [ ] 更新文档

## 相关文档

- [crates/app/src/cli/](../../crates/app/src/cli/)
- [crates/app/src/commands/](../../crates/app/src/commands/)
- [架构文档](../architecture.md)
