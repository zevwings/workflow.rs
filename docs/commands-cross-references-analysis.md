# src/commands 目录相互引用分析报告

## 概述

本报告分析了 `src/commands` 目录下所有跨模块的相互引用情况。根据要求，`src/commands` 目录下的模块之间禁止相互引用。

## 引用分类说明

- ✅ **允许的引用**：同一模块内的引用（如 `config/export.rs` → `config::helpers`）
- ❌ **禁止的引用**：跨模块的引用（如 `branch/create.rs` → `pr::helpers`）

---

## 跨模块引用清单

### 1. ❌ `check` 模块被多个模块引用

`check` 模块被以下模块引用（违反规则）：

| 引用文件 | 引用内容 | 用途 |
|---------|---------|------|
| `commit/amend.rs` | `use crate::commands::check;` | 环境检查 |
| `commit/reword.rs` | `use crate::commands::check;` | 环境检查 |
| `commit/squash.rs` | `use crate::commands::check;` | 环境检查 |
| `branch/rename.rs` | `use crate::commands::check;` | 环境检查 |
| `branch/delete.rs` | `use crate::commands::check;` | 环境检查 |
| `branch/sync.rs` | `use crate::commands::check;` | 环境检查 |
| `pr/create.rs` | `use crate::commands::check;` | 环境检查 |
| `pr/merge.rs` | `use crate::commands::check;` | 环境检查 |
| `pr/rebase.rs` | `use crate::commands::check;` | 环境检查 |
| `pr/pick.rs` | `use crate::commands::check;` | 环境检查 |
| `pr/sync.rs` | `use crate::commands::check;` | 环境检查 |
| `repo/clean.rs` | `use crate::commands::check;` | 环境检查 |
| `setup/command.rs` | `crate::commands::check::check::CheckCommand::verify_and_display_all` | 配置验证 |

**影响范围**：13 个文件

---

### 2. ❌ `pr` 模块被其他模块引用

`pr` 模块的 `helpers` 被以下模块引用（违反规则）：

| 引用文件 | 引用内容 | 用途 |
|---------|---------|------|
| `branch/create.rs` | `use crate::commands::pr::helpers::handle_stash_pop_result;` | 处理 stash pop 结果 |
| `branch/switch.rs` | `use crate::commands::pr::helpers::handle_stash_pop_result;` | 处理 stash pop 结果 |

**影响范围**：2 个文件

---

### 3. ❌ `setup` 模块引用其他模块

`setup` 模块引用了其他模块（违反规则）：

| 引用文件 | 引用内容 | 用途 |
|---------|---------|------|
| `setup/github.rs` | `use crate::commands::github::helpers::collect_github_account;` | 收集 GitHub 账号信息 |

**影响范围**：1 个文件

---

## 同一模块内的引用（允许）

以下引用属于同一模块内部，是允许的：

### `config` 模块内部引用
- `config/export.rs` → `config::helpers`, `config::validate`
- `config/import.rs` → `config::helpers`, `config::validate`
- `config/validate.rs` → `config::helpers`

### `setup` 模块内部引用
- `setup/command.rs` → `setup::log`, `setup::types`, `setup::{github, jira, llm}`
- `setup/github.rs` → `setup::types`
- `setup/jira.rs` → `setup::types`
- `setup/llm.rs` → `setup::types`
- `setup/log.rs` → `setup::types`

### `pr` 模块内部引用
- `pr/close.rs` → `pr::helpers`
- `pr/merge.rs` → `pr::helpers`
- `pr/rebase.rs` → `pr::helpers`
- `pr/create.rs` → `pr::helpers`
- `pr/pick.rs` → `pr::helpers`

### `branch` 模块内部引用
- `branch/rename.rs` → `branch::helpers`
- `branch/switch.rs` → `branch::helpers`
- `branch/delete.rs` → `branch::helpers`

### `commit` 模块内部引用
- `commit/amend.rs` → `commit::helpers`
- `commit/reword.rs` → `commit::helpers`
- `commit/squash.rs` → `commit::helpers`

### `stash` 模块内部引用
- `stash/apply.rs` → `stash::helpers`
- `stash/pop.rs` → `stash::helpers`

### `jira` 模块内部引用
- `jira/changelog.rs` → `jira::helpers`
- `jira/info.rs` → `jira::helpers`
- `jira/attachments.rs` → `jira::helpers`
- `jira/comments.rs` → `jira::helpers`
- `jira/related.rs` → `jira::helpers`

### `migrate` 模块内部引用
- `migrate/migrations.rs` → `migrate::history`

### `github` 模块内部引用
- `github/github.rs` → `github::helpers`

---

## 统计摘要

### 跨模块引用统计

| 被引用的模块 | 引用次数 | 引用文件数 |
|------------|---------|-----------|
| `check` | 13 | 13 |
| `pr` | 2 | 2 |
| `github` | 1 | 1 |
| **总计** | **16** | **16** |

### 需要重构的文件

需要重构以消除跨模块引用的文件共 **16 个**：

1. `commit/amend.rs`
2. `commit/reword.rs`
3. `commit/squash.rs`
4. `branch/rename.rs`
5. `branch/delete.rs`
6. `branch/sync.rs`
7. `branch/create.rs`
8. `branch/switch.rs`
9. `pr/create.rs`
10. `pr/merge.rs`
11. `pr/rebase.rs`
12. `pr/pick.rs`
13. `pr/sync.rs`
14. `repo/clean.rs`
15. `setup/command.rs`
16. `setup/github.rs`

---

## 重构建议

### 1. `check` 模块的引用问题

**问题**：`check` 模块被 13 个文件引用，用于环境检查。

**建议**：
- 将 `check` 模块的功能提取到 `src/lib/base/verify/` 或 `src/lib/base/check/` 中
- 所有命令模块都引用 `lib` 层的检查功能，而不是 `commands::check`

### 2. `pr::helpers::handle_stash_pop_result` 的引用问题

**问题**：`branch` 模块的两个文件引用了 `pr::helpers` 中的函数。

**建议**：
- 将 `handle_stash_pop_result` 函数移动到 `src/lib/git/stash.rs` 或 `src/lib/base/util/` 中
- 作为通用的 Git 操作辅助函数，不应该属于 `commands::pr` 模块

### 3. `setup::github` 引用 `github::helpers` 的问题

**问题**：`setup/github.rs` 引用了 `github::helpers::collect_github_account`。

**建议**：
- 将 `collect_github_account` 函数移动到 `src/lib/github/` 或 `src/lib/base/github/` 中
- 作为通用的 GitHub 配置收集功能，不应该属于 `commands::github` 模块

---

## 下一步行动

1. ✅ 完成分析报告（本文件）
2. ⏳ 重构 `check` 模块的引用（13 个文件）
3. ⏳ 重构 `handle_stash_pop_result` 的引用（2 个文件）
4. ⏳ 重构 `collect_github_account` 的引用（1 个文件）
5. ⏳ 添加 lint 规则或 CI 检查，防止未来出现跨模块引用
