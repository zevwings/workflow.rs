# CLI 模式恢复方案

## 🚀 快速执行清单

### ⚡ 完全移除 ratatui 和 crossterm

**决策**：**完全移除 ratatui 和 crossterm，不再使用 TUI 功能**

**Git 工作流**：**从 master 创建新分支，包含 feature/ratatui 和当前内容**

**执行方案**：

#### 方案 A：使用 merge（推荐，简单）⭐

```bash
# 1. 提交或暂存当前修改
git add -A
git stash  # 或 git commit -m "WIP: current changes"

# 2. 切换到 master
git checkout master
git pull origin master  # 如果有远程仓库

# 3. 创建新分支
git checkout -b feature/restore-cli

# 4. 合并 feature/ratatui 的内容
git merge feature/ratatui --no-commit

# 5. 恢复暂存的修改（如果有）
git stash pop

# 6. 提交合并结果
git add -A
git commit -m "merge: include feature/ratatui changes, ready to restore CLI"
```

**优点**：
- ✅ 简单直接，一条命令合并所有内容
- ✅ 保留完整的提交历史
- ✅ 自动处理所有文件

**缺点**：
- ⚠️ 会包含 feature/ratatui 的所有提交（包括不需要的）

---

#### 方案 B：使用 cherry-pick（精确控制）

如果需要选择性应用提交，可以使用 cherry-pick：

```bash
# 1. 提交或暂存当前修改
git add -A
git stash  # 或 git commit -m "WIP: current changes"

# 2. 切换到 master
git checkout master
git pull origin master  # 如果有远程仓库

# 3. 创建新分支
git checkout -b feature/restore-cli

# 4. 查看 feature/ratatui 的提交列表
git log feature/ratatui --oneline

# 5. 选择性地 cherry-pick 提交
# 方式 1：cherry-pick 所有提交（从 master 到 feature/ratatui）
git cherry-pick master..feature/ratatui

# 方式 2：cherry-pick 特定提交
# git cherry-pick <commit-hash1> <commit-hash2> ...

# 6. 恢复暂存的修改（如果有）
git stash pop

# 7. 提交（如果有未提交的修改）
git add -A
git commit -m "cherry-pick: include feature/ratatui changes, ready to restore CLI"
```

**优点**：
- ✅ 可以选择性地应用提交
- ✅ 可以跳过不需要的提交
- ✅ 更精确的控制

**缺点**：
- ⚠️ 需要知道要 cherry-pick 哪些提交
- ⚠️ 如果提交很多，会比较繁琐
- ⚠️ 可能产生冲突需要解决

---

#### 方案对比

| 方案 | 命令 | 复杂度 | 保留历史 | 选择性 | 推荐度 |
|------|------|--------|----------|--------|--------|
| **merge** | `git merge feature/ratatui` | 低 | ✅ 完整 | ❌ 全部 | ⭐⭐⭐⭐⭐ |
| **cherry-pick** | `git cherry-pick master..feature/ratatui` | 中 | ✅ 完整 | ✅ 可选 | ⭐⭐⭐⭐ |

---

#### 推荐方案

**如果 feature/ratatui 的所有内容都需要**：
- ✅ 使用 **merge**（方案 A）- 简单直接

**如果只需要 feature/ratatui 的部分提交**：
- ✅ 使用 **cherry-pick**（方案 B）- 精确控制

**对于当前场景（包含 feature/ratatui 和当前内容）**：
- ✅ 推荐使用 **merge**（方案 A）- 包含所有内容，简单直接

**分支结构**：
```
master
  ├── feature/ratatui (保留，包含 ratatui 实现)
  └── feature/restore-cli (新分支，将恢复传统 CLI)
```

**说明**：
- ✅ 从干净的 master 基础开始
- ✅ 包含 feature/ratatui 的所有修改
- ✅ 包含当前工作目录的修改
- ✅ 保留 feature/ratatui 分支完整
- ✅ 在新分支上恢复传统 CLI

**详细执行清单**：请参考 [`CLI_RESTORE_EXECUTION.md`](./CLI_RESTORE_EXECUTION.md)

