# Git 工作流需求文档

## 📋 需求概述

本文档描述 Git 工作流相关的需求，包括分支管理和 Commit 管理功能。

> **注意**：Stash 管理功能已迁移到独立的 [Stash 管理需求文档](./STASH_MANAGEMENT.md)。

**状态**: 📋 需求分析中（部分功能已完成）
**分类**: Git 工作流
**优先级**: 高优先级（分支创建/切换 ✅、Commit 修改 ✅）、中优先级（分支重命名 ✅、分支同步、Commit 压缩/重写 ✅）
**来源**: 从 `docs/todo/GIT_TODO.md` 迁移

---

## 🎯 需求目标

扩展 Git 工作流功能，提供更便捷的分支管理和 Commit 管理能力：
1. 简化分支创建和切换流程
2. 提供便捷的 Commit 历史修改功能

> **注意**：Stash 管理功能请参考 [Stash 管理需求文档](./STASH_MANAGEMENT.md)。

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
workflow branch switch feature/new-feature         # 直接指定分支名切换
workflow branch switch                              # 不带参数时，自动进入交互式选择
workflow branch switch --fuzzy                     # 明确指定使用模糊匹配/交互式选择
workflow branch switch feature/new-feature --create # 如果不存在则创建
```

##### 实现建议
- 使用 `GitBranch::checkout_branch()`
- 支持交互式选择（使用 `SelectDialog`，支持搜索功能）
- 自动 stash 未提交的更改

**状态**: ✅ 已完成

---

#### 1.3 `branch rename` - 重命名分支

##### 功能描述
重命名分支，支持本地和远程分支。提供交互式流程，引导用户完成分支重命名操作。

##### 功能要求
- 支持重命名指定分支
- 支持重命名当前分支
- 支持远程分支重命名
- 提供交互式流程，无需记忆复杂参数
- 多重验证和确认机制，防止误操作

##### 命令示例
```bash
workflow branch rename                              # 交互式重命名（推荐）
workflow branch rename old-name new-name           # 重命名指定分支（非交互式）
workflow branch rename --current new-name          # 重命名当前分支（非交互式）
```

##### 实现建议
- 使用 `git branch -m` 命令
- 支持远程分支重命名

**状态**: ✅ 已完成

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

**状态**: ✅ 已完成

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

**状态**: ✅ 已完成
- ✅ 支持修改 HEAD（使用 amend）
- ✅ 支持修改历史 commit（使用 rebase -i）
- ✅ 交互式选择 commit（支持 fuzzy-matcher）
- ✅ 无参数时默认使用 HEAD

---

### 3. Stash 管理

> **已迁移**：Stash 管理功能的详细需求已迁移到独立的 [Stash 管理需求文档](./STASH_MANAGEMENT.md)，请参考该文档获取完整的需求说明。

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
- [x] `branch create` 能够创建分支
- [x] 支持从 JIRA ticket 自动生成分支名
- [x] `branch switch` 能够快速切换分支
- [x] 支持模糊匹配和交互式选择（分支数量 > 25 时自动启用 fuzzy filter）
- [x] 支持分支不存在时询问是否创建
- [x] 自动处理未提交的更改（stash）
- [x] `branch rename` 能够重命名分支
  - [x] 交互式流程实现
  - [x] 支持本地分支重命名
  - [x] 支持远程分支重命名（交互式确认）
  - [x] 完整的分支名格式验证
- [ ] `branch sync` 能够同步分支（fetch + merge/rebase）

### Commit 管理
- [x] `commit amend` 能够修改最后一次 commit
  - [x] 支持修改提交消息
  - [x] 支持添加文件到最后一次 commit
  - [x] 支持不编辑消息直接提交（--no-edit）
  - [x] 完整的交互式界面
  - [x] 预览和确认机制
- [ ] `commit squash` 能够压缩多个 commits
- [x] `commit reword` 能够修改 commit 消息
  - [x] 支持修改 HEAD（使用 amend）
  - [x] 支持修改历史 commit（使用 rebase -i）
  - [x] 交互式选择 commit（支持 fuzzy-matcher）
  - [x] 无参数时默认使用 HEAD
  - [x] 完整的交互式界面
  - [x] 预览和确认机制
- [x] 所有操作都支持交互式界面

### Stash 管理
> **已迁移**：Stash 管理的验收标准请参考 [Stash 管理需求文档](./STASH_MANAGEMENT.md) 中的验收标准部分。

---

## 📊 优先级说明

### 高优先级
1. **`branch create`** - 创建分支（从 JIRA ticket 自动命名） ✅ 已完成
2. **`branch switch`** - 快速切换分支（模糊匹配） ✅ 已完成
3. **`commit amend`** - 修改最后一次 commit ✅ 已完成

### 中优先级
1. **`branch rename`** - 重命名分支 ✅ 已完成
2. **`branch sync`** - 同步分支
3. **`commit squash`** - 压缩多个 commits
4. **`commit reword`** - 修改 commit 消息 ✅ 已完成

> **注意**：Stash 管理的优先级说明请参考 [Stash 管理需求文档](./STASH_MANAGEMENT.md)。

---

## 🔗 依赖关系

### 实现顺序建议
1. **第一阶段**：分支管理基础功能 ✅ 已完成
   - `branch create` - 创建分支 ✅ 已完成
   - `branch switch` - 快速切换分支 ✅ 已完成
   - `branch rename` - 重命名分支 ✅ 已完成

2. **第二阶段**：Commit 管理 ✅ 部分完成
   - `commit amend` - 修改最后一次 commit ✅ 已完成
   - `commit reword` - 修改 commit 消息 ✅ 已完成

3. **第三阶段**：分支管理增强和 Commit 管理完善
   - `branch sync` - 同步分支 ✅ 已完成
   - `commit squash` - 压缩多个 commits
   - Stash 管理（参考 [Stash 管理需求文档](./STASH_MANAGEMENT.md)）

### 技术依赖
- 所有命令都需要 Git 仓库环境
- `branch create --from PROJ-123` 需要 JIRA 集成
- 交互式功能需要终端 UI 库支持

---

## 📚 相关文档

- [JIRA 模块待办事项](../todo/JIRA_TODO.md)
- 模板系统需求文档 - ✅ 已完成（2025-01-27）
- [Stash 管理需求文档](./STASH_MANAGEMENT.md) - Stash 管理功能的详细需求

---

**创建日期**: 2025-01-27
**最后更新**: 2025-01-27

## 📝 更新日志

- **2025-01-27**: `branch create` 功能已完成 ✅
- **2025-01-27**: `branch switch` 功能已完成 ✅
- **2025-01-27**: `branch rename` 功能已完成 ✅
- **2025-01-27**: `branch sync` 功能已完成 ✅
  - 支持 merge、rebase、squash 三种同步策略
  - 支持 fast-forward only 模式
  - 纯分支操作，无 PR 相关逻辑
  - 自动处理工作区状态（stash）
- **2025-01-27**: `commit amend` 功能已完成 ✅
  - 支持修改提交消息、添加文件、不编辑消息直接提交
  - 完整的交互式界面和预览确认机制
- **2025-01-27**: `commit reword` 功能已完成 ✅
  - 支持修改 HEAD（使用 amend）和历史 commit（使用 rebase -i）
  - 交互式选择 commit（支持 fuzzy-matcher）
  - 无参数时默认使用 HEAD
  - 完整的交互式界面和预览确认机制
