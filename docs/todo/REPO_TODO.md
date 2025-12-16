# Repo 配置模块待办事项

## 📋 概述

本文档列出 Repo 配置模块的待办功能，主要涉及配置层级优化和全局配置支持。

---

## ❌ 待实现功能

---

### 1. 分支前缀全局配置支持

#### 1.1 问题描述

**当前状态**：
- `[branch] prefix` 只支持仓库级配置（`.workflow/config.toml`）
- 个人偏好的前缀（如 `"zw"`）被提交到 Git，影响所有开发者
- 不符合"个人偏好应放在全局配置"的设计原则

**问题场景**：
```toml
# .workflow/config.toml (会被提交到 Git)
[branch]
prefix = "zw"  # 这是个人偏好，不应该强制所有开发者使用
```

#### 1.2 实现目标

- ✅ 支持全局配置中的分支前缀（`~/.workflow/config/workflow.toml`）
- ✅ 实现配置优先级：**仓库配置 > 全局配置 > 默认值**
- ✅ 保持向后兼容（现有仓库配置继续有效）

#### 1.3 实现步骤

1. **修改 `Settings` 结构体** (`src/lib/base/settings/settings.rs`)
   - 在 `Settings` 中添加 `branch_prefix: Option<String>` 字段
   - 在 TOML 解析中添加 `[branch] prefix` 支持

2. **修改 `RepoConfig::get_branch_prefix()`** (`src/lib/repo/config.rs`)
   - 优先读取仓库配置
   - 如果仓库配置不存在，读取全局配置
   - 如果都不存在，返回 `None`

3. **更新配置命令** (`src/commands/config/setup.rs`)
   - 在全局配置设置中添加分支前缀配置选项

4. **更新文档**
   - 更新 `docs/architecture/lib/REPO_ARCHITECTURE.md`
   - 更新 `docs/architecture/lib/SETTINGS_ARCHITECTURE.md`
   - 添加配置优先级说明

#### 1.4 配置示例

**全局配置** (`~/.workflow/config/workflow.toml`)：
```toml
[branch]
prefix = "zw"  # 个人默认前缀
```

**仓库配置** (`.workflow/config.toml`)：
```toml
[branch]
prefix = "feature"  # 项目统一前缀（覆盖全局配置）
```

**优先级**：
- 如果仓库配置存在 `prefix = "feature"`，使用 `"feature"`
- 如果仓库配置不存在，使用全局配置 `"zw"`
- 如果都不存在，返回 `None`

---

### 2. PR 自动接受变更类型全局配置支持

#### 2.1 问题描述

**当前状态**：
- `[pr] auto_accept_change_type` 只支持仓库级配置
- 个人工作习惯被提交到 Git，影响所有开发者
- 不符合"个人偏好应放在全局配置"的设计原则

**问题场景**：
```toml
# .workflow/config.toml (会被提交到 Git)
[pr]
auto_accept_change_type = true  # 这是个人偏好，不应该强制所有开发者使用
```

#### 2.2 实现目标

- ✅ 支持全局配置中的自动接受变更类型（`~/.workflow/config/workflow.toml`）
- ✅ 实现配置优先级：**仓库配置 > 全局配置 > 默认值（false）**
- ✅ 保持向后兼容（现有仓库配置继续有效）

#### 2.3 实现步骤

1. **修改 `Settings` 结构体** (`src/lib/base/settings/settings.rs`)
   - 在 `Settings` 中添加 `pr_auto_accept_change_type: Option<bool>` 字段
   - 在 TOML 解析中添加 `[pr] auto_accept_change_type` 支持

2. **修改 PR 创建逻辑** (`src/commands/pr/create.rs`)
   - 优先读取仓库配置（`RepoConfig::load()`）
   - 如果仓库配置不存在，读取全局配置（`Settings::get()`）
   - 如果都不存在，使用默认值 `false`

3. **更新配置命令** (`src/commands/config/setup.rs`)
   - 在全局配置设置中添加 PR 自动接受变更类型配置选项