---

### 5 分钟快速恢复

```bash
# 1. 确保在 feature/ratatui 分支
git checkout feature/ratatui

# 2. 提交当前修改（如果有未提交的）
git add -A
git commit -m "WIP: before removing ratatui and restoring CLI"

# 3. 修改 Cargo.toml（已完成 ✅）
# - 取消注释：colored, dialoguer, indicatif
# - 完全移除：ratatui, crossterm

# 4. 移除 UI 和 logging 目录
rm -rf src/lib/base/ui/
rm -rf src/lib/logging/  # 如果未被使用

# 5. 编译验证
cargo build

# 6. 修复编译错误（如果有）
# 主要检查是否有地方引用了 ui 或 logging 模块
```

---

### 详细执行步骤

#### 步骤 1：准备（2 分钟）

```bash
# 确保在 feature/ratatui 分支
git checkout feature/ratatui

# 检查当前状态
git status

# 提交所有修改（如果有）
git add -A
git commit -m "WIP: before removing ratatui and restoring CLI"
```

---

## 📋 完整执行清单

### 必须执行的操作

1. **修改 Cargo.toml** ✅（已完成）
   - [x] 取消注释：`colored = "2.1"`
   - [x] 取消注释：`dialoguer = "0.11"`
   - [x] 取消注释：`indicatif = "0.17"`
   - [x] 移除：`ratatui = "0.27"`
   - [x] 移除：`crossterm = "0.28"`

2. **移除代码目录**
   - [ ] 移除 `src/lib/base/ui/` 目录
   - [ ] 检查并移除 `src/lib/logging/` 目录（如果未被使用）

3. **修复模块引用**
   - [ ] 检查 `src/lib/base/mod.rs`（已确认：无需修改）
   - [ ] 检查 `src/lib.rs`（已确认：无需修改）
   - [ ] 处理 `src/lib/logging/logger.rs`（如果 logging 模块被使用）

4. **编译验证**
   - [ ] `cargo build` 成功
   - [ ] `cargo test` 通过

5. **功能验证**
   - [ ] 测试所有交互式命令
   - [ ] 验证日志输出正常

---

---

## 📋 方案概述

本方案旨在**完全移除 ratatui 和 crossterm**，恢复传统 CLI 模式，使用 `colored`、`dialoguer`、`indicatif` 等传统 CLI 库。

**目标**：
- ✅ **完全移除** ratatui 和 crossterm 依赖
- ✅ **完全移除** `src/lib/base/ui/` 目录
- ✅ 恢复传统 CLI 依赖（colored, dialoguer, indicatif）
- ✅ 确保所有命令正常工作
- ✅ 减少二进制体积（节省 ~700-1100 KB）

**预计时间**：1-2 小时（如果代码结构清晰）

**重要说明**：
- ⚠️ **不再使用 ratatui**，完全移除，不保留
- ⚠️ **不再使用 crossterm**，完全移除，不保留
- ✅ 所有 TUI 相关代码将被移除

---

## 🔍 当前状态分析

### 依赖状态

**已注释的传统 CLI 依赖**：
- `colored = "2.1"` ❌
- `dialoguer = "0.11"` ❌
- `indicatif = "0.17"` ❌

**已添加的 TUI 依赖**：
- `ratatui = "0.27"` ✅
- `crossterm = "0.28"` ✅

### 代码使用情况

**使用传统库的文件**（需要恢复依赖）：
- `src/lib/base/util/logger.rs` - 使用 `colored`
- `src/lib/base/util/confirm.rs` - 使用 `dialoguer::Confirm`
- `src/lib/base/http/retry.rs` - 使用 `dialoguer::Confirm`
- `src/commands/lifecycle/update.rs` - 使用 `indicatif::ProgressBar`
- 20+ 个命令文件使用 `dialoguer::Input`、`Select`、`MultiSelect`

**使用 TUI 的文件**（需要移除或替换）：
- `src/lib/base/ui/` - 整个目录（ratatui 组件）**需要完全移除**
- `src/lib/logging/logger.rs` - 使用了 `ui::theme` 和 `ratatui::style` **需要修复或移除**
- `src/lib/logging/` - 整个目录（如果未被使用，可以移除）

