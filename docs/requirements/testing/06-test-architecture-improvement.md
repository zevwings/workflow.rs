# 需求：测试架构系统化改进

**优先级**: 🟡 中
**类型**: Infrastructure Improvement
**影响范围**: 全项目测试体系
**预计工作量**: 9-13 天

## 📌 Rust 测试架构核心原则

> **⚠️ 重要说明：测试代码的正确组织方式**
>
> 在开始之前，必须理解 Rust 的测试架构原则：
>
> ### 三种测试类型
>
> | 类型 | 位置 | 用途 | 可见性 |
> |------|------|------|--------|
> | **单元测试** | `src/module.rs` 中的 `#[cfg(test)] mod tests` | 测试单个函数/结构体 | 可访问私有成员 |
> | **集成测试** | `tests/` 目录 | 端到端测试，测试公共 API | 只能访问公共接口 |
> | **测试辅助工具** | `src/testing/` 或 `src/test_utils/` | 提供测试数据工厂、Mock 等 | 通过 feature 控制 |
>
> ### ❌ 常见错误
>
> ```rust
> // ❌ 错误：不能从 src/ 导入 tests/ 的代码
> use tests::common::TestDataFactory;  // 编译错误！
>
> // ❌ 错误：tests/ 目录不应该有可复用的工具代码
> // tests/ 只用于集成测试，不是工具库
> ```
>
> ### ✅ 正确做法
>
> ```rust
> // ✅ 测试工具放在 crate 的 src/testing/ 中
> // crates/http/src/lib.rs
> #[cfg(any(test, feature = "testing"))]
> pub mod testing;
>
> // 单元测试中使用
> #[cfg(test)]
> mod tests {
>     use crate::testing::TestDataFactory;
> }
>
> // 其他 crate 使用（需要 features = ["testing"]）
> use http::testing::TestDataFactory;
> ```
>
> ### 本文档采用的方案
>
> - ✅ **测试工具** → `crates/*/src/testing/`（使用 feature flag）
> - ✅ **测试数据** → `tests/fixtures/`（只用于集成测试）
> - ✅ **单元测试** → `#[cfg(test)] mod tests`（与代码在同一文件）
>
> 这与项目现有的 `storage` crate 的模式一致。

---

## 问题描述

当前项目的测试架构虽然有基础设施（已配置 Criterion、cargo-tarpaulin、mockito 等工具），但缺乏系统化的测试数据管理、覆盖率监控和性能回归检测机制。

### 当前状态

**✅ 已有的良好基础**:
```
项目测试代码分布：
├── crates/storage/
│   ├── tests/stress_tests.rs           # ✓ 压力测试（360+ 行，但全部标记 #[ignore]）
│   ├── benches/git_services_bench.rs   # ✓ 性能基准测试（410+ 行）
│   └── src/git/testing.rs              # ✓ 测试辅助工具
├── crates/http/src/mock/server.rs      # ✓ Mock HTTP 服务器（60 行，功能简单）
├── make/Makefile.test.mk               # ✓ 测试命令（test, coverage）
├── make/Makefile.bench.mk              # ✓ 基准测试命令
└── .github/workflows/ci.yml            # ✓ CI 代码质量检查
```

**❌ 缺失的关键部分**:
```
缺少的测试基础设施：
├── tests/                              # ❌ 没有根级别测试目录
│   ├── fixtures/                       # ❌ 没有测试数据目录
│   │   ├── templates/                  # ❌ 没有数据模板
│   │   ├── scenarios/                  # ❌ 没有测试场景
│   │   └── mock_responses/             # ❌ 没有预定义 mock 响应
│   └── common/                         # ❌ 没有共享测试工具
│       ├── test_data_factory.rs        # ❌ 没有测试数据工厂
│       └── mock_server.rs              # ❌ 没有 Mock 服务器管理器
├── coverage.toml                       # ❌ 没有覆盖率配置
├── benches/cli_performance.rs          # ❌ 没有 CLI 性能测试
└── .git/hooks/pre-commit               # ❌ 没有预提交钩子
```

### 具体问题

**1. 测试覆盖率未监控**
- ❌ CI 中没有覆盖率报告作业
- ❌ 没有覆盖率阈值配置（目标 80%）
- ❌ 没有覆盖率趋势分析

**2. 测试数据管理混乱**
- ❌ 测试数据硬编码在各个测试文件中
- ❌ 没有统一的测试数据工厂模式
- ❌ Mock 服务器功能简单，缺乏预定义场景
- ⚠️ 重复的测试仓库创建代码（每个测试都在 setup）

**3. 性能测试不系统**
- ✅ 有 `git_services_bench.rs`（Commit/Branch/Blame/Tag）
- ❌ 缺少 CLI 启动时间、命令解析性能测试
- ❌ 缺少网络操作性能测试
- ❌ 没有性能回归检测机制

**4. 压力测试未充分利用**
- ⚠️ `stress_tests.rs` 全部标记 `#[ignore]`，默认不运行
- ❌ CI 中没有压力测试作业
- ⚠️ 开发者需要手动运行才能发现并发问题

**5. CI/CD 集成不完整**
- ✅ 有代码质量检查（rustfmt, clippy）
- ❌ 没有覆盖率报告上传（Codecov）
- ❌ 没有性能基准监控
- ❌ 没有预提交钩子防止低质量代码

## 为什么需要修改

### 1. **质量保障不足**

**当前风险**:
- 没有覆盖率监控，无法知道哪些代码未被测试
- 性能回归只能事后发现（用户报告慢了）
- 测试数据不一致，可能遗漏边缘场景

