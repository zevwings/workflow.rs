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
- **每个仓库的分支前缀可能不同**，需要灵活支持

**问题场景**：
```toml
# .workflow/config.toml (会被提交到 Git)
[branch]
prefix = "zw"  # 这是个人偏好，不应该强制所有开发者使用
```

**实际需求**：
- 项目 A 可能需要 `prefix = "feature"`
- 项目 B 可能需要 `prefix = "fix"`
- 个人项目可能需要 `prefix = "zw"`（不应提交到 Git）
- 每个仓库应该能够独立配置自己的前缀

**设计原则**：
- ✅ **仓库配置优先**：每个仓库可以有不同的前缀，仓库配置优先
- ✅ **全局配置作为回退**：全局配置主要用于个人项目或未配置仓库的回退值
- ✅ **避免个人偏好污染团队配置**：个人偏好的前缀不应提交到团队项目的 Git 仓库

#### 1.2 实现目标

- ✅ 支持全局配置中的分支前缀（`~/.workflow/config/workflow.toml`）
- ✅ 实现配置优先级：**仓库配置 > 全局配置 > 默认值**
- ✅ 保持向后兼容（现有仓库配置继续有效）
- ✅ **重要**：每个仓库的分支前缀可能不同，仓库配置优先；全局配置仅作为未配置仓库的回退值

#### 1.3 实现步骤

1. **修改 `Settings` 结构体** (`src/lib/base/settings/settings.rs`)
   - 在 `Settings` 中添加 `branch_prefix: Option<String>` 字段
   - 在 TOML 解析中添加 `[branch] prefix` 支持

2. **修改 `RepoConfig::get_branch_prefix()`** (`src/lib/repo/config.rs`)
   - **优先读取仓库配置**（每个仓库可能有不同的前缀）
   - 如果仓库配置不存在，读取全局配置（作为回退值，主要用于个人项目）
   - 如果都不存在，返回 `None`

3. **更新配置命令** (`src/commands/config/setup.rs`)
   - 在全局配置设置中添加分支前缀配置选项

4. **更新文档**
   - 更新 `docs/architecture/lib/REPO_ARCHITECTURE.md`
   - 更新 `docs/architecture/lib/SETTINGS_ARCHITECTURE.md`
   - 添加配置优先级说明

#### 1.4 配置示例

**使用场景说明**：
- **每个仓库的分支前缀可能不同**，因此仓库配置优先
- 全局配置主要用于：
  - 个人项目（不需要提交到 Git 的配置）
  - 未配置仓库的回退值
  - 个人默认偏好

**全局配置** (`~/.workflow/config/workflow.toml`)：
```toml
[branch]
prefix = "zw"  # 个人默认前缀（用于个人项目或未配置仓库的回退值）
```

**仓库配置示例 1** - 项目 A (`.workflow/config.toml`)：
```toml
[branch]
prefix = "feature"  # 项目 A 使用 "feature" 前缀
```

**仓库配置示例 2** - 项目 B (`.workflow/config.toml`)：
```toml
[branch]
prefix = "fix"  # 项目 B 使用 "fix" 前缀
```

**仓库配置示例 3** - 个人项目 (无仓库配置)：
- 使用全局配置 `"zw"` 作为回退值

**优先级**：
- ✅ **仓库配置优先**：如果仓库配置存在 `prefix = "feature"`，使用 `"feature"`（每个仓库可以不同）
- ✅ **全局配置作为回退**：如果仓库配置不存在，使用全局配置 `"zw"`（主要用于个人项目）
- ✅ **默认值**：如果都不存在，返回 `None`

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

**前置分析**：
- 已创建 [仓库配置组织方案分析](../requirements/REPO_CONFIG_ORGANIZATION_ANALYSIS.md)
- **推荐方案 3**：全局仓库特定配置（基于仓库标识符）
- 需要根据选定的方案完善文档

#### 3.2 实现目标

- ✅ 明确配置分类原则
- ✅ 完善配置优先级文档
- ✅ 提供配置选择指南

#### 3.3 配置分类原则

**全局配置**（个人偏好，不应提交到 Git）：
- `[branch] prefix` - 个人默认分支前缀（用于个人项目或未配置仓库的回退值）
  - ⚠️ **注意**：每个仓库的分支前缀可能不同，仓库配置优先
- `[pr] auto_accept_change_type` - 个人工作习惯（用于未配置仓库的回退值）
- 其他个人偏好设置

**仓库配置**（项目规范，应提交到 Git）：
- `[branch] prefix` - **项目特定分支前缀**（每个仓库可能不同，优先于全局配置）
- `[template.commit] use_scope` - 项目提交规范
- `[template.commit] default` - 项目提交模板
- `[template.branch]` - 项目分支命名规范
- `[branch] ignore` - 项目保护分支列表
- `[pr] auto_accept_change_type` - 项目统一行为（可选，覆盖全局配置）
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
- [仓库配置组织方案分析](../requirements/REPO_CONFIG_ORGANIZATION_ANALYSIS.md) - **配置文件组织方式对比分析**
- [仓库配置管理器架构设计分析](../requirements/REPO_CONFIG_ARCHITECTURE_DESIGN.md) - **配置管理器架构设计对比（拆分 vs 单一）**
- [仓库配置全局化实现文档](../requirements/REPO_CONFIG_IMPLEMENTATION.md) - **详细实现步骤和代码示例**

---

## 📝 更新记录

### 2025-01-27
- ✅ 创建 REPO_TODO 文档
- ✅ 记录分支前缀和 PR 自动接受变更类型的全局配置支持需求
- ✅ 记录配置优先级文档完善需求
- ✅ 创建仓库配置组织方案分析文档（方案 1 vs 方案 2 vs 方案 3 对比）
- ✅ 明确每个仓库的分支前缀可能不同，需要灵活支持
- ✅ **推荐方案 3**：全局仓库特定配置（`~/.workflow/config/repository.toml`，基于仓库标识符）
- ✅ 创建实现文档，包含详细实现步骤、代码示例和测试计划
- ✅ 确定使用 `repository.toml` 作为配置文件名，支持 iCloud 同步
- ✅ **推荐架构设计**：拆分为 `PublicRepoConfig` 和 `PrivateRepoConfig`，`RepoConfig` 提供统一接口

---

**最后更新**: 2025-01-27
**文档维护**: 定期审查，实施完成后及时更新状态
