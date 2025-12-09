# PR 创建 403 错误分析

## 错误信息

```
⚠️  Warning: Failed to create Pull Request (HTTP 403)
Response: {
  "message": "Resource not accessible by integration",
  "documentation_url": "https://docs.github.com/rest/pulls/pulls#create-a-pull-request",
  "status": "403"
}
```

## 问题分析

### 当前配置

查看 `.github/workflows/release.yml`：

1. **权限配置**（第16-19行）：
   ```yaml
   permissions:
     contents: write
     pull-requests: write  # ✅ 已配置
     statuses: write
   ```

2. **API 调用**（第331-341行）：
   ```bash
   curl -X POST \
     -H "Authorization: Bearer $GITHUB_TOKEN" \
     -H "Accept: application/vnd.github+json" \
     "https://api.github.com/repos/${{ github.repository }}/pulls"
   ```

### 可能的原因

#### 1. 仓库级别的 Actions 权限设置为"只读" ⚠️ **最可能**

即使工作流文件中声明了 `pull-requests: write`，如果仓库级别的 Actions 权限设置为"只读"，`GITHUB_TOKEN` 仍然无法创建 PR。

**检查方法**：
- 进入仓库：Settings → Actions → General → Workflow permissions
- 查看是否设置为 "Read and write permissions" 或 "Read repository contents and packages permissions"

**解决方案**：
- 将权限改为 "Read and write permissions"
- 或者勾选 "Allow GitHub Actions to create and approve pull requests"

#### 2. 分支保护规则限制

某些分支保护规则可能阻止通过 API 创建 PR，特别是如果：
- 分支保护规则要求特定的检查
- 分支保护规则要求代码审查
- 分支保护规则限制了 PR 创建者

**检查方法**：
- 进入仓库：Settings → Branches → Branch protection rules
- 检查 `master` 分支的保护规则

**解决方案**：
- 确保分支保护规则允许从 `bump-version-*` 分支创建 PR
- 或者临时放宽保护规则（不推荐）

#### 3. GITHUB_TOKEN 权限受限

在某些情况下，即使声明了权限，`GITHUB_TOKEN` 的权限可能仍然受限。

**解决方案**：
- 确保工作流文件中的 `permissions` 块正确配置
- 检查是否有其他工作流或设置覆盖了权限

## 解决方案

### 方案1：检查并修复仓库 Actions 权限（推荐 ⭐⭐⭐）

1. **进入仓库设置**：
   - 打开仓库：`https://github.com/zevwings/workflow.rs`
   - 点击 Settings → Actions → General

2. **检查 Workflow permissions**：
   - 找到 "Workflow permissions" 部分
   - 确保选择 "Read and write permissions"
   - 或者勾选 "Allow GitHub Actions to create and approve pull requests"

3. **保存更改**

### 方案2：添加更明确的权限声明

在工作流文件中，确保权限声明完整：

```yaml
permissions:
  contents: write
  pull-requests: write  # 创建和更新 PR
  issues: write          # 如果需要创建 issue（PR 也是 issue）
  statuses: write        # 创建状态检查
  checks: write          # 创建检查运行（可选，用于更高级的检查）
```

### 方案3：使用 Personal Access Token (PAT)（备选）

如果 `GITHUB_TOKEN` 仍然无法工作，可以使用 PAT：

1. **创建 PAT**：
   - 进入 GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
   - 创建新 token，勾选 `repo` 权限

2. **添加 Secret**：
   - 进入仓库：Settings → Secrets and variables → Actions
   - 添加 secret：`REPO_PAT`

3. **修改工作流**：
   ```yaml
   env:
     GITHUB_TOKEN: ${{ secrets.REPO_PAT }}
   ```

**注意**：PAT 需要妥善保管，定期轮换。

### 方案4：检查分支保护规则

1. **进入分支保护设置**：
   - Settings → Branches → Branch protection rules
   - 检查 `master` 分支的规则

2. **确保允许创建 PR**：
   - 检查是否有 "Restrict who can push to matching branches" 限制
   - 确保允许从 `bump-version-*` 分支创建 PR

## 验证步骤

修复后，验证步骤：

1. **触发工作流**：
   - 手动触发 release 工作流，或等待自动触发

2. **检查日志**：
   - 查看工作流运行日志
   - 确认 PR 创建成功（HTTP 201）

3. **检查 PR**：
   - 在 GitHub 上查看是否成功创建 PR
   - 确认 PR 标题和内容正确

## 当前状态

根据错误信息，最可能的原因是：

1. ✅ **仓库 Actions 权限设置为"只读"** - 需要改为"读写"
2. ⚠️ **分支保护规则限制** - 需要检查并调整
3. ⚠️ **GITHUB_TOKEN 权限受限** - 需要明确声明权限

## 推荐操作

**立即执行**：

1. 检查仓库 Settings → Actions → General → Workflow permissions
2. 确保设置为 "Read and write permissions"
3. 勾选 "Allow GitHub Actions to create and approve pull requests"
4. 重新运行工作流验证

**如果仍然失败**：

1. 检查分支保护规则
2. 考虑使用 PAT（临时方案）
3. 查看 GitHub 文档：https://docs.github.com/rest/pulls/pulls#create-a-pull-request
