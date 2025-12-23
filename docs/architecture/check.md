# 环境检查命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的环境检查命令模块架构。环境检查命令提供综合的环境验证功能，确保在执行关键操作前，Git 仓库状态和网络连接都处于正常状态。

**定位**：命令层专注于检查逻辑的组织和结果展示，核心检查功能由 `lib/git/` 和 `lib/base/http/` 模块提供。

**使用场景**：
- 在执行 PR 创建、分支清理等关键操作前自动运行
- 用户手动运行 `workflow check` 进行环境验证
- 作为其他命令的前置检查步骤

---

## 📁 相关文件

### CLI 入口层

环境检查命令现在作为 `workflow` 主命令的子命令，通过 `src/main.rs` 中的 `Commands::Check` 枚举定义。

```
src/main.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow check` 命令分发到对应的命令处理函数

### 命令封装层

```
src/commands/check/
├── mod.rs          # Check 命令模块声明（4 行）
└── check.rs        # 环境检查命令（~68 行）
```

**职责**：
- 组织检查步骤
- 格式化输出检查结果
- 调用核心业务逻辑层 (`lib/git/`、`lib/base/http/`) 的功能

### 依赖模块

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/git/`**：Git 操作（`GitRepo`、`GitCommit`）
  - `GitRepo::is-_git-_repo()` - 检查是否在 Git 仓库中
  - `GitCommit::status()` - 获取 Git 状态
- **`lib/base/http/`**：HTTP 客户端（`HttpClient`）
  - `HttpClient::global()` - 获取全局 HTTP 客户端
  - `HttpClient::stream()` - 发送 HTTP 请求

详细架构文档：参见 [Git 模块架构文档](../lib/git.md) 和 [HTTP 模块架构文档](../lib/http.md)

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
src/main.rs (workflow 主命令，参数解析)
  ↓
commands/check/check.rs (命令封装层)
  ↓
lib/git/* 和 lib/base/http/* (通过 API 调用，具体实现见相关模块文档)
```

### 命令分发流程

```
src/main.rs::main()
  ↓
Cli::parse() (解析命令行参数)
  ↓
match cli.subcommand {
  Commands::Check => CheckCommand::run-_all()
}
```

### 检查流程

```
CheckCommand::run-_all()
  ↓
[1/2] Git 仓库状态检查
  ├─ GitRepo::is-_git-_repo() (检查是否在 Git 仓库中)
  └─ GitCommit::status() (获取 Git 状态)
  ↓
[2/2] 网络连接检查
  ├─ HttpClient::global() (获取 HTTP 客户端)
  └─ HttpClient::stream() (检查到 GitHub 的连接)
  ↓
显示检查结果
```

---

## 功能说明

### 检查步骤

环境检查命令执行两个主要检查步骤：

#### 1. Git 仓库状态检查

**检查项**：
- 是否在 Git 仓库中
- Git 工作区状态（是否有未提交的更改）

**实现**：
- 使用 `GitRepo::is-_git-_repo()` 检查是否在 Git 仓库中
- 使用 `GitCommit::status()` 获取 Git 状态输出
- 如果工作区干净（无未提交更改），显示成功消息
- 如果有未提交更改，显示 Git 状态输出

**错误处理**：
- 如果不在 Git 仓库中，返回错误并中断检查
- 如果有未提交更改，显示状态但不中断（仅信息提示）

#### 2. 网络连接检查

**检查项**：
- 到 GitHub 的网络连接是否可用

**实现**：
- 使用 `HttpClient::global()` 获取全局 HTTP 客户端
- 发送 GET 请求到 `https://github.com`
- 设置 10 秒超时
- 检查响应状态码

**错误处理**：
- 如果连接失败，返回错误并中断检查
- 提供详细的错误信息和解决建议（网络问题、代理设置、防火墙限制等）

### 关键步骤说明

1. **检查顺序**：
   - 先检查 Git 仓库状态（本地检查，快速）
   - 再检查网络连接（需要网络请求，较慢）

2. **错误处理策略**：
   - Git 检查失败：立即中断，返回错误
   - 网络检查失败：返回错误，提供解决建议

3. **输出格式**：
   - 使用步骤编号（[1/2]、[2/2]）清晰标识检查进度
   - 使用分隔线（`log-_break!()`）分隔不同检查步骤
   - 使用不同日志级别（`log-_success!`、`log-_error!`、`log-_info!`）区分结果

---

## 🏗️ 架构设计

### 设计模式

#### 1. 命令模式

检查命令是一个独立的结构体，实现统一的方法接口：
- `CheckCommand::run-_all()` - 执行所有检查

#### 2. 工具函数模式

将检查逻辑封装到 `lib/` 中的工具函数，命令层只负责调用和展示：
- `GitRepo` - Git 仓库检查
- `GitCommit` - Git 状态检查
- `HttpClient` - 网络连接检查

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
   - `clap` 自动处理参数验证和错误提示

2. **命令层**：检查逻辑错误
   - Git 检查失败：返回错误，中断执行
   - 网络检查失败：返回错误，提供解决建议

3. **库层**：Git 操作错误、网络错误
   - 通过 `GitRepo`、`GitCommit`、`HttpClient` API 返回的错误信息
   - Git 操作失败、网络请求失败等

### 容错机制

- **Git 仓库不存在**：立即返回错误，不继续检查
- **网络连接失败**：返回详细错误信息，提供解决建议
- **超时处理**：网络检查设置 10 秒超时，避免长时间等待

---

## 📝 扩展性

### 添加新的检查项

1. 在 `check.rs` 的 `run-_all()` 方法中添加新的检查步骤
2. 调用相应的工具函数或库函数
3. 提供清晰的错误提示和解决建议
4. 更新检查步骤的编号和日志输出

**示例**：
```rust
// 3. 检查代理配置
log-_message!("[3/3] Checking proxy configuration...");
// ... 检查逻辑
```

---

## 📚 相关文档

- [主架构文档](../architecture.md)
- [Git 模块架构文档](../lib/git.md) - Git 操作相关
- [HTTP 模块架构文档](../lib/http.md) - HTTP 客户端相关
- [配置管理命令模块架构文档](./config.md) - 配置管理相关

---

## 📋 使用示例

### Check 命令

```bash
# 运行环境检查
workflow check
```

**输出示例**：
```
Running environment checks...

[1/2] Checking Git repository status...
✓ Git repository is clean (no uncommitted changes)

[2/2] Checking network connection to GitHub...
✓ GitHub network is available

✓ All checks passed
```

---

## ✅ 总结

环境检查命令层采用清晰的检查组织设计：

1. **综合检查**：Git 仓库状态和网络连接
2. **清晰输出**：步骤编号、分隔线、不同日志级别
3. **错误处理**：详细的错误信息和解决建议

**设计优势**：
- ✅ **可靠性**：确保关键操作前环境正常
- ✅ **清晰性**：步骤编号和格式化输出，易于理解
- ✅ **可扩展性**：易于添加新的检查项
- ✅ **用户友好**：详细的错误信息和解决建议

---

**最后更新**: 2025-12-16
