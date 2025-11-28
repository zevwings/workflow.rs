# PR Port / Migrate / Transplant 命名对比分析

## 📋 问题

应该使用 `pr port`、`pr migrate` 还是 `pr transplant`？

---

## 🔍 语义分析

### `port` 的含义

**在软件开发中**：
- **移植**：将代码从一个平台/环境移植到另一个平台/环境
- **保持逻辑不变**：移植过程中，代码逻辑保持不变，只是适配新环境
- **常见用法**：
  - "port code to another platform"（将代码移植到另一个平台）
  - "port changes to another branch"（将更改移植到另一个分支）

**特点**：
- ✅ 强调代码的"移植"，保持代码逻辑不变
- ✅ 在 Git 工作流中，cherry-pick 就是"移植"提交
- ✅ 语义准确：将提交从一个分支移植到另一个分支
- ✅ 简洁（4 个字母）

### `migrate` 的含义

**在软件开发中**：
- **迁移**：从一个系统/环境迁移到另一个系统/环境
- **可能涉及变化**：迁移过程中可能需要进行适配、转换
- **常见用法**：
  - "migrate database"（迁移数据库）
  - "migrate to new framework"（迁移到新框架）
  - "migrate codebase"（迁移代码库）

**特点**：
- ⚠️ 通常暗示更彻底的变化或系统级别的迁移
- ⚠️ 可能暗示需要适配、转换，而不仅仅是复制
- ⚠️ 在 Git 工作流中，`migrate` 不如 `port` 常见
- ⚠️ 较长（7 个字母）

### `transplant` 的含义

**在软件开发中**：
- **移植**：将代码从一个地方移植到另一个地方
- **强调动作**：更强调"移植"这个动作本身
- **常见用法**：
  - "transplant code"（移植代码）
  - "transplant changes"（移植更改）
  - 在 Git 中较少使用，但在某些工具中有使用

**特点**：
- ✅ 语义准确：准确描述"移植"操作
- ✅ 强调动作：更强调"移植"这个动作
- ⚠️ 在 Git 工作流中不如 `port` 常见
- ⚠️ 较长（10 个字母）
- ⚠️ 可能让人联想到医学上的"器官移植"

---

## 📊 详细对比

| 维度 | `port` | `migrate` | `transplant` |
|------|--------|-----------|--------------|
| **语义准确性** | ⭐⭐⭐⭐⭐ 准确（移植代码） | ⭐⭐⭐ 可能暗示更彻底的变化 | ⭐⭐⭐⭐⭐ 准确（移植代码） |
| **Git 工作流认知** | ⭐⭐⭐⭐ 在 Git 中常见 | ⭐⭐ 较少用于 Git 操作 | ⭐⭐ 较少用于 Git 操作 |
| **简洁性** | ⭐⭐⭐⭐⭐ 4 个字母 | ⭐⭐⭐⭐ 7 个字母 | ⭐⭐⭐ 10 个字母 |
| **业界使用** | ⭐⭐⭐⭐ 常见（backport/forwardport） | ⭐⭐ 较少用于分支操作 | ⭐⭐ 较少用于分支操作 |
| **用户理解度** | ⭐⭐⭐⭐ 容易理解 | ⭐⭐⭐ 可能需要解释 | ⭐⭐⭐ 可能需要解释 |
| **命令风格一致性** | ⭐⭐⭐⭐⭐ 符合现有风格 | ⭐⭐⭐⭐ 符合现有风格 | ⭐⭐⭐ 较长，不太符合简洁风格 |
| **输入便利性** | ⭐⭐⭐⭐⭐ 短，易输入 | ⭐⭐⭐⭐ 中等 | ⭐⭐⭐ 较长，输入不便 |

---

## 💡 使用场景对比

### 使用 `port` 的场景

```bash
# 将 develop 的修改移植到 master
workflow pr port develop master

# 语义：将代码从 develop 分支移植到 master 分支
# 理解：cherry-pick 提交，保持代码逻辑不变
# 输入：简短，4 个字母
```

### 使用 `migrate` 的场景

```bash
# 将 develop 的修改迁移到 master
workflow pr migrate develop master

# 语义：将代码从 develop 分支迁移到 master 分支
# 理解：可能暗示需要适配或转换（但实际上只是 cherry-pick）
# 输入：中等，7 个字母
```

### 使用 `transplant` 的场景

```bash
# 将 develop 的修改移植到 master
workflow pr transplant develop master

# 语义：将代码从 develop 分支移植到 master 分支
# 理解：准确描述"移植"操作
# 输入：较长，10 个字母
```

---

## 🎯 业界实践

### Git 工具中的命名

1. **backport/forwardport**：
   - 使用 `port` 后缀，表示"移植"
   - 例如：`git-backport`、GitHub Actions `backport`

2. **Cherry-pick 相关工具**：
   - 通常使用 `port`、`backport`、`forwardport`
   - 很少使用 `migrate` 或 `transplant`

3. **数据库/系统迁移工具**：
   - 通常使用 `migrate`（如 `rails migrate`、`django migrate`）
   - 用于系统级别的迁移