---

## 📝 恢复步骤

### 阶段 1：准备和备份（15 分钟）

#### 1.1 Git 工作流：从 master 创建新分支

**决策**：**从 master 创建新分支，包含 feature/ratatui 和当前内容**

**执行步骤**：

```bash
# 1. 保存当前修改（暂存或提交）
git status
git add -A
git stash  # 暂存当前修改（或使用 git commit）

# 2. 切换到 master 分支
git checkout master

# 3. 确保 master 是最新的
git pull origin master  # 如果有远程仓库

# 4. 从 master 创建新分支（用于恢复传统 CLI）
git checkout -b feature/restore-cli

# 5. 合并 feature/ratatui 的内容到新分支
git merge feature/ratatui --no-commit

# 6. 如果有暂存的修改，恢复
git stash pop  # 如果有暂存的修改

# 7. 提交合并结果
git add -A
git commit -m "merge: include feature/ratatui changes, ready to restore CLI

- Merge feature/ratatui branch into restore-cli
- Include all ratatui implementation
- Include current working directory changes
- Ready to remove ratatui and restore traditional CLI"
```

**分支结构**：
```
master
  ├── feature/ratatui (保留，包含 ratatui 实现)
  └── feature/restore-cli (新分支，将恢复传统 CLI)
```

**说明**：
- ✅ 从干净的 master 基础开始
- ✅ 包含 feature/ratatui 的所有修改
- ✅ 包含当前工作目录的修改
- ✅ 保留 feature/ratatui 分支完整
- ✅ 在新分支上恢复传统 CLI

#### 1.2 记录当前状态（可选）

```bash
# 在新分支上记录依赖树
cargo tree > dependencies-before-restore.txt

# 记录文件列表（如果目录还存在）
find src/lib/base/ui -type f > ui-files-before.txt 2>/dev/null || echo "UI directory not found"
find src/lib/logging -type f > logging-files-before.txt 2>/dev/null || echo "Logging directory not found"
```

---

### 阶段 2：恢复依赖（10 分钟）

#### 2.1 修改 Cargo.toml

编辑 `Cargo.toml`，**完全移除** ratatui 和 crossterm：

```toml
[dependencies]
# ... 其他依赖保持不变 ...

# 恢复传统 CLI 依赖
colored = "2.1"
dialoguer = "0.11"
indicatif = "0.17"

# 完全移除 TUI 依赖（不再使用）
# ratatui = "0.27"  # 已移除
# crossterm = "0.28"  # 已移除
```

**修改后的完整依赖列表**：

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
clap_complete = "4.5"
anyhow = "1.0"
duct = "0.13"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
chrono = "0.4"
# 传统 CLI 依赖
colored = "2.1"
dialoguer = "0.11"
indicatif = "0.17"
# 已移除：ratatui, crossterm
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
atty = "0.2"
reqwest = { version = "0.11", features = ["json", "blocking", "rustls-tls"], default-features = false }
regex = "1.10"
open = "5.0"
zip = "0.6"
flate2 = "1.0"
walkdir = "2.4"
tar = "0.4"
sha2 = "0.10"
dirs = "5.0"
```

#### 2.2 更新依赖

```bash
# 更新依赖
cargo update

# 验证依赖解析
cargo check
```

**预期结果**：
- ✅ 传统 CLI 依赖恢复
- ⚠️ 编译错误（因为代码仍在使用 ratatui）

---

### 阶段 3：移除 TUI 代码（30 分钟）

#### 3.1 移除 UI 和 logging 模块

```bash
# 完全移除整个 UI 目录（不再使用）
rm -rf src/lib/base/ui/

# 检查并移除 logging 目录（如果未被使用）
# 先检查是否被使用
grep -r "use.*logging" src/ || echo "logging module not used"
grep -r "logging::" src/ || echo "logging module not used"

