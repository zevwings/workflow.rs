# 日志输出重构计划

## 📋 概述

本文档描述将 `lib/` 层的日志输出迁移到 `commands/` 层的重构计划。此重构旨在：
- 遵循分层架构原则：lib 层专注业务逻辑，commands 层负责用户交互和输出
- 提高代码可复用性：lib 层可以在非 CLI 场景（如 GUI、测试、其他工具）中复用
- 改善可测试性：便于验证输出内容和格式
- 统一输出控制：commands 层统一管理所有用户可见的输出

---

## 🎯 重构目标

### 核心原则

1. **lib 层职责**：
   - 执行业务逻辑
   - 返回结果（`Result<T>` 或结构化数据）
   - 使用 `tracing::*` 进行结构化日志记录（用于调试和监控）
   - **不直接输出到控制台**

2. **commands 层职责**：
   - 解析命令行参数
   - 调用 lib 层的业务逻辑
   - 根据返回结果格式化输出
   - 处理用户交互（确认、选择等）
   - **负责所有用户可见的输出**

### 日志类型分类

#### 1. 结构化日志（保留在 lib 层）
- **用途**：调试、监控、问题排查
- **实现**：使用 `tracing::info!()`, `tracing::debug!()`, `tracing::error!()` 等
- **特点**：不直接输出到控制台，可通过日志框架配置输出

#### 2. 用户可见输出（迁移到 commands 层）
- **用途**：向用户展示操作结果、进度、错误信息
- **实现**：使用 `log_success!()`, `log_error!()`, `log_info!()` 等宏
- **特点**：直接输出到控制台，受日志级别控制

---

## 📊 影响范围分析

### 需要重构的模块

根据代码扫描，以下模块在 lib 层直接使用 `log_*` 宏：

#### 高优先级（高频使用、影响大）

1. **`src/lib/base/settings/settings.rs`**
   - 影响：`Settings::verify()` 被多个命令调用
   - 日志数量：~30+ 处
   - 调用者：`commands/config/show.rs`, `commands/config/setup.rs`

2. **`src/lib/jira/logs/download.rs`**
   - 影响：日志下载功能的核心模块
   - 日志数量：~20+ 处
   - 调用者：`commands/log/download.rs`

3. **`src/lib/git/config.rs`**
   - 影响：Git 配置管理
   - 日志数量：3 处
   - 调用者：`commands/github/github.rs`

#### 中优先级（功能重要但使用频率中等）

4. **`src/lib/completion/completion.rs`**
   - 影响：完成脚本管理
   - 日志数量：~15 处
   - 调用者：`commands/config/completion.rs`, `commands/lifecycle/*.rs`

5. **`src/lib/rollback/rollback.rs`**
   - 影响：回滚功能
   - 日志数量：~20 处
   - 调用者：`commands/lifecycle/update.rs`

6. **`src/lib/base/http/retry.rs`**
   - 影响：HTTP 重试逻辑
   - 日志数量：~10 处
   - 调用者：所有使用 HTTP 客户端的模块

7. **`src/lib/jira/status.rs`**
   - 影响：Jira 状态配置
   - 日志数量：~8 处
   - 调用者：`commands/jira/*.rs`

8. **`src/lib/jira/logs/clean.rs`**
   - 影响：日志清理
   - 日志数量：~15 处
   - 调用者：`commands/jira/clean.rs`

#### 低优先级（使用频率低或影响范围小）

9. `src/lib/git/commit.rs` - Git 提交（2 处）
10. `src/lib/git/pre_commit.rs` - 预提交钩子（5 处）
11. `src/lib/git/stash.rs` - Git stash（8 处）
12. `src/lib/jira/history.rs` - Jira 历史（3 处）
13. `src/lib/base/llm/client.rs` - LLM 客户端（1 处）
14. `src/lib/base/shell/reload.rs` - Shell 重载（5 处）
15. `src/lib/base/util/checksum.rs` - 校验和（2 处）
16. `src/lib/completion/generate.rs` - 完成脚本生成（3 处）
17. `src/lib/pr/github/platform.rs` - PR 平台（2 处）

---

## 🔄 重构策略

### 方案 1：返回结构化数据（推荐）

**适用场景**：大多数情况

**实现方式**：
- lib 层返回 `Result<T>` 或结构化数据
- commands 层根据返回结果格式化输出

**示例**：

```rust
// 重构前 (lib/git/config.rs)
pub fn set_global_user(email: &str, name: &str) -> Result<()> {
    cmd_run(&["config", "--global", "user.email", email])?;
    cmd_run(&["config", "--global", "user.name", name])?;

    log_info!("Git global config updated:");
    log_message!("  user.email: {}", email);
    log_message!("  user.name: {}", name);

    Ok(())
}

// 重构后 (lib/git/config.rs)
pub fn set_global_user(email: &str, name: &str) -> Result<()> {
    cmd_run(&["config", "--global", "user.email", email])?;
    cmd_run(&["config", "--global", "user.name", name])?;

    // 只记录结构化日志，不输出到控制台
    tracing::info!("Git global config updated", email = %email, name = %name);

    Ok(())
}

// commands 层 (commands/github/github.rs)
pub fn add() -> Result<()> {
    // ... 收集账号信息 ...

    GitConfig::set_global_user(&email, &name)?;

    // commands 层负责输出
    log_info!("Git global config updated:");
    log_message!("  user.email: {}", email);
    log_message!("  user.name: {}", name);

    Ok(())
}
```

### 方案 2：返回日志消息列表

**适用场景**：需要输出多条消息的复杂操作

**实现方式**：
- lib 层返回包含操作结果和消息列表的结构体
- commands 层统一打印消息

**示例**：

