# Backport vs Forwardport 选择指南

## 📋 概述

本文档详细说明 `backport` 和 `forwardport` 的区别、使用场景和选择依据。

---

## 🔍 基本定义

### Backport（向后移植）

**定义**：将**新版本**的修改应用到**旧版本**。

**方向**：从新 → 旧

**典型场景**：
- 将 `master`/`main` 的修复应用到 `release/v1.0`
- 将 `develop` 的修复应用到 `release/v1.0`
- 将最新版本的 bug 修复应用到旧版本分支

**示例**：
```bash
# 将 master 的修复应用到 release 分支
workflow pr backport master release/v1.0
```

### Forwardport（向前移植）

**定义**：将**旧版本**的修改应用到**新版本**。

**方向**：从旧 → 新

**典型场景**：
- 将 `release/v1.0` 的修复应用到 `master`/`main`
- 将 `release/v1.0` 的修复应用到 `develop`
- 确保新版本包含旧版本的所有修复

**示例**：
```bash
# 将 release 的修复应用到 master
workflow pr forwardport release/v1.0 master
```

---

## 🎯 选择依据

### 判断标准

选择 `backport` 还是 `forwardport` 主要看**版本的新旧关系**，而不是分支名称。

#### 关键问题：哪个版本更新？

- **如果源分支版本更新** → 使用 `backport`
- **如果目标分支版本更新** → 使用 `forwardport`

### 版本关系判断

#### 方法 1：基于分支命名约定

**常见分支命名**：
- `master`/`main` - 最新版本
- `develop` - 开发版本（通常比 release 新）
- `release/v1.0` - 发布版本（旧版本）
- `release/v2.0` - 发布版本（较新版本）

**判断规则**：
```
master > develop > release/v2.0 > release/v1.0
```

#### 方法 2：基于提交时间

- **较新的提交** → 新版本
- **较旧的提交** → 旧版本

#### 方法 3：基于语义版本

- **版本号更大** → 新版本
- **版本号更小** → 旧版本

---

## 📊 常见场景分析

### 场景 1：将 master 的修复应用到 release

**情况**：
- 源分支：`master`（最新版本）
- 目标分支：`release/v1.0`（旧版本）

**选择**：`backport` ✅

**原因**：从新版本到旧版本

```bash
workflow pr backport master release/v1.0
```

**典型场景**：
- 在 master 上修复了一个 bug
- 需要将这个修复也应用到已发布的 v1.0 版本
- 用户可能仍在使用 v1.0，需要修复

---

### 场景 2：将 release 的修复应用到 master

**情况**：
- 源分支：`release/v1.0`（旧版本）
- 目标分支：`master`（最新版本）

**选择**：`forwardport` ✅

**原因**：从旧版本到新版本

```bash
workflow pr forwardport release/v1.0 master
```

**典型场景**：
- 在 release/v1.0 上修复了一个紧急 bug
- 需要确保 master 也包含这个修复
- 避免 master 出现同样的 bug

---

### 场景 3：将 develop 的修改应用到 master

**情况**：
- 源分支：`develop`（开发版本）
- 目标分支：`master`（生产版本）

**判断**：取决于版本关系

**情况 A**：如果 `develop` 是开发分支，`master` 是稳定版本
- `develop` 通常包含更新的代码
- **选择**：`backport` ✅（从新到旧）

```bash
workflow pr backport develop master
```

**情况 B**：如果 `develop` 是旧版本，`master` 是新版本
- **选择**：`forwardport` ✅（从旧到新）

```bash
workflow pr forwardport develop master
```

**注意**：这取决于项目的分支策略，需要根据实际情况判断。

---

### 场景 4：跨 release 版本

**情况**：
- 源分支：`release/v1.0`
- 目标分支：`release/v2.0`

**判断**：
- `release/v2.0` 是较新版本
- **选择**：`forwardport` ✅（从旧到新）

```bash
workflow pr forwardport release/v1.0 release/v2.0
```

**典型场景**：
- v1.0 有一个重要的修复
- 需要确保 v2.0 也包含这个修复

---

## 🔄 实际工作流示例

### 典型 Git Flow 工作流

```
master (最新)
  ↑
  | forwardport
  |
develop (开发)
  ↑
  | backport
  |
release/v1.0 (旧版本)
```

**常见操作**：

1. **在 master 修复 bug** → `backport` 到 `release/v1.0`
   ```bash
   workflow pr backport master release/v1.0
   ```

