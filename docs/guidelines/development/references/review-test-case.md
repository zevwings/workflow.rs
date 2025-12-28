# 测试用例审查指南

> 本文档专为 AI 助手设计，提供测试用例检查的核心原则和实用指导，帮助 AI 高效地进行测试覆盖分析和测试用例生成。

## 📋 目录

- [核心原则](#-核心原则)
- [检查目标](#-检查目标)
- [测试边界和范围](#-测试边界和范围)
- [检查流程](#-检查流程)
- [项目测试现状](#-项目测试现状)
- [测试示例](#-测试示例)
- [快速检查脚本](#-快速检查脚本)
- [检查报告模板](#-检查报告模板)

---

## 🎯 核心原则

**测试边界**：测试我们自己的业务逻辑，不测试外部依赖和第三方库。

**检查重点**：
- ✅ 业务逻辑、数据转换、状态管理、错误处理
- ✅ CLI 参数解析、用户交互、输出格式化
- ✅ API 封装、重试机制、响应处理
- ❌ 外部工具功能、第三方库实现、远程 API 业务逻辑

---

## 🎯 检查目标

测试用例检查的主要目标：

1. **确保测试覆盖完整**：所有业务逻辑、CLI 命令、错误处理都有对应的测试用例
2. **确保测试边界正确**：测试我们自己的代码，不测试外部依赖和第三方库
3. **确保测试质量**：测试用例合理、有效，使用合适的测试工具和方法
4. **识别缺失测试**：发现未测试的功能和边界情况
5. **优化测试结构**：确保测试组织清晰，易于维护

### 检查范围

- **单元测试**：`#[cfg(test)]` 模块中的测试
- **集成测试**：`tests/` 目录中的测试文件
- **文档测试**：文档中的代码示例（doctest）
- **测试工具使用**：是否使用推荐的测试工具（rstest、pretty_assertions、mockito、insta 等）

### 检查原则

1. **测试边界原则**：测试我们自己的业务逻辑，不测试外部依赖和第三方库
2. **全面性原则**：检查所有模块的测试覆盖情况
3. **质量原则**：确保测试用例合理、有效，使用合适的测试工具
4. **可操作性原则**：检查结果应提供明确的改进建议

---

## 🎯 测试边界和范围

> **核心原则**：我们应该测试自己的业务逻辑，而不是测试外部依赖和第三方库的实现。

### ✅ 需要测试的内容

#### 1. 业务逻辑层

测试我们自己实现的业务规则和处理逻辑：

- ✅ **业务规则实现**
  - 分支前缀处理（如 `feature/` + 分支名）
  - 合并策略选择（如根据配置选择 `--no-ff` 或 `--ff-only`）
  - 数据验证规则（如分支名称验证、配置验证）
  - 业务流程控制（如 PR 创建流程、日志工作流）

- ✅ **数据转换和处理**
  - JSON/TOML 数据解析后的转换
  - 数据格式化（如日期格式、标题生成）
  - 数据聚合和计算（如统计、汇总）

- ✅ **状态管理**
  - 状态转换逻辑（如 Git 仓库状态、工作树状态）
  - 状态解析（如 `WorktreeStatus` 结构体的正确性）
  - 状态验证（如检查是否有未提交的更改）

- ✅ **数据结构的正确性**
  - 自定义数据结构（如 `CommitInfo`、`WorktreeStatus`、`BranchInfo`）
  - 数据结构的序列化/反序列化
  - 数据结构的默认值和验证

#### 2. CLI 命令层测试

测试命令行接口的正确性和用户体验：

- ✅ **命令参数解析**
  - 参数验证逻辑（必需参数、可选参数、默认值）
  - 参数类型转换和格式验证（如 Jira ID 格式、PR 标题长度）
  - 参数组合的有效性检查（互斥参数、依赖参数）
  - 错误参数的处理和友好提示

- ✅ **命令执行流程**
  - 命令的主要执行路径和业务逻辑
  - 前置条件检查（Git 仓库、配置文件、网络连接）
  - 命令间的依赖关系处理和执行顺序
  - 命令执行的状态管理和错误恢复

- ✅ **用户交互测试**
  - Dialog 组件的配置和验证逻辑
  - 用户输入的处理和验证（输入框、选择框、多选框）
  - 交互流程的正确性（确认、取消、重试）
  - 用户体验的一致性（提示信息、错误处理）

- ✅ **输出格式化**
  - 多种输出格式的正确性（Table、JSON、YAML、Markdown）
  - 输出内容的一致性和完整性
  - 错误消息和警告信息的格式化
  - 国际化和本地化支持

**CLI 测试示例**：
```rust
// ✅ 测试命令参数解析（我们的业务逻辑）
#[test]
fn test-_pr-_create-_args-_validation() {
    let args = PrCreateArgs {
        jira-_ticket: Some("PROJ-123".to-_string()),
        title: Some("Test PR".to-_string()),
        description: None,
        dry-_run: false,
    };

    // 测试我们的参数验证逻辑
    let result = validate-_pr-_create-_args(&args);
    assert!(result.is-_ok());

    // 测试无效的 Jira ID
    let invalid-_args = PrCreateArgs {
        jira-_ticket: Some("invalid-id".to-_string()),
        ..args
    };
    let result = validate-_pr-_create-_args(&invalid-_args);
    assert!(result.is-_err());
    assert!(result.unwrap-_err().to-_string().contains("Jira ID 格式无效"));
}

// ✅ 测试用户交互逻辑（我们的业务逻辑）
#[test]
fn test-_branch-_selection-_dialog-_config() {
    let branches = vec!["main", "develop", "feature/test"];
    let dialog = create-_branch-_selection-_dialog(&branches);

    // 测试我们的 Dialog 配置逻辑
    assert-_eq!(dialog.prompt(), "选择目标分支:");
    assert-_eq!(dialog.options().len(), 3);
    assert-_eq!(dialog.default-_index(), Some(0));
    assert!(dialog.enable-_filter()); // 启用模糊匹配
}

// ✅ 测试输出格式化（我们的业务逻辑）
#[test]
fn test-_pr-_list-_output-_formats() {
    let prs = create-_mock-_pr-_list();

    // 测试表格格式
    let table-_output = format-_pr-_list-_as-_table(&prs);
    assert!(table-_output.contains("ID"));
    assert!(table-_output.contains("Title"));
    assert!(table-_output.contains("Status"));

    // 测试 JSON 格式
    let json-_output = format-_pr-_list-_as-_json(&prs);
    let parsed: serde-_json::Value = serde-_json::from-_str(&json-_output).unwrap();
    assert!(parsed.is-_array());
    assert-_eq!(parsed.as-_array().unwrap().len(), prs.len());
}

// ✅ 测试命令执行前置条件（我们的业务逻辑）
#[test]
fn test-_pr-_create-_preconditions() {
    // 测试 Git 仓库检查
    let temp-_dir = tempfile::tempdir().unwrap();
    std::env::set-_current-_dir(&temp-_dir).unwrap();

    let result = PrCreateCommand::validate-_git-_repo();
    assert!(result.is-_err());
    assert!(result.unwrap-_err().to-_string().contains("不是 Git 仓库"));

    // 测试配置文件检查
    let result = PrCreateCommand::validate-_github-_config();
    assert!(result.is-_err());
    assert!(result.unwrap-_err().to-_string().contains("GitHub 配置"));
}
```

#### 3. 错误处理逻辑

测试我们如何处理错误，而不是测试错误是否会发生：

- ✅ **异常情况处理**
  - 检查错误情况是否被正确捕获
  - 检查错误恢复机制是否正确
  - 检查失败后的清理逻辑

- ✅ **错误消息和上下文**
  - 检查错误消息是否清晰准确
  - 检查错误上下文是否完整（使用 `anyhow::Context`）
  - 检查错误类型是否正确传递

- ✅ **错误传播**
  - 检查错误是否正确向上传播
  - 检查错误转换是否正确（如将底层错误转换为业务错误）

#### 3. 边界条件

测试各种边界情况和特殊输入：

- ✅ **输入边界**
  - 空输入（空字符串、空数组、空配置）
  - 最大/最小值（长度限制、数值范围）
  - 特殊字符（Unicode、换行符、特殊符号）
  - 无效输入（格式错误、类型错误）

- ✅ **输出边界**
  - 空结果处理
  - 超大结果处理
  - 格式化输出的正确性

- ✅ **并发和异步场景**
  - 异步函数的正确性测试（使用 `#[tokio::test]`）
  - 并发执行的安全性测试（数据竞争、死锁检测）
  - 并发限制和资源管理（如并发执行器的限制）
  - 超时和取消机制测试（任务超时、用户取消）
  - 异步错误处理和传播
  - 并发场景下的状态一致性

#### 4. 集成逻辑

测试我们如何封装和使用外部依赖：

- ✅ **API 调用封装**
  - 参数构造和传递
  - 请求配置（headers、auth、timeout）
  - 响应处理和数据提取

- ✅ **数据解析和转换**
  - API 响应的解析
  - 数据映射到内部数据结构
  - 错误响应的处理

- ✅ **重试和容错机制**
  - 重试逻辑的正确性（指数退避、最大重试次数）
  - 可重试错误的判断
  - 重试失败的处理

### ❌ 不需要测试的内容

#### 1. 外部依赖和第三方库

**核心原则**：不要测试外部工具和库的实现，它们已经有自己的测试。

#### 判断依据

使用以下问题判断是否需要测试：

1. **这段代码是谁写的？**
   - ❌ 外部库/工具的作者 → 不需要测试
   - ✅ 我们自己的团队 → 需要测试

2. **这段代码在哪里维护？**
   - ❌ 在外部仓库（如 crates.io、GitHub、系统工具） → 不需要测试
   - ✅ 在我们的项目中 → 需要测试

3. **测试的目的是什么？**
   - ❌ 验证外部库是否按文档工作 → 不需要测试（信任外部库）
   - ✅ 验证我们的代码逻辑是否正确 → 需要测试

#### 不需要测试的典型场景

- ❌ **外部命令行工具的功能**
  - 例如：Git、Docker、npm 等命令的基本功能
  - 不要测试：命令本身是否正确执行
  - 应该测试：我们如何构建命令参数、如何解析命令输出

- ❌ **第三方库的 API 实现**
  - 例如：HTTP 客户端、数据库驱动、序列化库
  - 不要测试：库的内部实现和协议处理
  - 应该测试：我们如何配置和使用这些库

- ❌ **远程 API 服务的业务逻辑**
  - 例如：GitHub API、Jira API、云服务 API
  - 不要测试：API 是否返回正确的业务数据
  - 应该测试：我们如何调用 API、如何处理 API 响应

- ❌ **标准库和系统调用**
  - 例如：文件系统、进程管理、网络操作
  - 不要测试：标准库的正确性
  - 应该测试：我们的文件处理逻辑、错误处理

- ❌ **语言和框架的核心功能**
  - 例如：Rust 标准库、语言特性、编译器行为
  - 不要测试：语言本身的功能
  - 应该测试：我们使用这些功能的业务逻辑

#### 2. 测试策略

对于外部依赖，采用以下策略：

- ✅ **使用 Mock 和 Stub 隔离测试**
  - 使用 `mockito` Mock HTTP API
  - 使用测试工具模拟 Git 仓库状态
  - 使用 Stub 模拟外部依赖的返回值

- ✅ **测试我们的代码如何使用外部依赖**
  - 测试我们传递给外部依赖的参数是否正确
  - 测试我们如何处理外部依赖的返回值
  - 测试我们如何处理外部依赖的错误

- ✅ **测试边界和异常情况**
  - 测试外部依赖返回错误时的处理
  - 测试外部依赖返回异常数据时的处理
  - 测试外部依赖超时或不可用时的处理

### 📚 具体示例

#### 示例 0: 项目实际结构对照

**实际项目模块结构**：

##### Core 业务模块 (`src/lib/`)
- ✅ **Base 模块** (`lib/base/`): HTTP 客户端、LLM 客户端、Settings、Dialog、Logger 等基础组件
- ✅ **Git 模块** (`lib/git/`): 分支管理、提交管理、仓库操作、Stash 管理
- ✅ **Jira 模块** (`lib/jira/`): API 集成、日志管理、附件处理、状态管理
- ✅ **PR 模块** (`lib/pr/`): GitHub 平台集成、LLM 生成、Body 解析
- ✅ **Branch 模块** (`lib/branch/`): 分支命名、LLM 生成、同步管理
- ✅ **Commit 模块** (`lib/commit/`): 提交修改、重写、压缩
- ✅ **Template 模块** (`lib/template/`): 模板配置、引擎、变量管理
- ✅ **Proxy 模块** (`lib/proxy/`): 代理配置生成和管理
- ✅ **Repo 模块** (`lib/repo/`): 仓库配置管理
- ✅ **Rollback 模块** (`lib/rollback/`): 操作回滚

##### CLI 命令层 (`src/commands/`)
- ✅ **配置管理**: `config/`, `github/`, `check/`, `proxy/`, `llm/`
- ✅ **业务功能**: `pr/`, `jira/`, `branch/`, `commit/`, `stash/`, `log/`
- ✅ **工具管理**: `lifecycle/`, `migrate/`, `repo/`, `alias/`, `tag/`

**实际测试工具配置** (`Cargo.toml`):
```toml
[dev-dependencies]
pretty_assertions = "1.4"    # 清晰的断言输出
rstest = "0.18"             # 参数化测试和 fixtures
mockito = "1.2"             # HTTP API Mock 测试
insta = "1.38"              # 快照测试（JSON 功能）
assert-_cmd = "2.0"          # CLI 命令测试
predicates = "3.0"          # 断言谓词
tempfile = "3.8"            # 临时文件和目录
```

**实际测试目录结构**:
```
tests/
├── base/           # Base 模块测试（LLM、Settings、Dialog、Util）
├── cli/            # CLI 命令测试（所有 commands/ 对应测试）
│   ├── basic-_cli.rs        # 基础 CLI 测试
│   ├── integration-_cli.rs  # CLI 集成测试
│   └── [各命令测试文件]    # PR、Branch、Config 等
├── completion/     # 自动补全测试
├── git/            # Git 模块测试（目前为空，需要补充）
├── http/           # HTTP 客户端测试
├── integration/    # 集成测试
├── jira/           # Jira 模块测试
├── pr/             # PR 模块测试
├── proxy/          # Proxy 模块测试
├── rollback/       # Rollback 模块测试
├── common/         # 共享测试工具
│   ├── cli-_helpers.rs      # CLI 测试辅助工具
│   ├── helpers.rs          # 通用测试工具
│   └── http-_helpers.rs     # HTTP 测试工具
└── fixtures/       # 测试数据文件
```

**测试覆盖现状**:
- 🟢 **已完整覆盖**: Base 模块（LLM、Settings、Dialog）、CLI 参数解析、PR 模块、Jira 模块
- 🟢 **CLI 测试工具**: 已添加 assert-_cmd、predicates、tempfile 和完整的测试辅助工具
- 🟡 **部分覆盖**: HTTP 模块、Completion 模块、Proxy 模块、CLI 集成测试（基础框架已建立）
- 🔴 **缺失覆盖**: Git 模块（测试文件为空）、Template 模块、Branch 模块、Commit 模块、Stash 模块

#### 示例 1: Git 模块

**✅ 应该测试的**：

```rust
// ✅ 测试分支前缀处理逻辑（我们的业务逻辑）
#[test]
fn test-_format-_branch-_name-_with-_prefix() {
    let result = format-_branch-_name("feature", "login");
    assert-_eq!(result, "feature/login");
}

// ✅ 测试合并策略选择逻辑（我们的业务逻辑）
#[test]
fn test-_merge-_strategy-_selection() {
    let strategy = determine-_merge-_strategy(true, false);
    assert-_eq!(strategy, MergeStrategy::NoFastForward);
}

// ✅ 测试分支名称验证逻辑（我们的业务逻辑）
#[test]
fn test-_validate-_branch-_name() {
    assert!(validate-_branch-_name("feature/login").is-_ok());
    assert!(validate-_branch-_name("invalid//name").is-_err());
    assert!(validate-_branch-_name("").is-_err());
}

// ✅ 测试 Git 命令执行失败时的错误处理（我们的错误处理）
#[test]
fn test-_branch-_create-_error-_handling() {
    // 使用 Mock 模拟 Git 命令失败
    let result = GitBranch::create("invalid/name");
    assert!(result.is-_err());
    assert!(result.unwrap-_err().to-_string().contains("分支名称无效"));
}

// ✅ 测试 CommitInfo 数据结构解析（我们的数据处理）
#[test]
fn test-_parse-_commit-_info() {
    let output = "abc123\nJohn Doe\n2024-01-01\nInitial commit";
    let info = CommitInfo::from-_output(output).unwrap();
    assert-_eq!(info.hash, "abc123");
    assert-_eq!(info.author, "John Doe");
}
```

**❌ 不应该测试的**：

```rust
// ❌ 不要测试 Git 命令本身是否正确（这是 Git 的责任）
#[test]
fn test-_git-_branch-_command-_creates-_branch() {
    // 这是在测试 Git 本身，而不是我们的代码
    Command::new("git").args(["branch", "test"]).status().unwrap();
    let output = Command::new("git").args(["branch", "--list", "test"]).output().unwrap();
    assert!(String::from-_utf8_lossy(&output.stdout).contains("test"));
}

// ❌ 不要测试 Git 参数的功能（这是 Git 的责任）
#[test]
fn test-_git-_merge-_ff-_only-_parameter() {
    // 这是在测试 Git 的 --ff-only 参数，而不是我们的代码
    Command::new("git").args(["merge", "--ff-only", "feature"]).status().unwrap();
}

// ❌ 不要测试 Git 的底层实现（这是 Git 的责任）
#[test]
fn test-_git-_internal-_merge-_algorithm() {
    // 这是在测试 Git 的合并算法，而不是我们的代码
}
```

#### 示例 2: HTTP 模块

**✅ 应该测试的**：

```rust
// ✅ 测试请求配置构建逻辑（我们的业务逻辑）
#[test]
fn test-_build-_request-_with-_auth() {
    let client = HttpClient::new();
    let request = client.request("https://api.example.com")
        .with-_auth("token", "abc123")
        .build();

    assert!(request.headers().contains-_key("Authorization"));
    assert-_eq!(request.headers().get("Authorization").unwrap(), "Bearer abc123");
}

// ✅ 测试重试逻辑（我们的业务逻辑）
#[test]
fn test-_retry-_on-_network-_error() {
    let mut mock-_server = mockito::Server::new();
    let mock = mock-_server.mock("GET", "/api")
        .with-_status(500)
        .expect(3)  // 应该重试 3 次
        .create();

    let result = HttpClient::new().get(&format!("{}/api", mock-_server.url())).await;
    assert!(result.is-_err());
    mock.assert();
}

// ✅ 测试响应数据解析（我们的业务逻辑）
#[test]
fn test-_parse-_api-_response() {
    let json = r#"{"id": 123, "name": "test"}"#;
    let data: ApiResponse = serde-_json::from-_str(json).unwrap();
    assert-_eq!(data.id, 123);
    assert-_eq!(data.name, "test");
}
```

**❌ 不应该测试的**：

```rust
// ❌ 不要测试 reqwest 是否正确发送 HTTP 请求（这是 reqwest 的责任）
#[test]
fn test-_reqwest-_sends-_http-_request() {
    // 这是在测试 reqwest 库，而不是我们的代码
    let response = reqwest::blocking::get("https://httpbin.org/get").unwrap();
    assert-_eq!(response.status(), 200);
}

// ❌ 不要测试 HTTP 协议的正确性（这是标准协议）
#[test]
fn test-_http-_protocol() {
    // 这是在测试 HTTP 协议，而不是我们的代码
}
```

#### 示例 3: Jira 模块

**✅ 应该测试的**：

```rust
// ✅ 测试 Jira API 请求构建（我们的业务逻辑）
#[test]
fn test-_build-_jira-_search-_request() {
    let query = build-_jira-_query("PROJECT-123");
    assert-_eq!(query, "project = PROJECT AND key = PROJECT-123");
}

// ✅ 测试 Jira 响应数据转换（我们的业务逻辑）
#[test]
fn test-_convert-_jira-_issue-_to-_internal-_format() {
    let jira-_issue = mock-_jira-_issue();
    let issue = Issue::from-_jira-_response(jira-_issue);
    assert-_eq!(issue.key, "PROJECT-123");
    assert-_eq!(issue.summary, "Test Issue");
}

// ✅ 测试日志格式化（我们的业务逻辑）
#[test]
fn test-_format-_worklog() {
    let worklog = Worklog {
        time-_spent: 3600,
        comment: "Fixed bug",
    };
    let formatted = format-_worklog(&worklog);
    assert-_eq!(formatted, "1h - Fixed bug");
}
```

**❌ 不应该测试的**：

```rust
// ❌ 不要测试 Jira API 本身的功能（这是 Jira 的责任）
#[test]
fn test-_jira-_api-_returns-_correct-_issue() {
    // 这是在测试 Jira API，而不是我们的代码
    let issue = jira-_client.get-_issue("PROJECT-123").await.unwrap();
    assert-_eq!(issue.fields.summary, "Expected Summary");
}

// ❌ 不要测试 Jira 的业务逻辑（这是 Jira 的责任）
#[test]
fn test-_jira-_calculates-_time-_tracking() {
    // 这是在测试 Jira 的时间跟踪逻辑，而不是我们的代码
}
```

#### 示例 4: 并发和异步测试

**✅ 应该测试的**：

```rust
// ✅ 测试异步函数的正确性（我们的业务逻辑）
#[tokio::test]
async fn test-_concurrent-_http-_requests() {
    let client = HttpClient::new();
    let urls = vec!["url1", "url2", "url3"];

    // 测试我们的并发请求逻辑
    let results = client.fetch-_all(urls).await;
    assert-_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is-_ok()));
}

// ✅ 测试并发执行器的限制（我们的业务逻辑）
#[tokio::test]
async fn test-_concurrent-_executor-_limits() {
    let executor = ConcurrentExecutor::new(2); // 最大2个并发
    let tasks = create-_test-_tasks(5); // 5个任务

    // 测试我们的并发控制逻辑
    let start-_time = Instant::now();
    let results = executor.execute-_all(tasks).await;
    let duration = start-_time.elapsed();

    // 验证结果和并发限制
    assert-_eq!(results.len(), 5);
    assert!(duration >= Duration::from-_millis(500)); // 至少需要3轮执行
}

// ✅ 测试超时和取消机制（我们的业务逻辑）
#[tokio::test]
async fn test-_task-_timeout-_handling() {
    let executor = ConcurrentExecutor::new(1);
    let timeout-_task = create-_long-_running-_task(Duration::from-_secs(10));

    // 测试我们的超时处理逻辑
    let result = tokio::time::timeout(
        Duration::from-_millis(100),
        executor.execute(timeout-_task)
    ).await;

    assert!(result.is-_err()); // 应该超时
}

// ✅ 测试并发安全性（我们的业务逻辑）
#[tokio::test]
async fn test-_concurrent-_state-_consistency() {
    let shared-_state = Arc::new(Mutex::new(Vec::new()));
    let tasks = (0..10).map(|i| {
        let state = shared-_state.clone();
        tokio::spawn(async move {
            let mut guard = state.lock().await;
            guard.push(i);
        })
    }).collect::<Vec<_>>();

    // 等待所有任务完成
    for task in tasks {
        task.await.unwrap();
    }

    // 验证状态一致性
    let final-_state = shared-_state.lock().await;
    assert-_eq!(final-_state.len(), 10);
}

// ✅ 测试异步错误处理（我们的业务逻辑）
#[tokio::test]
async fn test-_async-_error-_propagation() {
    let client = HttpClient::new();

    // 测试我们的异步错误处理
    let result = client.fetch-_with-_retry("invalid-url", 3).await;
    assert!(result.is-_err());

    let error = result.unwrap-_err();
    assert!(error.to-_string().contains("网络请求失败"));
    assert!(error.to-_string().contains("重试 3 次后仍然失败"));
}
```

**❌ 不应该测试的**：

```rust
// ❌ 不要测试 tokio 运行时的正确性（这是 tokio 的责任）
#[tokio::test]
async fn test-_tokio-_runtime-_behavior() {
    // 这是在测试 tokio 本身，而不是我们的代码
    tokio::spawn(async {
        tokio::time::sleep(Duration::from-_millis(100)).await;
    }).await.unwrap();
}

// ❌ 不要测试标准库的并发原语（这是标准库的责任）
#[test]
fn test-_mutex-_locking() {
    // 这是在测试 Mutex 的实现，而不是我们的代码
    let mutex = Mutex::new(0);
    let guard = mutex.lock().unwrap();
    assert-_eq!(*guard, 0);
}
```

### 🎯 测试边界总结

| 测试类型 | 应该测试 ✅ | 不应该测试 ❌ |
|---------|------------|--------------|
| **业务逻辑** | 我们的业务规则、数据转换、状态管理 | 外部工具的业务逻辑 |
| **错误处理** | 我们如何处理错误、错误消息、错误恢复 | 外部工具是否会产生错误 |
| **数据结构** | 我们的数据结构、序列化、验证 | 标准库的数据结构 |
| **API 集成** | 我们如何调用 API、处理响应、错误处理 | API 本身的实现和正确性 |
| **Git 操作** | 我们的 Git 封装、参数构建、结果解析 | Git 命令本身的功能 |
| **HTTP 请求** | 我们的请求配置、重试逻辑、响应处理 | HTTP 客户端库的实现 |
| **CLI 命令** | 我们的参数解析、执行流程、用户交互 | clap 库本身的参数解析功能 |
| **并发异步** | 我们的并发控制、异步逻辑、错误处理 | tokio 运行时和标准库并发原语 |

**关键原则**：**测试我们自己写的代码，信任外部依赖已经过充分测试。**

---

## 🔄 检查流程

### 步骤 1：项目结构扫描

#### 1.1 收集源代码信息
```bash
# 列出所有 lib 模块
echo "=== Core 业务模块 (src/lib/) ==="
find src/lib -name "*.rs" -not -name "mod.rs" | sort

# 列出所有 commands 模块
echo "=== CLI 命令模块 (src/commands/) ==="
find src/commands -name "*.rs" -not -name "mod.rs" | sort

# 统计模块数量
echo "Core 模块数量: $(find src/lib -name "*.rs" -not -name "mod.rs" | wc -l)"
echo "Commands 模块数量: $(find src/commands -name "*.rs" -not -name "mod.rs" | wc -l)"
```

#### 1.2 收集测试文件信息
```bash
# 列出所有测试文件
echo "=== 测试文件 (tests/) ==="
find tests -name "*.rs" | sort

# 统计测试文件数量
echo "测试文件数量: $(find tests -name "*.rs" | wc -l)"

# 检查测试目录结构
echo "=== 测试目录结构 ==="
tree tests/ -I "target|snapshots" 2>/dev/null || find tests -type d | sort
```

### 步骤 2：覆盖情况检查

#### 2.1 模块覆盖对比
```bash
# 创建模块覆盖检查脚本
cat > check-_coverage.sh << 'EOF'
#!/bin/bash
echo "=== 模块覆盖情况检查 ==="

echo "🟢 已覆盖的模块:"
for lib-_file in $(find src/lib -name "*.rs" -not -name "mod.rs"); do
    module-_name=$(basename $(dirname $lib-_file))
    test-_file="tests/${module-_name}/mod.rs"
    if [[ -f "$test-_file" ]] && [[ $(grep -c "#\[test\]" "$test-_file" 2>/dev/null || echo 0) -gt 0 ]]; then
        echo "  ✅ $module-_name ($(basename $lib-_file))"
    fi
done

echo ""
echo "🟡 部分覆盖的模块:"
for lib-_file in $(find src/lib -name "*.rs" -not -name "mod.rs"); do
    module-_name=$(basename $(dirname $lib-_file))
    test-_file="tests/${module-_name}/mod.rs"
    if [[ -f "$test-_file" ]] && [[ $(grep -c "#\[test\]" "$test-_file" 2>/dev/null || echo 0) -eq 0 ]]; then
        echo "  ⚠️  $module-_name (测试文件存在但为空)"
    fi
done

echo ""
echo "🔴 缺失覆盖的模块:"
for lib-_file in $(find src/lib -name "*.rs" -not -name "mod.rs"); do
    module-_name=$(basename $(dirname $lib-_file))
    test-_file="tests/${module-_name}/mod.rs"
    if [[ ! -f "$test-_file" ]]; then
        echo "  ❌ $module-_name (无测试文件)"
    fi
done
EOF

chmod +x check-_coverage.sh
./check-_coverage.sh
```

#### 2.2 功能覆盖检查
```bash
# 检查公共函数覆盖情况
echo "=== 公共函数覆盖检查 ==="
for module in src/lib/*/mod.rs; do
    module-_name=$(basename $(dirname $module))
    echo "检查模块: $module-_name"

    # 提取公共函数
    pub-_functions=$(grep -n "pub fn " $module 2>/dev/null | head -5)
    if [[ -n "$pub-_functions" ]]; then
        echo "  公共函数:"
        echo "$pub-_functions" | sed 's/^/    /'

        # 检查对应测试
        test-_file="tests/${module-_name}/mod.rs"
        if [[ -f "$test-_file" ]]; then
            test-_count=$(grep -c "#\[test\]" "$test-_file" 2>/dev/null || echo 0)
            echo "  测试用例数量: $test-_count"
        else
            echo "  ❌ 无测试文件"
        fi
    fi
    echo ""
done
```

### 步骤 3：测试质量评估

#### 3.1 测试工具使用检查
```bash
echo "=== 测试工具使用情况 ==="

# 检查 rstest 使用
rstest-_count=$(grep -r "#\[rstest\]" tests/ 2>/dev/null | wc -l)
echo "📊 rstest 参数化测试: $rstest-_count 个"

# 检查 pretty_assertions 使用
pretty_assertions=$(grep -r "use pretty_assertions" tests/ 2>/dev/null | wc -l)
echo "📊 pretty_assertions 使用: $pretty_assertions 个文件"

# 检查 insta 快照测试
insta-_count=$(grep -r "insta::" tests/ 2>/dev/null | wc -l)
echo "📊 insta 快照测试: $insta-_count 个"

# 检查 mockito Mock 测试
mockito-_count=$(grep -r "mockito::" tests/ 2>/dev/null | wc -l)
echo "📊 mockito Mock 测试: $mockito-_count 个"

# 检查 tokio 异步测试
tokio-_test-_count=$(grep -r "#\[tokio::test\]" tests/ 2>/dev/null | wc -l)
echo "📊 tokio 异步测试: $tokio-_test-_count 个"
```

#### 3.2 测试结构检查
```bash
echo "=== 测试结构和质量检查 ==="

# 检查测试命名规范
echo "🔍 测试命名规范检查:"
non-_standard-_tests=$(grep -r "fn test" tests/ | grep -v "fn test_" | wc -l)
if [[ $non-_standard-_tests -eq 0 ]]; then
    echo "  ✅ 所有测试都遵循 test_ 命名规范"
else
    echo "  ⚠️  发现 $non-_standard-_tests 个不规范的测试命名"
fi

# 检查测试文档注释
documented-_tests=$(grep -r "/// " tests/ 2>/dev/null | wc -l)
total-_tests=$(grep -r "#\[test\]" tests/ 2>/dev/null | wc -l)
echo "📝 测试文档覆盖: $documented-_tests/$total-_tests"

# 检查错误处理测试
error-_tests=$(grep -r "assert.*is-_err\|expect.*err\|unwrap-_err" tests/ 2>/dev/null | wc -l)
echo "🚨 错误处理测试: $error-_tests 个"

# 检查边界条件测试
boundary-_tests=$(grep -r "empty\|null\|zero\|max\|min\|boundary" tests/ 2>/dev/null | wc -l)
echo "🎯 边界条件测试: $boundary-_tests 个"
```

### 步骤 4：缺失测试识别

#### 4.1 生成缺失测试报告
```bash
cat > generate-_missing-_tests-_report.sh << 'EOF'
#!/bin/bash
echo "# 缺失测试分析报告"
echo ""
echo "## 📊 统计概览"
echo ""

# 统计总体情况
total-_lib-_modules=$(find src/lib -name "*.rs" -not -name "mod.rs" | wc -l)
total-_test-_files=$(find tests -name "*.rs" | wc -l)
total-_tests=$(grep -r "#\[test\]" tests/ 2>/dev/null | wc -l)

echo "- **Core 模块总数**: $total-_lib-_modules"
echo "- **测试文件总数**: $total-_test-_files"
echo "- **测试用例总数**: $total-_tests"
echo ""

# 计算覆盖率
covered-_modules=0
for lib-_file in $(find src/lib -name "*.rs" -not -name "mod.rs"); do
    module-_name=$(basename $(dirname $lib-_file))
    test-_file="tests/${module-_name}/mod.rs"
    if [[ -f "$test-_file" ]] && [[ $(grep -c "#\[test\]" "$test-_file" 2>/dev/null || echo 0) -gt 0 ]]; then
        covered-_modules=$((covered-_modules + 1))
    fi
done

coverage-_percent=$(echo "scale=1; $covered-_modules * 100 / $total-_lib-_modules" | bc -l 2>/dev/null || echo "0")
echo "- **模块覆盖率**: $covered-_modules/$total-_lib-_modules ($coverage-_percent%)"
echo ""

echo "## 🔴 完全缺失测试的模块"
echo ""
for lib-_file in $(find src/lib -name "*.rs" -not -name "mod.rs"); do
    module-_name=$(basename $(dirname $lib-_file))
    test-_file="tests/${module-_name}/mod.rs"
    if [[ ! -f "$test-_file" ]]; then
        echo "- ❌ **$module-_name** (\`$(basename $lib-_file)\`)"
        # 尝试识别主要功能
        pub-_functions=$(grep "pub fn " $lib-_file 2>/dev/null | head -3 | sed 's/.*pub fn \([^(]*\).*/  - \1()/')
        if [[ -n "$pub-_functions" ]]; then
            echo "  - 主要功能:"
            echo "$pub-_functions"
        fi
        echo ""
    fi
done

echo "## 🟡 测试文件为空的模块"
echo ""
for lib-_file in $(find src/lib -name "*.rs" -not -name "mod.rs"); do
    module-_name=$(basename $(dirname $lib-_file))
    test-_file="tests/${module-_name}/mod.rs"
    if [[ -f "$test-_file" ]] && [[ $(grep -c "#\[test\]" "$test-_file" 2>/dev/null || echo 0) -eq 0 ]]; then
        echo "- ⚠️  **$module-_name** (测试文件存在但无实际测试)"
        echo ""
    fi
done

echo "## 🟢 已完整覆盖的模块"
echo ""
for lib-_file in $(find src/lib -name "*.rs" -not -name "mod.rs"); do
    module-_name=$(basename $(dirname $lib-_file))
    test-_file="tests/${module-_name}/mod.rs"
    if [[ -f "$test-_file" ]] && [[ $(grep -c "#\[test\]" "$test-_file" 2>/dev/null || echo 0) -gt 0 ]]; then
        test-_count=$(grep -c "#\[test\]" "$test-_file")
        echo "- ✅ **$module-_name** ($test-_count 个测试)"
    fi
done
EOF

chmod +x generate-_missing-_tests-_report.sh
./generate-_missing-_tests-_report.sh
```

### 步骤 5：生成检查报告

#### 5.1 创建完整报告
```bash
cat > generate-_full-_report.sh << 'EOF'
#!/bin/bash
REPORT_FILE="report/TEST_COVERAGE_REPORT_$(date +%Y%m%d_%H%M%S).md"
mkdir -p report

echo "# 测试用例检查报告" > $REPORT_FILE
echo "" >> $REPORT_FILE
echo "**生成时间**: $(date '+%Y-%m-%d %H:%M:%S')" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# 执行所有检查并写入报告
echo "## 📈 覆盖情况总结" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# 这里可以调用之前的检查脚本并将结果追加到报告中
./check-_coverage.sh >> $REPORT_FILE 2>&1
echo "" >> $REPORT_FILE

./generate-_missing-_tests-_report.sh >> $REPORT_FILE 2>&1

echo "## 🛠️ 改进建议" >> $REPORT_FILE
echo "" >> $REPORT_FILE
echo "### 高优先级" >> $REPORT_FILE
echo "- [ ] 补充 Git 模块核心功能测试" >> $REPORT_FILE
echo "- [ ] 添加 Template 模块测试" >> $REPORT_FILE
echo "- [ ] 实现 Branch 模块测试" >> $REPORT_FILE
echo "" >> $REPORT_FILE
echo "### 中优先级" >> $REPORT_FILE
echo "- [ ] 完善 HTTP 模块重试逻辑测试" >> $REPORT_FILE
echo "- [ ] 增强 Proxy 模块配置验证测试" >> $REPORT_FILE
echo "" >> $REPORT_FILE

echo "报告已生成: $REPORT_FILE"
EOF

chmod +x generate-_full-_report.sh
./generate-_full-_report.sh
```

---

## 📊 项目测试现状

### 测试工具配置
```toml
[dev-dependencies]
pretty_assertions = "1.4"    # 清晰的断言输出
rstest = "0.18"             # 参数化测试
mockito = "1.2"             # HTTP Mock 测试
insta = "1.38"              # 快照测试
tempfile = "3.8"            # 临时文件管理
```

### 测试目录结构
```
tests/
├── base/           # Base 模块测试
├── cli/            # CLI 命令测试
├── git/            # Git 模块测试（目前为空）
├── jira/           # Jira 模块测试
├── pr/             # PR 模块测试
├── common/         # 共享测试工具
└── fixtures/       # 测试数据文件
```

### 当前覆盖状态
- 🟢 **已完整覆盖**：Base（LLM、Settings、Dialog）、PR、Jira、CLI 参数解析
- 🟡 **部分覆盖**：HTTP、Completion、Proxy
- 🔴 **缺失覆盖**：Git、Template、Branch、Commit、Stash

---
## 🎯 测试示例

### ✅ 好的测试示例

```rust
// ✅ 测试业务逻辑：分支名称格式化
#[test]
fn test-_format-_branch-_name() {
    assert-_eq!(format-_branch-_name("feature", "login"), "feature/login");
    assert-_eq!(format-_branch-_name("", "test"), "test");
}

// ✅ 测试错误处理：参数验证
#[test]
fn test-_validate-_jira-_id() {
    assert!(validate-_jira-_id("PROJ-123").is-_ok());
    assert!(validate-_jira-_id("invalid").is-_err());
}

// ✅ 使用 Mock 测试：HTTP API 调用
#[test]
async fn test-_github-_api-_call() {
    let mut server = mockito::Server::new();
    let mock = server.mock("GET", "/repos/owner/repo")
        .with-_status(200)
        .with-_body(r#"{"name": "repo"}"#)
        .create();

    let result = github-_client.get-_repo("owner/repo").await;
    assert!(result.is-_ok());
    mock.assert();
}

// ✅ 参数化测试：多种输入验证
#[rstest]
#[case("feature/test", true)]
#[case("invalid//name", false)]
#[case("", false)]
fn test-_branch-_name-_validation(#[case] name: &str, #[case] expected: bool) {
    assert-_eq!(is-_valid-_branch-_name(name), expected);
}
```

### ❌ 避免的测试

```rust
// ❌ 不要测试外部工具
#[test]
fn test-_git-_command() {
    Command::new("git").args(["status"]).status().unwrap();
}

// ❌ 不要测试第三方库
#[test]
fn test-_reqwest-_http() {
    reqwest::blocking::get("https://api.github.com").unwrap();
}
```

---

## 📋 快速检查脚本

### 检查测试覆盖
```bash
# 检查缺失的测试文件
for module in src/lib/*/mod.rs; do
    test-_file="tests/$(basename $(dirname $module))/mod.rs"
    if [[ ! -f "$test-_file" ]]; then
        echo "❌ 缺失测试: $module"
    fi
done

# 检查空的测试文件
find tests -name "*.rs" -exec sh -c 'if [ $(grep -c "#\[test\]" "$1") -eq 0 ]; then echo "⚠️  空测试文件: $1"; fi' _ {} \;
```

### 检查测试工具使用
```bash
# 检查是否使用推荐的测试工具
grep -r "use pretty_assertions" tests/ || echo "❌ 未使用 pretty_assertions"
grep -r "#\[rstest\]" tests/ || echo "❌ 未使用 rstest"
grep -r "insta::" tests/ || echo "❌ 未使用 insta"
grep -r "mockito::" tests/ || echo "❌ 未使用 mockito"
```

---

## 📊 检查报告模板

```markdown
# 测试用例检查报告

## 📈 覆盖情况总结
- **总测试文件数**: X 个
- **已覆盖模块**: X/Y (Z%)
- **测试用例总数**: ~X 个

## 🟢 已完整覆盖
- Base 模块 (LLM、Settings、Dialog)
- PR 模块 (GitHub 集成、LLM 生成)
- Jira 模块 (API 集成、日志管理)

## 🟡 部分覆盖
- HTTP 模块 (缺少重试逻辑测试)
- Proxy 模块 (缺少配置验证测试)

## 🔴 缺失覆盖
- Git 模块 (测试文件为空)
- Template 模块 (无测试文件)
- Branch 模块 (无测试文件)

## 🛠️ 改进建议
1. **优先级 1**: 补充 Git 模块核心功能测试
2. **优先级 2**: 添加 Template 和 Branch 模块测试
3. **优先级 3**: 完善 HTTP 和 Proxy 模块测试

## 📋 行动计划
- [ ] 创建 Git 模块测试框架
- [ ] 实现分支管理功能测试
- [ ] 添加提交管理功能测试
- [ ] 补充错误处理和边界条件测试
```

---

## 📚 参考文档

- [测试规范指南](../../testing/README.md) - 详细的测试组织和最佳实践
- [开发规范索引](../../development/README.md) - 开发规范总览
- [提交前检查指南](../workflows/pre-commit.md) - 测试检查的简要说明

---

## 📋 检查清单

### 测试边界检查清单

- [ ] 已明确测试边界：测试我们自己的业务逻辑，不测试外部依赖
- [ ] 已识别需要测试的内容（业务逻辑、CLI、错误处理等）
- [ ] 已识别不需要测试的内容（外部工具、第三方库等）

### 测试覆盖检查清单

- [ ] 所有 Lib 层模块都有对应的测试文件
- [ ] 所有 Commands 层模块都有对应的测试文件
- [ ] 新增的公共函数是否有单元测试？
- [ ] 新增的 CLI 命令是否有集成测试？
- [ ] 错误处理路径是否有测试？
- [ ] 边界情况是否有测试？

### 测试质量检查清单

- [ ] 是否使用了推荐的测试工具（rstest、pretty_assertions、mockito、insta 等）？
- [ ] 测试命名是否遵循 `test_` 规范？
- [ ] 测试是否有文档注释？
- [ ] 测试结构是否清晰，易于维护？

### 测试工具使用检查清单

- [ ] 是否使用 `rstest` 进行参数化测试？
- [ ] 是否使用 `pretty_assertions` 提供清晰的断言输出？
- [ ] 是否使用 `mockito` 进行 HTTP API Mock 测试？
- [ ] 是否使用 `insta` 进行快照测试（如适用）？
- [ ] 是否使用 `tokio::test` 进行异步测试（如适用）？

### 缺失测试识别检查清单

- [ ] 已识别完全缺失测试的模块
- [ ] 已识别测试文件为空的模块
- [ ] 已识别部分覆盖的模块
- [ ] 已生成缺失测试报告

---

**最后更新**: 2025-12-23