# 如果未被使用，移除
rm -rf src/lib/logging/
```

**移除的文件**：

**UI 模块**（完全移除）：
- `src/lib/base/ui/mod.rs`
- `src/lib/base/ui/dialogs.rs`
- `src/lib/base/ui/components.rs`
- `src/lib/base/ui/progress.rs`
- `src/lib/base/ui/theme.rs`
- `src/lib/base/ui/layout.rs`
- `src/lib/base/ui/log_viewer.rs`
- `src/lib/base/ui/file_log_viewer.rs`

**logging 模块**（如果未被使用，完全移除）：
- `src/lib/logging/mod.rs`
- `src/lib/logging/logger.rs`（包含 ratatui 依赖）
- `src/lib/logging/tracing.rs`（可能包含 ratatui 依赖）

#### 3.2 检查并修复模块引用

检查以下文件是否引用了 `ui` 模块：

```bash
# 搜索 ui 模块引用
grep -r "use.*ui::" src/
grep -r "from.*ui::" src/
grep -r "base::ui" src/
```

**需要检查的文件**：
- `src/lib/base/mod.rs` - 可能导出 `ui` 模块（已确认：无）
- `src/lib/logging/logger.rs` - **使用了 `ui::theme`** ⚠️
- `src/lib.rs` - 可能导出 `ui` 模块（已确认：无）

#### 3.3 处理 logging 模块

**发现**：`src/lib/logging/` 目录存在，但可能未被使用。

**检查是否被使用**：
```bash
# 检查是否有地方引用 logging 模块
grep -r "use.*logging" src/
grep -r "logging::" src/
```

**决策**：

**选项 A：如果 logging 模块未被使用**（推荐）：
```bash
# 完全移除 logging 目录（包含 ratatui 相关代码）
rm -rf src/lib/logging/
```

**选项 B：如果 logging 模块被使用**：
需要修复 `src/lib/logging/logger.rs`，移除 ratatui 依赖：

```rust
// 移除
// use crate::base::ui::theme::Theme;

// 替换 format_with_style 函数，使用 colored 替代
use colored::*;

// 修改 success/error/warning/info/debug 方法
pub fn success(message: impl fmt::Display) -> String {
    format!("{} {}", "✓".green().bold(), message)
}

pub fn error(message: impl fmt::Display) -> String {
    format!("{} {}", "✗".red().bold(), message)
}

// ... 类似修改其他方法
```

#### 3.4 修复模块引用

**`src/lib/base/mod.rs`**（已确认无需修改）：
- ✅ 没有 `pub mod ui;`，无需修改

**`src/lib.rs`**（已确认无需修改）：
- ✅ 没有导出 `ui` 模块，无需修改

**`src/lib/logging/logger.rs`**（需要处理）：
- ⚠️ 使用了 `crate::base::ui::theme::Theme`
- ⚠️ 使用了 `ratatui::style`
- 需要移除或修复（见 3.3）

---

### 阶段 4：修复编译错误（1-2 小时）

#### 4.1 检查编译错误

```bash
cargo build 2>&1 | tee build-errors.log
```

#### 4.2 修复错误分类

**错误类型 1：缺少 `colored`、`dialoguer`、`indicatif`**
- ✅ 已恢复依赖，应该自动解决

**错误类型 2：引用了 `ui` 模块**
- 需要移除或替换相关引用

**错误类型 3：使用了 ratatui 组件**
- 需要替换为传统 CLI 库

#### 4.3 修复步骤

**步骤 1：修复 `src/lib/base/mod.rs`**

```rust
// 移除
// pub mod ui;

// 确保其他模块正常导出
pub mod http;
pub mod llm;
pub mod prompt;
pub mod settings;
pub mod shell;
pub mod util;
```

**步骤 2：修复 `src/lib/logging/logger.rs`（如果存在）**

如果使用了 `ui::theme`，需要：
- 移除相关导入
- 使用 `colored` 替代（如果需要）

**步骤 3：检查 `src/lib.rs`**

```rust
// 移除 ui 相关导出
// pub mod ui;
```

**步骤 4：验证编译**

```bash
cargo build
```

---

### 阶段 5：验证功能（1 小时）

#### 5.1 编译验证

```bash
# 完整编译
cargo build --release

