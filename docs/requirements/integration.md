# 集成与扩展待办事项

> 本文档列出集成与扩展相关的待办功能，包括更多平台支持和通知系统。

---

## 📋 目录

- [概述](#-概述)
- [待实现功能](#-待实现功能)
- [优先级](#-优先级)
- [实现建议](#-实现建议)
- [相关文档](#-相关文档)

---

## 📋 概述

本文档列出集成与扩展相关的待办功能，包括更多平台支持和通知系统。

### 当前状态

- **状态**: ⏳ 待实施
- **实现度**: 10%
- **优先级**: 中
- **分类**: 集成与扩展模块

### 目标

扩展平台支持和通知系统，提供：
- 更多 Git 平台支持（GitLab、Bitbucket、Azure DevOps、Codeup）
- 完整的通知系统（桌面通知、邮件通知、Webhook 集成）
- 提升跨平台兼容性和用户体验

### 已完成

- ✅ GitHub PR 支持
- ✅ Codeup PR 支持（基础实现）

### 待实现

- ⏳ 更多平台支持（GitLab、Bitbucket、Azure DevOps）
- ⏳ 通知系统（桌面通知、邮件通知、Webhook 集成）

---

## ❌ 待实现功能

### 1. 更多平台支持

#### 1.1 GitLab 支持
- ❌ GitLab PR/MR 支持

**功能**：支持 GitLab PR/MR。

**实现建议**：
- 实现 `PlatformProvider` trait for GitLab
- 使用 GitLab API
- 参考 GitHub 的实现

**需要实现的功能**：
- PR 创建、合并、关闭
- PR 状态查询
- PR 列表查询
- PR 更新
- PR 评论、批准
- PR 同步（merge/rebase）

**GitLab API 端点**：
- `/projects/{id}/merge-_requests` - 创建/列出 MR
- `/projects/{id}/merge-_requests/{iid}` - 获取/更新 MR
- `/projects/{id}/merge-_requests/{iid}/merge` - 合并 MR
- `/projects/{id}/merge-_requests/{iid}/approvals` - 批准 MR

#### 1.2 Bitbucket 支持
- ❌ Bitbucket PR 支持

**功能**：支持 Bitbucket PR。

**实现建议**：
- 实现 `PlatformProvider` trait for Bitbucket
- 使用 Bitbucket API

**需要实现的功能**：
- PR 创建、合并、关闭
- PR 状态查询
- PR 列表查询
- PR 更新
- PR 评论、批准
- PR 同步（merge/rebase）

**Bitbucket API 端点**：
- `/2.0/repositories/{workspace}/{repo-_slug}/pullrequests` - 创建/列出 PR
- `/2.0/repositories/{workspace}/{repo-_slug}/pullrequests/{id}` - 获取/更新 PR
- `/2.0/repositories/{workspace}/{repo-_slug}/pullrequests/{id}/merge` - 合并 PR

#### 1.3 Azure DevOps 支持
- ❌ Azure DevOps PR 支持

**功能**：支持 Azure DevOps PR。

**实现建议**：
- 实现 `PlatformProvider` trait for Azure DevOps
- 使用 Azure DevOps REST API

**需要实现的功能**：
- PR 创建、合并、关闭
- PR 状态查询
- PR 列表查询
- PR 更新
- PR 评论、批准
- PR 同步（merge/rebase）

**Azure DevOps API 端点**：
- `/{organization}/{project}/_apis/git/repositories/{repositoryId}/pullrequests` - 创建/列出 PR
- `/{organization}/{project}/_apis/git/repositories/{repositoryId}/pullrequests/{pullRequestId}` - 获取/更新 PR
- `/{organization}/{project}/_apis/git/repositories/{repositoryId}/pullrequests/{pullRequestId}` - 合并 PR

#### 1.4 Codeup 支持
- ❌ Codeup PR 支持

**功能**：支持阿里云 Codeup PR。

**实现建议**：
- 实现 `PlatformProvider` trait for Codeup
- 使用 Codeup REST API
- 参考 GitHub 的实现

**需要实现的功能**：
- PR 创建、合并、关闭
- PR 状态查询
- PR 列表查询
- PR 更新
- PR 评论、批准
- PR 同步（merge/rebase）

**Codeup API 端点**：
- `/api/v3/projects/{project-_id}/code-_reviews` - 创建/列出 PR
- `/api/v3/projects/{project-_id}/code-_reviews/{review-_id}` - 获取/更新 PR
- `/api/v3/projects/{project-_id}/code-_reviews/{review-_id}/merge` - 合并 PR

**配置要求**：
- `CODEUP_PROJECT_ID` - Codeup 项目 ID
- `CODEUP_CSRF_TOKEN` - Codeup CSRF Token
- `CODEUP_COOKIE` - Codeup Cookie

---

### 2. 通知系统

#### 2.1 桌面通知
- ❌ PR 状态变更通知
- ❌ JIRA 更新通知
- ❌ 通知规则配置

**功能**：PR 状态变更、JIRA 更新时发送桌面通知。

**实现建议**：
- 使用 `notify-rust` 或类似库
- 支持配置通知规则

**通知事件**：
- PR 创建、合并、关闭
- PR 评论、review
- JIRA ticket 状态变更
- JIRA ticket 分配变更
- JIRA ticket 评论

**配置示例**：
```toml
[notifications]
enabled = true

[notifications.rules]
pr-_merged = true
pr-_commented = true
jira-_status-_changed = true
jira-_assigned = true
```

**命令示例**：
```bash
workflow notify enable                              # 启用通知
workflow notify disable                             # 禁用通知
workflow notify test                                 # 测试通知
```

#### 2.2 邮件通知
- ❌ 重要事件邮件通知
- ❌ HTML 邮件模板

**功能**：重要事件通知。

**实现建议**：
- 使用 SMTP 发送邮件
- 支持 HTML 邮件模板

**通知事件**：
- PR 合并（重要）
- JIRA ticket 解决（重要）
- 每日/每周摘要（可选）

**配置示例**：
```toml
[notifications.email]
enabled = true
smtp-_server = "smtp.example.com"
smtp-_port = 587
smtp-_username = "user@example.com"
smtp-_password = "password"
from = "workflow@example.com"
to = ["user@example.com"]
```

**命令示例**：
```bash
workflow notify email enable                        # 启用邮件通知
workflow notify email test                          # 测试邮件通知
```

#### 2.3 Webhook 集成
- ❌ 发送 webhook 请求
- ❌ 接收 webhook（需要 HTTP 服务器）

**功能**：集成外部系统。

**实现建议**：
- 支持发送 webhook 请求
- 支持接收 webhook（需要 HTTP 服务器）

**发送 Webhook**：
- PR 事件（创建、合并、关闭）
- JIRA 事件（状态变更、分配变更）

**接收 Webhook**：
- 监听外部系统事件
- 触发相应操作

**配置示例**：
```toml
[notifications.webhooks]
enabled = true

[notifications.webhooks.outgoing]
pr-_merged = "https://example.com/webhook/pr-merged"
jira-_status-_changed = "https://example.com/webhook/jira-status"

[notifications.webhooks.incoming]
enabled = true
port = 8080
path = "/webhook"
```

**命令示例**：
```bash
workflow webhook send --event pr-_merged --url https://example.com/webhook
workflow webhook server start                        # 启动 webhook 服务器
```

---

## 📊 优先级

### 高优先级
1. **更多平台支持**
   - GitLab 支持（如果团队使用 GitLab）

### 中优先级
1. **通知系统基础功能**
   - 桌面通知
   - Webhook 发送

2. **更多平台支持**
   - Bitbucket 支持
   - Azure DevOps 支持

### 低优先级
1. **通知系统增强**
   - 邮件通知
   - Webhook 接收（HTTP 服务器）

---

## 📝 实现建议

### 开发顺序
1. **第一阶段**：平台支持（根据实际需求选择）
   - GitLab 支持（如果团队使用）
   - 或 Bitbucket 支持
   - 或 Azure DevOps 支持

2. **第二阶段**：通知系统基础
   - 桌面通知
   - Webhook 发送

3. **第三阶段**：通知系统增强
   - 邮件通知
   - Webhook 接收

### 技术考虑
1. **平台抽象**：保持 `PlatformProvider` trait 的一致性
2. **通知库**：使用 `notify-rust` 或 `notify` crate 实现桌面通知
3. **邮件库**：使用 `lettre` 或类似库发送邮件
4. **HTTP 服务器**：使用 `axum` 或 `warp` 实现 webhook 接收
5. **配置管理**：在配置文件中定义通知规则
6. **错误处理**：通知失败时提供清晰的错误信息
7. **测试**：为新功能添加单元测试和集成测试
8. **文档**：及时更新文档和示例

### 实现细节

#### 平台 Provider 实现
```rust
pub struct GitLabProvider {
    client: reqwest::Client,
    base-_url: String,
    token: String,
}

impl PlatformProvider for GitLabProvider {
    async fn create-_pr(&self, params: CreatePrParams) -> Result<PullRequest> {
        // 实现 GitLab PR 创建
    }

    // ... 其他方法
}
```

#### 通知系统实现
```rust
use notify-_rust::Notification;

pub struct NotificationManager {
    desktop-_enabled: bool,
    email-_enabled: bool,
    webhook-_enabled: bool,
}

impl NotificationManager {
    pub async fn notify-_pr-_merged(&self, pr: &PullRequest) -> Result<()> {
        if self.desktop-_enabled {
            Notification::new()
                .summary("PR Merged")
                .body(&format!("PR #{} has been merged", pr.number))
                .show()?;
        }
        // ... 其他通知方式
        Ok(())
    }
}
```

---

## 📊 任务统计

| 状态 | 数量 | 说明 |
|-----|------|------|
| ✅ 已完成 | 2 个 | GitHub PR 支持、Codeup PR 支持（基础实现） |
| 🚧 进行中 | 0 个 | 暂无进行中的任务 |
| ⏳ 待实施 | 10 个 | 更多平台支持和通知系统功能 |
| **总计** | **12** | - |

---

## 📚 相关文档

- [JIRA 模块待办事项](./jira.md)

---

## ✅ 检查清单

实施本需求时，请确保：

- [ ] 保持 `PlatformProvider` trait 的一致性
- [ ] 为新功能添加单元测试和集成测试
- [ ] 及时更新文档和示例
- [ ] 确保新功能不影响现有功能
- [ ] 提供清晰的错误信息和配置说明

---

**最后更新**: 2025-12-23