**影响**:
- 生产环境 bug 风险增加
- 性能问题难以提前发现
- 重构时缺乏安全网

### 2. **开发效率低下**

**问题场景**:
```rust
// ❌ 当前：每个测试都要手动创建测试数据
#[test]
fn test_create_pr() {
    let pr_data = json!({
        "title": "Test PR",
        "body": "Test body",
        "head": "feature",
        "base": "main"
    });
    // 每次都要重复这些代码...
}

// ✅ 改进后：使用测试数据工厂
#[test]
fn test_create_pr() {
    let pr = TestDataFactory::github_pr()
        .with_title("Test PR")
        .build();
    // 简洁、可复用
}
```

### 3. **测试可维护性差**

**当前问题**:
- 测试数据散落在各处，修改困难
- Mock 服务器每次都要重新配置
- 缺乏测试最佳实践文档

**后果**:
- 新人上手困难
- 测试代码腐化速度快于业务代码
- 测试失败难以调试

## 解决方案

### 方案概览

分 5 个阶段系统化改进测试架构：

| 阶段 | 内容 | 工作量 | 优先级 |
|------|------|--------|--------|
| 阶段1 | 测试覆盖率监控系统 | 2-3天 | 🔴 高 |
| 阶段2 | 测试数据管理优化 | 3-4天 | 🔴 高 |
| 阶段3 | 系统化性能测试 | 2-3天 | 🟡 中 |
| 阶段4 | CI/CD 集成 | 1-2天 | 🔴 高 |
| 阶段5 | 文档和培训 | 1天 | 🟢 低 |

---

### 阶段1：测试覆盖率监控系统 (2-3天)

#### 1.1 创建覆盖率配置文件

**新增文件**: `coverage.toml`
```toml
# 覆盖率配置
[coverage]
# 整体目标
target = 80.0

# 最低要求
minimum = 75.0

# 分模块目标
[coverage.modules]
core = 85.0
cli = 75.0
storage = 85.0
services = 80.0
```

**修改文件**: 根 `Cargo.toml`
```toml
[workspace.package.metadata.tarpaulin]
target-coverage = 80.0
exclude = [
    "src/bin/*",
    "tests/*",
    "benches/*",
    "src/*/mod.rs"
]
output = ["Html", "Lcov", "Json"]
out = "coverage/"
run-types = ["Tests", "Doctests"]
```

**修改文件**: `.gitignore`
```gitignore
# 添加覆盖率相关
/coverage/
*.profraw
```

#### 1.2 增强 Makefile 覆盖率命令

**修改文件**: `make/Makefile.test.mk`
```makefile
# 添加覆盖率阈值检查
coverage-check: check-tarpaulin
	@echo "检查覆盖率是否达标..."
	cargo tarpaulin --skip-clean --out Json --output-dir coverage \
		--exclude-files "src/bin/*" \
		--exclude-files "tests/*" \
		--exclude-files "benches/*"
	@python3 scripts/check_coverage.py coverage/tarpaulin-report.json 75

# 覆盖率趋势分析（需要历史数据）
coverage-trend: check-tarpaulin
	@echo "生成覆盖率趋势报告..."
	@mkdir -p coverage/history
	cargo tarpaulin --skip-clean --out Json --output-dir coverage
	@cp coverage/tarpaulin-report.json coverage/history/$(shell date +%Y%m%d_%H%M%S).json
	@python3 scripts/analyze_coverage_trend.py coverage/history/
```

#### 1.3 新增覆盖率检查脚本

**新增文件**: `scripts/check_coverage.py`
```python
#!/usr/bin/env python3
"""检查覆盖率是否达标"""
import sys
import json

def check_coverage(report_file, threshold):
    with open(report_file) as f:
        data = json.load(f)

    coverage = data.get('coverage', 0)
    if coverage < threshold:
        print(f"❌ 覆盖率不达标: {coverage:.2f}% < {threshold}%")
        sys.exit(1)

    print(f"✅ 覆盖率达标: {coverage:.2f}% >= {threshold}%")
    sys.exit(0)

if __name__ == '__main__':
    check_coverage(sys.argv[1], float(sys.argv[2]))
```

---

### 阶段2：测试数据管理优化 (3-4天)

> **⚠️ Rust 最佳实践说明**：
>
> 测试辅助工具不应该放在 `tests/` 目录中，因为 `tests/` 是用于集成测试的，不应该被主代码导入。
>
> **正确的做法**（项目已采用此模式）：
> 1. **测试辅助工具** → 放在各 crate 的 `src/testing/` 或 `src/test_utils/` 模块中
> 2. **使用 feature flag** → `#[cfg(any(test, feature = "testing"))]` 控制编译
> 3. **集成测试** → 放在 `tests/` 目录，只用于端到端测试
> 4. **测试数据 fixtures** → 放在 `tests/fixtures/`，只给集成测试使用

#### 2.1 创建测试辅助工具目录结构

**方案 A：在现有 crates 中添加测试工具**（推荐，项目已采用此模式）

```bash
# 扩展现有的测试工具
crates/storage/src/git/testing.rs        # ✅ 已存在，扩展它
crates/http/src/testing/                 # 新增 HTTP 测试工具
crates/services/src/testing/             # 新增 Services 测试工具

# 集成测试的 fixtures（只用于 tests/ 目录）
tests/fixtures/
├── templates/                           # JSON 模板
├── scenarios/                           # 测试场景
└── mock_responses/                      # Mock 响应数据
```

**方案 B：创建独立的测试工具 crate**（可选，适合大型项目）

```bash
crates/test-utils/                       # 新 crate
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── data_factory.rs
    └── mock_server.rs
```

