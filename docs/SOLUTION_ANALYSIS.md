# GitHub Actions 403 错误解决方案分析

## 问题根源

工作流尝试通过 REST Statuses API 创建提交状态时收到 HTTP 403 "Resource not accessible by integration" 错误。

### 当前状态

查看 `.github/workflows/release.yml`：

1. **已有权限配置**（第16-19行）：
   ```yaml
   permissions:
     contents: write
     pull-requests: read
     statuses: write  # ✅ 已有
   ```

2. **当前实现**（第320-330行）：
   - 使用 Statuses API (`/statuses/{sha}`)
   - 使用 `Authorization: Bearer $GITHUB_TOKEN`
   - 环境变量已正确设置：`GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}`

3. **问题**：
   - 即使有 `statuses: write` 权限，仍可能遇到 403
   - 可能原因：
     - 仓库级别的 Actions 权限设置为"只读"
     - 分支保护规则要求特定的检查类型
     - GITHUB_TOKEN 在某些情况下权限受限

## 解决方案对比

### 方案1：添加 `checks: write` 权限（推荐 ⭐）

**优点**：
- ✅ 最简单快速
- ✅ 使用内置 GITHUB_TOKEN，无需额外配置
- ✅ 符合 GitHub Actions 最佳实践
- ✅ 安全性高（无需管理 PAT）

**实现**：
```yaml
permissions:
  contents: write
  statuses: write
  checks: write  # 新增：支持 Checks API
```

**适用场景**：
- 仓库 Actions 权限允许"读写"
- 分支保护规则接受 Actions 创建的检查

**注意事项**：
- 如果仓库级别设置为"只读"，需要在工作流中显式声明权限（已做）
- 某些分支保护规则可能要求检查由特定应用创建

---

### 方案2：使用 Checks API（更现代 ⭐⭐）

**优点**：
- ✅ 更现代的 API，GitHub 推荐用于 Actions
- ✅ 在 PR 和分支保护中显示更清晰
- ✅ 功能更强大（支持注释、输出等）
- ✅ 与 GitHub Actions 集成更好

**实现**：
```bash
curl -s -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "https://api.github.com/repos/${{ github.repository }}/check-runs" \
  -d '{
    "name": "Run check-status",
    "head_sha": "'"$COMMIT_SHA"'",
    "status": "completed",
    "conclusion": "success",
    "output": {
      "title": "check-status",
      "summary": "All checks passed (check-lint, tests, check-status)"
    }
  }'
```

**需要权限**：
```yaml
permissions:
  checks: write  # 必需
```

**注意事项**：
- 分支保护规则需要配置为接受 Check Run（而不是 Status）
- 检查名称必须与分支保护规则中配置的名称完全匹配

---

### 方案3：使用 Personal Access Token (PAT)（备选）

**优点**：
- ✅ 可以绕过某些分支保护限制
- ✅ 权限更灵活

**缺点**：
- ❌ 需要手动创建和管理 PAT
- ❌ 安全性较低（需要妥善保管 secret）
- ❌ 不符合 GitHub Actions 最佳实践
- ❌ PAT 可能过期或被撤销

**实现**：
```yaml
env:
  REPO_PAT: ${{ secrets.REPO_PAT }}
```

```bash
# API 调用
-H "Authorization: token $REPO_PAT"

# Git push
git remote set-url origin https://x-access-token:$REPO_PAT@github.com/zevwings/workflow.rs.git
git push origin master
```

**适用场景**：
- 无法修改工作流权限
- 分支保护规则要求特定应用创建检查
- 需要绕过分支保护规则

---

### 方案4：创建 PR 而不是直接推送（最安全 ⭐⭐⭐）

**优点**：
- ✅ 最安全，符合 Git 工作流最佳实践
- ✅ 不需要绕过分支保护
- ✅ 可以代码审查
- ✅ 使用 GITHUB_TOKEN 即可（推送分支不需要特殊权限）

**实现**：
```bash
# 创建分支并推送
git checkout -b bump-version-${NEW_VERSION}
git commit -m "chore: bump version to ${NEW_VERSION}"
git push origin bump-version-${NEW_VERSION}

# 创建 PR
curl -s -X POST \
  -H "Authorization: token $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github+json" \
  "https://api.github.com/repos/${{ github.repository }}/pulls" \
  -d '{
    "title": "chore: bump version to '"$NEW_VERSION"'",
    "head": "bump-version-'"$NEW_VERSION"'",
    "base": "master",
    "body": "Automated version bump"
  }'
```

**适用场景**：
- 希望保持代码审查流程
- 分支保护规则严格
- 自动化程度要求不高（需要手动合并 PR）

---

## 推荐实施路径

### 快速修复（立即尝试）

1. **添加 `checks: write` 权限**
   ```yaml
   permissions:
     contents: write
     statuses: write
     checks: write  # 新增
   ```

2. **验证当前实现**
   - 确保使用 `Bearer $GITHUB_TOKEN`（已正确）
   - 确保环境变量已设置（已正确）

3. **如果仍然 403**：
   - 检查仓库设置：Settings → Actions → General → Workflow permissions
   - 确保设置为"Read and write"或"Allow GitHub Actions to create and approve pull requests"

### 长期优化（推荐）

**迁移到 Checks API**：
- 更符合 GitHub Actions 设计
- 更好的用户体验
- 更强大的功能

**步骤**：
1. 添加 `checks: write` 权限
2. 修改代码使用 Checks API
3. 更新分支保护规则以接受 Check Run

---

## 当前代码问题分析

### 已正确实现的部分

1. ✅ 权限声明：已有 `statuses: write`
2. ✅ Token 使用：正确使用 `$GITHUB_TOKEN`
3. ✅ 环境变量：已正确设置
4. ✅ 错误处理：有适当的错误检查

### 可能的问题

1. ⚠️ **缺少 `checks: write`**：如果将来迁移到 Checks API 需要此权限
2. ⚠️ **Authorization 格式**：当前使用 `Bearer`，Statuses API 也接受 `token` 前缀
3. ⚠️ **分支保护配置**：可能要求检查由特定应用创建

### 建议的修改

1. **立即添加 `checks: write` 权限**（为未来迁移做准备）
2. **考虑迁移到 Checks API**（更现代、更可靠）
3. **如果必须使用 PAT**，确保安全存储和轮换

---

## 总结

**最佳实践推荐**：
1. 短期：添加 `checks: write` 权限，保持当前 Statuses API 实现
2. 中期：迁移到 Checks API
3. 长期：考虑使用 PR 流程（如果需要代码审查）

**安全性排序**：
1. PR 流程（最安全）
2. Checks API + GITHUB_TOKEN
3. Statuses API + GITHUB_TOKEN
4. PAT（最不安全，但有时必需）

**实施难度排序**：
1. 添加权限（最简单）
2. 迁移到 Checks API（中等）
3. 使用 PAT（需要额外配置）
4. PR 流程（需要修改工作流逻辑）
