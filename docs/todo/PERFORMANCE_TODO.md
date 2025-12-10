# 性能优化待办事项

## 📋 概述

本文档列出性能优化相关的待办功能，包括缓存机制和并发处理。

---

## ❌ 待实现功能

### 1. 缓存机制

#### 1.1 API 响应缓存
- ❌ API 响应缓存（减少重复请求）
- ❌ 缓存过期策略
- ❌ 手动刷新缓存

**功能**：减少重复的 API 请求，提高响应速度。

**实现建议**：
- 使用内存缓存（如 `moka`）
- 支持缓存过期策略（TTL）
- 支持手动刷新缓存
- 支持缓存统计（命中率、大小等）

**缓存策略**：
- **短期缓存**（1-5 分钟）：JIRA ticket 基本信息、PR 状态
- **中期缓存**（10-30 分钟）：JIRA ticket 列表、PR 列表
- **长期缓存**（1 小时+）：项目配置、用户信息

**命令示例**：
```bash
workflow cache clear                               # 清除所有缓存
workflow cache clear --type jira                  # 清除 JIRA 缓存
workflow cache stats                               # 显示缓存统计
workflow jira info PROJ-123 --refresh             # 强制刷新缓存
```

**实现示例**：
```rust
use moka::future::Cache;

pub struct ApiCache {
    jira_cache: Cache<String, JiraTicket>,
    pr_cache: Cache<String, PullRequest>,
}

impl ApiCache {
    pub async fn get_jira_ticket(&self, key: &str) -> Option<JiraTicket> {
        self.jira_cache.get(key).await
    }

    pub async fn set_jira_ticket(&self, key: String, ticket: JiraTicket) {
        self.jira_cache.insert(key, ticket).await;
    }
}
```

#### 1.2 本地数据缓存
- ❌ 本地数据缓存（缓存 tickets/PRs 信息）
- ❌ 增量更新
- ❌ 缓存索引

**功能**：缓存 tickets/PRs 信息到本地文件，减少 API 调用。

**实现建议**：
- 缓存到本地文件（JSON/SQLite）
- 支持增量更新（只更新变更的数据）
- 支持缓存索引（快速查找）
- 支持缓存压缩（节省空间）

**缓存格式**：
- **JSON 格式**：简单易读，适合小量数据
- **SQLite 格式**：支持查询和索引，适合大量数据

**命令示例**：
```bash
workflow cache sync                                # 同步缓存（增量更新）
workflow cache rebuild                             # 重建缓存
workflow cache size                                 # 显示缓存大小
```

#### 1.3 智能刷新
- ❌ 按需刷新缓存
- ❌ 后台自动刷新
- ❌ 缓存失效检测

**功能**：智能管理缓存，按需刷新。

**实现建议**：
- 检测数据是否过期
- 后台自动刷新常用数据
- 支持缓存失效检测（基于 ETag、Last-Modified 等）

**刷新策略**：
- **按需刷新**：访问时检查是否过期，过期则刷新
- **后台刷新**：定期刷新常用数据
- **事件驱动**：数据变更时自动失效缓存

---

### 2. 并发处理

#### 2.1 并行下载
- ❌ 并行下载多个附件
- ❌ 并发数限制
- ❌ 下载进度显示

**功能**：并行下载多个附件，提高下载速度。

**实现建议**：
- 使用 `tokio` 或 `rayon` 实现并行
- 支持并发数限制（避免过载）
- 显示下载进度（进度条）

**命令示例**：
```bash
workflow jira attachments PROJ-123 --parallel      # 并行下载附件
workflow jira attachments PROJ-123 --parallel --max-concurrent 5  # 限制并发数
```

**实现示例**：
```rust
use tokio::task::JoinSet;

pub async fn download_attachments_parallel(
    attachments: Vec<Attachment>,
    max_concurrent: usize,
) -> Result<Vec<PathBuf>> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut tasks = JoinSet::new();

    for attachment in attachments {
        let semaphore = semaphore.clone();
        tasks.spawn(async move {
            let _permit = semaphore.acquire().await?;
            download_attachment(&attachment).await
        });
    }

    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        results.push(result??);
    }
    Ok(results)
}
```

#### 2.2 批量 API 调用
- ❌ 合并多个 API 请求
- ❌ 并发 API 请求
- ❌ 请求去重

**功能**：优化 API 调用，减少请求次数和延迟。

**实现建议**：
- 使用批量 API（如果平台支持）
- 使用并发请求（如果平台不支持批量）
- 支持请求去重（相同请求只发送一次）

**批量 API 示例**：
```rust
// 批量获取 JIRA tickets
pub async fn get_tickets_batch(keys: Vec<String>) -> Result<Vec<JiraTicket>> {
    // 如果 JIRA 支持批量 API
    if let Some(batch_api) = self.batch_api {
        return batch_api.get_tickets(keys).await;
    }

    // 否则使用并发请求
    let tasks: Vec<_> = keys.into_iter()
        .map(|key| self.get_ticket(&key))
        .collect();
    futures::future::join_all(tasks).await
        .into_iter()
        .collect()
}
```

---

## 📊 优先级

### 高优先级
1. **API 响应缓存**
   - API 响应缓存（减少重复请求）
   - 缓存过期策略

2. **并行下载**
   - 并行下载多个附件
   - 并发数限制

### 中优先级
1. **本地数据缓存**
   - 本地数据缓存（缓存 tickets/PRs 信息）
   - 增量更新

2. **批量 API 调用**
   - 合并多个 API 请求
   - 并发 API 请求

3. **智能刷新**
   - 按需刷新缓存
   - 后台自动刷新

---

## 📝 实现建议

### 开发顺序
1. **第一阶段**：API 响应缓存
   - 实现内存缓存
   - 支持缓存过期策略
   - 支持手动刷新

2. **第二阶段**：并行下载
   - 实现并行下载
   - 支持并发数限制
   - 显示下载进度

3. **第三阶段**：本地数据缓存和批量 API
   - 实现本地数据缓存
   - 实现批量 API 调用
   - 实现智能刷新

### 技术考虑
1. **缓存库**：使用 `moka` 或 `cached` 作为内存缓存
2. **并发库**：使用 `tokio` 或 `rayon` 实现并发
3. **存储格式**：使用 JSON 或 SQLite 存储本地缓存
4. **性能监控**：添加性能指标（缓存命中率、响应时间等）
5. **测试**：为新功能添加单元测试和集成测试
6. **文档**：及时更新文档和示例

### 性能指标

#### 缓存指标
- **缓存命中率**：缓存命中次数 / 总请求次数
- **缓存大小**：内存使用量、磁盘使用量
- **缓存效率**：节省的 API 请求次数

#### 并发指标
- **并发数**：同时进行的任务数
- **吞吐量**：单位时间内完成的任务数
- **延迟**：平均响应时间

---

## 📚 相关文档

- [JIRA 模块待办事项](./JIRA_TODO.md)
- [Git 工作流待办事项](./GIT_TODO.md)

---

**最后更新**: 2025-12-09