**本文档采用方案 A**（与项目现有模式一致）

#### 2.2 扩展 HTTP 测试工具模块

**新增目录**: `crates/http/src/testing/`

**新增文件**: `crates/http/src/testing/mod.rs`
```rust
//! HTTP 测试辅助工具
//!
//! 提供 Mock 服务器管理和测试数据工厂

mod mock_server;
mod data_factory;

pub use mock_server::MockServerManager;
pub use data_factory::{TestDataFactory, GitHubPRBuilder, JiraIssueBuilder};
```

**新增文件**: `crates/http/src/testing/data_factory.rs`

> 💡 **为什么放在 `crates/http/src/testing/` 而不是 `tests/common/`？**
>
> 因为 `tests/` 目录是用于集成测试的，不能被其他模块导入。
> 测试工具需要在多个地方使用（单元测试、集成测试、基准测试），
> 所以应该作为 crate 的一部分，通过 feature flag 控制编译。

```rust
//! 测试数据工厂
//!
//! 提供统一的测试数据创建接口

use serde_json::{json, Value};

/// 测试数据工厂
pub struct TestDataFactory;

impl TestDataFactory {
    /// 创建 GitHub PR 构建器
    pub fn github_pr() -> GitHubPRBuilder {
        GitHubPRBuilder::default()
    }

    /// 创建 Jira Issue 构建器
    pub fn jira_issue() -> JiraIssueBuilder {
        JiraIssueBuilder::default()
    }

    /// 创建配置构建器
    pub fn config() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

/// GitHub PR 构建器
#[derive(Default)]
pub struct GitHubPRBuilder {
    title: Option<String>,
    body: Option<String>,
    head: Option<String>,
    base: Option<String>,
}

impl GitHubPRBuilder {
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn with_head(mut self, head: impl Into<String>) -> Self {
        self.head = Some(head.into());
        self
    }

    pub fn with_base(mut self, base: impl Into<String>) -> Self {
        self.base = Some(base.into());
        self
    }

    pub fn build(self) -> Value {
        json!({
            "title": self.title.unwrap_or_else(|| "Test PR".to_string()),
            "body": self.body.unwrap_or_else(|| "Test body".to_string()),
            "head": self.head.unwrap_or_else(|| "feature".to_string()),
            "base": self.base.unwrap_or_else(|| "main".to_string()),
            "state": "open",
            "number": 1,
            "user": {
                "login": "testuser",
                "id": 1
            }
        })
    }
}

/// Jira Issue 构建器
#[derive(Default)]
pub struct JiraIssueBuilder {
    summary: Option<String>,
    description: Option<String>,
    issue_type: Option<String>,
}

impl JiraIssueBuilder {
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_issue_type(mut self, issue_type: impl Into<String>) -> Self {
        self.issue_type = Some(issue_type.into());
        self
    }

    pub fn build(self) -> Value {
        json!({
            "fields": {
                "summary": self.summary.unwrap_or_else(|| "Test Issue".to_string()),
                "description": self.description.unwrap_or_else(|| "Test description".to_string()),
                "issuetype": {
                    "name": self.issue_type.unwrap_or_else(|| "Task".to_string())
                },
                "project": {
                    "key": "TEST"
                }
            }
        })
    }
}

/// 配置构建器
#[derive(Default)]
pub struct ConfigBuilder {
    github_token: Option<String>,
    jira_url: Option<String>,
}

impl ConfigBuilder {
    pub fn with_github_token(mut self, token: impl Into<String>) -> Self {
        self.github_token = Some(token.into());
        self
    }

    pub fn with_jira_url(mut self, url: impl Into<String>) -> Self {
        self.jira_url = Some(url.into());
        self
    }

    pub fn build(self) -> Value {
        json!({
            "github": {
                "token": self.github_token.unwrap_or_else(|| "test_token".to_string())
            },
            "jira": {
                "url": self.jira_url.unwrap_or_else(|| "https://test.atlassian.net".to_string())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_pr_builder() {
        let pr = TestDataFactory::github_pr()
            .with_title("My PR")
            .with_head("feature-branch")
            .build();

        assert_eq!(pr["title"], "My PR");
        assert_eq!(pr["head"], "feature-branch");
        assert_eq!(pr["base"], "main"); // 默认值
    }

    #[test]
    fn test_jira_issue_builder() {
        let issue = TestDataFactory::jira_issue()
            .with_summary("My Issue")
            .with_issue_type("Bug")
            .build();

        assert_eq!(issue["fields"]["summary"], "My Issue");
        assert_eq!(issue["fields"]["issuetype"]["name"], "Bug");
    }
}
```

#### 2.3 增强 Mock 服务器管理

