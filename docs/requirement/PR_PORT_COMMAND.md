# PR Port 命令设计文档

## 📋 文档概述

本文档详细描述 `pr port` 命令的设计，该命令用于从源分支 cherry-pick 提交到目标分支并创建新 PR。

**文档创建时间**: 2024-12

---

## 一、命令概述

**命令名称**: `pr port`

**功能描述**: 从源分支（FROM_BRANCH）cherry-pick 所有提交到目标分支（TO_BRANCH），创建新分支并自动创建新的 PR。

**命令格式**:
```bash
workflow pr port <FROM_BRANCH> <TO_BRANCH> [OPTIONS]
```

---

## 二、功能设计

### 2.1 核心功能

1. **检测源分支的提交**
   - 找到 FROM_BRANCH 相对于 TO_BRANCH 的所有新提交
   - 支持从 PR 获取提交列表（如果源分支有 PR）

2. **创建新分支**
   - 基于 TO_BRANCH 创建新分支
   - 自动生成分支名（如：`port-develop-to-master-20241201`）
   - 支持自定义分支名

3. **Cherry-pick 提交**
   - 按顺序 cherry-pick 所有提交
   - 处理冲突（暂停并提示用户）

4. **创建新 PR**
   - 自动创建基于 TO_BRANCH 的新 PR
   - 从源 PR 复制标题和描述（如果存在）
   - 支持自定义 PR 标题和描述

### 2.2 参数设计

**必需参数**:
- `FROM_BRANCH` - 源分支名称（要 cherry-pick 的分支）
- `TO_BRANCH` - 目标分支名称（新 PR 的 base 分支）

**可选参数**:
- `--branch-name <NAME>` - 新分支名称（如果不指定，自动生成）
- `--pr-title <TITLE>` - PR 标题（如果不指定，从源 PR 复制或自动生成）
- `--pr-description <DESC>` - PR 描述（如果不指定，从源 PR 复制）
- `--no-create-pr` - 只 cherry-pick，不创建 PR
- `--dry-run` - 预览模式，显示将要执行的操作但不实际执行
- `--force` - 如果新分支已存在，强制覆盖（谨慎使用）

### 2.3 工作流程

```
1. 验证 FROM_BRANCH 存在（本地或远程）
2. 验证 TO_BRANCH 存在（本地或远程）
3. 拉取最新代码（fetch）
4. 检测 FROM_BRANCH 相对于 TO_BRANCH 的新提交
   - 如果没有新提交，提示并退出
5. 检查源分支是否有 PR
   - 如果有，获取 PR 信息（标题、描述、提交列表）
6. 生成新分支名（如果未指定）
   - 格式：port-{FROM_BRANCH}-to-{TO_BRANCH}-{timestamp}
7. 在 TO_BRANCH 上创建新分支
8. Cherry-pick 所有提交
   - 如果有冲突，暂停并提示用户解决
9. 推送到远程
10. 创建新 PR（如果未指定 --no-create-pr）
    - 使用源 PR 的标题和描述（如果存在）
    - 或使用自定义的标题和描述
11. 显示新 PR 的 URL
```

---

## 三、使用场景

### 场景 1：将 develop 的修改应用到 master

```bash
# 在 develop 上完成开发后，需要应用到 master
workflow pr port develop master

# 结果：
# - 创建新分支 port-develop-to-master-20241201
# - Cherry-pick develop 的所有新提交
# - 创建基于 master 的新 PR
```

### 场景 2：将 feature 分支应用到 release 分支

```bash
workflow pr port feature-branch release/v1.0

# 结果：
# - 创建新分支 port-feature-branch-to-release-v1.0-20241201
# - Cherry-pick feature-branch 的所有提交
# - 创建基于 release/v1.0 的新 PR
```

### 场景 3：自定义分支名和 PR 标题

```bash
workflow pr port develop master \
  --branch-name hotfix-security \
  --pr-title "Security fix: Apply develop changes to master"
```

### 场景 4：只 cherry-pick，不创建 PR

```bash
workflow pr port develop master --no-create-pr

# 结果：
# - 创建新分支并 cherry-pick
# - 不创建 PR（用户可以手动创建）
```

### 场景 5：预览模式

```bash
workflow pr port develop master --dry-run

# 结果：
# - 显示将要执行的操作
# - 不实际执行
```

