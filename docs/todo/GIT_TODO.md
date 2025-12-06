# Git 工作流待办事项

## 📋 概述

本文档列出 Git 工作流相关的待办功能，包括分支管理、Commit 管理和 Stash 管理。

---

## ✅ 已完成功能

- ✅ `branch clean` - 清理本地分支
- ✅ `branch ignore` - 管理分支忽略列表

---

## ❌ 待实现功能

### 1. 分支管理增强

#### 1.1 `branch create` - 创建分支
- ❌ 创建分支（支持从 JIRA ticket 自动命名）

**命令示例**：
```bash
workflow branch create feature/new-feature         # 创建分支
workflow branch create --from PROJ-123             # 从 JIRA ticket 创建
workflow branch create --from master               # 从指定分支创建
workflow branch create --checkout                  # 创建并切换
```

**实现建议**：
- 使用 `GitBranch::create()` 和 `GitBranch::checkout_branch()`
- 支持分支命名模板（配置文件）
- 自动提取 JIRA ID 并验证

#### 1.2 `branch switch` - 快速切换分支
- ❌ 快速切换分支（支持模糊匹配）

**命令示例**：
```bash
workflow branch switch feature/new-feature         # 切换分支
workflow branch switch --fuzzy                     # 模糊匹配选择
workflow branch switch --create                    # 如果不存在则创建
```

**实现建议**：
- 使用 `GitBranch::checkout_branch()`
- 支持交互式选择（fuzzy finder）
- 自动 stash 未提交的更改

#### 1.3 `branch rename` - 重命名分支
- ❌ 重命名分支

**命令示例**：
```bash
workflow branch rename old-name new-name           # 重命名分支
workflow branch rename --current new-name          # 重命名当前分支
```

**实现建议**：
- 使用 `git branch -m` 命令
- 支持远程分支重命名

#### 1.4 `branch compare` - 对比分支差异
- ❌ 对比分支差异

**命令示例**：
```bash
workflow branch compare branch1 branch2            # 对比两个分支
workflow branch compare branch1 --base master      # 对比与 base 的差异
workflow branch compare --stat                     # 只显示统计
```

#### 1.5 `branch sync` - 同步分支
- ❌ 同步分支（fetch + merge/rebase）

**命令示例**：
```bash
workflow branch sync                                # 同步当前分支
workflow branch sync branch-name                    # 同步指定分支
workflow branch sync --rebase                      # 使用 rebase
```

**实现建议**：
- 可以基于 `pr sync` 的实现
- 支持自动推送

---

### 2. Commit 管理

#### 2.1 `commit amend` - 修改最后一次 commit
- ❌ 修改最后一次 commit

**命令示例**：
```bash
workflow commit amend                              # 修改最后一次 commit
workflow commit amend --message "New message"      # 修改消息
workflow commit amend --no-edit                    # 不编辑消息
```

**实现建议**：
- 使用 `git commit --amend`
- 支持交互式编辑

#### 2.2 `commit squash` - 压缩多个 commits
- ❌ 压缩多个 commits

**命令示例**：
```bash
workflow commit squash HEAD~3                      # 压缩最近 3 个 commits
workflow commit squash --interactive               # 交互式选择
```

**实现建议**：
- 使用 `git rebase -i`
- 支持交互式选择要压缩的 commits

#### 2.3 `commit reword` - 修改 commit 消息
- ❌ 修改 commit 消息

**命令示例**：
```bash
workflow commit reword HEAD                        # 修改最后一次 commit 消息
workflow commit reword HEAD~2                     # 修改倒数第二个
```

#### 2.4 `commit history` - 查看 commit 历史
- ❌ 查看 commit 历史（支持过滤）

**命令示例**：
```bash
workflow commit history                            # 查看历史
workflow commit history --author user@example.com  # 按作者过滤
workflow commit history --since "2024-01-01"       # 按时间过滤
workflow commit history --grep "fix"               # 搜索消息
```

---

### 3. Stash 管理

#### 3.1 `stash list` - 列出所有 stash
- ❌ 列出所有 stash

**命令示例**：
```bash
workflow stash list                                # 列出所有 stash
workflow stash list --stat                         # 显示统计信息
```

#### 3.2 `stash apply` - 应用 stash
- ❌ 应用 stash

**命令示例**：
```bash
workflow stash apply                               # 应用最新的 stash
workflow stash apply stash@{1}                     # 应用指定的 stash
```

#### 3.3 `stash drop` - 删除 stash
- ❌ 删除 stash

**命令示例**：
```bash
workflow stash drop                                # 删除最新的 stash
workflow stash drop stash@{1}                      # 删除指定的 stash
```

#### 3.4 `stash pop` - 应用并删除 stash
- ❌ 应用并删除 stash

**命令示例**：
```bash
workflow stash pop                                 # 应用并删除最新的 stash
workflow stash pop stash@{1}                       # 应用并删除指定的 stash
```

**实现建议**：
- 使用 `GitBranch::stash_push()` 和 `GitBranch::stash_pop()`
- 支持交互式选择 stash

---

## 📊 优先级

### 高优先级
1. **分支管理增强**
   - `branch create` - 创建分支（从 JIRA ticket 自动命名）
   - `branch switch` - 快速切换分支（模糊匹配）

2. **Commit 管理**
   - `commit amend` - 修改最后一次 commit
   - `commit history` - 查看 commit 历史（过滤）

### 中优先级
1. **分支管理增强**
   - `branch rename` - 重命名分支
   - `branch compare` - 对比分支差异
   - `branch sync` - 同步分支

2. **Commit 管理**
   - `commit squash` - 压缩多个 commits
   - `commit reword` - 修改 commit 消息

3. **Stash 管理**
   - `stash list` - 列出所有 stash
   - `stash apply` - 应用 stash
   - `stash drop` - 删除 stash
   - `stash pop` - 应用并删除 stash

---

## 📝 实现建议

### 开发顺序
1. **第一阶段**：分支管理基础功能
   - `branch create` - 创建分支
   - `branch switch` - 快速切换分支

2. **第二阶段**：Commit 管理
   - `commit amend` - 修改最后一次 commit
   - `commit history` - 查看 commit 历史

3. **第三阶段**：分支管理增强和 Stash 管理
   - `branch rename`、`branch compare`、`branch sync`
   - `stash list`、`stash apply`、`stash drop`、`stash pop`

### 技术考虑
1. **Git 操作**：使用 `git2` crate 或直接调用 git 命令
2. **错误处理**：处理 Git 操作失败的情况
3. **交互式选择**：使用 fuzzy finder 提供更好的用户体验
4. **测试**：为新功能添加单元测试和集成测试
5. **文档**：及时更新文档和示例

---

## 📚 相关文档

- [功能拓展分析文档](./FEATURE_EXTENSIONS.md)

---

**最后更新**: 2024-12-19