**新增文件**: `tests/common/mock_server.rs`
```rust
//! Mock 服务器管理器
//!
//! 提供统一的 Mock HTTP 服务器管理接口

use http::mock::MockServer;
use mockito::Mock;
use std::collections::HashMap;

/// Mock 服务器管理器
pub struct MockServerManager {
    servers: HashMap<String, MockServer>,
}

impl MockServerManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }

    /// 创建 GitHub Mock 服务器
    pub fn github(&mut self) -> &mut MockServer {
        self.servers
            .entry("github".to_string())
            .or_insert_with(MockServer::new)
    }

    /// 创建 Jira Mock 服务器
    pub fn jira(&mut self) -> &mut MockServer {
        self.servers
            .entry("jira".to_string())
            .or_insert_with(MockServer::new)
    }

    /// 获取服务器 URL
    pub fn url(&self, name: &str) -> Option<String> {
        self.servers.get(name).map(|s| s.url())
    }

    /// 设置 GitHub PR 列表 Mock
    pub fn setup_github_pr_list(&mut self, prs: Vec<serde_json::Value>) -> Mock {
        let body = serde_json::json!(prs).to_string();
        self.github()
            .mock("GET", "/repos/owner/repo/pulls")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create()
    }

    /// 设置 GitHub PR 创建 Mock
    pub fn setup_github_pr_create(&mut self, pr: serde_json::Value) -> Mock {
        let body = pr.to_string();
        self.github()
            .mock("POST", "/repos/owner/repo/pulls")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create()
    }

    /// 设置 Jira Issue 创建 Mock
    pub fn setup_jira_issue_create(&mut self, issue: serde_json::Value) -> Mock {
        let body = issue.to_string();
        self.jira()
            .mock("POST", "/rest/api/2/issue")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create()
    }

    /// 设置错误响应 Mock
    pub fn setup_error_response(&mut self, service: &str, status: usize, message: &str) -> Mock {
        let body = serde_json::json!({
            "error": message
        })
        .to_string();

        let server = match service {
            "github" => self.github(),
            "jira" => self.jira(),
            _ => panic!("Unknown service: {}", service),
        };

        server
            .mock("GET", mockito::Matcher::Any)
            .with_status(status)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create()
    }
}

impl Default for MockServerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_server_manager() {
        let mut manager = MockServerManager::new();

        let github_url = manager.github().url();
        assert!(github_url.starts_with("http://"));

        let jira_url = manager.jira().url();
        assert!(jira_url.starts_with("http://"));
    }

    #[test]
    fn test_setup_github_pr_list() {
        let mut manager = MockServerManager::new();
        let _mock = manager.setup_github_pr_list(vec![]);

        // Mock 已创建，可以使用
        assert!(manager.url("github").is_some());
    }
}
```

#### 2.4 创建测试模板文件

**新增文件**: `tests/fixtures/templates/github_pr.json`
```json
{
  "title": "{{title}}",
  "body": "{{body}}",
  "head": "{{head}}",
  "base": "{{base}}",
  "state": "{{state|open}}",
  "number": {{number|1}},
  "user": {
    "login": "{{user|testuser}}",
    "id": {{user_id|1}}
  },
  "created_at": "{{created_at|2024-01-01T00:00:00Z}}",
  "updated_at": "{{updated_at|2024-01-01T00:00:00Z}}"
}
```

**新增文件**: `tests/fixtures/templates/jira_issue.json`
```json
{
  "key": "{{key|TEST-1}}",
  "fields": {
    "summary": "{{summary}}",
    "description": "{{description}}",
    "issuetype": {
      "name": "{{issue_type|Task}}"
    },
    "project": {
      "key": "{{project|TEST}}"
    },
    "status": {
      "name": "{{status|To Do}}"
    }
  }
}
```

#### 2.5 创建测试场景文件

**新增文件**: `tests/fixtures/scenarios/auth_failure.json`
```json
{
  "name": "认证失败场景",
  "description": "模拟 GitHub/Jira 认证失败",
  "github": {
    "status": 401,
    "response": {
      "message": "Bad credentials"
    }
  },
  "jira": {
    "status": 401,
    "response": {
      "errorMessages": ["Client must be authenticated to access this resource."]
    }
  }
}
```

**新增文件**: `tests/fixtures/scenarios/network_timeout.json`
```json
{
  "name": "网络超时场景",
  "description": "模拟网络请求超时",
  "timeout": 5000,
  "delay": 10000
}
```

---

### 阶段3：系统化性能测试 (2-3天)

#### 3.1 新增 CLI 性能基准测试

**新增文件**: `benches/cli_performance.rs`
```rust
//! CLI 性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::process::Command;

/// 测试帮助命令性能
fn bench_help_command(c: &mut Criterion) {
    c.bench_function("cli_help", |b| {
        b.iter(|| {
            Command::new("cargo")
                .args(&["run", "--release", "--", "--help"])
                .output()
                .expect("Failed to execute command");
        });
    });
}

/// 测试版本命令性能
fn bench_version_command(c: &mut Criterion) {
    c.bench_function("cli_version", |b| {
        b.iter(|| {
            Command::new("cargo")
                .args(&["run", "--release", "--", "--version"])
                .output()
                .expect("Failed to execute command");
        });
    });
}

/// 测试 CLI 启动时间
fn bench_startup_time(c: &mut Criterion) {
    c.bench_function("cli_startup", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();
            Command::new("cargo")
                .args(&["run", "--release", "--", "help"])
                .output()
                .expect("Failed to execute command");
            black_box(start.elapsed());
        });
    });
}

criterion_group!(
    cli_benches,
    bench_help_command,
    bench_version_command,
    bench_startup_time,
);

criterion_main!(cli_benches);
```

#### 3.2 新增核心操作基准测试

**新增文件**: `benches/core_operations.rs`
```rust
//! 核心操作性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// 测试字符串操作性能
fn bench_string_operations(c: &mut Criterion) {
    c.bench_function("string_format", |b| {
        b.iter(|| {
            let s = black_box("feature/PROJ-123-some-feature");
            let _result = format!("Branch: {}", s);
        });
    });
}

/// 测试分支名称解析性能
fn bench_branch_parsing(c: &mut Criterion) {
    c.bench_function("branch_parse", |b| {
        b.iter(|| {
            let branch = black_box("feature/PROJ-123-some-feature");
            let _parts: Vec<&str> = branch.split('/').collect();
        });
    });
}

criterion_group!(
    core_benches,
    bench_string_operations,
    bench_branch_parsing,
);

criterion_main!(core_benches);
```