---

## 四、边界情况处理

### 情况 1：源分支没有新提交

**处理方式**:
- 检测到 FROM_BRANCH 相对于 TO_BRANCH 没有新提交
- 提示用户：`No new commits to cherry-pick from {FROM_BRANCH} to {TO_BRANCH}`
- 退出，不创建分支和 PR

### 情况 2：Cherry-pick 冲突

**处理方式**:
- 暂停 cherry-pick 操作
- 显示冲突文件列表
- 提示用户解决冲突：
  ```
  Cherry-pick conflict detected!
  Please resolve conflicts in:
    - file1.rs
    - file2.rs

  After resolving:
    1. git add <resolved-files>
    2. git cherry-pick --continue
    3. workflow pr port --continue  # 继续执行
  ```
- 提供 `--continue` 选项继续执行
- 提供 `--abort` 选项中止操作

### 情况 3：源分支有多个 PR

**处理方式**:
- 检测到源分支有多个 PR（open 状态）
- 提示用户选择要使用的 PR：
  ```
  Multiple PRs found for branch 'develop':
    1. PR #123: Fix bug (open)
    2. PR #124: Add feature (open)

  Which PR to use? [1-2]:
  ```
- 或使用最新创建的 PR（默认）

### 情况 4：新分支已存在

**处理方式**:
- 检测到新分支名已存在
- 提示用户：
  ```
  Branch 'port-develop-to-master-20241201' already exists.
  Options:
    1. Use existing branch
    2. Generate new branch name
    3. Force overwrite (--force)
  ```
- 或使用 `--force` 强制覆盖

### 情况 5：源分支不存在

**处理方式**:
- 检查 FROM_BRANCH 是否存在（本地和远程）
- 如果不存在，提示错误并退出
- 提供建议：检查分支名拼写，或先创建分支

### 情况 6：目标分支不存在

**处理方式**:
- 检查 TO_BRANCH 是否存在（本地和远程）
- 如果不存在，提示错误并退出
- 提供建议：检查分支名拼写

---

## 五、实现细节

### 5.1 检测新提交

```rust
// 伪代码
fn get_new_commits(from_branch: &str, to_branch: &str) -> Result<Vec<String>> {
    // 使用 git rev-list 获取 from_branch 相对于 to_branch 的提交
    // git rev-list to_branch..from_branch
    // 返回提交哈希列表
}
```

### 5.2 生成分支名

```rust
// 伪代码
fn generate_branch_name(from: &str, to: &str) -> String {
    let timestamp = chrono::Utc::now().format("%Y%m%d");
    format!("port-{}-to-{}-{}", from, to, timestamp)
}
```

### 5.3 从源 PR 获取信息

```rust
// 伪代码
fn get_source_pr_info(branch: &str) -> Result<Option<PrInfo>> {
    // 1. 查找分支对应的 PR
    // 2. 获取 PR 标题、描述、提交列表
    // 3. 返回 PR 信息
}
```

---

## 六、与其他命令的关系

### 6.1 与 `pr rebase` 的区别

| 特性 | `pr port` | `pr rebase` |
|------|-----------|-------------|
| **操作方式** | Cherry-pick | Rebase |
| **创建新分支** | ✅ 是 | ❌ 否（在当前分支操作） |
| **创建新 PR** | ✅ 是 | ❌ 否（更新现有 PR） |
| **使用场景** | 跨分支移植代码 | 修正当前分支的基础分支 |
| **结果** | 新分支 + 新 PR | 修改现有分支 + 更新 PR base |

### 6.2 与 `pr sync` 的区别

| 特性 | `pr port` | `pr sync` |
|------|-----------|-----------|
| **操作对象** | 从分支 A 到分支 B | 从分支 A 到当前分支 |
| **创建新分支** | ✅ 是 | ❌ 否（在当前分支操作） |
| **创建 PR** | ✅ 是（可选） | ❌ 否 |
| **Cherry-pick** | ✅ 是 | ❌ 否（使用 merge/rebase） |
| **使用场景** | 跨分支移植代码 | 同步基础分支更新 |
| **典型用法** | `pr port develop master` | `pr sync master`（在 feature 分支上） |

### 6.3 选择建议

