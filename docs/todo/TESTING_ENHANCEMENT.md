# 测试增强计划文档

## 📋 概述

本文档分析了 Workflow CLI 项目的测试覆盖情况，并制定了详细的测试增强计划。目标是提高代码质量、减少回归问题，并确保新功能的稳定性。

**当前测试状态：**
- 现有测试文件：7 个
- 测试覆盖模块：部分基础模块
- 测试类型：主要是单元测试和少量集成测试

**目标：**
- 提高测试覆盖率至 80%+
- 为所有核心模块添加测试
- 增加集成测试和端到端测试
- 建立持续集成测试流程

---

## 📊 现有测试分析

### 已存在的测试文件

| 测试文件 | 覆盖模块 | 测试类型 | 完整度 |
|---------|---------|---------|--------|
| `tests/logger.rs` | `logging::logger` | 单元测试 | ⭐⭐⭐⭐⭐ 完整 |
| `tests/settings.rs` | `base::settings` | 单元测试 | ⭐⭐ 基础 |
| `tests/paths_integration.rs` | `base::settings::paths` | 集成测试 | ⭐⭐⭐⭐ 较完整 |
| `tests/llm_client.rs` | `base::llm::client` | 单元测试 | ⭐⭐⭐ 中等 |
| `tests/jira_helpers.rs` | `jira::helpers` | 单元测试 | ⭐⭐ 基础 |
| `tests/github.rs` | `pr::helpers` | 单元测试 | ⭐ 基础 |
| `tests/body_parser.rs` | `pr::body_parser` | 单元测试 | ⭐⭐⭐ 中等 |

### 测试覆盖评估

**已覆盖（部分）：**
- ✅ Logger 模块（完整）
- ✅ Paths 模块（较完整）
- ✅ LLM 客户端 JSON 解析（中等）
- ✅ Jira 辅助函数（基础）
- ✅ PR Body 解析（中等）
- ✅ GitHub URL 解析（基础）

**未覆盖或覆盖不足：**
- ❌ HTTP 客户端和重试机制
- ❌ UI 组件（dialogs, components, progress）
- ❌ Git 操作模块
- ❌ Jira 客户端核心功能
- ❌ PR 平台集成
- ❌ 工具函数模块
- ❌ Shell 检测和配置
- ❌ 代理管理
- ❌ 回滚管理
- ❌ 补全生成
- ❌ 命令层测试

---

## 🎯 测试增强计划

### 阶段 1：核心基础设施测试（优先级：高）

#### 1.1 HTTP 模块测试

**文件：** `tests/http_client.rs`

**需要测试的功能：**
- [ ] HTTP 客户端初始化
- [ ] GET/POST/PUT/DELETE 请求
- [ ] 请求头设置和认证
- [ ] 响应解析（JSON、文本）
- [ ] 错误处理（网络错误、HTTP 错误）
- [ ] 超时处理

**文件：** `tests/http_retry.rs`

**需要测试的功能：**
- [ ] 重试配置（默认值、自定义配置）
- [ ] 指数退避算法
- [ ] 可重试错误判断
- [ ] 最大重试次数限制
- [ ] 交互式确认（模拟用户输入）
- [ ] 非交互式模式
- [ ] 延迟计算准确性

**测试示例：**
```rust
#[test]
fn test_retry_config_default() {
    let config = HttpRetryConfig::default();
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_delay, 1);
    assert_eq!(config.max_delay, 30);
}

#[test]
fn test_exponential_backoff() {
    // 测试指数退避计算
}

#[test]
fn test_retry_on_network_error() {
    // 测试网络错误重试
}
```

#### 1.2 工具函数模块测试

**文件：** `tests/util.rs`

**需要测试的功能：**

**Browser (`base::util::browser`):**
- [ ] URL 打开功能（需要 mock）
- [ ] 错误处理

**Checksum (`base::util::checksum`):**
- [ ] SHA256 计算
- [ ] 文件哈希验证
- [ ] 哈希解析
- [ ] URL 构建

**Clipboard (`base::util::clipboard`):**
- [ ] 复制功能（需要 mock）
- [ ] 平台特定行为