#### 3.3 新增网络操作基准测试

**新增文件**: `benches/network_operations.rs`
```rust
//! 网络操作性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use http::mock::MockServer;

/// 测试 HTTP 重试机制性能
fn bench_http_retry(c: &mut Criterion) {
    let mut server = MockServer::new();
    let _mock = server
        .mock("GET", "/test")
        .with_status(200)
        .with_body("OK")
        .create();

    c.bench_function("http_retry_immediate_success", |b| {
        b.iter(|| {
            let url = black_box(format!("{}/test", server.url()));
            // 模拟立即成功的请求
            let _ = reqwest::blocking::get(&url);
        });
    });
}

criterion_group!(network_benches, bench_http_retry);

criterion_main!(network_benches);
```

#### 3.4 更新 Cargo.toml 配置

**修改文件**: 根 `Cargo.toml`
```toml
# 添加新的基准测试配置

[[bench]]
name = "cli_performance"
harness = false

[[bench]]
name = "core_operations"
harness = false

[[bench]]
name = "network_operations"
harness = false
```

#### 3.5 增强 Makefile 基准测试命令

**修改文件**: `make/Makefile.bench.mk`
```makefile
# 添加细分的基准测试命令

# 运行 CLI 性能测试
bench-cli:
	@echo "运行 CLI 性能基准测试..."
	cargo bench --bench cli_performance

# 运行核心操作测试
bench-core:
	@echo "运行核心操作基准测试..."
	cargo bench --bench core_operations

# 运行网络操作测试
bench-network:
	@echo "运行网络操作基准测试..."
	cargo bench --bench network_operations

# 性能回归检测
bench-regression:
	@echo "运行性能回归检测..."
	@if [ ! -d "target/criterion" ]; then \
		echo "错误: 没有基线数据，请先运行 make bench"; \
		exit 1; \
	fi
	cargo bench -- --save-baseline regression
	@python3 scripts/check_performance_regression.py

# 性能对比
bench-compare:
	@echo "对比性能基准..."
	cargo bench -- --save-baseline current
	@echo "对比当前基准与上一次基准..."
	@python3 scripts/compare_benchmarks.py target/criterion/

# CI 性能监控
bench-ci:
	@echo "CI 性能监控..."
	cargo bench --no-fail-fast -- --save-baseline ci-$(shell date +%Y%m%d)
```

---

### 阶段4：CI/CD 集成 (1-2天)

#### 4.1 增强 GitHub Actions 工作流

**修改文件**: `.github/workflows/ci.yml`

添加覆盖率作业：
```yaml
  # 测试覆盖率报告
  coverage:
    name: 📊 Test Coverage
    runs-on: ubuntu-latest
    needs: check-skip-ci
    if: needs.check-skip-ci.outputs.should_skip != 'true'
    steps:
      - name: 📥 Checkout repository
        uses: actions/checkout@v4

      - name: 🔧 Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: 💾 Cache Cargo dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-

      - name: 📦 Install system dependencies
        run: bash scripts/dev/shell/deps/install-basic.sh

      - name: 📦 Install cargo-tarpaulin
        run: cargo install cargo-tarpaulin

      - name: 📊 Generate coverage report
        run: make coverage-ci

      - name: 📤 Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: ./coverage/lcov.info
          fail_ci_if_error: true
          token: ${{ secrets.CODECOV_TOKEN }}

      - name: ✅ Check coverage threshold
        run: |
          python3 scripts/check_coverage.py coverage/tarpaulin-report.json 75
```

添加性能基准作业：
```yaml
  # 性能基准测试
  performance:
    name: ⚡ Performance Benchmarks
    runs-on: ubuntu-latest
    needs: check-skip-ci
    if: needs.check-skip-ci.outputs.should_skip != 'true'
    steps:
      - name: 📥 Checkout repository
        uses: actions/checkout@v4

      - name: 🔧 Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: 💾 Cache Cargo dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-

      - name: 📦 Install system dependencies
        run: bash scripts/dev/shell/deps/install-basic.sh

      - name: ⚡ Run benchmarks
        run: make bench-ci

      - name: 📤 Archive benchmark results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results
          path: target/criterion/
          retention-days: 30
```

#### 4.2 创建预提交钩子

**新增文件**: `scripts/git-hooks/pre-commit`
```bash
#!/bin/bash
# Pre-commit hook for code quality checks

set -e

echo "🔍 Running pre-commit checks..."

# 1. 代码格式检查
echo "  ✨ Checking code formatting..."
if ! cargo fmt --check; then
    echo "❌ Code formatting check failed. Run 'cargo fmt' to fix."
    exit 1
fi

# 2. Clippy 检查
echo "  🔍 Running Clippy..."
if ! cargo clippy -- -D warnings; then
    echo "❌ Clippy check failed. Fix the warnings above."
    exit 1
fi

# 3. 运行测试
echo "  🧪 Running tests..."
if ! cargo test; then
    echo "❌ Tests failed. Fix the failing tests."
    exit 1
fi

# 4. 覆盖率检查（可选，较慢）
if [ "${CHECK_COVERAGE:-0}" = "1" ]; then
    echo "  📊 Checking coverage..."
    if ! make coverage-check; then
        echo "❌ Coverage check failed."
        exit 1
    fi
fi

echo "✅ All pre-commit checks passed!"
```

**修改文件**: `make/Makefile.tools.mk`
```makefile
# 添加 Git hooks 安装命令

# 安装 Git hooks
install-hooks:
	@echo "安装 Git hooks..."
	@chmod +x scripts/git-hooks/pre-commit
	@ln -sf ../../scripts/git-hooks/pre-commit .git/hooks/pre-commit
	@echo "✅ Git hooks 已安装"

# 更新 setup 命令
setup: install-tools install-hooks
	@echo "✅ 开发环境设置完成"
```