# 检查二进制大小
ls -lh target/release/workflow
```

**预期结果**：
- ✅ 编译成功
- ✅ 二进制大小减少 ~700-1100 KB

#### 5.2 功能测试

**测试交互式命令**：

```bash
# 测试确认对话框
workflow github remove  # 需要确认

# 测试输入
workflow jira info  # 需要输入 ticket ID

# 测试选择
workflow github switch  # 需要选择账号

# 测试进度条
workflow update  # 显示下载进度
```

**测试列表**：

- [ ] `workflow github list` - 列表显示
- [ ] `workflow github add` - 交互式输入
- [ ] `workflow github remove` - 确认对话框
- [ ] `workflow github switch` - 选择对话框
- [ ] `workflow jira info` - 输入对话框
- [ ] `workflow jira attachments` - 输入对话框
- [ ] `workflow log download` - 输入对话框
- [ ] `workflow log search` - 输入对话框
- [ ] `workflow pr create` - 输入和多选
- [ ] `workflow config setup` - 输入和选择
- [ ] `workflow update` - 进度条显示

#### 5.3 运行测试套件

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test --lib
cargo test --test '*'
```

---

### 阶段 6：清理和优化（30 分钟）

#### 6.1 清理未使用的代码

```bash
# 检查是否有未使用的导入
cargo clippy -- -W unused-imports

# 检查是否有死代码
cargo clippy -- -W dead-code
```

#### 6.2 更新文档

更新以下文档（如果存在）：
- `README.md` - 移除 TUI 相关说明
- `docs/` - 更新架构文档
- `CHANGELOG.md` - 记录恢复操作

#### 6.3 提交更改

```bash
# 检查更改
git status
git diff

# 提交
git add -A
git commit -m "restore: revert to traditional CLI mode

- Restore colored, dialoguer, indicatif dependencies
- Remove ratatui and crossterm dependencies
- Remove src/lib/base/ui/ directory
- Fix module references and imports
- Reduce binary size by ~700-1100 KB"
```

---

## 🔧 详细修复指南

### 修复模块引用

#### 修复 `src/lib/base/mod.rs`

```rust
//! Base 基础设施模块

pub mod http;
pub mod llm;
pub mod prompt;
pub mod settings;
pub mod shell;
pub mod util;

// 移除以下行
// pub mod ui;

// 重新导出保持不变
pub use http::{...};
pub use util::{...};
```

#### 修复 `src/lib/logging/logger.rs`（如果存在）

如果文件使用了 `ui::theme`：

```rust
// 移除
// use crate::base::ui::theme::Theme;

// 如果需要颜色，使用 colored
use colored::*;
```

---

### 验证命令功能

#### 测试确认对话框

```bash
# 应该显示 dialoguer 风格的确认
workflow github remove
# 输出：Remove account 'xxx'? [y/N]:
```

#### 测试输入对话框

```bash
# 应该显示 dialoguer 风格的输入
workflow jira info
# 输出：Enter Jira ticket ID (e.g., PROJ-123): _
```

#### 测试选择对话框

```bash
# 应该显示 dialoguer 风格的选择
workflow github switch
# 输出：
# Select account to switch to:
# > account1
#   account2
```

#### 测试进度条

```bash
# 应该显示 indicatif 风格的进度条
workflow update
# 输出：
# [████████████████░░░░░░░░] 60% Downloading...
```

---

## ⚠️ 潜在问题和解决方案

### 问题 1：编译错误 - 找不到 `ui` 模块

**错误信息**：
```
error[E0432]: unresolved import `crate::base::ui`
```

**解决方案**：
1. 检查所有引用 `ui` 模块的文件
2. 移除或注释相关导入
3. 如果功能需要，使用传统 CLI 库替代

---

### 问题 2：`confirm` 函数使用了 ratatui

**如果 `src/lib/base/util/confirm.rs` 使用了 ratatui**：

需要恢复为 `dialoguer` 实现：