4. **代码移植工具**：
   - 某些工具使用 `transplant`（如 Mercurial 的 `transplant` 扩展）
   - 但在 Git 生态系统中较少见

### 结论

- **Git 分支操作**：更常用 `port`、`backport`、`forwardport`
- **系统迁移**：更常用 `migrate`
- **代码移植**：`transplant` 在某些工具中使用，但在 Git 中不常见

---

## ✅ 推荐建议

### 推荐方案 1：保持 `port`（最推荐 ⭐⭐⭐⭐⭐）

**理由**：

1. **语义准确**：
   - `port` 准确描述了 cherry-pick 操作的本质（移植代码）
   - 与 `backport`/`forwardport` 等标准术语一致

2. **业界标准**：
   - Git 工作流中，`backport`/`forwardport` 是标准术语
   - `port` 与这些术语一致

3. **用户理解**：
   - `port` 在 Git 上下文中更容易理解
   - 用户可能已经熟悉 `backport` 概念

4. **简洁性**：
   - `port` 最短（4 个字母），输入最方便
   - 符合现有命令的简洁风格（create, merge, sync, rebase）

5. **一致性**：
   - 与现有命令风格一致
   - 都是简洁的动词

### 推荐方案 2：使用 `transplant`（备选 ⭐⭐⭐）

**理由**：

1. **语义准确**：
   - `transplant` 准确描述了"移植"操作
   - 语义上可能比 `port` 更直观（强调"移植"动作）

2. **专业术语**：
   - 在某些版本控制工具中使用（如 Mercurial）
   - 在代码移植场景中语义清晰

**缺点**：

- ❌ **名称较长**（10 个字母），输入不便
- ❌ 在 Git 工作流中不如 `port` 常见
- ❌ 可能让人联想到医学上的"器官移植"
- ❌ 不符合现有命令的简洁风格

### 推荐方案 3：使用 `migrate`（不推荐 ⭐⭐）

**理由**：
- ✅ 在某些框架中常见（Rails、Django）

**缺点**：
- ❌ 语义可能不够准确（暗示需要适配，但实际只是复制）
- ❌ 在 Git 工作流中不如 `port` 常见
- ❌ 名称较长（7 个字母）
- ❌ 可能与系统迁移工具混淆

---

## 📊 综合评分

| 命名 | 语义准确性 | Git 认知度 | 简洁性 | 输入便利性 | 业界标准 | 综合评分 |
|------|-----------|-----------|--------|-----------|---------|---------|
| `port` | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | **⭐⭐⭐⭐⭐** |
| `transplant` | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| `migrate` | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ |

---

## 🎯 最终建议

### 强烈推荐保持 `pr port`

**原因**：

1. **最佳平衡**：
   - 语义准确 ✅
   - 简洁易用 ✅
   - 业界标准 ✅
   - 用户友好 ✅

2. **符合现有风格**：
   - 与现有命令（create, merge, sync, rebase）风格一致
   - 都是简洁的动词

3. **输入便利**：
   - 最短的命令名，输入最方便
   - 用户使用频率高，简洁性很重要

### 如果确实想使用 `transplant`

**可以考虑**：

1. **添加别名**：同时支持 `port` 和 `transplant`
   ```bash
   workflow pr port develop master        # 主命令（推荐）
   workflow pr transplant develop master # 别名（可选）
   ```

2. **权衡考虑**：
   - `transplant` 语义准确，但名称较长
   - 如果用户更偏好语义清晰度，可以考虑
   - 但需要权衡输入便利性

### 不推荐使用 `migrate`

**原因**：
- 语义不够准确（暗示需要适配）
- 在 Git 工作流中不常见
- 可能与系统迁移工具混淆

---

## 📝 实际使用对比

### 日常使用频率

假设用户每天使用 10 次：

```bash
# 使用 port（推荐）
workflow pr port develop master
# 输入：4 个字母，快速

# 使用 transplant
workflow pr transplant develop master
# 输入：10 个字母，较慢

# 使用 migrate
workflow pr migrate develop master
# 输入：7 个字母，中等
```

**结论**：`port` 在频繁使用时优势明显。

---

## 🔗 相关参考

- [Git Backport 工具](https://github.com/zeebe-io/backport)
- [GitHub Backport Action](https://github.com/zeebe-io/backport-action)
- [Git Cherry-pick 文档](https://git-scm.com/docs/git-cherry-pick)
- [Mercurial Transplant Extension](https://www.mercurial-scm.org/wiki/TransplantExtension)

---

## 📋 总结

| 选项 | 推荐度 | 主要优势 | 主要劣势 |
|------|--------|---------|---------|
| `port` | ⭐⭐⭐⭐⭐ | 简洁、标准、易用 | 可能需要解释含义 |
| `transplant` | ⭐⭐⭐ | 语义准确、专业 | 较长、不常见 |
| `migrate` | ⭐⭐ | 在某些框架中常见 | 语义不准确、不常见 |

**最终建议**：保持 `pr port`，这是最佳选择。

