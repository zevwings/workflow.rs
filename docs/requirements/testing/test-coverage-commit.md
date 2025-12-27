# Commit 模块测试覆盖率改进计划

> Commit 模块测试覆盖率分析与改进方案

**状态**: 📋 待实施
**当前覆盖率**: 26.3% (90/342 行)
**目标覆盖率**: >80%
**需要提升**: +53.7% (+184 行)
**优先级**: ⭐⭐⭐ 高（核心业务逻辑）

---

## 📋 目录

- [执行摘要](#-执行摘要)
- [当前状态分析](#-当前状态分析)
- [测试覆盖缺失分析](#-测试覆盖缺失分析)
- [测试改进计划](#-测试改进计划)
- [实施优先级](#-实施优先级)
- [相关文档](#-相关文档)

---

## 📊 执行摘要

### 模块概述

Commit 模块是 Workflow CLI 的核心业务模块之一，负责：
- **Amend 操作**：修改最近的 commit（消息和文件）
- **Reword 操作**：修改 commit 消息（支持历史 commit）
- **Squash 操作**：压缩多个 commits 为一个
- **预览生成**：格式化显示操作预览信息
- **Rebase 集成**：使用 rebase 修改历史 commits

### 代码规模

| 指标 | 数值 |
|------|------|
| 总代码行数 | 1,165 行 |
| 可测试行数 | 342 行 |
| 已覆盖行数 | 90 行 |
| 未覆盖行数 | 252 行 |
| 测试代码行数 | 930 行 |

### 当前覆盖率

| 文件 | 覆盖率 | 已覆盖/可测试 | 状态 |
|------|--------|---------------|------|
| `amend.rs` | 95.7% | 44/46 | 🟢 优秀 |
| `reword.rs` | 21.5% | 28/130 | 🔴 低 |
| `squash.rs` | 10.8% | 18/166 | 🔴 极低 |
| **总计** | **26.3%** | **90/342** | 🔴 **需改进** |

### 核心问题

1. **squash.rs 覆盖率极低**（10.8%）：Squash 是复杂功能，但测试严重不足
2. **reword.rs 覆盖率低**（21.5%）：历史 commit reword 功能缺少测试
3. **Rebase 逻辑未测试**：Reword 和 Squash 都依赖 rebase，但 rebase 逻辑几乎未测试
4. **测试质量不均**：Amend 测试充分（95.7%），但 Reword 和 Squash 测试不足

---

## 📈 当前状态分析

### ✅ 已有测试

#### tests/commit/ 目录结构

```
tests/commit/
├── mod.rs      # 模块声明
├── amend.rs    # Amend 操作测试 ✅ 95.7%
├── reword.rs   # Reword 操作测试 ⚠️ 21.5%
└── squash.rs   # Squash 操作测试 ❌ 10.8%
```

#### 已覆盖功能

1. **amend.rs (95.7% 覆盖)** - **优秀**：
   - ✅ 创建预览信息
   - ✅ 格式化预览显示
   - ✅ 生成完成消息
   - ✅ 检查是否已推送
   - ❌ 仅缺少 2 行边界情况

2. **reword.rs (21.5% 覆盖)** - **不足**：
   - ✅ 格式化 commit 信息
   - ✅ 创建预览信息
   - ✅ 格式化预览显示
   - ❌ 历史 commit reword 未测试
   - ❌ Rebase 编辑器配置未测试
   - ❌ Rebase 执行未测试
   - ❌ 冲突处理未测试

3. **squash.rs (10.8% 覆盖)** - **严重不足**：
   - ✅ 创建预览信息（部分）
   - ✅ 格式化预览显示（部分）
   - ❌ 获取分支 commits 未测试
   - ❌ Squash 执行未测试
   - ❌ Rebase 编辑器配置未测试
   - ❌ Rebase 执行未测试
   - ❌ 冲突处理未测试

### 🚧 未覆盖功能

#### 1. reword.rs (102 行未覆盖) - **高优先级**

**核心功能**：
- 历史 commit reword 执行
- Rebase 编辑器配置
- Rebase 交互式执行
- 冲突检测和处理
- Stash 管理

**未测试的关键函数**：
```rust
// 历史 commit reword（核心功能）
pub fn reword_history(options: RewordHistoryOptions) -> Result<RewordHistoryResult>

// Rebase 编辑器配置
fn setup_rebase_editor_config(
    commit_sha: &str,
    new_message: &str,
) -> Result<RebaseEditorConfig>

// 执行 rebase
fn execute_rebase(
    commit_sha: &str,
    config: &RebaseEditorConfig,
) -> Result<bool>

// 清理编辑器配置
fn cleanup_rebase_editor_config(config: &RebaseEditorConfig) -> Result<()>

// 检查冲突
fn check_rebase_conflicts() -> Result<bool>

// 中止 rebase
fn abort_rebase() -> Result<()>
```

**测试难点**：
- 需要 Git 仓库环境
- 需要创建历史 commits
- 需要模拟 rebase 过程
- 需要模拟冲突场景
- 需要处理临时文件（编辑器脚本）

#### 2. squash.rs (148 行未覆盖) - **最高优先级**

**核心功能**：
- 获取分支 commits
- Squash 执行
- Rebase 编辑器配置
- Rebase 交互式执行
- 冲突检测和处理
- Stash 管理

**未测试的关键函数**：
```rust
// 获取分支 commits（重要功能）
pub fn get_branch_commits(current_branch: &str) -> Result<Vec<CommitInfo>>

// Squash 执行（核心功能）
pub fn squash(options: SquashOptions) -> Result<SquashResult>

// Rebase 编辑器配置
fn setup_rebase_editor_config(
    commit_shas: &[String],
    new_message: &str,
) -> Result<RebaseEditorConfig>

// 执行 rebase
fn execute_rebase(
    base_sha: &str,
    config: &RebaseEditorConfig,
) -> Result<bool>

// 清理编辑器配置
fn cleanup_rebase_editor_config(config: &RebaseEditorConfig) -> Result<()>

// 检查冲突
fn check_rebase_conflicts() -> Result<bool>

// 中止 rebase
fn abort_rebase() -> Result<()>

// 创建 sequence editor 脚本
fn create_sequence_editor_script(
    commit_shas: &[String],
    script_path: &Path,
) -> Result<()>

// 创建 message editor 脚本
fn create_message_editor_script(
    new_message: &str,
    script_path: &Path,
) -> Result<()>
```

**测试难点**：
- 需要 Git 仓库环境
- 需要创建多个 commits
- 需要检测基础分支
- 需要模拟 rebase 过程
- 需要模拟冲突场景
- 需要处理临时文件（编辑器脚本）

#### 3. amend.rs (2 行未覆盖) - **低优先级**

**未覆盖的边界情况**：
- 某些错误处理分支
- 某些格式化边界情况

---

## 🔍 测试覆盖缺失分析

### 1. squash.rs - Squash 操作（148 行未覆盖）

#### 缺失的测试场景

**获取分支 commits**：
- [ ] 从默认分支创建的分支
- [ ] 从非默认分支创建的分支
- [ ] 检测基础分支失败时的 fallback
- [ ] 没有 commits 的分支
- [ ] 获取 commit 信息失败

**Squash 执行**：
- [ ] 压缩 2 个 commits
- [ ] 压缩多个 commits（3+）
- [ ] 压缩所有分支 commits
- [ ] 自动 stash 未提交更改
- [ ] 不 stash 时有未提交更改（应失败）

**Rebase 编辑器配置**：
- [ ] 创建 sequence editor 脚本
- [ ] 创建 message editor 脚本
- [ ] 脚本内容正确性
- [ ] 脚本权限正确性（Unix）

**Rebase 执行**：
- [ ] Rebase 成功
- [ ] Rebase 冲突
- [ ] Rebase 失败
- [ ] 环境变量设置正确

**冲突处理**：
- [ ] 检测冲突
- [ ] 中止 rebase
- [ ] 恢复 stash

**清理**：
- [ ] 清理编辑器脚本
- [ ] 清理失败时的处理

**错误处理**：
- [ ] 无效的 commit SHA
- [ ] Commits 不连续
- [ ] 基础分支不存在
- [ ] Rebase 失败恢复

#### 建议的测试文件

```
tests/commit/
├── squash_basic.rs       # 新建：基础 squash 测试
├── squash_rebase.rs      # 新建：Rebase 逻辑测试
└── squash_conflicts.rs   # 新建：冲突处理测试
```

#### 测试策略

1. **使用 Git 测试仓库**：
   - 创建临时 Git 仓库
   - 创建多个 commits
   - 模拟分支结构

2. **测试 Rebase 编辑器**：
   - 验证脚本文件创建
   - 验证脚本内容
   - 验证环境变量设置

3. **模拟冲突**：
   - 创建冲突的 commits
   - 验证冲突检测
   - 验证 rebase 中止

### 2. reword.rs - Reword 操作（102 行未覆盖）

#### 缺失的测试场景

**历史 commit reword**：
- [ ] Reword 历史 commit（非 HEAD）
- [ ] Reword HEAD commit
- [ ] 自动 stash 未提交更改
- [ ] 不 stash 时有未提交更改（应失败）

**Rebase 编辑器配置**：
- [ ] 创建 sequence editor 脚本
- [ ] 创建 message editor 脚本
- [ ] 脚本内容正确性
- [ ] 脚本权限正确性（Unix）

**Rebase 执行**：
- [ ] Rebase 成功
- [ ] Rebase 冲突
- [ ] Rebase 失败
- [ ] 环境变量设置正确

**冲突处理**：
- [ ] 检测冲突
- [ ] 中止 rebase
- [ ] 恢复 stash

**清理**：
- [ ] 清理编辑器脚本
- [ ] 清理失败时的处理

**错误处理**：
- [ ] 无效的 commit SHA
- [ ] Commit 不存在
- [ ] Rebase 失败恢复

#### 建议的测试文件

```
tests/commit/
├── reword_history.rs     # 新建：历史 reword 测试
├── reword_rebase.rs      # 新建：Rebase 逻辑测试
└── reword_conflicts.rs   # 新建：冲突处理测试
```

#### 测试策略

1. **使用 Git 测试仓库**：
   - 创建临时 Git 仓库
   - 创建多个 commits
   - 测试不同位置的 commits

2. **测试 Rebase 编辑器**：
   - 验证脚本文件创建
   - 验证脚本内容
   - 验证环境变量设置

3. **模拟冲突**：
   - 创建冲突的 commits
   - 验证冲突检测
   - 验证 rebase 中止

### 3. amend.rs - Amend 操作（2 行未覆盖）

#### 缺失的测试场景

**边界情况**：
- [ ] 某些错误处理分支
- [ ] 某些格式化边界情况

#### 测试策略

1. **增强现有测试**：
   - 添加更多边界情况
   - 添加错误处理测试

---

## 📝 测试改进计划

### 阶段 1：高优先级测试（目标：50% 覆盖率）

#### 1.1 squash.rs 基础测试（预计 +70 行覆盖）

**文件**：`tests/commit/squash_basic.rs`

**测试用例**：
```rust
// 获取分支 commits 测试
#[test]
fn test_get_branch_commits_from_default_branch() { }

#[test]
fn test_get_branch_commits_from_non_default_branch() { }

#[test]
fn test_get_branch_commits_no_commits() { }

// 基础 squash 测试
#[test]
fn test_squash_two_commits() { }

#[test]
fn test_squash_multiple_commits() { }

#[test]
fn test_squash_all_branch_commits() { }

// Stash 管理测试
#[test]
fn test_squash_with_auto_stash() { }

#[test]
fn test_squash_without_stash_fails_with_uncommitted_changes() { }

// 预览测试
#[test]
fn test_create_squash_preview() { }

#[test]
fn test_format_squash_preview() { }
```

**工作量估计**：4-5 天

#### 1.2 reword.rs 历史 reword 测试（预计 +50 行覆盖）

**文件**：`tests/commit/reword_history.rs`

**测试用例**：
```rust
// 历史 reword 测试
#[test]
fn test_reword_history_commit() { }

#[test]
fn test_reword_head_commit() { }

// Stash 管理测试
#[test]
fn test_reword_with_auto_stash() { }

#[test]
fn test_reword_without_stash_fails_with_uncommitted_changes() { }

// 错误处理测试
#[test]
fn test_reword_invalid_commit_sha() { }

#[test]
fn test_reword_non_existing_commit() { }
```

**工作量估计**：3-4 天

### 阶段 2：中优先级测试（目标：70% 覆盖率）

#### 2.1 squash.rs Rebase 逻辑测试（预计 +40 行覆盖）

**文件**：`tests/commit/squash_rebase.rs`

**测试用例**：
```rust
// Rebase 编辑器配置测试
#[test]
fn test_setup_rebase_editor_config() { }

#[test]
fn test_sequence_editor_script_content() { }

#[test]
fn test_message_editor_script_content() { }

#[test]
fn test_script_permissions_on_unix() { }

// Rebase 执行测试
#[test]
fn test_execute_rebase_success() { }

#[test]
fn test_execute_rebase_with_env_vars() { }

// 清理测试
#[test]
fn test_cleanup_rebase_editor_config() { }
```

**工作量估计**：2-3 天

#### 2.2 reword.rs Rebase 逻辑测试（预计 +30 行覆盖）

**文件**：`tests/commit/reword_rebase.rs`

**测试用例**：
```rust
// Rebase 编辑器配置测试
#[test]
fn test_setup_rebase_editor_config() { }

#[test]
fn test_sequence_editor_script_content() { }

#[test]
fn test_message_editor_script_content() { }

// Rebase 执行测试
#[test]
fn test_execute_rebase_success() { }

#[test]
fn test_execute_rebase_with_env_vars() { }

// 清理测试
#[test]
fn test_cleanup_rebase_editor_config() { }
```

**工作量估计**：2 天

### 阶段 3：完善测试（目标：>80% 覆盖率）

#### 3.1 squash.rs 冲突处理测试（预计 +30 行覆盖）

**文件**：`tests/commit/squash_conflicts.rs`

**测试用例**：
```rust
// 冲突检测测试
#[test]
fn test_squash_detect_conflicts() { }

#[test]
fn test_squash_abort_on_conflicts() { }

#[test]
fn test_squash_restore_stash_on_conflicts() { }

// Rebase 失败测试
#[test]
fn test_squash_rebase_failure() { }

#[test]
fn test_squash_rebase_failure_cleanup() { }
```

**工作量估计**：2 天

#### 3.2 reword.rs 冲突处理测试（预计 +20 行覆盖）

**文件**：`tests/commit/reword_conflicts.rs`

**测试用例**：
```rust
// 冲突检测测试
#[test]
fn test_reword_detect_conflicts() { }

#[test]
fn test_reword_abort_on_conflicts() { }

#[test]
fn test_reword_restore_stash_on_conflicts() { }

// Rebase 失败测试
#[test]
fn test_reword_rebase_failure() { }

#[test]
fn test_reword_rebase_failure_cleanup() { }
```

**工作量估计**：1-2 天

#### 3.3 amend.rs 完善测试（预计 +2 行覆盖）

**增强现有测试**：
- 添加更多边界情况
- 添加错误处理测试

**工作量估计**：0.5 天

---

## 🎯 实施优先级

### P0 - 立即实施（2 周内）

| 任务 | 预计覆盖提升 | 工作量 | 负责人 |
|------|-------------|--------|--------|
| squash.rs 基础测试 | +20.5% | 4-5 天 | TBD |
| reword.rs 历史 reword 测试 | +14.6% | 3-4 天 | TBD |

**预期结果**：覆盖率从 26.3% 提升到 61.4%

### P1 - 短期实施（1 个月内）

| 任务 | 预计覆盖提升 | 工作量 | 负责人 |
|------|-------------|--------|--------|
| squash.rs Rebase 逻辑测试 | +11.7% | 2-3 天 | TBD |
| reword.rs Rebase 逻辑测试 | +8.8% | 2 天 | TBD |

**预期结果**：覆盖率从 61.4% 提升到 81.9%

### P2 - 中期实施（2 个月内）

| 任务 | 预计覆盖提升 | 工作量 | 负责人 |
|------|-------------|--------|--------|
| squash.rs 冲突处理测试 | +8.8% | 2 天 | TBD |
| reword.rs 冲突处理测试 | +5.8% | 1-2 天 | TBD |
| amend.rs 完善测试 | +0.6% | 0.5 天 | TBD |

**预期结果**：覆盖率从 81.9% 提升到 >97%（实际约 85-90%，考虑到部分代码难以测试）

---

## 📚 相关文档

### 项目文档

- [测试覆盖度提升综合方案](./test-coverage-improvement.md) - 整体测试覆盖率改进计划
- [测试标准](../../guidelines/testing/README.md) - 项目测试标准和最佳实践
- [开发指南](../../guidelines/development/README.md) - 开发规范和流程

### 架构文档

- [Commit 模块架构](../../architecture/commit.md) - Commit 模块设计文档
- [Git 模块架构](../../architecture/git.md) - Git 操作封装

### 源代码

- `src/lib/commit/` - Commit 模块源代码
- `tests/commit/` - Commit 模块测试代码

---

**最后更新**: 2025-12-24