4. **更新文档**
   - 更新 `docs/architecture/lib/REPO_ARCHITECTURE.md`
   - 更新 `docs/architecture/lib/SETTINGS_ARCHITECTURE.md`
   - 添加配置优先级说明

#### 2.4 配置示例

**全局配置** (`~/.workflow/config/workflow.toml`)：
```toml
[pr]
auto_accept_change_type = true  # 个人默认行为
```

**仓库配置** (`.workflow/config.toml`)：
```toml
[pr]
auto_accept_change_type = false  # 项目统一行为（覆盖全局配置）
```

**优先级**：
- 如果仓库配置存在 `auto_accept_change_type = false`，使用 `false`
- 如果仓库配置不存在，使用全局配置 `true`
- 如果都不存在，使用默认值 `false`

---

### 3. 配置优先级文档完善

#### 3.1 问题描述

**当前状态**：
- 配置优先级规则不够清晰
- 用户不清楚哪些配置应该放在全局，哪些应该放在仓库
- 缺少配置选择指南

#### 3.2 实现目标

- ✅ 明确配置分类原则
- ✅ 完善配置优先级文档
- ✅ 提供配置选择指南

#### 3.3 配置分类原则

**全局配置**（个人偏好，不应提交到 Git）：
- `[branch] prefix` - 个人默认分支前缀
- `[pr] auto_accept_change_type` - 个人工作习惯
- 其他个人偏好设置

**仓库配置**（项目规范，应提交到 Git）：
- `[template.commit] use_scope` - 项目提交规范
- `[template.commit] default` - 项目提交模板
- `[template.branch]` - 项目分支命名规范
- `[branch] ignore` - 项目保护分支列表
- 其他项目统一规范

#### 3.4 实现步骤

1. **更新架构文档**
   - `docs/architecture/lib/REPO_ARCHITECTURE.md` - 添加配置优先级说明
   - `docs/architecture/lib/SETTINGS_ARCHITECTURE.md` - 添加全局配置说明

2. **创建配置指南**
   - `docs/guidelines/CONFIG_GUIDELINES.md` - 配置选择和使用指南

3. **更新 README**
   - 在项目 README 中添加配置说明链接

---

## ✅ 已完成功能

---

### 1. 仓库配置基础功能

- ✅ 仓库配置加载和保存 (`RepoConfig::load()`, `RepoConfig::save()`)
- ✅ 分支前缀仓库配置支持 (`RepoConfig::get_branch_prefix()`)
- ✅ 忽略分支列表配置 (`RepoConfig::get_ignore_branches()`)
- ✅ 模板配置支持（commit、branch、pull_requests）
- ✅ PR 自动接受变更类型仓库配置支持

### 2. 模板配置优先级

- ✅ 模板配置支持全局和仓库两级配置
- ✅ 仓库配置优先于全局配置 (`TemplateConfig::load()`)
- ✅ 配置合并逻辑完善

---

## 📊 实现进度

| 功能模块 | 状态 | 完成度 | 优先级 |
|---------|------|--------|--------|
| 分支前缀全局配置 | ⏳ 待实施 | 0% | 高 |
| PR 自动接受全局配置 | ⏳ 待实施 | 0% | 高 |
| 配置优先级文档 | ⏳ 待实施 | 0% | 中 |

---

## 🔗 相关文档

- [Repo 模块架构文档](../architecture/lib/REPO_ARCHITECTURE.md)
- [Settings 模块架构文档](../architecture/lib/SETTINGS_ARCHITECTURE.md)
- [Template 模块架构文档](../architecture/lib/TEMPLATE_ARCHITECTURE.md)
- [配置合理性分析](../requirements/REPO_CONFIG_ANALYSIS.md)（待创建）

---

## 📝 更新记录

### 2025-01-27
- ✅ 创建 REPO_TODO 文档
- ✅ 记录分支前缀和 PR 自动接受变更类型的全局配置支持需求
- ✅ 记录配置优先级文档完善需求

---

**最后更新**: 2025-01-27
**文档维护**: 定期审查，实施完成后及时更新状态
