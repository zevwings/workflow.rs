# 工作流自动化待办事项

## 📋 概述

本文档列出工作流自动化相关的待办功能，包括钩子系统。

---

## ❌ 待实现功能

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

### 3. 批量操作

#### 3.1 `batch update-jira` - 批量更新 JIRA
- ❌ 批量更新多个 JIRA tickets

**功能**：批量更新多个 JIRA tickets。

**命令示例**：
```bash
workflow batch update-jira --file tickets.txt --status "Done"  # 从文件读取
workflow batch update-jira "PROJ-123,PROJ-124" --status "Done"  # 从参数读取
```

**实现建议**：
- 支持从文件读取 ticket 列表
- 支持并行处理以提高效率
- 提供进度显示和错误处理

#### 3.2 `batch create-pr` - 批量创建 PR
- ❌ 批量创建 PR（从多个分支）

**功能**：批量创建 PR（从多个分支）。

**命令示例**：
```bash
workflow batch create-pr --file branches.txt       # 从文件读取分支列表
```

**实现建议**：
- 支持从文件读取分支列表
- 支持并行创建 PR
- 提供进度显示和错误处理

#### 3.3 `batch merge` - 批量合并 PR
- ❌ 批量合并 PR

**功能**：批量合并 PR。

**命令示例**：
```bash
workflow batch merge --file prs.txt                # 从文件读取 PR 列表
workflow batch merge --status "approved"            # 合并所有已批准的 PR
```

**实现建议**：
- 支持从文件读取 PR 列表
- 支持按状态过滤（如合并所有已批准的 PR）
- 支持并行合并（如果安全）
- 提供进度显示和错误处理

---

## 📊 优先级

### 高优先级
（暂无）

### 中优先级
1. **钩子系统**
   - Pre-commit hooks（提交前检查）
   - Post-merge hooks（合并后自动操作）
   - Pre-push hooks（推送前检查）

2. **批量操作**
   - `batch update-jira` - 批量更新 JIRA
   - `batch create-pr` - 批量创建 PR
   - `batch merge` - 批量合并 PR

---

## 📝 实现建议

### 开发顺序
1. **第一阶段**：钩子系统
   - Pre-commit hooks
   - Post-merge hooks
   - Pre-push hooks

2. **第二阶段**：批量操作
   - `batch update-jira` - 批量更新 JIRA
   - `batch create-pr` - 批量创建 PR
   - `batch merge` - 批量合并 PR

### 技术考虑
1. **Git Hooks**：使用 `git2` crate 管理 Git hooks
2. **配置管理**：在配置文件中定义钩子规则
3. **错误处理**：钩子失败时提供清晰的错误信息
4. **测试**：为新功能添加单元测试和集成测试
5. **文档**：及时更新文档和示例

### 实现细节

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

- [模板系统需求文档](../requirements/TEMPLATE_SYSTEM.md) - 已转换为需求文档
- [JIRA 模块待办事项](./JIRA_TODO.md)
- [Git 工作流待办事项](./GIT_TODO.md)
- [配置管理待办事项](./CONFIG_TODO.md)

---

**最后更新**: 2025-12-09
