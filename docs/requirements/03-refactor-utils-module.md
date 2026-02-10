# 需求：重构 utils 模块，提升业务内聚性

**优先级**: 🟡 中
**类型**: Refactoring
**影响范围**: `crates/app/src/utils/`

## 问题描述

当前 `utils/` 模块包含与特定业务领域强相关的工具函数，违反了"通用工具模块"的定位。

### 当前状态

```
crates/app/src/utils/
├── mod.rs
├── branch.rs           # 分支相关工具（与 commands/branch 强相关）
├── jira.rs             # Jira 交互辅助（与 commands/jira 强相关）
└── pull_request.rs     # PR 生成辅助（与 commands/pr 强相关）
```

### 模块职责分析

| 文件 | 职责 | 实际使用者 |
|------|------|-----------|
| `branch.rs` | 分支名称生成、类型判断、slug 转换 | `commands/branch/*` |
| `jira.rs` | Jira ID 交互式输入、状态配置检查 | `commands/jira/*` |
| `pull_request.rs` | PR 标题/正文生成 | `commands/pr/create` |

## 为什么需要修改

### 1. **违反单一职责原则**

`utils/` 名称暗示"通用工具"，但实际内容是**业务辅助函数**：

```rust
// utils/jira.rs - 这不是通用工具，而是 Jira 命令的辅助函数
pub async fn get_jira_id_interactive(
    jira_repo: Arc<dyn JiraRepository>,
    // ...
) -> Result<String> {
    // Jira 特定的交互逻辑
}
```

**问题**:
- 命名误导：`utils` 应该是纯函数工具（如字符串处理、日期格式化）
- 实际是业务逻辑的一部分，应该与命令实现放在一起

### 2. **降低代码内聚性**

相关代码分散在不同目录：

```
commands/jira/
├── info.rs          # Jira info 命令
├── clean.rs         # Jira clean 命令
└── ...

utils/
└── jira.rs          # ❌ Jira 辅助函数却在这里
```

**开发者困惑**:
- "Jira 相关代码在哪里？" → 需要同时查看 `commands/jira` 和 `utils/jira.rs`
- 修改 Jira 功能需要跨目录操作

### 3. **增加跨模块依赖**

```rust
// commands/jira/info.rs
use crate::utils::jira::get_jira_id_interactive;  // ❌ 跨目录依赖

// 理想情况
use super::utils::get_jira_id_interactive;  // ✅ 模块内依赖
```

### 4. **不利于模块独立性**

如果未来需要将 `commands/jira` 提取为独立 crate：

- ❌ 当前：还需要携带 `utils/jira.rs`
- ✅ 重构后：整个 `commands/jira` 目录就是完整的功能

## 解决方案

### 目标结构

```
crates/app/src/commands/
├── branch/
│   ├── utils.rs        # ← 移动自 utils/branch.rs
│   ├── create.rs
│   ├── list.rs
│   └── mod.rs
├── jira/
│   ├── utils.rs        # ← 移动自 utils/jira.rs
│   ├── info.rs
│   ├── clean.rs
│   └── mod.rs
└── pr/
    ├── utils.rs        # ← 移动自 utils/pull_request.rs
    ├── create/
    ├── merge.rs
    └── mod.rs
```

### 迁移步骤

#### 步骤 1：移动文件

```bash
# 移动 utils/branch.rs → commands/branch/utils.rs
mv crates/app/src/utils/branch.rs crates/app/src/commands/branch/utils.rs

# 移动 utils/jira.rs → commands/jira/utils.rs
mv crates/app/src/utils/jira.rs crates/app/src/commands/jira/utils.rs

# 移动 utils/pull_request.rs → commands/pr/utils.rs
mv crates/app/src/utils/pull_request.rs crates/app/src/commands/pr/utils.rs
```

#### 步骤 2：更新模块声明

```rust
// commands/branch/mod.rs
mod utils;  // ✅ 添加

pub use utils::{
    branch_type_from_branch_name,
    generate_branch_name_from_jira,
    to_slug,
};
```

#### 步骤 3：更新导入路径

```rust
// 迁移前
use crate::utils::jira::get_jira_id_interactive;

// 迁移后
use super::utils::get_jira_id_interactive;
// 或
use crate::commands::jira::utils::get_jira_id_interactive;
```

#### 步骤 4：清理 utils 目录

```bash
# 如果 utils 目录为空，删除
rm -rf crates/app/src/utils/

# lib.rs 移除声明
# pub(crate) mod utils;  // ❌ 删除这行
```

### 代码示例

**迁移前**:
```rust
// utils/jira.rs
pub async fn get_jira_id_interactive(...) { ... }

// commands/jira/info.rs
use crate::utils::jira::get_jira_id_interactive;
```

**迁移后**:
```rust
// commands/jira/utils.rs
pub async fn get_jira_id_interactive(...) { ... }

// commands/jira/info.rs
use super::utils::get_jira_id_interactive;

// commands/jira/mod.rs
mod utils;
pub use utils::get_jira_id_interactive;  // 如需要公开导出
```

## 影响评估

### 影响范围

| 类型 | 影响 |
|------|------|
| **文件移动** | 3 个文件迁移到对应命令目录 |
| **导入路径** | 所有使用 `utils::*` 的地方需要更新 |
| **模块声明** | `lib.rs` 移除 `utils` 模块 |
| **破坏性** | 无（内部重构） |

### 优点

- ✅ **提升内聚性**：业务逻辑集中在同一目录
- ✅ **降低耦合**：减少跨目录依赖
- ✅ **提高可读性**：代码组织更清晰
- ✅ **便于模块化**：每个命令模块更独立

### 风险

- ⚠️ **迁移工作量**：需要更新所有导入路径
- ⚠️ **测试需求**：确保功能不受影响

## 实施计划

### 阶段 1：准备（0.5 天）
- [ ] 使用 `rg` 搜索所有 `use crate::utils` 的位置
- [ ] 记录所有需要更新的文件

### 阶段 2：迁移（1 天）
- [ ] 移动 `utils/branch.rs` → `commands/branch/utils.rs`
- [ ] 移动 `utils/jira.rs` → `commands/jira/utils.rs`
- [ ] 移动 `utils/pull_request.rs` → `commands/pr/utils.rs`
- [ ] 更新所有导入路径

### 阶段 3：清理（0.5 天）
- [ ] 删除空的 `utils/` 目录
- [ ] 更新 `lib.rs`
- [ ] 运行测试验证

## 验证方法

```bash
# 1. 检查编译
cargo build

# 2. 运行测试
cargo test

# 3. 搜索是否还有旧的 utils 导入
rg "use crate::utils::" crates/app/src/

# 4. 验证 utils 目录已删除
ls crates/app/src/utils 2>&1 | grep "No such file"
```

## 相关文档

- [crates/app/src/utils/](../../crates/app/src/utils/)
- [crates/app/src/commands/](../../crates/app/src/commands/)
- [02-merge-cli-commands.md](./02-merge-cli-commands.md)（可以一起重构）
