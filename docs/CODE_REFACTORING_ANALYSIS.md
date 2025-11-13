# 代码重构分析：重复代码抽取和简化

本文档分析了代码库中的重复代码模式，并提供了重构建议。

## 📋 目录

1. [Stash 操作模式](#1-stash-操作模式)
2. ~~[平台提供者调用模式](#2-平台提供者调用模式)~~ ✅ 已完成
3. [分支清理模式](#3-分支清理模式)
4. [用户确认模式](#4-用户确认模式)

---

## 1. Stash 操作模式

### 🔍 问题描述

在多个文件中，stash pop 的错误处理逻辑几乎完全相同：

**重复位置：**
- `src/commands/pr/integrate.rs` (3处)
- `src/commands/pr/create.rs` (1处)
- `src/commands/pr/merge.rs` (1处)
- `src/commands/pr/close.rs` (1处)

**重复代码示例：**
```rust
if let Err(e) = Git::stash_pop() {
    log_warning!("Failed to restore stashed changes: {}", e);
    log_warning!("You can manually restore them with: git stash pop");
    // 有时还有额外的日志
} else {
    log_success!("Stashed changes restored");
}
```

### ✅ 重构建议

**直接修改 `stash_pop()` 方法：** `src/lib/git/stash.rs`

直接在 `stash_pop()` 方法中统一处理日志输出，这样所有调用者都不需要重复的日志处理代码。

```rust
impl Git {
    pub fn stash_pop() -> Result<()> {
        let result = cmd("git", &["stash", "pop"]).run();

        match result {
            Ok(_) => {
                log_success!("Stashed changes restored");
                Ok(())
            }
            Err(e) => {
                if Self::has_unmerged()? {
                    // 冲突处理（已有详细日志）
                    // ...
                } else {
                    // 非冲突错误，输出警告并返回错误
                    log_warning!("Failed to restore stashed changes: {}", e);
                    log_warning!("You can manually restore them with: git stash pop");
                    Err(e).context("Failed to pop stash")
                }
            }
        }
    }
}
```

**影响范围：**
- 可以减少约 30-40 行重复代码
- 统一错误处理和日志输出
- 便于后续维护和修改
- 不需要新增方法，直接增强现有方法

---

---

## 3. 分支清理模式

### 🔍 问题描述

`merge.rs` 和 `close.rs` 中的 `cleanup_after_*` 方法几乎完全相同：

**重复位置：**
- `src/commands/pr/merge.rs::cleanup_after_merge()` (约 75 行)
- `src/commands/pr/close.rs::cleanup_after_close()` (约 67 行)

**重复逻辑：**
1. 检查是否在默认分支
2. 更新远程分支信息 (fetch)
3. 检查并 stash 未提交的更改
4. 切换到默认分支
5. 拉取最新代码
6. 删除本地分支
7. 恢复 stash
8. 清理远程分支引用 (prune)

### ✅ 重构建议

**创建共享清理函数：** `src/commands/pr/helpers.rs`

```rust
/// 通用的分支清理逻辑
pub fn cleanup_branch_after_operation(
    current_branch: &str,
    default_branch: &str,
    operation_name: &str, // 用于日志消息
) -> Result<()> {
    // 如果当前分支已经是默认分支，不需要清理
    if current_branch == default_branch {
        log_info!("Already on default branch: {}", default_branch);
        return Ok(());
    }

    log_info!("Switching to default branch: {}", default_branch);

    // 1. 更新远程分支信息
    Git::fetch()?;

    // 2. 检查并 stash 未提交的更改
    let has_stashed = Git::has_commit()?;
    if has_stashed {
        log_info!("Stashing local changes before switching branches...");
        Git::stash_push(Some(&format!("Auto-stash before {} cleanup", operation_name)))?;
    }

    // 3. 切换到默认分支
    Git::checkout_branch(default_branch)
        .with_context(|| format!("Failed to checkout default branch: {}", default_branch))?;

    // 4. 更新本地默认分支
    Git::pull(default_branch)
        .with_context(|| format!("Failed to pull latest changes from {}", default_branch))?;

    // 5. 删除本地分支
    if branch_exists_locally(current_branch)? {
        log_info!("Deleting local branch: {}", current_branch);
        Git::delete(current_branch, false)
            .or_else(|_| {
                log_info!("Branch may not be fully merged, trying force delete...");
                Git::delete(current_branch, true)
            })
            .context("Failed to delete local branch")?;
        log_success!("Local branch deleted: {}", current_branch);
    } else {
        log_info!("Local branch already deleted: {}", current_branch);
    }

    // 6. 恢复 stash
    if has_stashed {
        log_info!("Restoring stashed changes...");
        Git::try_stash_pop_with_logging(); // 使用新的辅助函数
    }

    // 7. 清理远程分支引用
    if let Err(e) = Git::prune_remote() {
        log_info!("Warning: Failed to prune remote references: {}", e);
        log_info!("This is a non-critical cleanup operation. Local cleanup is complete.");
    }

    log_success!(
        "Cleanup completed: switched to {} and deleted local branch {}",
        default_branch,
        current_branch
    );

    Ok(())
}

/// 检查本地分支是否存在
pub fn branch_exists_locally(branch_name: &str) -> Result<bool> {
    let (exists_local, _) =
        Git::is_branch_exists(branch_name).context("Failed to check if branch exists")?;
    Ok(exists_local)
}
```

**影响范围：**
- 可以减少约 100+ 行重复代码
- 统一清理逻辑
- 便于修复和优化

---

## 4. 用户确认模式

### 🔍 问题描述

多个地方使用类似的 `Confirm::new()` 模式，但可以进一步简化。

**重复位置：**
- `src/commands/uninstall.rs` (2处)
- `src/commands/pr/integrate.rs` (1处)
- `src/commands/pr/create.rs` (3处)
- `src/commands/setup.rs` (2处)
- `src/lib/log/clean.rs` (1处)

**重复代码示例：**
```rust
let should_continue = Confirm::new()
    .with_prompt("Continue anyway?")
    .default(false)
    .interact()
    .context("Failed to get user confirmation")?;
```

### ✅ 重构建议

**创建辅助宏或函数：** `src/lib/utils/confirm.rs` (新文件)

```rust
use dialoguer::Confirm;
use anyhow::{Context, Result};

/// 确认操作，失败时返回错误
pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    Confirm::new()
        .with_prompt(prompt)
        .default(default)
        .interact()
        .context("Failed to get user confirmation")
}

/// 确认操作，取消时返回错误
pub fn confirm_or_cancel(prompt: &str, default: bool) -> Result<()> {
    if confirm(prompt, default)? {
        Ok(())
    } else {
        anyhow::bail!("Operation cancelled by user");
    }
}
```

**影响范围：**
- 可以减少约 20-30 行重复代码
- 统一确认逻辑
- 便于修改默认行为

---

## 📊 总结

### 预计减少的代码行数

| 类别 | 预计减少行数 | 优先级 |
|------|------------|--------|
| Stash 操作模式 | 30-40 行 | 高 |
| ~~平台提供者调用模式~~ | ~~100+ 行~~ | ~~已完成~~ |
| 分支清理模式 | 100+ 行 | 高 |
| 用户确认模式 | 20-30 行 | 中 |
| ~~日志文件检查模式~~ | ~~10-15 行~~ | ~~已完成~~ |
| ~~分支存在检查模式~~ | ~~15-20 行~~ | ~~已完成~~ |
| ~~错误处理模式~~ | ~~40-50 行~~ | ~~已完成~~ |
| **总计** | **150-290 行** | - |

### 重构优先级建议

1. **高优先级**（立即重构）：
   - 分支清理模式（代码重复最多）
   - Stash 操作模式（简单且影响大）

2. **中优先级**（近期重构）：
   - 用户确认模式

3. **低优先级**（可选）：
   - 无

### 重构注意事项

1. **保持向后兼容**：确保重构后的 API 与现有代码兼容
2. **测试覆盖**：重构后需要运行完整的测试套件
3. **渐进式重构**：建议一次重构一个模式，避免大规模改动
4. **文档更新**：重构后更新相关文档

---

## 🚀 实施建议

### 第一步：创建辅助函数模块

1. 扩展 `src/lib/git/stash.rs` - 修改 stash_pop 方法（已完成）
2. 扩展 `src/lib/pr/provider.rs` - 添加平台提供者调度函数（已完成）
3. 扩展 `src/commands/pr/helpers.rs` - 添加清理辅助函数
4. 创建 `src/lib/utils/confirm.rs` - 添加确认辅助函数

### 第二步：逐步重构

1. 先重构 stash 操作（最简单）
2. 然后重构平台提供者调用（已完成）
3. 最后重构分支清理（影响最大）

### 第三步：测试和验证

1. 运行所有测试
2. 手动测试关键功能
3. 检查代码覆盖率

---

## 📝 相关文件

- `src/lib/git/stash.rs` - Stash 操作
- `src/lib/git/branch.rs` - 分支操作
- `src/commands/pr/helpers.rs` - PR 辅助函数
- `src/lib/pr/provider.rs` - 平台提供者 trait 和调度函数
- `src/lib/log/mod.rs` - 日志相关函数