**使用 `pr port` 当：**
- 需要跨分支移植代码
- 需要创建新的 PR
- 需要 cherry-pick 特定提交
- 源分支和目标分支不同
- 不想修改现有分支

---

## 七、错误处理和用户体验

### 7.1 错误消息

- **源分支不存在**: `Source branch '{FROM_BRANCH}' does not exist. Please check the branch name.`
- **目标分支不存在**: `Target branch '{TO_BRANCH}' does not exist. Please check the branch name.`
- **没有新提交**: `No new commits to cherry-pick from '{FROM_BRANCH}' to '{TO_BRANCH}'.`
- **Cherry-pick 冲突**: `Cherry-pick conflict detected. Please resolve conflicts and use '--continue' to proceed.`
- **新分支已存在**: `Branch '{branch_name}' already exists. Use '--force' to overwrite or specify a different name.`

### 7.2 成功消息

```
✓ Created branch: port-develop-to-master-20241201
✓ Cherry-picked 3 commits from develop
✓ Pushed to remote
✓ Created PR: #456
  URL: https://github.com/owner/repo/pull/456
```

### 7.3 交互式提示

#### 冲突解决提示

```
⚠️  Conflict detected during cherry-pick

Conflicted files:
  - src/file1.rs
  - src/file2.rs

To resolve:
  1. Edit conflicted files
  2. git add <resolved-files>
  3. workflow pr port --continue

To abort:
  workflow pr port --abort
```

---

## 八、实现考虑

### 8.1 技术实现要点

1. **提交检测**
   - 使用 `git rev-list TO_BRANCH..FROM_BRANCH` 获取新提交
   - 处理空结果（没有新提交）

2. **PR 信息获取**
   - 通过 PR API 获取源分支的 PR 信息
   - 获取 PR 的提交列表、标题、描述

3. **分支名生成**
   - 确保分支名唯一
   - 处理特殊字符（替换为 `-`）

4. **Cherry-pick 流程**
   - 逐个 cherry-pick 提交
   - 处理冲突（暂停、提示、继续）

5. **PR 创建**
   - 使用现有的 `create_pull_request` API
   - 从源 PR 复制信息或使用自定义信息

### 8.2 依赖模块

- `lib/git/branch.rs` - 分支操作（需要添加 cherry-pick 方法）
- `lib/pr/platform.rs` - PR 操作（创建 PR、获取 PR 信息）
- `lib/git/commit.rs` - 提交操作（获取提交信息）

### 8.3 需要新增的 Git 方法

#### 在 `GitBranch` 中添加

```rust
impl GitBranch {
    /// Cherry-pick 提交到当前分支
    pub fn cherry_pick(commit: &str) -> Result<()>;

    /// 获取两个分支之间的提交列表
    pub fn get_commits_between(base: &str, head: &str) -> Result<Vec<String>>;
}
```

#### 在 `GitCommit` 中添加

```rust
impl GitCommit {
    /// 获取提交信息
    pub fn get_commit_message(commit: &str) -> Result<String>;

    /// 获取提交的完整信息（包括作者、时间等）
    pub fn get_commit_info(commit: &str) -> Result<CommitInfo>;
}
```

---

## 九、测试场景

1. **正常流程**
   - 源分支有提交，目标分支存在
   - 成功创建分支、cherry-pick、创建 PR

2. **无新提交**
   - 源分支没有新提交
   - 正确提示并退出

3. **Cherry-pick 冲突**
   - 模拟冲突
   - 验证暂停和继续流程

4. **新分支已存在**
   - 分支名冲突
   - 验证处理逻辑

5. **源分支有 PR**
   - 验证从 PR 复制信息

---

## 十、未来扩展

- **支持多个源分支**: `pr port branch1 branch2 master`
- **选择性 cherry-pick**: `pr port develop master --commits abc123 def456`
- **批量操作**: `pr port develop master release/v1.0`（应用到多个目标分支）

---

## 十一、总结

**`pr port`**:
- 用途：跨分支移植代码并创建新 PR
- 特点：创建新分支、cherry-pick、自动创建 PR
- 场景：将 develop 的修改应用到 master

**设计原则**:
1. **自动化**: 尽量减少手动步骤
2. **安全性**: 使用安全的分支操作
3. **用户友好**: 清晰的错误消息和操作指导
4. **灵活性**: 支持多种使用场景和选项