```rust
// lib 层
pub struct VerificationResult {
    pub success: bool,
    pub messages: Vec<LogMessage>,
}

pub enum LogMessage {
    Info(String),
    Success(String),
    Warning(String),
    Error(String),
}

pub fn verify(&self) -> Result<VerificationResult> {
    let mut messages = Vec::new();

    // 执行验证逻辑
    if let Some(ref email) = self.jira.email {
        messages.push(LogMessage::Info(format!("Email: {}", email)));
    }

    // ... 更多验证逻辑 ...

    Ok(VerificationResult {
        success: true,
        messages,
    })
}

// commands 层
pub fn show() -> Result<()> {
    let settings = Settings::load();
    let result = settings.verify()?;

    for msg in result.messages {
        match msg {
            LogMessage::Info(s) => log_info!("{}", s),
            LogMessage::Success(s) => log_success!("{}", s),
            LogMessage::Warning(s) => log_warning!("{}", s),
            LogMessage::Error(s) => log_error!("{}", s),
        }
    }

    Ok(())
}
```

### 方案 3：使用回调函数

**适用场景**：需要实时输出进度信息的长时间操作

**实现方式**：
- lib 层接受可选的回调函数参数
- 通过回调通知进度和状态

**示例**：

```rust
// lib 层
pub fn download_logs<F>(jira_id: &str, on_progress: Option<F>) -> Result<()>
where
    F: Fn(&str),
{
    // ... 下载逻辑 ...

    if let Some(ref callback) = on_progress {
        callback("Downloading attachment 1/3...");
    }

    // ... 继续下载 ...

    Ok(())
}

// commands 层
pub fn download(jira_id: &str) -> Result<()> {
    let callback = |msg: &str| {
        log_info!("{}", msg);
    };

    JiraLogs::download_logs(jira_id, Some(callback))?;

    Ok(())
}
```

---

## 📝 重构步骤

### 阶段 1：准备和规划（1-2 天）

1. ✅ 完成影响范围分析（本文档）
2. 创建重构分支
3. 确定每个模块的重构方案
4. 编写测试用例（确保重构后功能不变）

### 阶段 2：高优先级模块重构（3-5 天）

按优先级顺序重构：

1. **`src/lib/base/settings/settings.rs`**
   - 方案：返回 `VerificationResult` 结构体
   - 影响文件：
     - `src/lib/base/settings/settings.rs`
     - `src/commands/config/show.rs`
     - `src/commands/config/setup.rs`

2. **`src/lib/jira/logs/download.rs`**
   - 方案：使用回调函数（实时进度）+ 返回结果
   - 影响文件：
     - `src/lib/jira/logs/download.rs`
     - `src/commands/log/download.rs`

3. **`src/lib/git/config.rs`**
   - 方案：返回结构化数据
   - 影响文件：
     - `src/lib/git/config.rs`
     - `src/commands/github/github.rs`

### 阶段 3：中优先级模块重构（5-7 天）

4. `src/lib/completion/completion.rs`
5. `src/lib/rollback/rollback.rs`
6. `src/lib/base/http/retry.rs`
7. `src/lib/jira/status.rs`
8. `src/lib/jira/logs/clean.rs`

### 阶段 4：低优先级模块重构（3-5 天）

9-17. 剩余模块按使用频率逐个重构

### 阶段 5：清理和优化（2-3 天）

1. 移除 lib 层中不再需要的 `log_*` 宏导入
2. 统一使用 `tracing::*` 进行结构化日志记录
3. 更新文档和注释
4. 代码审查和测试

---

## 🧪 测试策略

### 单元测试

- 确保 lib 层函数返回正确的数据结构
- 验证错误处理逻辑

### 集成测试

- 验证 commands 层输出格式正确
- 确保用户体验不受影响

### 手动测试

- 运行所有相关命令，验证输出
- 检查日志级别控制是否正常工作

---

## ⚠️ 注意事项

### 1. 向后兼容性

- 如果某些 lib 层函数被外部代码直接调用，需要考虑兼容性
- 可以通过提供包装函数或保持旧 API 一段时间来过渡

### 2. 错误处理

- lib 层应该返回详细的错误信息
- commands 层负责将错误转换为用户友好的消息

### 3. 调试日志

- 保留 `tracing::debug!()` 用于调试
- 这些日志不会输出到控制台，但可以通过日志框架配置查看

### 4. 进度信息

- 对于长时间运行的操作，考虑使用回调或事件机制
- 避免阻塞操作期间无法显示进度

---

## 📈 成功标准

重构完成后，应该满足以下标准：

1. ✅ lib 层不再直接使用 `log_*` 宏（除了 `tracing::*`）
2. ✅ 所有用户可见的输出都在 commands 层
3. ✅ lib 层函数返回结构化数据或 `Result<T>`
4. ✅ 所有现有功能正常工作
5. ✅ 测试覆盖率不降低
6. ✅ 代码可读性和可维护性提升

---

## 🔗 相关文档

- [架构设计文档](../architecture/ARCHITECTURE.md)
- [开发指南](../guidelines/DEVELOPMENT_GUIDELINES.md)
- [命令架构文档](../architecture/commands/)
- [库架构文档](../architecture/lib/)

---

## 📅 时间估算

- **总时间**：约 15-20 个工作日
- **阶段 1**：1-2 天
- **阶段 2**：3-5 天
- **阶段 3**：5-7 天
- **阶段 4**：3-5 天
- **阶段 5**：2-3 天

---

## 🎯 下一步行动

1. 评审本重构计划
2. 创建重构分支：`refactor/logging-separation`
3. 开始阶段 1 的准备工作
4. 按阶段逐步执行重构

---

**最后更新**：2024-12-19
**状态**：规划中
**负责人**：待分配
