# 工作流自动化待办事项

## 📋 概述

本文档列出工作流自动化相关的待办功能，包括模板系统和钩子系统。

---

## ❌ 待实现功能

### 1. 模板系统

#### 1.1 PR 模板
- ❌ PR 模板（根据 JIRA ticket 自动生成）

**功能**：根据 JIRA ticket 自动生成 PR 描述模板。

**实现建议**：
- 从 JIRA ticket 提取信息（summary、description、labels 等）
- 使用模板引擎（如 `handlebars`）生成 PR 描述
- 支持自定义模板（配置文件）

**配置示例**：
```toml
[pr.templates]
default = """
## Description
{{jira_summary}}

## Related Ticket
{{jira_key}}

## Changes
- [ ] Feature
- [ ] Bug fix
- [ ] Documentation
"""
```

#### 1.2 Commit 模板
- ❌ Commit 模板（标准化格式）

**功能**：标准化 commit 消息格式。

**实现建议**：
- 支持 Conventional Commits 格式
- 自动提取 JIRA ID
- 支持交互式填写

**配置示例**：
```toml
[commit.templates]
default = """
{{type}}({{scope}}): {{subject}}

{{body}}

Closes {{jira_key}}
"""
```

#### 1.3 分支命名模板
- ❌ 分支命名模板（根据 JIRA ticket 自动生成）

**功能**：根据 JIRA ticket 自动生成分支名。

**实现建议**：
- 支持模板变量（`{{jira_key}}`、`{{jira_type}}`、`{{summary}}` 等）
- 自动清理和规范化分支名

**配置示例**：
```toml
[branch.templates]
default = "{{jira_key}}-{{summary_slug}}"
feature = "feature/{{jira_key}}-{{summary_slug}}"
bugfix = "bugfix/{{jira_key}}-{{summary_slug}}"
```

---

### 2. 钩子系统

#### 2.1 Pre-commit hooks
- ❌ Pre-commit hooks（提交前检查）

**功能**：提交前检查（lint、test、JIRA 格式）。

**实现建议**：
- 使用 Git hooks（`.git/hooks/pre-commit`）
- 支持自定义检查规则
- 支持跳过检查（`--no-verify`）

**检查项**：
- Commit 消息格式检查
- JIRA ID 格式验证
- 代码 lint 检查（可选）
- 单元测试（可选）

**配置示例**：
```toml
[hooks.pre-commit]
enabled = true
checks = [
    "commit-format",
    "jira-id",
    # "lint",
    # "test",
]
```

#### 2.2 Post-merge hooks
- ❌ Post-merge hooks（合并后自动操作）

**功能**：合并后自动操作（更新 JIRA、清理分支）。

**实现建议**：
- 使用 Git hooks（`.git/hooks/post-merge`）
- 支持自定义操作脚本

**操作项**：
- 自动更新 JIRA 状态
- 清理已合并的分支
- 发送通知（可选）

**配置示例**：
```toml
[hooks.post-merge]
enabled = true
actions = [
    "update-jira-status",
    "clean-merged-branches",
]
```

#### 2.3 Pre-push hooks
- ❌ Pre-push hooks（推送前检查）

**功能**：推送前检查。

**实现建议**：
- 使用 Git hooks（`.git/hooks/pre-push`）
- 检查 PR 状态、CI 状态等

**检查项**：
- PR 状态检查
- CI 状态检查（可选）
- 分支保护规则检查（可选）

**配置示例**：
```toml
[hooks.pre-push]
enabled = true
checks = [
    "pr-status",
    # "ci-status",
]
```

---

## 📊 优先级

### 高优先级
1. **模板系统**
   - PR 模板（根据 JIRA ticket 自动生成）
   - Commit 模板（标准化格式）
   - 分支命名模板（根据 JIRA ticket 自动生成）

### 中优先级
1. **钩子系统**
   - Pre-commit hooks（提交前检查）
   - Post-merge hooks（合并后自动操作）
   - Pre-push hooks（推送前检查）

---

## 📝 实现建议

### 开发顺序
1. **第一阶段**：模板系统
   - PR 模板
   - Commit 模板
   - 分支命名模板

2. **第二阶段**：钩子系统
   - Pre-commit hooks
   - Post-merge hooks
   - Pre-push hooks

### 技术考虑
1. **模板引擎**：使用 `handlebars` 或 `tera` 作为模板引擎
2. **Git Hooks**：使用 `git2` crate 管理 Git hooks
3. **配置管理**：在配置文件中定义模板和钩子规则
4. **错误处理**：钩子失败时提供清晰的错误信息
5. **测试**：为新功能添加单元测试和集成测试
6. **文档**：及时更新文档和示例

### 实现细节

#### 模板系统实现
```rust
// 模板引擎示例
use handlebars::Handlebars;

pub struct TemplateEngine {
    handlebars: Handlebars,
}

impl TemplateEngine {
    pub fn render_pr_template(&self, jira_ticket: &JiraTicket) -> Result<String> {
        let data = json!({
            "jira_key": jira_ticket.key,
            "jira_summary": jira_ticket.summary,
            "jira_description": jira_ticket.description,
        });
        self.handlebars.render("pr_template", &data)
    }
}
```

#### 钩子系统实现
```rust
// Git hooks 管理示例
use git2::Repository;

pub struct GitHooks {
    repo: Repository,
}

impl GitHooks {
    pub fn install_pre_commit_hook(&self, script: &str) -> Result<()> {
        let hook_path = self.repo.path().join("hooks/pre-commit");
        std::fs::write(&hook_path, script)?;
        // 设置执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755))?;
        }
        Ok(())
    }
}
```

---

## 📚 相关文档

- [功能拓展分析文档](./FEATURE_EXTENSIONS.md)

---

**最后更新**: 2024-12-19
