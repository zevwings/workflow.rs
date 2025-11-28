# PR Port 和 Rebase 命令索引

## 📋 文档概述

本文档是 PR Port 和 Rebase 两个命令的索引文档。每个命令的详细设计文档已拆分为独立文档。

**注意**：`pr sync` 命令的详细设计已整合到 [PR 命令模块架构文档](../architecture/commands/PR_COMMAND_ARCHITECTURE.md#2-同步分支命令-syncrs)。

**文档创建时间**: 2024-12
**最后更新**: 2024-12

---

## 📚 命令文档

### 1. `pr port` - 跨分支移植代码

**文档**: [PR_PORT_COMMAND.md](./PR_PORT_COMMAND.md)

**功能**: 从源分支 cherry-pick 提交到目标分支并创建新 PR

**快速参考**:
```bash
workflow pr port <FROM_BRANCH> <TO_BRANCH> [OPTIONS]
```

**典型用法**:
```bash
# 将 develop 的修改应用到 master
workflow pr port develop master
```

---

### 2. `pr rebase` - 修正基础分支

**文档**: [PR_REBASE_COMMAND.md](./PR_REBASE_COMMAND.md)

**功能**: 将当前分支 rebase 到目标分支并更新 PR 的 base 分支

**快速参考**:
```bash
workflow pr rebase <TARGET_BRANCH> [OPTIONS]
```

**典型用法**:
```bash
# 修正错误的基础分支（代码在 master 上，但应该在 develop 基础上）
git checkout master
workflow pr rebase develop
```

---

### 3. `pr sync` - 同步分支更新

**文档**: [PR 命令模块架构文档](../architecture/commands/PR_COMMAND_ARCHITECTURE.md#2-同步分支命令-syncrs)

**功能**: 将源分支的更改同步到当前分支，支持 merge、rebase 或 squash 三种方式

**快速参考**:
```bash
workflow pr sync <SOURCE_BRANCH> [OPTIONS]
```

**典型用法**:
```bash
# 同步 master 到当前分支
workflow pr sync master

# 使用 rebase 方式
workflow pr sync master --rebase
```

---

## 🔄 命令对比

| 特性 | `pr port` | `pr rebase` | `pr sync` |
|------|-----------|-------------|-----------|
| **操作对象** | 从分支 A 到分支 B | 当前分支到目标分支 | 从分支 A 到当前分支 |
| **操作方式** | Cherry-pick | Rebase | Merge、Rebase 或 Squash |
| **创建新分支** | ✅ 是 | ❌ 否 | ❌ 否 |
| **创建新 PR** | ✅ 是（可选） | ❌ 否 | ❌ 否 |
| **更新 PR base** | ❌ 否 | ✅ 是 | ❌ 否 |
| **使用场景** | 跨分支移植代码 | 修正当前分支的基础分支 | 同步基础分支更新 |
| **典型用法** | `pr port develop master` | `pr rebase develop` | `pr sync master` |

---

## 📖 选择建议

### 使用 `pr port` 当：
- 需要跨分支移植代码
- 需要创建新的 PR
- 需要 cherry-pick 特定提交
- 源分支和目标分支不同
- 不想修改现有分支

### 使用 `pr rebase` 当：
- 需要修正当前分支的基础分支
- 需要更新现有 PR 的 base 分支
- 当前分支有 PR，但基础分支错误
- 需要重写提交历史（保持线性）

### 使用 `pr sync` 当：
- 需要同步基础分支到当前分支
- 不需要创建新 PR
- 不需要更新 PR 的 base 分支
- 需要 merge、rebase 或 squash 整个分支
- 在当前分支上操作

---

## 🔗 相关文档

- [功能扩展文档](./FEATURE_EXTENSIONS.md) - 其他 PR 相关功能扩展建议

---

## 📝 文档维护

- 每个命令的详细设计请参考对应的独立文档
- 本文档仅作为索引和快速参考
- 如有更新，请同步更新对应的命令文档