```rust
use anyhow::{Context, Result};
use dialoguer::Confirm;

pub fn confirm(prompt: &str, default: bool, message: Option<&str>) -> Result<bool> {
    let confirmed = Confirm::new()
        .with_prompt(prompt)
        .default(default)
        .interact()
        .context("Failed to get user confirmation")?;

    if !confirmed {
        if let Some(msg) = message {
            anyhow::bail!("{}", msg);
        }
    }

    Ok(confirmed)
}
```

---

### 问题 3：进度条使用了 ratatui

**如果 `src/commands/lifecycle/update.rs` 使用了 ratatui 进度条**：

需要恢复为 `indicatif` 实现：

```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(size);
pb.set_style(
    ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .progress_chars("#>-"),
);
```

---

### 问题 4：命令使用了 ratatui 对话框

**如果命令文件使用了 `InputDialog`、`SelectDialog` 等**：

需要替换为 `dialoguer`：

```rust
// 替换前（ratatui）
use crate::base::ui::dialogs::InputDialog;
let input = InputDialog::new("Enter value").show()?;

// 替换后（dialoguer）
use dialoguer::Input;
let input: String = Input::new()
    .with_prompt("Enter value")
    .interact()?;
```

---

## 📊 验证清单

### 编译验证

- [ ] `cargo build` 成功
- [ ] `cargo build --release` 成功
- [ ] `cargo test` 通过
- [ ] `cargo clippy` 无严重警告

### 功能验证

- [ ] 所有交互式命令正常工作
- [ ] 确认对话框显示正常
- [ ] 输入对话框显示正常
- [ ] 选择对话框显示正常
- [ ] 进度条显示正常
- [ ] 日志输出颜色正常

### 体积验证

- [ ] 二进制大小减少
- [ ] 依赖数量减少
- [ ] 编译时间减少

---

## 🎯 预期结果

### 依赖变化

**完全移除**（不再使用）：
- ❌ `ratatui = "0.27"` - 完全移除
- ❌ `crossterm = "0.28"` - 完全移除

**恢复**（传统 CLI）：
- ✅ `colored = "2.1"` - 恢复
- ✅ `dialoguer = "0.11"` - 恢复
- ✅ `indicatif = "0.17"` - 恢复

### 代码变化

**完全移除**（不再使用）：
- ❌ `src/lib/base/ui/` 目录（整个目录）- 完全移除
- ❌ 所有 ratatui 相关代码引用

**保持不变**：
- ✅ 所有命令文件（使用传统 CLI 库）
- ✅ `src/lib/base/util/` 目录
- ✅ 其他核心模块

**需要修复**：
- ⚠️ `src/lib/base/mod.rs` - 移除 `pub mod ui;`
- ⚠️ `src/lib/logging/logger.rs` - 如果使用了 `ui::theme`，需要修复
- ⚠️ `src/lib.rs` - 如果导出了 `ui` 模块，需要移除

### 体积变化

- **减少**：~700-1100 KB（移除 ratatui + crossterm）
- **增加**：~100-200 KB（恢复传统 CLI 库）
- **净减少**：~500-900 KB

---

## 📚 相关文档

- `docs/todo/CLI_RESTORE_ANALYSIS.md` - 详细分析文档
- `.git-separation-plan.md` - Git 分支分离计划

---

## ✅ 完成标准

恢复方案完成的标准：

1. ✅ 所有依赖正确恢复/移除
2. ✅ 代码编译成功
3. ✅ 所有测试通过
4. ✅ 所有交互式命令正常工作
5. ✅ 二进制体积减少
6. ✅ 代码已提交

---

## 🔄 回滚方案

如果恢复过程中出现问题，可以回滚：

```bash
# 回滚到备份分支
git checkout backup/ratatui-state

# 或回滚最后一次提交
git reset --hard HEAD~1
```

---

## 📝 后续优化（可选）

恢复完成后，可以考虑以下优化：

1. **引入 `inquire`** 替代 `dialoguer`（更好的交互体验）
2. **引入 `tabled`** 用于列表展示（美观的表格）
3. **优化 `indicatif` 进度条样式**（更好的视觉效果）

详见 `docs/todo/CLI_RESTORE_ANALYSIS.md` 中的增强方案。
