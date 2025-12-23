# 贡献指南

> 欢迎贡献 Workflow CLI！本文档提供了如何开始贡献、提交 PR、以及项目开发规范的快速指南。

---

## 📋 目录

- [快速开始](#-快速开始)
- [如何贡献](#-如何贡献)
- [Git 工作流](#-git-工作流)
- [Git 操作最佳实践](#-git-操作最佳实践)
- [提交规范](#-提交规范)
- [PR 描述模板](#-pr-描述模板)
- [代码审查](#-代码审查)
- [开发规范](#-开发规范)
- [相关文档](#-相关文档)

---

## 🚀 快速开始

### 环境设置

首次开发前，请先安装所需的开发工具：

```bash
make setup
```

这会自动安装：
- `rustfmt` - 代码格式化工具
- `clippy` - 代码检查工具
- `rust-analyzer` - 语言服务器（从源码构建）

### 开发流程概览

1. **Fork 仓库**：Fork 项目到你的 GitHub 账号
2. **克隆仓库**：`git clone https://github.com/your-username/workflow.rs.git`
3. **创建分支**：从 `master` 创建功能分支
4. **开发**：实现功能或修复 bug
5. **测试**：运行测试确保代码正确
6. **提交**：遵循提交规范提交代码
7. **推送**：推送到你的 Fork
8. **创建 PR**：创建 Pull Request 到主仓库

---

## 💡 如何贡献

### 报告问题

如果你发现了 bug 或有功能建议，请通过 GitHub Issues 报告：

1. **检查现有 Issues**：确保问题未被报告
2. **创建新 Issue**：提供清晰的问题描述
3. **包含信息**：
   - 问题描述
   - 复现步骤
   - 预期行为
   - 实际行为
   - 环境信息（操作系统、Rust 版本等）

### 提交 PR

提交 PR 前，请确保：

- [ ] 代码已格式化（`cargo fmt`）
- [ ] 通过 Clippy 检查（`cargo clippy`）
- [ ] 所有测试通过（`cargo test`）
- [ ] 添加了必要的文档注释
- [ ] 遵循了错误处理规范
- [ ] PR 描述完整（见下方 PR 描述模板）

### PR 审查流程

1. **创建 PR**：使用 PR 描述模板填写完整信息
2. **等待审查**：维护者会审查你的代码
3. **响应反馈**：根据审查意见修改代码
4. **合并**：审查通过后，维护者会合并 PR

---

## 🌿 Git 工作流

### 分支策略

- **`feature/*`**：功能分支，从 `master` 创建，用于新功能开发
- **`fix/*`**：修复分支，从 `master` 创建，用于修复 bug
- **`hotfix/*`**：热修复分支，用于紧急修复生产问题

### 分支命名

- 功能分支：`feature/jira-attachments`
- 修复分支：`fix/pr-merge-error`
- 热修复分支：`hotfix/critical-bug`

**注意**：Workflow CLI 支持通过模板系统自定义分支命名格式。详细配置方法请参考 [模板配置指南](./docs/guidelines/template.md#分支命名模板-templatebranch)。

### 工作流程

1. **创建分支**：从 `master` 创建新分支
2. **开发**：在分支上进行开发
3. **提交**：遵循提交规范（见下方）
4. **推送**：推送到远程仓库
5. **创建 PR**：创建 Pull Request 到 `master`
6. **代码审查**：等待代码审查
7. **合并**：审查通过后合并到 `master`

---

## 🔧 Git 操作最佳实践

### Rebase vs Merge

#### 何时使用 Rebase

- **功能分支同步**：在提交 PR 前，将 `master` 的最新更改同步到你的分支
- **保持线性历史**：希望保持 Git 历史记录简洁线性
- **本地分支整理**：整理本地提交历史

**示例**：

```bash
# 同步 master 到当前分支
git checkout feature/my-feature
git rebase master

# 如果有冲突，解决后继续
git rebase --continue
```

#### 何时使用 Merge

- **合并 PR**：合并 Pull Request 到主分支（通常由维护者执行）
- **保留分支历史**：希望保留完整的分支历史记录
- **协作分支**：多人协作的分支，避免 rebase 冲突

**示例**：

```bash
# 合并功能分支到 master
git checkout master
git merge feature/my-feature
```

#### 推荐做法

- **提交 PR 前**：使用 `rebase` 同步 `master` 到你的分支
- **合并 PR 时**：使用 `merge` 保留完整历史
- **避免**：不要 rebase 已经推送到远程且其他人可能基于它工作的分支

### 分支清理规则

#### 本地分支清理

定期清理已合并的本地分支：

```bash
# 使用 Workflow CLI 清理已合并的分支
workflow branch clean

# 预览模式（不实际删除）
workflow branch clean --dry-run
```

**保留的分支**：
- `master` / `main`
- `develop`（如果存在）
- 当前分支
- 忽略列表中的分支（通过 `workflow branch ignore add <BRANCH_NAME>` 添加）

#### 远程分支清理

合并 PR 后，通常会自动删除远程分支。如果未自动删除，可以手动删除：

```bash
# 删除远程分支
git push origin --delete feature/my-feature
```

### 冲突解决流程

#### 预防冲突

1. **频繁同步**：定期将 `master` 同步到你的分支
2. **小步提交**：频繁提交小改动，避免大范围冲突
3. **沟通协调**：与团队成员协调，避免同时修改相同文件

#### 解决冲突

**Rebase 冲突**：

```bash
# 开始 rebase
git rebase master

# 如果遇到冲突，Git 会暂停
# 1. 查看冲突文件
git status

# 2. 手动解决冲突（编辑文件，删除冲突标记）
# 冲突标记示例：
# <<<<<<< HEAD
# 你的更改
# =======
# master 的更改
# >>>>>>> master

# 3. 标记为已解决
git add <resolved-file>

# 4. 继续 rebase
git rebase --continue

# 5. 如果所有冲突都解决了，rebase 完成
# 6. 强制推送（因为历史已重写）
git push --force-with-lease origin feature/my-feature
```

**Merge 冲突**：

```bash
# 开始 merge
git merge master

# 如果遇到冲突，解决步骤同上
# 1. 解决冲突
# 2. 标记为已解决
git add <resolved-file>

# 3. 完成 merge
git commit

# 4. 推送
git push origin feature/my-feature
```

#### 冲突解决最佳实践

- **理解冲突**：仔细阅读冲突内容，理解双方更改的意图
- **保留必要更改**：确保不丢失重要的代码或功能
- **测试验证**：解决冲突后，运行测试确保功能正常
- **寻求帮助**：如果不确定如何解决，在 PR 中询问维护者

---

## 📋 提交规范

### Conventional Commits

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<type>(<scope>): <subject>

<body>

<footer>
```

### 提交类型

- **`feat`**：新功能
- **`fix`**：修复 bug
- **`docs`**：文档更新
- **`style`**：代码格式调整（不影响功能）
- **`refactor`**：代码重构
- **`test`**：测试相关
- **`chore`**：构建过程或辅助工具的变动
- **`perf`**：性能优化
- **`ci`**：CI/CD 配置变更

### 提交示例

```bash
# 功能提交
feat(jira): add attachments download command

Add new command to download all attachments from a JIRA ticket.
The command supports filtering by file type and size.

Closes #123

# 修复提交
fix(pr): handle merge conflict error

Fix the issue where PR merge fails silently when there's a merge conflict.
Now the command will display a clear error message.

Fixes #456

# 文档提交
docs: update development guidelines

Add error handling best practices section.

# 重构提交
refactor(http): simplify retry logic

Extract retry logic into a separate module for better maintainability.
```

### 提交信息要求

- **主题行**：不超过 50 个字符，使用祈使语气
- **正文**：详细说明变更原因和方式，每行不超过 72 个字符
- **页脚**：引用相关 issue（如 `Closes #123`）

**注意**：Workflow CLI 支持通过模板系统自定义提交消息格式，包括是否使用 Conventional Commits 格式。详细配置方法请参考 [模板配置指南](./docs/guidelines/template.md#提交消息模板-templatecommit)。

---

## 📝 PR 描述模板

创建 Pull Request 时，请使用以下模板填写 PR 描述。这有助于审查者快速理解你的更改。

### PR 描述必须包含的内容

#### 1. 变更说明

**必须包含**：清晰描述本次 PR 的变更内容

```markdown
## 变更说明

本次 PR 实现了 [功能描述] / 修复了 [问题描述]。

主要变更：
- [变更点 1]
- [变更点 2]
- [变更点 3]
```

#### 2. 测试说明

**必须包含**：说明如何测试本次变更

```markdown
## 测试说明

### 测试步骤
1. [步骤 1]
2. [步骤 2]
3. [步骤 3]

### 测试结果
- [测试结果 1]
- [测试结果 2]

### 测试覆盖
- [ ] 单元测试已添加/更新
- [ ] 集成测试已添加/更新
- [ ] 手动测试已完成
```

#### 3. 相关文档更新说明

**必须包含**：说明是否更新了相关文档

```markdown
## 文档更新

- [ ] README.md 已更新（如适用）
- [ ] 架构文档已更新（如适用）
- [ ] 开发规范文档已更新（如适用）
- [ ] CHANGELOG.md 已更新（如适用）

### 文档变更说明
[说明具体更新了哪些文档和内容]
```

#### 4. 破坏性变更说明（如有）

**如有破坏性变更，必须包含**：说明破坏性变更和迁移路径

```markdown
## 破坏性变更

### 变更内容
[描述破坏性变更的具体内容]

### 影响范围
[说明哪些功能或 API 受到影响]

### 迁移指南
[提供迁移步骤和示例代码]
```

### PR 描述模板（完整版）

```markdown
## 变更说明

[清晰描述本次 PR 的变更内容]

### 主要变更
- [变更点 1]
- [变更点 2]
- [变更点 3]

## 测试说明

### 测试步骤
1. [步骤 1]
2. [步骤 2]
3. [步骤 3]

### 测试结果
- [测试结果 1]
- [测试结果 2]

### 测试覆盖
- [ ] 单元测试已添加/更新
- [ ] 集成测试已添加/更新
- [ ] 手动测试已完成

## 文档更新

- [ ] README.md 已更新（如适用）
- [ ] 架构文档已更新（如适用）
- [ ] 开发规范文档已更新（如适用）
- [ ] CHANGELOG.md 已更新（如适用）

### 文档变更说明
[说明具体更新了哪些文档和内容]

## 破坏性变更（如有）

### 变更内容
[描述破坏性变更的具体内容]

### 影响范围
[说明哪些功能或 API 受到影响]

### 迁移指南
[提供迁移步骤和示例代码]

## 相关 Issue

Closes #123
Related to #456
```

### PR 描述模板（简化版）

对于小型修复或文档更新，可以使用简化版：

```markdown
## 变更说明

[简要描述变更内容]

## 测试说明

[简要说明测试方法或测试结果]

## 文档更新

- [ ] 相关文档已更新（如适用）
```

---

## 👀 代码审查

### 审查清单

提交 PR 前，确保：

- [ ] 代码已格式化（`cargo fmt`）
- [ ] 通过 Clippy 检查（`cargo clippy`）
- [ ] 所有测试通过（`cargo test`）
- [ ] 添加了必要的文档注释
- [ ] 遵循了错误处理规范
- [ ] 提交信息符合规范
- [ ] PR 描述完整（包含变更说明、测试说明、文档更新说明）
- [ ] 没有引入新的警告

### 审查重点

- **功能正确性**：代码是否实现了预期功能
- **代码质量**：是否遵循了代码风格和最佳实践
- **错误处理**：是否正确处理了错误情况
- **性能**：是否有性能问题
- **安全性**：是否有安全漏洞
- **可维护性**：代码是否易于理解和维护
- **文档完整性**：相关文档是否已同步更新

### 审查流程

1. **自动检查**：CI/CD 会自动运行格式化和测试检查
2. **代码审查**：维护者会审查代码质量和功能正确性
3. **反馈处理**：根据审查意见修改代码
4. **批准合并**：审查通过后，维护者会批准并合并 PR

---

## 📚 开发规范

### 代码风格

- **格式化**：使用 `cargo fmt` 格式化代码
- **Lint 检查**：使用 `cargo clippy` 检查代码质量
- **命名规范**：
  - 模块/函数/变量：`snake_case`
  - 类型/Trait：`PascalCase`
  - 常量：`SCREAMING_SNAKE_CASE`

### 错误处理

- **错误类型**：使用 `color_eyre::Result<T>`
- **错误信息**：提供清晰的错误信息和上下文
- **错误处理模式**：考虑所有错误情况，避免使用 `unwrap()`

### 文档注释

- **公共 API**：所有公共 API 必须包含 `///` 文档注释
- **注释内容**：包括参数说明、返回值说明、错误说明、使用示例

### 测试规范

- **单元测试**：放在对应模块的 `#[cfg(test)]` 模块中
- **集成测试**：放在 `tests/` 目录
- **测试覆盖率**：目标覆盖率 > 80%，关键业务逻辑 > 90%

### 模块组织

遵循三层架构：
- **CLI 入口层** (`bin/`, `main.rs`)：命令行参数解析和命令分发
- **命令封装层** (`commands/`)：CLI 命令封装，处理用户交互
- **核心业务逻辑层** (`lib/`)：所有业务逻辑实现

---

## 🔗 相关文档

### 开发规范

- [开发规范文档](./docs/guidelines/development.md) - 详细的开发规范和最佳实践
- [测试规范文档](./docs/guidelines/testing.md) - 测试规范和测试方法
- [文档编写指南](./docs/guidelines/document.md) - 文档编写规范和模板

### 工作流指南

- [提交前检查指南](./docs/guidelines/development/workflows/pre-commit.md) - 提交前快速检查（5-15分钟）
- [综合深入检查指南](./docs/guidelines/development/workflows/review.md) - 综合深入检查（2-4小时）

### 架构文档

- [总体架构文档](./docs/architecture/architecture.md) - 项目整体架构设计
- [CLI 架构文档](./docs/architecture/cli.md) - CLI 层架构设计

### 模板配置

- [模板配置指南](./docs/guidelines/template.md) - 分支命名、提交消息、PR 模板配置

---

## ✅ 检查清单

提交 PR 前，请使用以下清单检查：

### 代码质量

- [ ] 代码已格式化（`cargo fmt`）
- [ ] 通过 Clippy 检查（`cargo clippy`）
- [ ] 所有测试通过（`cargo test`）
- [ ] 没有引入新的警告

### 代码规范

- [ ] 添加了必要的文档注释
- [ ] 遵循了错误处理规范
- [ ] 遵循了命名规范
- [ ] 遵循了模块组织规范

### Git 规范

- [ ] 提交信息符合 Conventional Commits 格式
- [ ] 分支命名符合规范
- [ ] 已同步 `master` 分支（使用 rebase）

### PR 规范

- [ ] PR 描述包含变更说明
- [ ] PR 描述包含测试说明
- [ ] PR 描述包含文档更新说明
- [ ] 如有破坏性变更，已说明迁移路径

### 文档更新

- [ ] README.md 已更新（如适用）
- [ ] 架构文档已更新（如适用）
- [ ] 开发规范文档已更新（如适用）
- [ ] CHANGELOG.md 已更新（如适用）

---

**最后更新**: 2025-12-23