**Format (`base::util::format`):**
- [ ] 文件大小格式化（B, KB, MB, GB, TB）
- [ ] 边界值测试

**String (`base::util::string`):**
- [ ] 敏感值掩码
- [ ] 各种输入格式

**Unzip (`base::util::unzip`):**
- [ ] ZIP 解压
- [ ] TAR.GZ 解压
- [ ] 错误处理（损坏文件、权限错误）

**Platform (`base::util::platform`):**
- [ ] 平台检测
- [ ] 不同平台行为

**测试示例：**
```rust
#[test]
fn test_format_size() {
    assert_eq!(format_size(0), "0 B");
    assert_eq!(format_size(1024), "1.0 KB");
    assert_eq!(format_size(1024 * 1024), "1.0 MB");
}

#[test]
fn test_mask_sensitive_value() {
    assert_eq!(mask_sensitive_value("secret123"), "*******");
    assert_eq!(mask_sensitive_value("short"), "*****");
}
```

#### 1.3 Shell 模块测试

**文件：** `tests/shell.rs`

**需要测试的功能：**

**Shell 检测 (`base::shell::detect`):**
- [ ] Shell 类型检测（zsh, bash, fish, etc.）
- [ ] 配置文件路径检测
- [ ] 默认 Shell 检测

**Shell 配置 (`base::shell::config`):**
- [ ] 配置读取和写入
- [ ] 配置合并
- [ ] 配置验证

**Shell 重载 (`base::shell::reload`):**
- [ ] 重载命令生成
- [ ] 不同 Shell 的重载方式

---

### 阶段 2：UI 组件测试（优先级：高）

#### 2.1 对话框组件测试

**文件：** `tests/ui_dialogs.rs`

**需要测试的功能：**

**InputDialog:**
- [ ] 基本输入功能
- [ ] 占位符显示
- [ ] 默认值设置
- [ ] 初始文本设置
- [ ] 空值验证
- [ ] 自定义验证器
- [ ] 错误消息显示
- [ ] 非交互式模式（非 TTY）

**SelectDialog:**
- [ ] 选项列表显示
- [ ] 键盘导航（上下箭头）
- [ ] 选择确认（Enter）
- [ ] 取消操作（Esc）
- [ ] 默认选项
- [ ] 空列表处理

**MultiSelectDialog:**
- [ ] 多选功能
- [ ] 空格键切换选择
- [ ] 全选/取消全选
- [ ] 选择状态显示
- [ ] 最少/最多选择限制

**ConfirmDialog:**
- [ ] 确认/取消功能
- [ ] 默认值设置
- [ ] 键盘快捷键（Y/N）

**测试挑战：**
- UI 组件需要终端交互，测试需要使用 mock 或测试框架
- 考虑使用 `crossterm` 的测试模式或创建测试专用的终端后端

**测试策略：**
```rust
// 使用 mock 终端进行测试
#[test]
fn test_input_dialog_basic() {
    // 创建 mock 终端
    // 模拟用户输入
    // 验证输出
}

#[test]
fn test_input_dialog_non_interactive() {
    // 测试非 TTY 环境
    // 应该回退到简单输入
}
```

#### 2.2 其他 UI 组件测试

**文件：** `tests/ui_components.rs`

**需要测试的功能：**

**ProgressBar:**
- [ ] 进度更新
- [ ] 百分比计算
- [ ] 完成状态
- [ ] 自定义消息

**Layout:**
- [ ] 布局计算
- [ ] 响应式布局
- [ ] 边界情况

**Theme:**
- [ ] 样式应用
- [ ] 颜色配置

**LogViewer:**
- [ ] 日志显示
- [ ] 过滤功能
- [ ] 搜索功能
- [ ] 滚动功能

**FileLogViewer:**
- [ ] 文件加载
- [ ] 分页加载（大文件）
- [ ] 搜索和过滤

---

### 阶段 3：Git 模块测试（优先级：高）

#### 3.1 Git 操作测试

**文件：** `tests/git.rs`

**需要测试的功能：**

