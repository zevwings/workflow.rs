# 分支保护规则配置指南

## 当前实现方式

**工作流现在使用 Pull Request 方式更新版本**（推荐方案）：
- ✅ 工作流会自动创建分支并推送版本更新
- ✅ 自动创建 Pull Request
- ✅ 不需要绕过分支保护规则
- ✅ 更安全，符合 Git 工作流最佳实践
- ✅ 可以代码审查后再合并

**工作流程**：
1. 工作流检测到需要更新版本
2. 创建新分支 `bump-version-{version}`
3. 提交版本更新（Cargo.toml 和 Cargo.lock）
4. 推送到新分支
5. 自动创建 Pull Request
6. 等待手动合并（或配置自动合并）

**优点**：
- 不需要配置分支保护规则的特殊权限
- 不需要使用 PAT
- 可以审查更改后再合并
- 更符合开源项目的最佳实践

---

## 历史问题说明（已解决）

~~当 GitHub Actions 工作流尝试推送版本更新到受保护的 `master` 分支时，会遇到以下错误：~~

~~```
remote: error: GH006: Protected branch update failed for refs/heads/master.
remote: - Required status check "Run check-status" is expected.
```~~

**此问题已通过改用 PR 方式解决。** 以下方案仅作为历史参考。

---

## 历史解决方案（仅供参考）

### 方案 1：允许 GitHub Actions 绕过状态检查（推荐）

这是最安全和推荐的方法，因为只有通过所有检查的工作流才能推送。

**配置步骤：**

1. 进入仓库设置：
   - 访问：`Settings` → `Branches` → `Branch protection rules`
   - 找到 `master` 分支的保护规则（点击编辑）

2. **方法 A：允许 GitHub Actions 推送（推荐）**

   **在传统分支保护规则中（Settings → Branches → Branch protection rules）：**

   找到 **"Restrict who can push to matching branches"** 选项：
   - 勾选此选项
   - 在下方出现的输入框中，输入 `github-actions[bot]` 并添加
   - 这样 GitHub Actions 就可以直接推送到受保护的分支

   **说明：**
   - "Restrict who can push to matching branches" = 限制谁可以推送到匹配的分支（这是我们需要的）
   - "Restrict pushes that create matching branches" = 限制创建匹配分支的推送（这是另一个独立选项，用于限制创建新分支，不是子选项，也不是我们需要的）

   **如果使用 Rulesets（Settings → Rules → Rulesets）：**
   - 找到或创建针对 `master` 分支的 ruleset
   - 在 ruleset 配置中，查找以下任一选项：
     - "Push restrictions" 或 "Restrict pushes"
     - "Bypass list" 或 "Allow bypass"
     - "Actor restrictions"
   - 添加 `github-actions[bot]` 到允许列表
   - 注意：Rulesets 的界面可能因仓库类型而异

   **如果仍然找不到这些选项：**
   - 可能你的仓库使用的是企业版 GitHub 或特殊配置
   - 建议直接使用**方案 2（PAT）**，这是最通用的解决方案

3. **方法 B：允许绕过状态检查（不推荐，可能无效）**

   在 "Require status checks to pass before merging" 部分：
   - 找到 "Do not allow bypassing the above settings" 选项
   - **取消勾选**此选项（不勾选 = 允许某些角色绕过）
   - ⚠️ **注意**：即使取消勾选，GitHub Actions bot 默认可能也没有绕过权限
   - ⚠️ 这个方法可能不够，建议使用方法 A

   **关于 "Do not allow bypassing the above settings"：**
   - **勾选** = 不允许任何人绕过状态检查（包括 GitHub Actions）→ 会导致推送失败
   - **不勾选** = 允许某些角色绕过，但 GitHub Actions bot 可能不在允许列表中

**推荐使用方法 A**，因为：
- ✅ 更明确：直接允许 GitHub Actions 推送
- ✅ 更安全：只允许 GitHub Actions，不影响其他设置
- ✅ 更简单：不需要修改其他保护规则

**优点：**
- ✅ 更安全：只有通过所有检查的工作流才能推送
- ✅ 不需要 PAT
- ✅ 符合 GitHub 最佳实践

### 方案 2：使用 Personal Access Token (PAT)

如果无法修改分支保护规则，可以使用 PAT。

