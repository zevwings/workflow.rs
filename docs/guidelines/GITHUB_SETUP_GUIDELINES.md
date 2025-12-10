# GitHub 配置指南

> 本文档描述了 Workflow CLI 项目在 GitHub 上需要配置的内容，包括 Secrets、Variables、分支保护规则等。

---

## 📋 目录

- [Repository Secrets](#-repository-secrets)
- [Repository Variables](#-repository-variables)
- [分支保护规则](#-分支保护规则)
- [Workflow 权限配置](#-workflow-权限配置)
- [验证配置](#-验证配置)
- [故障排除](#-故障排除)

---

## 🔐 Repository Secrets

Repository Secrets 用于存储敏感信息，如 Personal Access Token (PAT)。这些信息在 workflow 运行时会被注入，但不会在日志中显示。

### 配置位置

**Settings → Secrets and variables → Actions → Secrets**

### 必需的 Secrets

#### 1. WORKFLOW_PAT

**用途**：用于所有需要写权限的操作（创建 PR、合并 PR、推送 tag、更新 Homebrew 等）

**配置步骤**：

1. 创建 Personal Access Token (PAT)：
   - 访问：https://github.com/settings/tokens
   - 点击 "Generate new token (classic)"
   - 选择权限：
     - ✅ `repo`（完整仓库访问权限）
     - ✅ `workflow`（更新 GitHub Actions workflow）
   - 设置过期时间（建议：90 天或更长）
   - 点击 "Generate token"
   - **重要**：立即复制 token，离开页面后将无法再次查看

2. 添加到 Repository Secrets：
   - 访问仓库：Settings → Secrets and variables → Actions
   - 点击 "New repository secret"
   - Name: `WORKFLOW_PAT`
   - Secret: 粘贴刚才复制的 token
   - 点击 "Add secret"

**验证**：
- 检查 workflow 日志，确认 token 可以正常使用
- 测试创建 PR 是否成功

**安全注意事项**：
- ⚠️ 不要将 token 提交到代码仓库
- ⚠️ 定期轮换 token（建议每 90 天）
- ⚠️ 如果 token 泄露，立即撤销并重新创建
- ⚠️ 只授予必要的权限（`repo` 和 `workflow`）

---

## 📝 Repository Variables

Repository Variables 用于存储非敏感配置信息，可以在 workflow 中引用。

### 配置位置

**Settings → Secrets and variables → Actions → Variables**

### 可选的 Variables

#### 1. WORKFLOW_USER_NAME

**用途**：用于验证 `bump-version-*` PR 的创建者（在 CI workflow 中使用）

**默认值**：`github.repository_owner`（仓库所有者，如果未设置，使用此默认值）

**配置步骤**：

1. 访问仓库：Settings → Secrets and variables → Actions
2. 点击 "New repository variable"
3. Name: `WORKFLOW_USER_NAME`
4. Value: GitHub 用户名（例如：`zevwings`）
5. 点击 "Add variable"

**说明**：
- 此变量必须与 `WORKFLOW_PAT` 的所有者匹配
- 如果 `WORKFLOW_PAT` 的所有者是 `zevwings`，则 `WORKFLOW_USER_NAME` 应该设置为 `zevwings`
- 如果未设置此变量，workflow 会使用默认值 `github.repository_owner`（仓库所有者）
- 此变量仅在 CI workflow 中用于验证 PR 创建者，不用于 Git 提交配置
- Git 提交使用 `github-actions[bot]` 作为提交者

**验证**：
- 创建 `bump-version-*` PR 后，检查 CI 验证是否通过
- 如果验证失败，检查 PR 创建者是否与 `WORKFLOW_USER_NAME` 匹配

---

## 🛡️ 分支保护规则

分支保护规则确保只有通过 CI 检查的代码才能合并到受保护的分支。

### 配置位置

**Settings → Branches → Branch protection rules**

### 必需的规则

#### 1. master 分支保护规则

**配置步骤**：

1. 访问：Settings → Branches
2. 点击 "Add rule" 或编辑现有的 master 分支规则
3. 配置以下选项：

**基本设置**：
- ✅ **Require a pull request before merging**
  - ✅ Require approvals: `1`（至少 1 个批准）
  - ✅ Dismiss stale pull request approvals when new commits are pushed
  - ✅ Require review from Code Owners（如果配置了 CODEOWNERS）

**状态检查**：
- ✅ **Require status checks to pass before merging**
  - ✅ Require branches to be up to date before merging
  - ✅ Status checks that are required:
    - ✅ `Run check-status`（必须通过）

**其他设置**：
- ✅ **Require conversation resolution before merging**
- ✅ **Do not allow bypassing the above settings**（管理员也不能绕过）
- ✅ **Restrict who can push to matching branches**（可选，限制推送权限）

**保存**：点击 "Save changes"

**验证**：
- 尝试合并一个未通过 CI 的 PR，应该被阻止
- 尝试合并一个通过 CI 的 PR，应该可以合并

---

## ⚙️ Workflow 权限配置

Workflow 权限在 workflow 文件中配置，确保 workflow 有足够的权限执行操作。

### 当前配置

#### release.yml

```yaml
permissions:
  contents: write      # 允许创建 Release 和上传文件
  pull-requests: write # 允许创建 PR（用于版本更新）
  statuses: write      # 允许创建状态检查（用于满足分支保护规则）
```

#### ci.yml

```yaml
permissions:
  contents: read       # 允许读取代码
  pull-requests: read   # 允许读取 PR 信息
```

**说明**：
- 这些权限配置已经在 workflow 文件中设置
- 通常不需要在 GitHub 设置中额外配置
- 如果遇到权限问题，检查 workflow 文件中的 `permissions` 配置

---

## ✅ 验证配置

配置完成后，需要验证以下功能是否正常工作：

### 1. Token 验证

**测试步骤**：
1. 手动触发 release workflow
2. 检查 workflow 日志，确认没有 token 相关的错误
3. 确认可以成功创建 PR

**预期结果**：
- ✅ Workflow 可以正常运行
- ✅ 可以成功创建 PR
- ✅ PR 可以触发 CI workflow

### 2. PR 验证

**测试步骤**：
1. Release workflow 创建 `bump-version-*` PR
2. 检查 CI workflow 是否被触发
3. 检查 `check-skip-ci` job 是否验证通过
4. 检查 `check-status` job 是否成功

**预期结果**：
- ✅ CI workflow 被触发
- ✅ `check-skip-ci` 验证通过（PR 创建者匹配 `WORKFLOW_USER_NAME`）
- ✅ `check-status` 成功
- ✅ PR 可以合并

### 3. Tag 和 Release 验证

**测试步骤**：
1. 合并 `bump-version-*` PR
2. 检查是否创建了 tag
3. 检查 tag push 是否触发了 build job
4. 检查是否创建了 Release

**预期结果**：
- ✅ Tag 成功创建和推送
- ✅ Build job 被触发
- ✅ Release 成功创建

### 4. Homebrew 更新验证

**测试步骤**：
1. Release 创建后，检查 `update-homebrew` job
2. 检查 homebrew-workflow 仓库是否有更新

**预期结果**：
- ✅ Homebrew formula 成功更新
- ✅ 更改推送到 homebrew-workflow 仓库

---

## 🔧 故障排除

### 问题 1：PR 无法触发 CI

**症状**：创建 PR 后，CI workflow 没有运行

**可能原因**：
- `WORKFLOW_PAT` 未配置或配置错误
- 使用了 `GITHUB_TOKEN` 而不是 `WORKFLOW_PAT` 创建 PR

**解决方案**：
1. 检查 `WORKFLOW_PAT` 是否正确配置
2. 检查 release.yml 中是否使用 `secrets.WORKFLOW_PAT`
3. 确认 token 有 `repo` 权限

### 问题 2：CI 验证失败

**症状**：`check-skip-ci` job 失败，提示 PR 创建者不匹配

**可能原因**：
- `WORKFLOW_PUI` 未配置或配置错误
- PR 创建者与 `WORKFLOW_PUI` 不匹配

**解决方案**：
1. 检查 `WORKFLOW_PUI` 是否与 `WORKFLOW_PAT` 的所有者匹配
2. 检查 PR 创建者是否是 `WORKFLOW_PUI` 指定的用户
3. 确认 `WORKFLOW_PAT` 的所有者是正确的用户

### 问题 3：无法合并 PR

**症状**：PR 无法合并，提示需要状态检查通过

**可能原因**：
- 分支保护规则要求 `check-status` 通过
- `check-status` job 失败或未运行

**解决方案**：
1. 检查分支保护规则配置
2. 检查 `check-status` job 的状态
3. 确认 `check-skip-ci` job 成功并设置了 `should_skip=true`

### 问题 4：Tag 推送未触发 Build

**症状**：Tag 推送后，build job 没有运行

**可能原因**：
- 使用了 `GITHUB_TOKEN` 而不是 `WORKFLOW_PAT` 推送 tag
- Tag 格式不正确（应该是 `v*` 格式）

**解决方案**：
1. 检查 release.yml 中 tag 推送是否使用 `WORKFLOW_PAT`
2. 检查 tag 格式是否符合 `v*` 模式
3. 确认 workflow 监听 `tags: - 'v*'` 事件

### 问题 5：Homebrew 更新失败

**症状**：`update-homebrew` job 失败

**可能原因**：
- `WORKFLOW_PAT` 没有 homebrew-workflow 仓库的写权限
- Token 配置错误

**解决方案**：
1. 确认 `WORKFLOW_PAT` 有 `repo` 权限
2. 确认 token 的所有者对 homebrew-workflow 仓库有写权限
3. 检查 release.yml 中是否使用 `secrets.WORKFLOW_PAT`

---

## 📚 相关文档

- [GitHub Actions: Using secrets in a workflow](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [GitHub Actions: Using variables in a workflow](https://docs.github.com/en/actions/learn-github-actions/variables)
- [GitHub: Managing a branch protection rule](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches)
- [GitHub: Creating a personal access token](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token)

---

## 🔄 配置检查清单

在配置完成后，使用以下清单验证所有配置：

### Repository Secrets
- [ ] `WORKFLOW_PAT` 已配置
- [ ] Token 有 `repo` 权限
- [ ] Token 有 `workflow` 权限（如果需要）
- [ ] Token 未过期

### Repository Variables
- [ ] `WORKFLOW_USER_NAME` 已配置（可选，默认使用仓库所有者）
- [ ] `WORKFLOW_USER_NAME` 与 `WORKFLOW_PAT` 的所有者匹配

### 分支保护规则
- [ ] master 分支保护规则已配置
- [ ] 要求 PR 才能合并
- [ ] 要求 `check-status` 状态检查通过
- [ ] 不允许绕过保护规则

### 功能验证
- [ ] 可以创建 PR
- [ ] PR 可以触发 CI
- [ ] CI 验证通过
- [ ] PR 可以合并
- [ ] Tag 可以创建和推送
- [ ] Tag push 触发 build
- [ ] Release 可以创建
- [ ] Homebrew 可以更新

---

## 💡 最佳实践

1. **Token 管理**：
   - 定期轮换 token（建议每 90 天）
   - 使用最小权限原则（只授予必要的权限）
   - 如果 token 泄露，立即撤销

2. **变量管理**：
   - 使用变量存储非敏感配置
   - 为变量设置合理的默认值
   - 文档化所有变量的用途

3. **分支保护**：
   - 保护所有重要分支（master、main 等）
   - 要求 CI 检查通过才能合并
   - 不允许绕过保护规则

4. **监控和日志**：
   - 定期检查 workflow 运行情况
   - 关注失败的工作流
   - 及时处理权限和配置问题

---

**最后更新**：2025-12-10