---

### 阶段5：文档和培训 (1天)

#### 5.1 创建测试指南文档

**新增文件**: `docs/testing/README.md`
```markdown
# 测试指南

## 快速开始

### 运行测试

```bash
# 运行所有测试
make test

# 运行所有测试（包括被忽略的）
make test-all

# 运行特定 crate 的测试
cargo test -p storage

# 运行特定测试
cargo test test_create_pr
```

### 查看覆盖率

```bash
# 生成覆盖率报告
make coverage

# 打开覆盖率报告
make coverage-open

# 检查覆盖率是否达标
make coverage-check
```

### 运行性能测试

```bash
# 运行所有基准测试
make bench

# 运行 CLI 性能测试
make bench-cli

# 运行核心操作测试
make bench-core

# 性能回归检测
make bench-regression
```

## 编写测试

### 使用测试数据工厂

```rust
use tests::common::TestDataFactory;

#[test]
fn test_create_github_pr() {
    // 使用默认值
    let pr = TestDataFactory::github_pr().build();

    // 自定义值
    let pr = TestDataFactory::github_pr()
        .with_title("My Feature")
        .with_head("feature-branch")
        .build();

    assert_eq!(pr["title"], "My Feature");
}
```

### 使用 Mock 服务器

```rust
use tests::common::MockServerManager;

#[test]
fn test_github_api_call() {
    let mut manager = MockServerManager::new();

    // 设置 GitHub PR 列表 Mock
    let _mock = manager.setup_github_pr_list(vec![
        TestDataFactory::github_pr().build()
    ]);

    // 使用 Mock 服务器 URL
    let url = manager.url("github").unwrap();
    // 进行 API 调用测试...
}
```

## 最佳实践

### 1. 测试命名规范

```rust
// ✅ 好的命名
#[test]
fn test_create_pr_with_valid_data_should_succeed() { }

#[test]
fn test_create_pr_with_invalid_token_should_return_auth_error() { }

// ❌ 不好的命名
#[test]
fn test1() { }

#[test]
fn test_pr() { }
```

### 2. 使用测试数据工厂

```rust
// ✅ 使用工厂
let pr = TestDataFactory::github_pr()
    .with_title("Test")
    .build();

// ❌ 硬编码
let pr = json!({
    "title": "Test",
    "body": "...",
    // 大量重复代码
});
```

### 3. 测试隔离

```rust
// ✅ 使用 tempfile 创建临时目录
#[test]
fn test_file_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    // 测试代码...
    // temp_dir 会自动清理
}

// ❌ 使用固定目录
#[test]
fn test_file_operations() {
    std::fs::create_dir("/tmp/test").unwrap();
    // 可能与其他测试冲突
}
```

## 常见问题

### Q: 如何跳过慢测试？

```rust
#[test]
#[ignore]  // 默认不运行
fn slow_test() {
    // 耗时测试
}
```

运行时：
```bash
# 只运行快速测试
cargo test

# 运行所有测试（包括慢测试）
cargo test -- --include-ignored
```

### Q: 如何调试测试失败？

```bash
# 显示测试输出
cargo test -- --nocapture

# 运行单个测试
cargo test test_name -- --nocapture
```

### Q: 如何测试并发代码？

```rust
use serial_test::serial;

#[test]
#[serial]  // 串行运行，避免并发冲突
fn test_shared_resource() {
    // 测试代码
}
```
```

#### 5.2 创建最佳实践文档

