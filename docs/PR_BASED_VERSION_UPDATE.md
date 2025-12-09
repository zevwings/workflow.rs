# 基于 Pull Request 的版本更新机制

## 概述

工作流已更新为使用 Pull Request 方式更新版本，而不是直接推送到受保护的 `master` 分支。这是更安全、更符合最佳实践的方法。

## 工作流程

### 1. 触发条件

当代码推送到 `master` 分支时，`create-tag` job 会运行并检查是否需要创建新 tag。

### 2. 版本检查

`create-tag` job 会：
- 从 `Cargo.toml` 读取当前版本
- 检查对应的 tag 是否已存在
- 如果 tag 已存在但指向不同的提交，则标记为需要递增版本

### 3. 创建版本更新 PR

如果需要递增版本，工作流会：

1. **创建新分支**：`bump-version-{version}`
   ```bash
   git checkout -b bump-version-1.5.1
   ```

2. **更新版本文件**：
   - 更新 `Cargo.toml` 中的版本号
   - 更新 `Cargo.lock` 中的版本号

3. **提交更改**：
   ```bash
   git commit -m "chore: bump version to 1.5.1"
   ```

4. **推送到新分支**：
   ```bash
   git push origin bump-version-1.5.1
   ```

5. **创建 Pull Request**：
   - 使用 GitHub API 自动创建 PR
   - PR 标题：`chore: bump version to {version}`
   - PR 描述包含：
     - 版本更新说明
     - 更改的文件列表
     - 工作流运行链接

### 4. 自动合并 PR

PR 创建后，工作流会自动：
- ✅ **等待 CI 检查通过**：工作流会轮询 PR 状态，等待最多 10 分钟
- ✅ **自动合并**：当 PR 可合并时（所有检查通过），自动使用 squash merge 合并
- ✅ **错误处理**：如果超时或合并失败，会显示错误信息并提供手动合并链接

**合并方式**：使用 `squash merge`，保持提交历史整洁

**超时处理**：如果 10 分钟内 PR 仍未可合并，工作流会：
- 显示超时信息
- 提供手动合并的 PR 链接
- 不会导致工作流失败（允许后续手动处理）

### 5. 自动创建 Tag

PR 合并到 `master` 后：
- 触发新的工作流运行（push 事件）
- `create-tag` job 检测到版本已更新
- 创建并推送对应的 tag（如 `v1.5.1`）

### 6. 构建和发布

Tag 创建后：
- 触发构建 job，为所有平台构建二进制文件
- 创建 GitHub Release
- 上传所有平台的构建产物
- 更新 Homebrew Formula

## 自动合并机制

工作流实现了智能的自动合并机制：

1. **创建 PR 后立即开始等待**
   - 每 30 秒检查一次 PR 状态
   - 最多等待 10 分钟

2. **检查 PR 可合并性**
   - 检查 `mergeable` 状态
   - 检查是否已合并
   - 等待 CI 检查完成

3. **自动合并**
   - 使用 `squash merge` 方式
   - 合并提交信息：`chore: bump version to {version}`
   - 包含工作流运行信息

4. **错误处理**
   - 如果 PR 已合并，跳过合并步骤
   - 如果超时，提供手动合并链接
   - 如果合并失败，显示错误信息

**配置选项**（未来可扩展）：
- 可以通过环境变量配置等待时间
- 可以通过环境变量配置是否启用自动合并
- 可以通过环境变量配置合并方式（squash/merge/rebase）

## 优势

### 安全性
- ✅ 不需要绕过分支保护规则
- ✅ 不需要使用高权限的 PAT
- ✅ 符合 GitHub 安全最佳实践

### 可审查性
- ✅ 版本更新可以通过 PR 进行审查
- ✅ 可以查看具体的更改内容
- ✅ 可以添加评论和讨论

### 自动化
- ✅ PR 创建完全自动化
- ✅ PR 合并完全自动化（等待 CI 后自动合并）
- ✅ 无需手动干预即可完成整个流程
- ✅ 合并后自动触发后续流程（创建 tag 和发布）

### 灵活性
- ✅ 自动合并，无需手动干预
- ✅ 如果自动合并失败，仍可手动合并
- ✅ 超时机制确保不会无限等待
- ✅ 可以添加额外的检查步骤（通过分支保护规则）

## 配置要求

### 工作流权限

工作流已配置以下权限：

```yaml
permissions:
  contents: write      # 创建 Release 和推送 tag
  pull-requests: write # 创建 Pull Request
  statuses: write      # 创建状态检查
```

### 分支保护规则

**无需特殊配置**：
- PR 方式不需要绕过分支保护规则
- 只需要确保允许从 `bump-version-*` 分支创建 PR（默认允许）

**可选配置**：
- 可以配置自动合并规则（当 CI 通过时自动合并）
- 可以配置 PR 模板，添加额外的检查步骤

## PR 内容示例

### PR 标题
```
chore: bump version to 1.5.1
```

### PR 描述
```
Automated version bump to 1.5.1

This PR was created automatically by the release workflow.

**Changes:**
- Updated version in Cargo.toml to 1.5.1
- Updated version in Cargo.lock to 1.5.1

**Workflow Run:** https://github.com/zevwings/workflow.rs/actions/runs/123456789
```

## 故障排除

### PR 创建失败

如果 PR 创建失败（HTTP 非 201），工作流会：
- 显示错误信息
- 提供手动创建 PR 的链接
- 检查是否已存在相同版本的 PR

### 分支已存在

如果分支 `bump-version-{version}` 已存在：
- Git push 会失败
- 工作流会显示错误信息
- 需要手动删除旧分支或使用不同的版本号

### 自动合并超时

如果 PR 在 10 分钟内未变为可合并状态：
- 工作流会显示超时信息
- 提供手动合并的 PR 链接
- 可能的原因：
  - CI 检查仍在运行（超过 10 分钟）
  - 分支保护规则要求手动审查
  - 存在合并冲突
  - 其他检查未通过

**解决方案**：
1. 检查 PR 页面，查看 CI 检查状态
2. 如果检查通过但仍未合并，手动合并 PR
3. 如果存在合并冲突，解决冲突后手动合并

### 自动合并失败

如果自动合并失败（HTTP 非 200）：
- 工作流会显示错误信息
- 提供手动合并的 PR 链接
- 可能的原因：
  - 权限不足
  - PR 已被合并
  - 分支保护规则阻止自动合并
  - 合并冲突

**解决方案**：
1. 检查 PR 是否已合并
2. 检查分支保护规则设置
3. 手动合并 PR

### PR 合并后未触发 tag 创建

确保：
1. PR 已成功合并到 `master`
2. 合并触发了 push 事件
3. `create-tag` job 正常运行
4. 版本号已正确更新

## 与旧方案的对比

| 特性 | 直接推送（旧） | PR 方式（新） |
|------|---------------|--------------|
| 安全性 | ⚠️ 需要绕过分支保护 | ✅ 不需要特殊配置 |
| 可审查性 | ❌ 无法审查 | ✅ 可以审查 |
| 自动化 | ✅ 完全自动 | ✅ 完全自动（需合并 PR） |
| 配置复杂度 | ⚠️ 需要配置分支保护 | ✅ 无需特殊配置 |
| 符合最佳实践 | ❌ 不符合 | ✅ 符合 |

## 相关文档

- [分支保护规则配置指南](./BRANCH_PROTECTION_SETUP.md)
- [解决方案分析](./SOLUTION_ANALYSIS.md)