**配置步骤：**

1. **创建 PAT**：
   - 访问：https://github.com/settings/tokens
   - 点击 "Generate new token" → "Generate new token (classic)"
   - 配置：
     - Note: "Workflow Push Token for workflow.rs"
     - Expiration: 根据需要选择（建议至少 1 年）
     - Scopes: 勾选 `repo`（Full control of private repositories）
   - 点击 "Generate token"
   - **复制 token**（只显示一次）

2. **添加到仓库 Secrets**：
   - 进入：`Settings` → `Secrets and variables` → `Actions`
   - 点击 "New repository secret"
   - Name: `WORKFLOW_PUSH_TOKEN`
   - Secret: 粘贴 token
   - 点击 "Add secret"

3. **修改工作流使用 PAT**：
   在 `.github/workflows/release.yml` 中，修改 `Commit version update` 步骤：

   ```yaml
   - name: Commit version update
     if: steps.check_tag.outputs.needs_increment == 'true'
     shell: bash
     env:
       GITHUB_TOKEN: ${{ secrets.WORKFLOW_PUSH_TOKEN }}  # 使用 PAT
     run: |
       # ... 其余代码保持不变
   ```

**注意：**
- ⚠️ PAT 有更高的权限，需要妥善保管
- ⚠️ PAT 过期后需要更新
- ⚠️ 如果 PAT 泄露，需要立即撤销

## 推荐方案

**如果找不到 "Restrict who can push to matching branches" 选项：**

**直接使用方案 2（PAT）**，这是最可靠和通用的解决方案：
- ✅ 适用于所有仓库类型（包括 Rulesets）
- ✅ 不依赖分支保护规则的具体选项
- ✅ 100% 可靠
- ⚠️ 需要管理 token（但工作流已支持自动使用）

**如果找到了 "Restrict who can push to matching branches" 选项：**

**推荐使用方案 1**，因为：
1. 更安全：不需要存储高权限的 PAT
2. 更简单：不需要管理 token 的过期和更新
3. 符合最佳实践：GitHub Actions 应该能够管理自己的提交

## 快速参考

### 选项说明

- **"Restrict who can push to matching branches"**
  - 作用：限制谁可以推送到匹配的分支
  - 位置：传统分支保护规则中
  - 这是我们需要的选项

- **"Restrict pushes that create matching branches"**
  - 作用：限制创建匹配分支的推送（创建新分支）
  - 位置：传统分支保护规则中
  - 这是另一个独立选项，不是子选项，也不是我们需要的

### 如果找不到选项

如果在 Rulesets 中找不到相关选项，建议：
1. 直接使用**方案 2（PAT）**，这是最通用和可靠的解决方案
2. 或者联系 GitHub 支持，了解你的仓库类型的具体配置方法

## 当前工作流程

### 版本更新流程（使用 PR 方式）

1. **触发条件**：当有代码推送到 `master` 分支时
2. **检查版本**：`create-tag` job 检查当前版本是否需要创建新 tag
3. **如果需要递增版本**：
   - 创建新分支 `bump-version-{version}`
   - 更新 `Cargo.toml` 和 `Cargo.lock` 中的版本号
   - 提交并推送到新分支
   - 自动创建 Pull Request
4. **合并 PR**：手动或自动合并 PR 到 `master`
5. **创建 Tag**：PR 合并后触发新的工作流运行，创建并推送 tag
6. **构建和发布**：tag 创建后触发构建和发布流程

### 优势

- ✅ **安全性**：不需要绕过分支保护规则
- ✅ **可审查**：版本更新可以通过 PR 进行审查
- ✅ **自动化**：PR 创建完全自动化
- ✅ **灵活性**：可以选择何时合并 PR

### 注意事项

- PR 需要手动合并（或配置自动合并规则）
- 确保分支保护规则允许从 `bump-version-*` 分支创建 PR
- PR 合并后会自动触发后续的 tag 创建和发布流程

---

## 验证配置

**使用 PR 方式时**：
- 工作流会自动创建 PR，无需额外配置
- 只需确保有权限创建 PR（工作流已配置 `pull-requests: write`）
- 合并 PR 后会自动触发后续流程

**历史方案验证**（已弃用）：
~~配置完成后，运行工作流应该能够成功推送版本更新到 `master` 分支。~~