**新增文件**: `docs/testing/BEST_PRACTICES.md`
```markdown
# 测试最佳实践

## 测试数据管理

### 原则
1. **DRY（Don't Repeat Yourself）**: 使用测试数据工厂避免重复
2. **可读性优先**: 测试数据应该清晰表达意图
3. **独立性**: 每个测试应该独立，不依赖其他测试

### 示例

#### ✅ 好的做法
```rust
#[test]
fn test_pr_creation() {
    let pr = TestDataFactory::github_pr()
        .with_title("Add feature X")
        .with_head("feature/x")
        .build();

    let result = create_pr(pr);
    assert!(result.is_ok());
}
```

#### ❌ 不好的做法
```rust
#[test]
fn test_pr_creation() {
    let pr = json!({
        "title": "Add feature X",
        "body": "This PR adds feature X",
        "head": "feature/x",
        "base": "main",
        "state": "open",
        // ... 大量重复代码
    });

    let result = create_pr(pr);
    assert!(result.is_ok());
}
```

## Mock 服务器使用

### 原则
1. **最小化 Mock**: 只 Mock 必要的外部依赖
2. **真实性**: Mock 响应应该接近真实 API
3. **可维护性**: 使用预定义的 Mock 场景

### 示例

#### ✅ 好的做法
```rust
#[test]
fn test_github_api_integration() {
    let mut manager = MockServerManager::new();
    let _mock = manager.setup_github_pr_list(vec![
        TestDataFactory::github_pr().build()
    ]);

    // 测试代码使用 manager.url("github")
}
```

#### ❌ 不好的做法
```rust
#[test]
fn test_github_api_integration() {
    let mut server = MockServer::new();
    server.mock("GET", "/pulls")
        .with_status(200)
        .with_body(r#"{"title":"..."}"#)  // 硬编码
        .create();

    // 每个测试都要重复配置
}
```

## 性能测试原则

### 1. 基线建立
```bash
# 首次运行建立基线
cargo bench -- --save-baseline initial

# 后续对比
cargo bench -- --baseline initial
```

### 2. 稳定的测试环境
- 关闭不必要的后台程序
- 使用相同的硬件配置
- 多次运行取平均值

### 3. 有意义的基准
```rust
// ✅ 测试实际使用场景
fn bench_realistic_workflow(c: &mut Criterion) {
    c.bench_function("realistic_pr_creation", |b| {
        b.iter(|| {
            // 模拟真实的 PR 创建流程
        });
    });
}

// ❌ 测试不现实的场景
fn bench_unrealistic(c: &mut Criterion) {
    c.bench_function("empty_loop", |b| {
        b.iter(|| {
            // 空循环没有实际意义
        });
    });
}
```

## 覆盖率提升技巧

### 1. 识别未覆盖代码
```bash
# 生成覆盖率报告
make coverage-open

# 查看未覆盖的代码行
# 红色表示未覆盖
```

### 2. 优先级排序
1. 核心业务逻辑（最高优先级）
2. 错误处理路径
3. 边缘情况
4. 工具函数

### 3. 排除不必要的代码
```rust
// 使用 #[cfg(test)] 排除测试代码
#[cfg(test)]
mod tests { }

// 排除生成代码
// 在 Cargo.toml 中配置 tarpaulin 排除
```
```

---

## 验证方法

### 阶段1验证：覆盖率监控
```bash
# 1. 安装 tarpaulin
cargo install cargo-tarpaulin

# 2. 生成覆盖率报告
make coverage

# 3. 检查报告生成
ls -la coverage/tarpaulin-report.html

# 4. 验证覆盖率阈值检查
make coverage-check

# 5. 检查 CI 覆盖率作业
# 创建 PR 并查看 CI 运行结果
```

### 阶段2验证：测试数据工厂
```bash
# 1. 检查目录结构
tree tests/

# 2. 运行测试数据工厂测试
cargo test -p workflow --test common

# 3. 使用工厂创建测试数据
cargo test test_data_factory

# 4. 验证 Mock 服务器
cargo test mock_server_manager
```

### 阶段3验证：性能测试
```bash
# 1. 运行所有基准测试
make bench

# 2. 运行 CLI 性能测试
make bench-cli

# 3. 运行核心操作测试
make bench-core

# 4. 查看性能报告
ls -la target/criterion/
open target/criterion/report/index.html
```

### 阶段4验证：CI/CD集成
```bash
# 1. 安装 Git hooks
make install-hooks

# 2. 测试预提交钩子
git commit -m "test"
# 应该运行格式检查、Clippy、测试

# 3. 创建 PR 验证 CI
# 检查覆盖率作业是否运行
# 检查性能基准作业是否运行

# 4. 检查 Codecov 集成
# 访问 Codecov 网站查看覆盖率报告
```

### 阶段5验证：文档
```bash
# 1. 检查文档是否存在
ls -la docs/testing/

# 2. 阅读测试指南
cat docs/testing/README.md

# 3. 按照指南运行测试
# 验证所有命令是否正常工作
```

---

## 影响评估

### 新增文件统计
```
新增文件：
├── coverage.toml                           # 覆盖率配置
├── scripts/
│   ├── check_coverage.py                  # 覆盖率检查脚本
│   ├── analyze_coverage_trend.py          # 覆盖率趋势分析
│   ├── check_performance_regression.py    # 性能回归检测
│   ├── compare_benchmarks.py              # 性能对比
│   └── git-hooks/pre-commit               # 预提交钩子
├── tests/
│   ├── common/
│   │   ├── mod.rs                         # 测试公共模块
│   │   ├── test_data_factory.rs           # 测试数据工厂（~200 行）
│   │   ├── mock_server.rs                 # Mock 服务器管理（~150 行）
│   │   └── data_migration.rs              # 数据迁移
│   └── fixtures/
│       ├── templates/
│       │   ├── github_pr.json
│       │   └── jira_issue.json
│       └── scenarios/
│           ├── auth_failure.json
│           └── network_timeout.json
├── benches/
│   ├── cli_performance.rs                 # CLI 性能测试（~100 行）
│   ├── core_operations.rs                 # 核心操作测试（~80 行）
│   └── network_operations.rs              # 网络操作测试（~60 行）
└── docs/testing/
    ├── README.md                          # 测试指南
    └── BEST_PRACTICES.md                  # 最佳实践

总计：~20 个新文件，~1500 行代码
```

### 修改文件
```
修改文件：
├── Cargo.toml                             # 添加 benchmark 配置
├── .gitignore                             # 添加 coverage/
├── make/Makefile.test.mk                  # 增强覆盖率命令
├── make/Makefile.bench.mk                 # 增强性能测试命令
├── make/Makefile.tools.mk                 # 添加 install-hooks
├── .github/workflows/ci.yml               # 添加覆盖率和性能作业
└── .github/workflows/release.yml          # 添加覆盖率验证

总计：7 个文件修改
```

### 破坏性评估
- **破坏性**: 无（纯增量添加，不修改现有测试）
- **向后兼容**: 100%（现有测试继续工作）
- **迁移成本**: 低（可选择性迁移到新架构）
- **学习曲线**: 中等（需要理解新的测试工具）

### 性能影响
- **CI 时间增加**: +5-10 分钟（覆盖率 + 性能测试）
- **本地开发**: 无影响（按需运行）
- **预提交钩子**: +30-60 秒（可配置）

---

## 实施计划

### 第1周：基础设施（阶段1+2）
**目标**: 建立测试覆盖率监控和数据管理系统

**Day 1-2**:
- [ ] 创建 `coverage.toml` 配置
- [ ] 更新 Makefile 覆盖率命令
- [ ] 添加覆盖率检查脚本
- [ ] 修改 CI 添加覆盖率作业

**Day 3-5**:
- [ ] 创建 `tests/` 目录结构
- [ ] 实现测试数据工厂（`test_data_factory.rs`）
- [ ] 实现 Mock 服务器管理器（`mock_server.rs`）
- [ ] 创建测试模板和场景文件

**Day 5 验收**:
- [ ] 覆盖率报告可以生成
- [ ] 测试数据工厂单元测试通过
- [ ] Mock 服务器管理器测试通过

### 第2周：性能测试和集成（阶段3+4）
**目标**: 完善性能测试体系，集成到 CI/CD

**Day 6-8**:
- [ ] 创建 CLI 性能基准测试
- [ ] 创建核心操作基准测试
- [ ] 创建网络操作基准测试
- [ ] 更新 Makefile 基准测试命令

**Day 9-10**:
- [ ] 修改 CI 添加性能基准作业
- [ ] 创建预提交钩子
- [ ] 更新 Makefile 添加 `install-hooks`
- [ ] 实现性能回归检测脚本

**Day 10 验收**:
- [ ] 所有基准测试可以运行
- [ ] CI 中覆盖率和性能作业正常
- [ ] 预提交钩子正常工作

### 第3周：文档和优化（阶段5）
**目标**: 完善文档，逐步迁移现有测试

**Day 11**:
- [ ] 创建测试指南文档
- [ ] 创建最佳实践文档
- [ ] 更新 README.md 添加测试章节

**Day 12-13**:
- [ ] 选择 2-3 个模块作为示例迁移到新架构
- [ ] 优化和调整工具（根据实际使用反馈）
- [ ] 团队培训和知识分享

**Day 13 验收**:
- [ ] 文档完整且可用
- [ ] 至少 2 个模块使用新测试架构
- [ ] 团队成员熟悉新工具

---

## 成功指标

### 量化指标
- [ ] **测试覆盖率**: 从当前水平提升到 ≥ 75%（目标 80%）
- [ ] **CI 通过率**: ≥ 95%
- [ ] **性能基准**: 建立 50+ 个基准测试
- [ ] **测试数据复用率**: ≥ 60% 测试使用数据工厂

### 质量指标
- [ ] **可维护性**: 测试代码重复率 < 20%
- [ ] **可读性**: 新人上手时间 < 1 天
- [ ] **稳定性**: 测试波动率 < 5%
- [ ] **文档完整性**: 所有工具有使用文档

---

## 风险和缓解措施

### 🔴 高风险
**风险**: CI 时间增加过多（>15分钟）
- **缓解**:
  - 使用 GitHub Actions 缓存
  - 并行运行作业
  - 覆盖率和性能测试只在特定分支运行

**风险**: 团队学习成本高
- **缓解**:
  - 详细文档和示例
  - 团队培训会议
  - 逐步迁移，不强制一次性切换

### 🟡 中风险
**风险**: 测试数据工厂设计不合理
- **缓解**:
  - 先在小范围试点
  - 收集反馈快速迭代
  - 保持向后兼容

**风险**: 性能基准不稳定
- **缓解**:
  - 使用相对性能指标
  - 设置合理的阈值范围（±5%）
  - 多次运行取平均值

### 🟢 低风险
**风险**: 工具版本兼容性
- **缓解**:
  - 锁定工具版本
  - 定期更新（每季度）
  - 维护兼容性矩阵

---

## 相关资源

### 外部文档
- [cargo-tarpaulin 文档](https://github.com/xd009642/tarpaulin)
- [Criterion.rs 指南](https://bheisler.github.io/criterion.rs/book/)
- [mockito 文档](https://docs.rs/mockito/)
- [GitHub Actions 文档](https://docs.github.com/en/actions)

### 内部文件
- [测试架构改进 TODO](./test-architecture-improvement.md)
- [现有压力测试](../../crates/storage/tests/stress_tests.rs)
- [现有基准测试](../../crates/storage/benches/git_services_bench.rs)
- [CI 配置](../../.github/workflows/ci.yml)

---

## 附录

### A. 快速命令参考

```bash
# 测试相关
make test                    # 运行测试
make test-all                # 运行所有测试（包括 ignored）
make coverage                # 生成覆盖率报告
make coverage-open           # 打开覆盖率报告
make coverage-check          # 检查覆盖率阈值

# 性能测试
make bench                   # 运行所有基准测试
make bench-cli               # CLI 性能测试
make bench-core              # 核心操作测试
make bench-network           # 网络操作测试
make bench-regression        # 性能回归检测

# 工具安装
make setup                   # 设置开发环境
make install-hooks           # 安装 Git hooks
```

### B. 常见问题 FAQ

**Q1: 为什么需要测试数据工厂？**
A: 避免测试代码重复，提高可维护性，统一测试数据格式。

**Q2: 预提交钩子会不会很慢？**
A: 默认只运行格式检查和 Clippy（~10秒），覆盖率检查是可选的。

**Q3: 现有测试需要全部迁移吗？**
A: 不需要。可以选择性迁移，现有测试继续工作。

**Q4: 性能基准测试多久运行一次？**
A: 本地按需运行，CI 在 PR 合并到 master 时运行。

**Q5: 如何排除某些代码的覆盖率统计？**
A: 在 `coverage.toml` 中配置排除规则，或使用 `#[cfg(not(tarpaulin_include))]`。

---

**文档版本**: 1.0
**创建日期**: 2024-02-10
**最后更新**: 2024-02-10
**负责人**: [待指定]
