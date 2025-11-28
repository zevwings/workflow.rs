# PR Integrate 命令总结

## 📋 文档概述

本文档总结 `pr integrate` 命令的用途、功能和使用场景。

**文档创建时间**: 2024-12

---

## 一、命令概述

**命令名称**: `pr integrate`

**功能描述**: 将指定分支合并到当前分支（本地 Git 操作），并自动处理相关的 PR 和分支清理工作。

**命令格式**:
```bash
workflow pr integrate <SOURCE_BRANCH> [OPTIONS]
```

**与 `pr merge` 的区别**:
- `pr integrate`: 本地 Git 合并操作，将分支合并到当前分支
- `pr merge`: 通过 API 合并 PR（远程操作）

---

## 二、核心功能

### 2.1 主要功能

1. **分支合并**
   - 将源分支（SOURCE_BRANCH）合并到当前分支
   - 支持多种合并策略（fast-forward、squash、普通合并）

2. **工作区管理**
   - 自动检测未提交的更改
   - 提示用户 stash 或中止操作
   - 合并后自动恢复 stash

3. **PR 管理**
   - 合并后自动推送更新当前分支的 PR（如果存在）
   - 自动关闭被合并分支的 PR（如果存在）

4. **分支清理**
   - 合并成功后自动删除源分支（本地和远程）
   - 安全处理分支删除（不会删除当前分支）

5. **安全检查**
   - 运行预检查（pre-flight checks）
   - 禁止合并默认分支到其他分支
   - 自动处理合并冲突

### 2.2 参数说明

**必需参数**:
- `SOURCE_BRANCH` - 要合并的源分支名称

**可选参数**:
- `--ff-only` - 只允许 fast-forward 合并（如果无法 fast-forward 则失败）
- `--squash` - 使用 squash 合并（将所有提交压缩为一个）
- `--no-push` - 不推送到远程（默认会推送）

---

## 三、工作流程

```
1. 运行预检查（check::CheckCommand::run_all()）
2. 获取当前分支
3. 检查工作区状态并 stash（如果需要）
4. 验证并准备源分支
   - 检查分支是否存在（本地或远程）
   - 禁止合并默认分支
   - 如果只在远程，先 fetch
5. 确定合并策略（--ff-only, --squash, default）
6. 执行合并（GitBranch::merge_branch()）
7. 处理合并结果
   - 如果成功：更新当前分支的 PR，关闭源分支的 PR
   - 如果冲突：提示用户手动解决
8. 恢复 stash（如果有）
9. 推送到远程（如果未指定 --no-push）
10. 删除被合并的源分支（本地和远程）
```

---

## 四、使用场景

### 场景 1：合并功能分支到当前分支

```bash
# 当前在 feature-main 分支，需要合并 feature-sub 分支
git checkout feature-main
workflow pr integrate feature-sub

# 结果：
# - 将 feature-sub 合并到 feature-main
# - 更新 feature-main 的 PR（如果存在）
# - 关闭 feature-sub 的 PR（如果存在）
# - 删除 feature-sub 分支（本地和远程）
```

### 场景 2：使用 squash 合并

```bash
workflow pr integrate feature-sub --squash

# 结果：
# - 将 feature-sub 的所有提交压缩为一个提交
# - 合并到当前分支
```

### 场景 3：只允许 fast-forward 合并

```bash
workflow pr integrate feature-sub --ff-only

# 结果：
# - 只允许 fast-forward 合并
# - 如果无法 fast-forward，失败并提示
```

### 场景 4：合并但不推送

```bash
workflow pr integrate feature-sub --no-push

# 结果：
# - 合并到本地
# - 不推送到远程（用户可以手动推送）
```

---

## 五、与其他命令的区别

### 5.1 与 `pr merge` 的区别