**GitCommit (`git::commit`):**
- [ ] 提交状态检查
- [ ] 文件暂存
- [ ] 提交创建
- [ ] 提交消息验证
- [ ] 推送操作
- [ ] 错误处理（无变更、无远程等）

**GitBranch (`git::branch`):**
- [ ] 分支创建
- [ ] 分支切换
- [ ] 分支检查（是否存在、是否干净）
- [ ] 默认分支获取
- [ ] 合并策略
- [ ] 分支删除

**GitCherryPick (`git::cherry_pick`):**
- [ ] Cherry-pick 应用
- [ ] 冲突检测
- [ ] 继续操作
- [ ] 中止操作
- [ ] 状态检查

**GitRepo (`git::repo`):**
- [ ] Git 仓库检测
- [ ] 远程仓库类型识别（GitHub, Codeup）
- [ ] 远程 URL 解析

**GitStash (`git::stash`):**
- [ ] Stash push
- [ ] Stash pop
- [ ] 冲突检测

**GitPreCommit (`git::pre_commit`):**
- [ ] Pre-commit hook 检测
- [ ] Hook 执行
- [ ] 执行结果处理

**GitConfig (`git::config`):**
- [ ] 配置读取
- [ ] 配置设置
- [ ] 配置验证

**测试策略：**
- 使用临时 Git 仓库进行测试
- 使用 `git2` crate 或直接调用 `git` 命令
- 测试后清理临时仓库

**测试示例：**
```rust
use tempfile::TempDir;
use std::process::Command;

#[test]
fn test_git_commit() {
    let temp_dir = TempDir::new().unwrap();
    // 初始化 Git 仓库
    // 创建测试文件
    // 执行提交操作
    // 验证结果
    // 清理
}
```

---

### 阶段 4：Jira 模块测试（优先级：中）

#### 4.1 Jira 客户端测试

**文件：** `tests/jira_client.rs`

**需要测试的功能：**

**JiraClient (`jira::client`):**
- [ ] 客户端初始化
- [ ] 认证（Basic Auth, API Token）
- [ ] 错误处理

**Jira API (`jira::api`):**
- [ ] Issue 获取
- [ ] Issue 创建
- [ ] Issue 更新
- [ ] 用户信息获取
- [ ] 项目信息获取
- [ ] 状态转换

**JiraTicket (`jira::ticket`):**
- [ ] Ticket 信息获取
- [ ] Ticket 更新
- [ ] 评论添加
- [ ] 附件处理

**JiraStatus (`jira::status`):**
- [ ] 状态配置加载
- [ ] 状态转换验证
- [ ] 状态映射

**JiraUsers (`jira::users`):**
- [ ] 用户信息管理
- [ ] 用户缓存
- [ ] 用户查找

**JiraHistory (`jira::history`):**
- [ ] 工作历史记录
- [ ] 历史查询
- [ ] 历史导出

**JiraLogs (`jira::logs`):**
- [ ] 日志下载
- [ ] 日志搜索
- [ ] 日志清理
- [ ] 日志压缩

**测试策略：**
- 使用 mock HTTP 客户端（如 `mockito`）
- 测试 Jira API 响应解析
- 测试错误处理

**测试示例：**
```rust
use mockito::Server;

#[test]
fn test_jira_get_issue() {
    let mut server = Server::new();
    let mock = server.mock("GET", "/rest/api/2/issue/PROJ-123")
        .with_status(200)
        .with_body(r#"{"key": "PROJ-123", "fields": {...}}"#)
        .create();

    // 测试获取 Issue
    // 验证解析结果
}
```

#### 4.2 Jira 辅助函数测试（扩展）

**文件：** `tests/jira_helpers.rs`（扩展现有文件）

**需要增加的测试：**
- [ ] `validate_jira_ticket_format` - Ticket 格式验证
- [ ] 更多边界情况测试
- [ ] 错误输入处理

---

### 阶段 5：PR 模块测试（优先级：中）

#### 5.1 PR 平台测试

**文件：** `tests/pr_platform.rs`

**需要测试的功能：**

