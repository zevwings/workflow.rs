# Push Spinner 输出问题分析与方案

## 问题描述

1. **原始需求**：所有 `git push` 操作都应该显示 loading spinner
2. **实际问题**：`git push` 的输出（如 `remote: Resolving deltas...`）与 spinner 动画混合，导致显示混乱
3. **错误修改**：完全移除了 spinner，改用 `log_info!`，导致回退了功能

## 问题根源

1. **Spinner 实现**：使用 `indicatif::ProgressBar`，通过 ANSI 转义序列更新同一行
2. **Git 输出**：`git push` 通过 `duct::cmd` 执行，直接输出到 stdout/stderr
3. **冲突**：当 git 输出内容时，会干扰 spinner 的 ANSI 转义序列，导致显示混乱

## 解决方案分析

### 方案1：使用 indicatif 的 suspend/resume（推荐）

**原理**：
- `ProgressBar` 有 `suspend()` 和 `resume()` 方法
- 在执行有输出的操作前暂停 spinner
- 操作完成后恢复或完成 spinner

**优点**：
- 保持 spinner 功能
- 输出清晰，不混合
- 使用 indicatif 原生功能

**缺点**：
- 需要修改 Spinner 实现

**实现**：
```rust
// 在 Spinner 中添加新方法
pub fn with_output<F, T, E>(message: impl AsRef<str>, operation: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
{
    let spinner = Self::new(message);
    spinner.inner.suspend(|| {
        operation()
    })?;
    spinner.finish();
    result
}
```

### 方案2：手动管理 spinner（临时方案）

**原理**：
- 创建 spinner
- 暂停 spinner
- 执行 git push
- 完成 spinner

**优点**：
- 简单，不需要修改 Spinner
- 可以立即实现

**缺点**：
- 代码重复
- 不够优雅

**实现**：
```rust
let spinner = Spinner::new("Pushing to remote...");
spinner.inner.suspend(|| {
    GitBranch::push(&current_branch, false)?;
    Ok(())
})?;
spinner.finish();
```

### 方案3：捕获 git 输出后显示

**原理**：
- 捕获 git push 的输出
- 在 spinner 完成后显示

**优点**：
- 保持 spinner
- 输出清晰

**缺点**：
- 用户看不到实时进度
- 需要修改 `cmd_run` 实现

## 推荐方案

**推荐使用方案1**：在 `Spinner` 中添加 `with_output()` 方法，使用 indicatif 的 `suspend()` 功能。

### 实现步骤

1. 修改 `Spinner` 实现，添加 `with_output()` 方法
2. 在所有 git push 的地方使用 `Spinner::with_output()` 替代 `Spinner::with()`
3. 保持其他操作的 `Spinner::with()` 不变

### 代码示例

```rust
// src/lib/base/indicator/spinner.rs
impl Spinner {
    /// 使用 spinner 执行一个会产生输出的操作
    ///
    /// 在执行操作前会暂停 spinner，让子进程的输出正常显示，
    /// 操作完成后恢复并完成 spinner。
    ///
    /// # 参数
    ///
    /// * `message` - 要显示的消息文本
    /// * `operation` - 要执行的操作（闭包）
    ///
    /// # 返回
    ///
    /// 返回操作的结果
    ///
    /// # 示例
    ///
    /// ```rust
    /// Spinner::with_output("Pushing to remote...", || {
    ///     GitBranch::push(&current_branch, false)
    /// })?;
    /// ```
    pub fn with_output<F, T, E>(message: impl AsRef<str>, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let spinner = Self::new(message);
        let result = spinner.inner.suspend(|| {
            operation()
        });
        spinner.finish();
        result
    }
}
```

**注意**：`suspend()` 方法会临时隐藏进度条，执行函数，然后重新绘制。这正好适合我们的需求。

## 需要修改的文件

1. `src/lib/base/indicator/spinner.rs` - 添加 `with_output()` 方法
2. 所有使用 `git push` 的地方（约11处）：
   - `src/commands/pr/update.rs`
   - `src/commands/pr/sync.rs` (2处)
   - `src/commands/pr/create.rs` (5处)
   - `src/commands/pr/rebase.rs` (1处)
   - `src/commands/pr/pick.rs` (2处)