2. **在 release/v1.0 修复 bug** → `forwardport` 到 `master`
   ```bash
   workflow pr forwardport release/v1.0 master
   ```

3. **在 develop 开发新功能** → `backport` 到 `master`（如果 develop 更新）
   ```bash
   workflow pr backport develop master
   ```

---

## 📋 决策流程图

```
开始
  ↓
确定源分支和目标分支
  ↓
判断哪个版本更新？
  ↓
┌─────────────────┬─────────────────┐
│  源分支更新      │  目标分支更新    │
│  (新 → 旧)      │  (旧 → 新)      │
└─────────────────┴─────────────────┘
        ↓                    ↓
    BACKPORT            FORWARDPORT
        ↓                    ↓
    结束 ←─────────────────┘
```

---

## 🎯 快速判断表

| 源分支 | 目标分支 | 版本关系 | 选择 | 命令示例 |
|--------|---------|---------|------|---------|
| `master` | `release/v1.0` | 新 → 旧 | `backport` | `pr backport master release/v1.0` |
| `release/v1.0` | `master` | 旧 → 新 | `forwardport` | `pr forwardport release/v1.0 master` |
| `develop` | `master` | 新 → 旧* | `backport`* | `pr backport develop master` |
| `release/v1.0` | `release/v2.0` | 旧 → 新 | `forwardport` | `pr forwardport release/v1.0 release/v2.0` |
| `feature/new` | `master` | 新 → 旧 | `backport` | `pr backport feature/new master` |

*注：`develop` 和 `master` 的关系取决于项目策略，需要根据实际情况判断。

---

## 💡 最佳实践

### 1. 明确版本关系

在使用前，先明确：
- 哪个分支包含更新的代码？
- 哪个分支是目标环境？

### 2. 使用语义化命名

如果分支命名清晰（如 `release/v1.0`、`release/v2.0`），判断会更容易。

### 3. 文档化分支策略

在项目文档中明确：
- 各分支的版本关系
- 何时使用 backport，何时使用 forwardport

### 4. 统一使用一个命令（推荐）

如果命令支持双向移植（如 `pr port`），可以：
- 使用统一的 `pr port` 命令
- 让工具自动判断方向
- 减少用户选择困难

---

## 🔧 实现建议

### 方案 1：提供两个独立命令

```bash
# 向后移植
workflow pr backport <FROM_BRANCH> <TO_BRANCH>

# 向前移植
workflow pr forwardport <FROM_BRANCH> <TO_BRANCH>
```

**优点**：
- 语义清晰
- 用户明确知道操作方向

**缺点**：
- 用户需要判断使用哪个
- 可能造成选择困难

### 方案 2：统一命令，自动判断（推荐）

```bash
# 统一命令，自动判断方向
workflow pr port <FROM_BRANCH> <TO_BRANCH>
```

**优点**：
- 用户不需要判断
- 减少选择困难
- 更灵活

**缺点**：
- 需要工具自动判断版本关系（可能不准确）

### 方案 3：统一命令，提供方向选项

```bash
# 默认自动判断
workflow pr port <FROM_BRANCH> <TO_BRANCH>

# 明确指定方向
workflow pr port <FROM_BRANCH> <TO_BRANCH> --backport
workflow pr port <FROM_BRANCH> <TO_BRANCH> --forwardport
```

**优点**：
- 默认自动判断，减少选择
- 需要时可以明确指定

**缺点**：
- 实现较复杂

---

## 📝 总结

### 核心原则

**Backport**：新版本 → 旧版本
- 将最新修复应用到旧版本
- 确保旧版本也包含修复

**Forwardport**：旧版本 → 新版本
- 将旧版本修复应用到新版本
- 确保新版本包含所有修复

### 判断方法

1. **看版本号**：版本号大的 → 新版本
2. **看分支名**：`master` > `develop` > `release/v2.0` > `release/v1.0`
3. **看提交时间**：较新的提交 → 新版本

### 推荐方案

**使用统一的 `pr port` 命令**：
- 不限定方向，更灵活
- 减少用户选择困难
- 在文档中说明 backport/forwardport 的概念
- 让用户理解操作的本质，而不是纠结命令名

---

## 🔗 相关参考

- [Git Backport 工具](https://github.com/zeebe-io/backport)
- [GitHub Backport Action](https://github.com/zeebe-io/backport-action)
- [Git Cherry-pick 文档](https://git-scm.com/docs/git-cherry-pick)
- [Git Flow 工作流](https://nvie.com/posts/a-successful-git-branching-model/)