**GitHub Platform (`pr::github`):**
- [ ] PR 创建
- [ ] PR 获取
- [ ] PR 更新
- [ ] PR 合并
- [ ] PR 评论
- [ ] 用户信息获取
- [ ] 错误处理

**Codeup Platform (`pr::codeup`):**
- [ ] PR 创建
- [ ] PR 获取
- [ ] PR 更新
- [ ] PR 合并
- [ ] 用户信息获取
- [ ] 错误处理

**Platform Detection (`pr::platform`):**
- [ ] 平台类型检测
- [ ] URL 解析
- [ ] 平台切换

**测试策略：**
- 使用 mock HTTP 客户端
- 测试不同平台的 API 差异
- 测试错误响应处理

#### 5.2 PR 辅助函数测试

**文件：** `tests/pr_helpers.rs`

**需要测试的功能：**
- [ ] `generate_branch_name` - 分支名生成
- [ ] `generate_commit_title` - 提交标题生成
- [ ] `generate_pull_request_body` - PR Body 生成
- [ ] `get_current_branch_pr_id` - 当前分支 PR ID 获取
- [ ] `detect_repo_type` - 仓库类型检测

#### 5.3 PR LLM 测试

**文件：** `tests/pr_llm.rs`

**需要测试的功能：**
- [ ] PR 摘要生成
- [ ] 文件变更摘要
- [ ] Prompt 构建
- [ ] LLM 响应解析

---

### 阶段 6：其他模块测试（优先级：低）

#### 6.1 代理管理测试

**文件：** `tests/proxy.rs`

**需要测试的功能：**
- [ ] 代理配置生成
- [ ] 代理启用/禁用
- [ ] 系统代理读取
- [ ] 代理验证
- [ ] 不同平台行为

#### 6.2 回滚管理测试

**文件：** `tests/rollback.rs`

**需要测试的功能：**
- [ ] 备份创建
- [ ] 备份恢复
- [ ] 备份列表
- [ ] 备份清理
- [ ] 备份验证

#### 6.3 补全生成测试

**文件：** `tests/completion.rs`

**需要测试的功能：**
- [ ] 补全文件生成（不同 Shell）
- [ ] 补全内容验证
- [ ] 补全安装
- [ ] 补全更新

#### 6.4 Prompt 模块测试

**文件：** `tests/prompt.rs`

**需要测试的功能：**
- [ ] Prompt 模板加载
- [ ] Prompt 变量替换
- [ ] 语言检测
- [ ] 语言指令生成

#### 6.5 Settings 模块测试（扩展）

**文件：** `tests/settings.rs`（扩展现有文件）

**需要增加的测试：**
- [ ] 配置加载（各种场景）
- [ ] 配置验证
- [ ] 配置合并
- [ ] 默认值应用
- [ ] 配置保存

---

### 阶段 7：命令层测试（优先级：中）

#### 7.1 命令测试

**文件：** `tests/commands/`

为每个命令模块创建测试文件：

**Branch 命令：**
- [ ] `tests/commands/branch.rs`
  - Branch clean
  - Branch ignore
  - Branch helpers

**Config 命令：**
- [ ] `tests/commands/config.rs`
  - Config setup
  - Config completion
  - Config log
  - Config show

**GitHub 命令：**
- [ ] `tests/commands/github.rs`
  - GitHub 操作
  - GitHub helpers

**Jira 命令：**
- [ ] `tests/commands/jira.rs`
  - Jira info
  - Jira attachments
  - Jira clean

**PR 命令：**
- [ ] `tests/commands/pr.rs`
  - PR create
  - PR pick
  - PR rebase
  - PR sync
  - PR merge
  - PR comment
  - PR summarize

**Log 命令：**
- [ ] `tests/commands/log.rs`
  - Log download
  - Log find
  - Log search

**测试策略：**
- 使用 mock 依赖（Git、HTTP、文件系统）
- 测试命令参数解析
- 测试命令执行流程
- 测试错误处理

---

### 阶段 8：集成测试（优先级：高）

#### 8.1 端到端测试

**文件：** `tests/integration/`

**需要测试的场景：**