| 特性 | `pr integrate` | `pr merge` |
|------|----------------|------------|
| **操作类型** | 本地 Git 合并 | 远程 API 合并 |
| **操作对象** | 分支合并到当前分支 | 通过 API 合并 PR |
| **使用场景** | 合并功能分支 | 合并并关闭 PR |
| **结果** | 更新当前分支 | 关闭 PR |

### 5.2 与 `pr sync` 的区别

| 特性 | `pr integrate` | `pr sync` |
|------|----------------|-----------|
| **操作类型** | 合并（merge） | 同步（merge 或 rebase） |
| **默认行为** | Merge | Merge（可切换为 rebase） |
| **使用场景** | 集成功能分支 | 同步基础分支更新 |
| **语义** | 将功能合并进来 | 与基础分支保持同步 |
| **参数** | `source_branch`（必需） | `SOURCE_BRANCH`（必需） |
| **特殊选项** | `--squash`, `--ff-only`, `--no-push` | `--rebase`, `--ff-only`, `--no-push` |
| **分支清理** | ✅ 自动删除源分支 | ❌ 不删除分支 |
| **PR 管理** | ✅ 更新当前 PR，关闭源分支 PR | ❌ 不管理 PR |

### 5.3 选择建议

**使用 `pr integrate` 当：**
- 需要将另一个功能分支的更改合并到当前分支
- 需要合并后自动清理源分支
- 需要自动更新当前分支的 PR
- 需要自动关闭被合并分支的 PR
- 合并完成后不再需要源分支

**使用 `pr sync` 当：**
- 需要将基础分支（如 master、develop）的最新更改同步到当前分支
- 不需要删除分支
- 不需要管理 PR
- 需要 merge 或 rebase 整个分支

---

## 六、边界情况处理

### 情况 1：工作区有未提交更改

**处理方式**:
- 自动检测未提交更改
- 提示用户选择：stash 并继续，或中止操作
- 合并后自动恢复 stash

### 情况 2：源分支不存在

**处理方式**:
- 检查分支是否存在（本地和远程）
- 如果只在远程，先 fetch
- 如果都不存在，提示错误并退出

### 情况 3：合并冲突

**处理方式**:
- 检测合并冲突
- 提示用户手动解决：
  ```
  Merge conflicts detected!
  Please resolve the conflicts manually:
    1. Review conflicted files
    2. Resolve conflicts
    3. Stage resolved files: git add <files>
    4. Complete the merge: git commit
    5. Push when ready: git push
  ```
- 如果合并失败，自动恢复 stash

### 情况 4：源分支是默认分支

**处理方式**:
- 禁止合并默认分支到其他分支
- 提示错误：`Cannot integrate default branch 'master' into current branch.`

### 情况 5：源分支是当前分支

**处理方式**:
- 跳过删除操作
- 提示：`Source branch 'xxx' is the current branch, skipping deletion`

---

## 七、典型使用流程

### 场景：合并功能分支

```bash
# 1. 切换到目标分支
git checkout feature-main

# 2. 合并源分支
workflow pr integrate feature-sub

# 自动执行：
# - 检查工作区状态
# - 合并 feature-sub 到 feature-main
# - 更新 feature-main 的 PR（如果存在）
# - 关闭 feature-sub 的 PR（如果存在）
# - 删除 feature-sub 分支（本地和远程）
# - 推送更新到远程
```

---

## 八、总结

**`pr integrate` 的核心用途**:

1. **集成功能分支**: 将功能分支合并到当前分支
2. **自动化工作流**: 自动处理 PR 更新、分支清理等操作
3. **本地 Git 操作**: 在本地执行合并，然后推送到远程
4. **分支管理**: 合并后自动清理不再需要的分支

**适用场景**:
- 合并功能分支到主功能分支
- 合并子功能到父功能
- 合并完成后需要清理源分支的场景

**设计特点**:
- **自动化**: 尽量减少手动步骤
- **安全性**: 运行预检查，禁止危险操作
- **用户友好**: 清晰的提示和错误处理
- **完整性**: 自动处理 PR 和分支清理

