# Git 工作流需求文档

## 📋 需求概述

本文档描述 Git 工作流相关的需求，包括分支管理、Commit 管理和 Stash 管理功能。

**状态**: 📋 需求分析中
**分类**: Git 工作流
**优先级**: 高优先级（分支创建/切换、Commit 修改）、中优先级（分支同步/重命名、Commit 压缩/重写、Stash 管理）
**来源**: 从 `docs/todo/GIT_TODO.md` 迁移

---

## 🎯 需求目标

扩展 Git 工作流功能，提供更便捷的分支管理、Commit 管理和 Stash 管理能力：
1. 简化分支创建和切换流程
2. 提供便捷的 Commit 历史修改功能
3. 完善 Stash 管理功能

---

## 📝 详细需求

### 1. 分支管理增强

#### 1.1 `branch create` - 创建分支

##### 功能描述
创建分支，支持从 JIRA ticket 自动命名。

##### 功能要求
- 支持直接指定分支名创建
- 支持从 JIRA ticket 自动生成分支名
- 支持从指定分支创建
- 支持创建并自动切换

##### 命令示例
```bash
workflow branch create feature/new-feature         # 创建分支
workflow branch create --from PROJ-123             # 从 JIRA ticket 创建
workflow branch create --from master               # 从指定分支创建
workflow branch create --checkout                  # 创建并切换
```

##### 实现建议
- 使用 `GitBranch::create()` 和 `GitBranch::checkout_branch()`
- 支持分支命名模板（配置文件）
- 自动提取 JIRA ID 并验证

---

#### 1.2 `branch switch` - 快速切换分支

##### 功能描述
快速切换分支，支持模糊匹配和交互式选择。

##### 功能要求
- 支持直接指定分支名切换
- 支持模糊匹配选择分支
- 支持如果分支不存在则创建
- 自动处理未提交的更改（stash）

##### 命令示例
```bash
workflow branch switch feature/new-feature         # 切换分支
workflow branch switch --fuzzy                     # 模糊匹配选择
workflow branch switch --create                    # 如果不存在则创建
```

##### 实现建议
- 使用 `GitBranch::checkout_branch()`
- 支持交互式选择（fuzzy finder）
- 自动 stash 未提交的更改

---

#### 1.3 `branch rename` - 重命名分支

##### 功能描述
重命名分支，支持本地和远程分支。

##### 功能要求
- 支持重命名指定分支
- 支持重命名当前分支
- 支持远程分支重命名

##### 命令示例
```bash
workflow branch rename old-name new-name           # 重命名分支
workflow branch rename --current new-name          # 重命名当前分支
```

##### 实现建议
- 使用 `git branch -m` 命令
- 支持远程分支重命名

---

#### 1.4 `branch sync` - 同步分支

##### 功能描述
同步分支（fetch + merge/rebase），保持分支与远程同步。

##### 功能要求
- 支持同步当前分支
- 支持同步指定分支
- 支持使用 rebase 方式同步
- 支持自动推送

##### 命令示例
```bash
workflow branch sync                                # 同步当前分支
workflow branch sync branch-name                    # 同步指定分支
workflow branch sync --rebase                      # 使用 rebase
```

##### 实现建议
- 可以基于 `pr sync` 的实现
- 支持自动推送

---

### 2. Commit 管理

#### 2.1 `commit amend` - 修改最后一次 commit

##### 功能描述
修改最后一次 commit，支持修改消息和内容。

##### 功能要求
- 支持修改 commit 消息
- 支持添加文件到最后一次 commit
- 支持不编辑消息直接提交

##### 命令示例
```bash
workflow commit amend                              # 修改最后一次 commit
workflow commit amend --message "New message"      # 修改消息
workflow commit amend --no-edit                    # 不编辑消息
```

##### 实现建议
- 使用 `git commit --amend`
- 支持交互式编辑

---

#### 2.2 `commit squash` - 压缩多个 commits

##### 功能描述
压缩多个 commits 为一个，简化提交历史。

##### 功能要求
- 支持压缩指定数量的 commits
- 支持交互式选择要压缩的 commits
- 支持自定义压缩后的 commit 消息

##### 命令示例
```bash
workflow commit squash HEAD~3                      # 压缩最近 3 个 commits
workflow commit squash --interactive               # 交互式选择
```

##### 实现建议
- 使用 `git rebase -i`
- 支持交互式选择要压缩的 commits

---

#### 2.3 `commit reword` - 修改 commit 消息

##### 功能描述
修改指定 commit 的消息，不改变提交内容。

##### 功能要求
- 支持修改最后一次 commit 消息
- 支持修改历史 commit 消息（通过 rebase）
- 支持交互式编辑

##### 命令示例
```bash
workflow commit reword HEAD                        # 修改最后一次 commit 消息
workflow commit reword HEAD~2                     # 修改倒数第二个
```

##### 实现建议
- 使用 `git rebase -i` 的 reword 功能
- 提供友好的交互式界面

---

### 3. Stash 管理

#### 3.1 `stash list` - 列出所有 stash

##### 功能描述
列出所有 stash 条目，显示详细信息。