**Git 工作流：**
- [ ] `tests/integration/git_workflow.rs`
  - 完整 Git 操作流程
  - 分支创建 → 提交 → PR 创建

**Jira 工作流：**
- [ ] `tests/integration/jira_workflow.rs`
  - Jira Ticket 创建 → 工作历史记录 → 日志下载

**PR 工作流：**
- [ ] `tests/integration/pr_workflow.rs`
  - PR 创建 → LLM 摘要生成 → PR 合并

**配置管理：**
- [ ] `tests/integration/config_workflow.rs`
  - 配置初始化 → 配置更新 → 配置验证

**测试策略：**
- 使用真实的外部服务（可选，或使用 mock）
- 测试完整用户场景
- 验证数据一致性

---

## 📝 测试工具和依赖

### 推荐的测试依赖

在 `Cargo.toml` 的 `[dev-dependencies]` 中添加：

```toml
[dev-dependencies]
# Mock HTTP 客户端
mockito = "1.0"
wiremock = "0.5"

# 临时文件和目录
tempfile = "3.8"

# 测试工具
assert_matches = "1.5"
pretty_assertions = "1.4"

# 异步测试（如果需要）
tokio-test = "0.4"

# 文件系统操作
walkdir = "2.4"  # 可能已存在

# Git 测试
git2 = "0.18"  # 用于 Git 操作测试
```

### 测试工具函数

创建 `tests/common/mod.rs` 用于共享测试工具：

```rust
// tests/common/mod.rs
pub mod fixtures;
pub mod mocks;
pub mod helpers;

// 测试工具函数
pub fn setup_test_env() {
    // 设置测试环境
}

pub fn cleanup_test_env() {
    // 清理测试环境
}

pub fn create_temp_git_repo() -> TempDir {
    // 创建临时 Git 仓库
}
```

---

## 🎯 测试优先级和时间估算

### 优先级排序

1. **高优先级（立即开始）：**
   - HTTP 模块测试
   - 工具函数测试
   - Git 模块测试
   - UI 组件测试（基础功能）

2. **中优先级（第二阶段）：**
   - Jira 模块测试
   - PR 模块测试
   - 命令层测试
   - 集成测试

3. **低优先级（第三阶段）：**
   - 代理管理测试
   - 回滚管理测试
   - 补全生成测试
   - Prompt 模块测试

### 时间估算

| 阶段 | 任务 | 估算时间 |
|------|------|---------|
| 阶段 1 | 核心基础设施测试 | 1-2 周 |
| 阶段 2 | UI 组件测试 | 1 周 |
| 阶段 3 | Git 模块测试 | 1-2 周 |
| 阶段 4 | Jira 模块测试 | 1 周 |
| 阶段 5 | PR 模块测试 | 1 周 |
| 阶段 6 | 其他模块测试 | 1 周 |
| 阶段 7 | 命令层测试 | 1-2 周 |
| 阶段 8 | 集成测试 | 1 周 |
| **总计** | | **8-11 周** |

---

## 📋 测试最佳实践

### 1. 测试命名规范

```rust
// 好的命名
#[test]
fn test_http_client_get_request() { }

#[test]
fn test_git_commit_with_message() { }

// 避免的命名
#[test]
fn test1() { }
#[test]
fn test_thing() { }
```

### 2. 测试组织结构

```rust
mod http_client {
    mod get_request {
        #[test]
        fn test_success() { }

        #[test]
        fn test_timeout() { }

        #[test]
        fn test_network_error() { }
    }

    mod post_request {
        // ...
    }
}
```

### 3. 测试数据管理

- 使用 fixtures 目录存放测试数据
- 使用临时文件/目录进行测试
- 测试后清理资源

### 4. Mock 和 Stub

- 使用 mock 替代外部依赖（HTTP、文件系统）
- 使用 stub 提供简单实现
- 避免测试依赖外部服务

### 5. 测试覆盖率

- 目标覆盖率：80%+
- 关注核心业务逻辑
- 测试边界情况和错误处理

### 6. 持续集成

- 在 CI/CD 中运行所有测试
- 测试失败时阻止合并
- 定期检查测试覆盖率

---

