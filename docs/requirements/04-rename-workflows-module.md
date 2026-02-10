# 需求：重命名 workflows 模块为 interactive

**优先级**: 🟡 中
**类型**: Refactoring
**影响范围**: `crates/app/src/workflows/`

## 问题描述

当前 `workflows` 模块命名容易引起误解，实际职责是"交互式工作流"（配置向导、用户交互、验证流程），而不是传统意义的"业务工作流"。

### 当前状态

```
crates/app/src/workflows/
├── core/           # 核心抽象（WorkflowContext, WorkflowStage）
├── display/        # 显示格式化
├── platforms/      # 平台实现（GitHub, Jira, LLM, Log）
└── setup/          # 设置向导
```

### 模块实际职责

```rust
// workflows/core/context.rs
pub struct WorkflowContext {
    mode: WorkflowMode,  // Interactive or NonInteractive
    // ...
}

// workflows/platforms/github.rs
pub fn github_stage() -> Box<dyn WorkflowStage> {
    // 交互式配置 GitHub
}

// workflows/setup/orchestrator.rs
pub async fn run_setup_wizard() {
    // 运行交互式配置向导
}
```

**实际用途**：
- ✓ 交互式配置向导（setup wizard）
- ✓ 用户输入验证（validation）
- ✓ 平台账号配置（platform setup）
- ✓ 显示格式化（display formatting）

**非用途**：
- ✗ GitHub Actions workflows
- ✗ 业务流程编排（business process）
- ✗ CI/CD 流水线

## 为什么需要修改

### 1. **命名误导性强**

**问题场景**：

开发者看到 `workflows` 模块，可能会误以为：
- 这是 GitHub Actions workflows 的管理代码
- 这是业务工作流引擎（如 Temporal, Camunda）
- 这是任务编排系统

**实际情况**：
- 这只是 CLI 的交互式配置向导和用户界面

### 2. **与主流概念冲突**

在软件开发中，`workflow` 通常指：

| 概念 | 含义 | 示例 |
|------|------|------|
| **GitHub Workflows** | CI/CD 自动化流程 | `.github/workflows/ci.yml` |
| **Business Workflows** | 业务流程编排 | 订单审批流程、发布流程 |
| **ETL Workflows** | 数据处理流水线 | Airflow, Prefect |

**本项目的 workflows**：
- 仅仅是 **交互式 UI 逻辑**
- 与上述主流概念完全不同

### 3. **降低代码可读性**

```rust
// 当前命名
use crate::workflows::setup::run_setup_wizard;
// 问题：workflows 是什么工作流？

// 改进后
use crate::interactive::setup::run_setup_wizard;
// 清晰：这是交互式设置向导
```

### 4. **与项目实际名称冲突**

项目名称就叫 `workflow-rs`，但 `workflows` 模块与项目核心功能无关：

```
workflow-rs/              # 项目：Git workflow 工具
└── crates/app/src/
    └── workflows/        # ❌ 容易混淆：这不是项目的 workflow 功能
```

## 解决方案

### 推荐命名：`interactive`

```
crates/app/src/interactive/
├── core/           # 交互核心抽象
├── display/        # 显示格式化
├── platforms/      # 平台配置交互
└── setup/          # 设置向导
```

**理由**：
- ✅ 准确描述职责：交互式用户界面
- ✅ 与 `WorkflowMode::Interactive` 一致
- ✅ 避免与主流概念冲突
- ✅ 提高代码可读性

### 备选命名

| 名称 | 优点 | 缺点 |
|------|------|------|
| `wizard` | 强调向导功能 | 过于局限（不仅仅是向导） |
| `ui` | 简洁 | 过于宽泛（CLI 也是 UI） |
| `prompts` | 强调用户输入 | 不够准确（还包括显示） |
| `interactive` | 准确、清晰 | 稍长 |

### 迁移步骤

#### 步骤 1：重命名目录

```bash
# 重命名 workflows → interactive
mv crates/app/src/workflows crates/app/src/interactive
```

#### 步骤 2：更新模块声明

```rust
// lib.rs
// 迁移前
pub(crate) mod workflows;

// 迁移后
pub(crate) mod interactive;
```

#### 步骤 3：更新所有导入

```bash
# 批量替换
find crates/app/src -type f -name "*.rs" -exec sed -i '' 's/crate::workflows/crate::interactive/g' {} +
find crates/app/src -type f -name "*.rs" -exec sed -i '' 's/use workflows::/use interactive::/g' {} +
```

#### 步骤 4：更新文档注释

```rust
// interactive/mod.rs
//! 交互式工作流模块
//!
//! 提供 CLI 命令的用户交互逻辑，包括配置向导、验证流程、显示格式化等。
```

### 代码示例

**迁移前**:
```rust
// lib.rs
pub(crate) mod workflows;
pub use workflows::{WorkflowContext, WorkflowExecutor};

// commands/setup/mod.rs
use crate::workflows::setup::run_setup_wizard;
```

**迁移后**:
```rust
// lib.rs
pub(crate) mod interactive;
pub use interactive::{WorkflowContext, WorkflowExecutor};

// commands/setup/mod.rs
use crate::interactive::setup::run_setup_wizard;
```

## 影响评估

### 影响范围

| 类型 | 影响 |
|------|------|
| **目录重命名** | `workflows/` → `interactive/` |
| **模块声明** | `lib.rs` 更新 |
| **导入路径** | 所有 `use crate::workflows` 更新为 `use crate::interactive` |
| **破坏性** | 无（内部重构） |

### 搜索影响范围

```bash
# 查找所有使用 workflows 的地方
rg "workflows::" crates/app/src/
rg "use workflows" crates/app/src/
rg "mod workflows" crates/app/src/
```

### 优点

- ✅ **提高可读性**：命名准确描述职责
- ✅ **避免混淆**：不再与 GitHub Workflows 等概念冲突
- ✅ **概念一致性**：与 `WorkflowMode::Interactive` 对齐
- ✅ **降低认知负担**：新开发者更容易理解

### 风险

- ⚠️ **迁移工作量**：需要更新所有导入路径
- ⚠️ **测试需求**：确保重命名后功能正常

## 实施计划

### 阶段 1：准备（0.5 天）
- [ ] 搜索所有 `workflows` 引用
- [ ] 确认影响范围
- [ ] 准备回滚方案

### 阶段 2：重命名（1 天）
- [ ] 重命名目录 `workflows/` → `interactive/`
- [ ] 更新 `lib.rs` 模块声明
- [ ] 批量替换导入路径
- [ ] 更新文档注释

### 阶段 3：验证（0.5 天）
- [ ] 运行 `cargo build`
- [ ] 运行 `cargo test`
- [ ] 手动测试交互式命令（`workflow setup`, `workflow jira info`, 等）
- [ ] 更新项目文档

## 验证方法

```bash
# 1. 检查编译
cargo build

# 2. 运行测试
cargo test

# 3. 确认没有遗留的 workflows 引用
rg "workflows::" crates/app/src/
rg "use workflows" crates/app/src/

# 4. 确认目录已重命名
ls crates/app/src/interactive
ls crates/app/src/workflows 2>&1 | grep "No such file"
```

## 相关文档

- [crates/app/src/workflows/](../../crates/app/src/workflows/)
- [架构文档](../architecture.md)
- [开发指南](../development.md)