##### 功能要求
- 列出所有 stash 条目
- 显示每个 stash 的创建时间、消息等信息
- 支持显示统计信息

##### 命令示例
```bash
workflow stash list                                # 列出所有 stash
workflow stash list --stat                         # 显示统计信息
```

##### 实现建议
- 使用 `git stash list` 命令
- 解析并格式化输出
- 支持表格或列表显示

---

#### 3.2 `stash apply` - 应用 stash

##### 功能描述
应用指定的 stash，保留 stash 条目。

##### 功能要求
- 支持应用最新的 stash
- 支持应用指定的 stash
- 支持交互式选择 stash

##### 命令示例
```bash
workflow stash apply                               # 应用最新的 stash
workflow stash apply stash@{1}                     # 应用指定的 stash
```

##### 实现建议
- 使用 `GitBranch::stash_apply()` 或 `git stash apply`
- 支持交互式选择 stash

---

#### 3.3 `stash drop` - 删除 stash

##### 功能描述
删除指定的 stash 条目。

##### 功能要求
- 支持删除最新的 stash
- 支持删除指定的 stash
- 支持交互式选择要删除的 stash
- 提供确认提示（避免误删）

##### 命令示例
```bash
workflow stash drop                                # 删除最新的 stash
workflow stash drop stash@{1}                     # 删除指定的 stash
```

##### 实现建议
- 使用 `git stash drop` 命令
- 支持交互式选择
- 提供安全确认机制

---

#### 3.4 `stash pop` - 应用并删除 stash

##### 功能描述
应用 stash 并删除该条目，相当于 apply + drop。

##### 功能要求
- 支持应用并删除最新的 stash
- 支持应用并删除指定的 stash
- 支持交互式选择 stash

##### 命令示例
```bash
workflow stash pop                                 # 应用并删除最新的 stash
workflow stash pop stash@{1}                      # 应用并删除指定的 stash
```

##### 实现建议
- 使用 `GitBranch::stash_pop()` 或 `git stash pop`
- 支持交互式选择 stash

---

## 🔧 技术实现

### Git 操作库
- 使用 `git2` crate 进行 Git 操作
- 对于复杂操作，可以调用 `git` 命令
- 保持与现有 Git 模块的一致性

### 交互式界面
- 使用 `dialoguer` 或类似库实现交互式选择
- 使用 fuzzy finder 提供分支选择功能
- 提供清晰的错误提示和确认机制

### 分支命名模板
- 支持配置文件中的分支命名模板
- 支持从 JIRA ticket 自动提取信息
- 验证分支名格式

### 错误处理
- 处理 Git 操作失败的情况
- 提供详细的错误信息
- 支持操作回滚（如果可能）

---

## ✅ 验收标准

### 分支管理
- [ ] `branch create` 能够创建分支
- [ ] 支持从 JIRA ticket 自动生成分支名
- [ ] `branch switch` 能够快速切换分支
- [ ] 支持模糊匹配和交互式选择
- [ ] `branch rename` 能够重命名分支
- [ ] `branch sync` 能够同步分支（fetch + merge/rebase）

### Commit 管理
- [ ] `commit amend` 能够修改最后一次 commit
- [ ] `commit squash` 能够压缩多个 commits
- [ ] `commit reword` 能够修改 commit 消息
- [ ] 所有操作都支持交互式界面

### Stash 管理
- [ ] `stash list` 能够列出所有 stash
- [ ] `stash apply` 能够应用 stash
- [ ] `stash drop` 能够删除 stash
- [ ] `stash pop` 能够应用并删除 stash
- [ ] 支持交互式选择 stash

---

## 📊 优先级说明

### 高优先级
1. **`branch create`** - 创建分支（从 JIRA ticket 自动命名）
2. **`branch switch`** - 快速切换分支（模糊匹配）
3. **`commit amend`** - 修改最后一次 commit

### 中优先级
1. **`branch rename`** - 重命名分支
2. **`branch sync`** - 同步分支
3. **`commit squash`** - 压缩多个 commits
4. **`commit reword`** - 修改 commit 消息
5. **Stash 管理** - 所有 stash 相关命令

---

## 🔗 依赖关系

### 实现顺序建议
1. **第一阶段**：分支管理基础功能
   - `branch create` - 创建分支
   - `branch switch` - 快速切换分支

2. **第二阶段**：Commit 管理
   - `commit amend` - 修改最后一次 commit

3. **第三阶段**：分支管理增强和 Stash 管理
   - `branch rename`、`branch sync`
   - `commit squash`、`commit reword`
   - `stash list`、`stash apply`、`stash drop`、`stash pop`

### 技术依赖
- 所有命令都需要 Git 仓库环境
- `branch create --from PROJ-123` 需要 JIRA 集成
- 交互式功能需要终端 UI 库支持

---

## 📚 相关文档

- [Git 工作流待办事项](../todo/GIT_TODO.md)
- [JIRA 模块待办事项](../todo/JIRA_TODO.md)
- [工作流自动化待办事项](../todo/WORKFLOW_TODO.md)

---

**创建日期**: 2025-01-27
**最后更新**: 2025-01-27