## 🔍 测试检查清单

### 每个测试文件应包含：

- [ ] 模块文档注释
- [ ] 测试用例覆盖正常流程
- [ ] 测试用例覆盖错误情况
- [ ] 测试用例覆盖边界值
- [ ] 测试数据清理
- [ ] 测试独立性（不依赖其他测试）

### 每个功能模块应包含：

- [ ] 单元测试（核心功能）
- [ ] 集成测试（模块间交互）
- [ ] 错误处理测试
- [ ] 性能测试（如适用）

---

## 📈 测试覆盖率目标

### 当前状态（估算）

- 总体覆盖率：~15-20%
- 核心模块覆盖率：~30%
- 工具模块覆盖率：~10%
- 命令层覆盖率：~5%

### 目标状态

- 总体覆盖率：**80%+**
- 核心模块覆盖率：**90%+**
- 工具模块覆盖率：**85%+**
- 命令层覆盖率：**70%+**

### 覆盖率工具

使用 `cargo-tarpaulin` 或 `cargo-llvm-cov` 测量覆盖率：

```bash
# 安装
cargo install cargo-tarpaulin

# 运行覆盖率测试
cargo tarpaulin --out Html

# 查看报告
open tarpaulin-report.html
```

---

## 🚀 实施计划

### 第一步：准备工作（1-2 天）

1. 添加测试依赖到 `Cargo.toml`
2. 创建测试工具模块 `tests/common/`
3. 设置测试覆盖率工具
4. 创建测试目录结构

### 第二步：开始实施（按优先级）

1. **周 1-2：** HTTP 模块和工具函数测试
2. **周 3：** UI 组件基础测试
3. **周 4-5：** Git 模块测试
4. **周 6：** Jira 模块测试
5. **周 7：** PR 模块测试
6. **周 8：** 命令层和其他模块测试
7. **周 9：** 集成测试
8. **周 10：** 测试优化和覆盖率提升

### 第三步：持续改进

1. 每次添加新功能时同步添加测试
2. 定期审查测试覆盖率
3. 重构测试代码以提高可维护性
4. 优化测试执行时间

---

## 📚 参考资源

### Rust 测试文档

- [The Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust by Example - Testing](https://doc.rust-lang.org/rust-by-example/testing.html)

### 测试工具

- [mockito](https://docs.rs/mockito/) - HTTP mock 服务器
- [tempfile](https://docs.rs/tempfile/) - 临时文件/目录
- [cargo-tarpaulin](https://docs.rs/cargo-tarpaulin/) - 代码覆盖率

### 测试最佳实践

- [Rust Testing Best Practices](https://github.com/rust-lang/rfcs/blob/master/text/2318-test-attribute.md)
- [Testing in Rust](https://blog.rust-lang.org/2020/03/12/Rust-1.42.html)

---

## 📝 更新日志

- **2024-XX-XX：** 创建测试增强计划文档
- 后续更新将记录在此处

---

## ✅ 完成状态跟踪

### 阶段 1：核心基础设施测试
- [ ] HTTP 客户端测试
- [ ] HTTP 重试测试
- [ ] 工具函数测试
- [ ] Shell 模块测试

### 阶段 2：UI 组件测试
- [ ] 对话框组件测试
- [ ] 其他 UI 组件测试

### 阶段 3：Git 模块测试
- [ ] Git 操作测试

### 阶段 4：Jira 模块测试
- [ ] Jira 客户端测试
- [ ] Jira 辅助函数测试（扩展）

### 阶段 5：PR 模块测试
- [ ] PR 平台测试
- [ ] PR 辅助函数测试
- [ ] PR LLM 测试

### 阶段 6：其他模块测试
- [ ] 代理管理测试
- [ ] 回滚管理测试
- [ ] 补全生成测试
- [ ] Prompt 模块测试
- [ ] Settings 模块测试（扩展）

### 阶段 7：命令层测试
- [ ] 各命令模块测试

### 阶段 8：集成测试
- [ ] 端到端测试

---

**文档维护者：** 开发团队
**最后更新：** 2024-XX-XX
**版本：** 1.0
