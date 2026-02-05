---
name: code-review
description: 审查当前分支相对于基准分支的 Rust 代码变更，检查代码质量、最佳实践、API 设计和错误处理。触发词：代码审查、review、质量检查。
---

# 代码审查

**检查范围**: 当前分支相对于基准分支的所有变更

## 检查流程

1. 运行 `make lint`（格式化 + Clippy + Check）
2. 按下方清单逐项检查
3. 输出审查报告

> 修复问题可运行 `make fix`

## 检查清单

### 代码精简
- [ ] 只包含核心内容，无冗余代码
- [ ] 无向后兼容代码
- [ ] 无 dead code / unused imports

### 代码组织
- [ ] 导入顺序：标准库 → 第三方 → 项目内部
- [ ] 测试模块导入放在 `mod tests {}` 顶部
- [ ] 最小化可见性：优先 `pub(crate)` / `pub(super)`
- [ ] 内部实现保持私有

### Git 操作规范
- [ ] 禁止使用 `Command::new("git")` 调用 git 命令行
- [ ] 统一使用 `git2` 库完成所有 Git 操作

### 模块导出策略
- [ ] 层层导出需要公开的类型到 crate 根
- [ ] 避免导出仅在 crate 内部使用的子模块
- [ ] crate 根 `lib.rs` 只 re-export 真正需要公开的 API

### 模块导入规则
- [ ] **同一模块内部**：使用 `super::` 引用兄弟模块
  - 如 `hooks/tool_executor.rs` 引用 `hooks/context.rs`
  - 使用 `super::context::HookContext` 而非 `crate::git::services::hooks::context::HookContext`
- [ ] **跨模块（crate 内部）**：使用 `crate::` 全路径
  - 单个：`use crate::rollback::create_backup`
  - 多个：`use crate::rollback::{create_backup, rollback, BackupInfo}`
- [ ] **crate 外部**：从 crate 根导入
  - 单个：`use toolkit::create_backup`
  - 多个：`use toolkit::{create_backup, rollback, BackupInfo}`

### API 设计
- [ ] 超过 3 个参数封装为结构体
- [ ] 布尔参数考虑替换为枚举
- [ ] 使用 `impl Into<T>` / `impl AsRef<T>` 提升灵活性
- [ ] 复杂构造使用 Builder 模式
- [ ] 公开 API 有 `///` 文档注释

### 错误处理
- [ ] 使用 `thiserror` 或自定义错误类型
- [ ] 使用 `anyhow::Context` 添加上下文
- [ ] 避免 `unwrap()` / `expect()` 滥用
- [ ] 使用 `?` 操作符传播错误

### 所有权与生命周期
- [ ] 避免不必要的 `clone()`
- [ ] 参数优先使用 `&str` 而非 `String`
- [ ] 考虑使用 `Cow<str>` 减少分配

### 性能优化
- [ ] 集合预分配：`Vec::with_capacity()`
- [ ] 使用 Iterator 链式操作
- [ ] 避免循环中重复分配

### 测试规范
- [ ] 测试命名：`test_<行为>_<条件>_<预期>`
- [ ] 使用 `#[cfg(test)]` 限定测试代码
- [ ] 测试中使用 `?` 或 assert，避免 `unwrap()`

## 报告格式

```markdown
## 审查报告

### 🔴 Critical
- [文件:行号] 问题 | 建议

### 🟡 Warning
- [文件:行号] 问题 | 建议

### 🟢 Info
- [文件:行号] 问题 | 建议

### ✅ 通过项
- [x] 检查项列表
```
