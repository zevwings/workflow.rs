# 测试文档编写指南

> **目标**: 为项目创建完善的测试文档，帮助团队成员快速上手
> **优先级**: 🔴 P0 (高)
> **预计时间**: 1 天
> **依赖**: [测试覆盖率监控](./testing-01-coverage-monitoring.md), [系统化性能测试](./testing-02-performance-testing.md), [CI/CD 集成](./testing-03-cicd-integration.md)

---

## 🎯 目标和范围

### 实施目标
1. ✅ 创建项目级测试文档目录 (`docs/testing/`)
2. ✅ 编写测试快速入门指南
3. ✅ 编写测试最佳实践文档
4. ✅ 为主要 crate 创建测试使用文档 (`TESTING.md`)
5. ✅ 编写测试工具使用示例
6. ✅ 创建常见问题 FAQ

### 产出物
```
workflow.rs/
├── docs/
│   └── testing/
│       ├── README.md                # 测试指南总览
│       ├── QUICK_START.md           # 快速入门
│       ├── BEST_PRACTICES.md        # 最佳实践
│       ├── FAQ.md                   # 常见问题
│       └── TROUBLESHOOTING.md       # 故障排查
├── crates/
│   ├── domain/TESTING.md            # Domain 测试文档
│   ├── http/TESTING.md              # HTTP 测试文档
│   ├── services/TESTING.md          # Services 测试文档
│   ├── storage/TESTING.md           # Storage 测试文档
│   └── llm/TESTING.md               # LLM 测试文档
└── README.md                        # 更新主 README
```

---

## 📊 当前状态

### ✅ 已有基础
- ✅ Testing 模块已实现
- ✅ 测试工具已创建
- ✅ 覆盖率和性能测试已配置

### ❌ 缺失部分
- ❌ 没有 `docs/testing/` 目录
- ❌ 没有测试快速入门指南
- ❌ 没有测试最佳实践文档
- ❌ 各 crate 没有 `TESTING.md`
- ❌ 没有测试工具使用示例
- ❌ README 中没有测试章节

---

## 📋 前置条件

### 知识准备
- 了解项目的测试架构
- 熟悉各 crate 的 testing 模块
- 了解覆盖率和性能测试工具

### 准备工作
```bash
# 确认测试基础设施已就绪
ls -la coverage.toml
ls -la benches/
ls -la crates/*/src/testing/

# 确认测试可以运行
cargo test
make coverage
make bench
```

---

## 🔨 详细实施步骤

### Step 1: 创建测试文档目录 (5 分钟)

#### 1.1 创建目录结构

```bash
# 创建测试文档目录
mkdir -p docs/testing

# 确认目录已创建
ls -la docs/testing/
```

---

### Step 2: 编写测试指南总览 (30 分钟)

#### 2.1 创建 `docs/testing/README.md`

```markdown
# 测试指南

欢迎阅读 Workflow 项目测试指南！本文档帮助你快速了解和使用项目的测试体系。

---

## 📚 文档导航

### 快速开始
- [快速入门指南](./QUICK_START.md) - 5 分钟上手测试
- [常见问题 FAQ](./FAQ.md) - 快速找到答案

### 深入学习
- [测试最佳实践](./BEST_PRACTICES.md) - 编写高质量测试
- [故障排查指南](./TROUBLESHOOTING.md) - 解决测试问题

### Crate 专项文档
- [Domain 测试文档](../../crates/domain/TESTING.md)
- [HTTP 测试文档](../../crates/http/TESTING.md)
- [Services 测试文档](../../crates/services/TESTING.md)
- [Storage 测试文档](../../crates/storage/TESTING.md)
- [LLM 测试文档](../../crates/llm/TESTING.md)

---

## 🎯 测试体系概览

### 测试类型

| 类型 | 位置 | 用途 | 运行命令 |
|------|------|------|----------|
| **单元测试** | `src/**/*.rs` 中的 `#[cfg(test)]` | 测试单个函数/模块 | `cargo test` |
| **集成测试** | `tests/*.rs` | 端到端测试 | `cargo test --test *` |
| **性能测试** | `benches/*.rs` | 性能基准 | `cargo bench` |
| **压力测试** | `tests/stress_tests.rs` | 并发/负载测试 | `cargo test -- --ignored` |

### 测试工具

| 工具 | 用途 | 位置 |
|------|------|------|
| **TestDataFactory** | 创建测试数据 | `crates/*/src/testing/` |
| **MockServer** | Mock HTTP 服务 | `crates/http/src/testing/` |
| **MockServices** | Mock 业务服务 | `crates/services/src/testing/` |
| **Git 测试辅助** | 创建测试仓库 | `crates/storage/src/testing/` |

### 测试命令

```bash
# 运行测试
make test                    # 运行所有测试
make test-all                # 包括被忽略的测试

# 覆盖率
make coverage                # 生成覆盖率报告
make coverage-check          # 检查覆盖率阈值
make coverage-open           # 打开 HTML 报告

# 性能测试
make bench                   # 运行所有基准测试
make bench-cli               # CLI 性能测试
make bench-regression        # 性能回归检测

# 开发工具
make install-hooks           # 安装预提交钩子
make test-hooks              # 测试钩子
```

---

## 💡 快速示例

### 编写单元测试

```rust
// src/branch/mod.rs

pub fn parse_branch_name(name: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = name.split('/').collect();
    if parts.len() == 2 {
        Some((parts[0], parts[1]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_branch() {
        let result = parse_branch_name("feature/add-login");
        assert_eq!(result, Some(("feature", "add-login")));
    }

    #[test]
    fn test_parse_invalid_branch() {
        let result = parse_branch_name("invalid");
        assert_eq!(result, None);
    }
}
```

### 使用测试数据工厂

```rust
use http::testing::TestDataFactory;

#[test]
fn test_create_pull_request() {
    // 使用默认值
    let pr = TestDataFactory::github_pr().build();

    // 自定义值
    let pr = TestDataFactory::github_pr()
        .with_title("Add new feature")
        .with_head("feature/new-feature")
        .build();

    assert_eq!(pr["title"], "Add new feature");
}
```

### 使用 Mock 服务器

```rust
use http::testing::{MockServerManager, TestDataFactory};

#[test]
fn test_github_api_call() {
    let mut manager = MockServerManager::new();

    // 设置 Mock 响应
    let pr_data = TestDataFactory::github_pr().build();
    let _mock = manager.setup_github_pr_list(vec![pr_data]);

    // 使用 Mock 服务器 URL
    let url = manager.url("github").unwrap();

    // 调用你的 API 客户端，它会访问 Mock 服务器
    // let client = GitHubClient::new(&url);
    // let result = client.list_prs();
    // assert!(result.is_ok());
}
```

---

## 📊 测试覆盖率

### 当前状态

查看最新覆盖率: [![codecov](https://codecov.io/gh/YOUR_USERNAME/workflow.rs/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USERNAME/workflow.rs)

### 覆盖率目标

| Crate | 目标 | 当前 |
|-------|------|------|
| domain | 85% | - |
| storage | 85% | - |
| services | 80% | - |
| http | 75% | - |
| llm | 70% | - |

---

## 🚀 CI/CD 集成

测试自动在以下场景运行：

### Pull Request
- ✅ 单元测试
- ✅ 集成测试
- ✅ 覆盖率检查
- ✅ 代码格式和 Clippy

### Main 分支
- ✅ 所有 PR 检查
- ✅ 性能基准测试
- ✅ 压力测试

### Pre-commit 钩子
- ✅ 代码格式检查
- ✅ Clippy 检查
- ✅ 单元测试

---

## 🎓 学习路径

### 新手（0-1 周）
1. 阅读 [快速入门指南](./QUICK_START.md)
2. 运行 `cargo test` 查看现有测试
3. 编写第一个单元测试
4. 使用测试数据工厂

### 进阶（1-2 周）
1. 阅读 [测试最佳实践](./BEST_PRACTICES.md)
2. 使用 Mock 服务器编写集成测试
3. 运行 `make coverage` 查看覆盖率
4. 了解性能测试

### 高级（2+ 周）
1. 为新功能编写完整测试套件
2. 提高覆盖率到 80%+
3. 编写性能基准测试
4. 优化测试性能

---

## 📞 获取帮助

- **文档问题**: 在 GitHub 上提 Issue
- **测试失败**: 查看 [故障排查指南](./TROUBLESHOOTING.md)
- **最佳实践**: 查看 [测试最佳实践](./BEST_PRACTICES.md)
- **常见问题**: 查看 [FAQ](./FAQ.md)

---

**最后更新**: 2025-02-11
```

创建此文件：

```bash
# 复制上述内容到文件
cat > docs/testing/README.md << 'EOF'
# (粘贴上述 markdown 内容)
EOF
```

---

### Step 3: 编写快速入门指南 (45 分钟)

#### 3.1 创建 `docs/testing/QUICK_START.md`

这个文档帮助新手在 5 分钟内上手：

```markdown
# 测试快速入门

5 分钟快速上手 Workflow 项目测试！

---

## 🚀 运行你的第一个测试

### 1. 运行所有测试

```bash
cargo test
```

你应该看到类似的输出：

```
running 42 tests
test domain::tests::test_branch_parse ... ok
test http::tests::test_mock_server ... ok
test storage::tests::test_git_operations ... ok
...
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### 2. 运行特定 crate 的测试

```bash
# 只测试 domain crate
cargo test -p domain

# 只测试 http crate
cargo test -p http
```

### 3. 运行特定测试

```bash
# 运行名称包含 "branch" 的测试
cargo test branch

# 运行确切的测试
cargo test test_parse_branch_name
```

---

## ✍️ 编写你的第一个测试

### 示例：测试一个简单函数

假设你有一个函数：

```rust
// src/utils.rs

pub fn format_branch_name(prefix: &str, name: &str) -> String {
    format!("{}/{}", prefix, name)
}
```

为它添加测试：

```rust
// src/utils.rs 底部

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_branch_name() {
        let result = format_branch_name("feature", "add-login");
        assert_eq!(result, "feature/add-login");
    }
}
```

运行测试：

```bash
cargo test test_format_branch_name
```

---

## 🎨 使用测试数据工厂

测试数据工厂帮助你快速创建复杂的测试数据。

### 示例 1: 创建 GitHub PR 数据

```rust
use http::testing::TestDataFactory;

#[test]
fn test_pr_creation() {
    // 使用默认值
    let pr = TestDataFactory::github_pr().build();

    // 自定义字段
    let pr = TestDataFactory::github_pr()
        .with_title("My awesome feature")
        .with_head("feature/awesome")
        .build();

    // 使用数据
    assert_eq!(pr["title"], "My awesome feature");
    assert_eq!(pr["head"], "feature/awesome");
    assert_eq!(pr["base"], "main"); // 默认值
}
```

### 示例 2: 创建领域实体

```rust
use domain::testing::TestEntityFactory;

#[test]
fn test_branch_entity() {
    let branch = TestEntityFactory::branch()
        .with_name("feature/test")
        .as_current()
        .build();

    assert_eq!(branch.name, "feature/test");
    assert!(branch.is_current);
}
```

---

## 🎭 使用 Mock 服务器

Mock 服务器让你模拟 HTTP 请求，无需真实的外部服务。

### 示例：Mock GitHub API

```rust
use http::testing::{MockServerManager, TestDataFactory};

#[test]
fn test_list_pull_requests() {
    // 1. 创建 Mock 服务器
    let mut manager = MockServerManager::new();

    // 2. 准备测试数据
    let pr1 = TestDataFactory::github_pr()
        .with_title("PR 1")
        .build();
    let pr2 = TestDataFactory::github_pr()
        .with_title("PR 2")
        .build();

    // 3. 设置 Mock 响应
    let _mock = manager.setup_github_pr_list(vec![pr1, pr2]);

    // 4. 获取 Mock 服务器 URL
    let url = manager.url("github").unwrap();

    // 5. 使用 URL 调用你的代码
    // let client = GitHubClient::new(&url);
    // let result = client.list_prs();
    // assert_eq!(result.unwrap().len(), 2);
}
```

---

## 📊 查看测试覆盖率

### 生成覆盖率报告

```bash
make coverage
```

### 打开 HTML 报告

```bash
make coverage-open
```

报告会在浏览器中打开，显示：
- 整体覆盖率百分比
- 每个文件的覆盖率
- 哪些代码行没有被测试（红色标记）

### 检查覆盖率是否达标

```bash
make coverage-check
```

如果覆盖率低于 75%，命令会失败并显示：
```
❌ 覆盖率不达标: 65.50% < 75.00%
```

---

## ⚡ 性能测试

### 运行性能基准测试

```bash
# 运行所有基准测试
make bench

# 运行特定基准测试
make bench-cli        # CLI 性能
make bench-core       # 核心操作
```

### 查看性能报告

```bash
# 在浏览器中打开
open target/criterion/report/index.html
```

---

## 🪝 安装预提交钩子

预提交钩子在每次 `git commit` 前自动运行检查。

### 安装

```bash
make install-hooks
```

### 使用

```bash
# 正常提交（会触发检查）
git commit -m "feat: add new feature"

# 跳过检查（不推荐）
git commit --no-verify -m "wip: work in progress"
```

钩子会检查：
- ✅ 代码格式 (`cargo fmt`)
- ✅ Clippy 警告 (`cargo clippy`)
- ✅ 单元测试 (`cargo test`)

---

## 🎯 常用测试命令

```bash
# 运行测试
cargo test                           # 所有测试
cargo test -p domain                 # 特定 crate
cargo test test_name                 # 特定测试
cargo test -- --nocapture            # 显示 println! 输出
cargo test -- --ignored              # 运行被忽略的测试

# 覆盖率
make coverage                        # 生成报告
make coverage-open                   # 打开报告
make coverage-check                  # 检查阈值

# 性能测试
make bench                           # 运行基准测试
make bench-cli                       # CLI 性能

# 开发工具
make install-hooks                   # 安装预提交钩子
make test-hooks                      # 测试钩子
```

---

## 🐛 遇到问题？

### 测试失败

```bash
# 1. 查看详细输出
cargo test -- --nocapture

# 2. 运行单个测试调试
cargo test test_name -- --nocapture

# 3. 使用 dbg! 宏
fn my_function() {
    let value = compute();
    dbg!(&value);  // 打印调试信息
}
```

### 覆盖率报告生成失败

```bash
# 1. 确认 cargo-tarpaulin 已安装
cargo install cargo-tarpaulin

# 2. 重新生成
make coverage
```

### 更多帮助

- 查看 [故障排查指南](./TROUBLESHOOTING.md)
- 查看 [FAQ](./FAQ.md)

---

## 📚 下一步

- 阅读 [测试最佳实践](./BEST_PRACTICES.md)
- 查看各 crate 的测试文档:
  - [HTTP 测试文档](../../crates/http/TESTING.md)
  - [Domain 测试文档](../../crates/domain/TESTING.md)
  - [Services 测试文档](../../crates/services/TESTING.md)

---

**预计阅读时间**: 5 分钟
**最后更新**: 2025-02-11
```

创建此文件：

```bash
cat > docs/testing/QUICK_START.md << 'EOF'
# (粘贴上述内容)
EOF
```

---

### Step 4: 编写测试最佳实践文档 (60 分钟)

#### 4.1 创建 `docs/testing/BEST_PRACTICES.md`

```markdown
# 测试最佳实践

编写高质量测试的指南和建议。

---

## 📐 测试设计原则

### FIRST 原则

好的测试应该是 **FIRST**:

- **F**ast (快速): 测试应该快速运行
- **I**ndependent (独立): 测试之间不应有依赖
- **R**epeatable (可重复): 每次运行结果一致
- **S**elf-Validating (自验证): 测试明确通过或失败
- **T**imely (及时): 测试与代码同步编写

### AAA 模式

每个测试应该遵循 **Arrange-Act-Assert** 模式：

```rust
#[test]
fn test_create_pull_request() {
    // Arrange: 准备测试数据
    let pr_data = TestDataFactory::github_pr()
        .with_title("Test PR")
        .build();

    // Act: 执行被测试的操作
    let result = create_pull_request(pr_data);

    // Assert: 验证结果
    assert!(result.is_ok());
    assert_eq!(result.unwrap().title, "Test PR");
}
```

---

## ✍️ 测试命名规范

### 命名模式

使用清晰、描述性的测试名称：

```rust
// ✅ 好的命名
#[test]
fn test_parse_valid_branch_name_should_return_parts() { }

#[test]
fn test_parse_empty_branch_name_should_return_none() { }

#[test]
fn test_create_pr_with_invalid_token_should_return_auth_error() { }

// ❌ 不好的命名
#[test]
fn test1() { }

#[test]
fn test_parse() { }

#[test]
fn it_works() { }
```

### 命名模板

```
test_<function>_<scenario>_should_<expected_result>

例如:
test_divide_by_zero_should_return_error
test_parse_valid_json_should_succeed
test_send_request_with_timeout_should_retry
```

---

## 🎨 使用测试数据工厂

### 为什么使用数据工厂

```rust
// ❌ 硬编码（不推荐）
#[test]
fn test_something() {
    let pr = json!({
        "title": "Test",
        "body": "Body",
        "head": "feature",
        "base": "main",
        "state": "open",
        "number": 1,
        "user": {
            "login": "user",
            "id": 1
        }
        // ... 大量重复代码
    });
}

// ✅ 使用工厂（推荐）
#[test]
fn test_something() {
    let pr = TestDataFactory::github_pr()
        .with_title("Test")
        .build();
}
```

### 数据工厂最佳实践

1. **只覆盖必要字段**:
```rust
// ✅ 好 - 只覆盖关键字段
let pr = TestDataFactory::github_pr()
    .with_title("Important title")
    .build();

// ❌ 不好 - 覆盖太多字段
let pr = TestDataFactory::github_pr()
    .with_title("Test")
    .with_body("Body")
    .with_head("feature")
    .with_base("main")
    .build();
```

2. **为不同场景创建命名构建器**:
```rust
impl TestDataFactory {
    pub fn open_pr() -> GitHubPRBuilder {
        Self::github_pr()
            .with_state("open")
    }

    pub fn closed_pr() -> GitHubPRBuilder {
        Self::github_pr()
            .with_state("closed")
    }
}
```

---

## 🎭 Mock 使用最佳实践

### Mock 服务器

```rust
// ✅ 好的做法
#[test]
fn test_github_api() {
    let mut manager = MockServerManager::new();
    let pr_data = TestDataFactory::github_pr().build();

    // 清晰地设置预期
    let _mock = manager.setup_github_pr_list(vec![pr_data]);

    // 使用 mock
    let url = manager.url("github").unwrap();
    // ... 测试代码
}

// ❌ 不好的做法
#[test]
fn test_github_api() {
    let mut server = MockServer::new();

    // 手动构建 JSON
    server.mock("GET", "/pulls")
        .with_status(200)
        .with_body(r#"{"title":"..."}"#)  // 硬编码
        .create();
}
```

### Mock 服务

```rust
use services::testing::MockBranchService;

#[test]
fn test_branch_listing() {
    // 创建 mock 服务
    let mock_service = MockBranchService::new();

    // 添加测试数据
    let branch = TestEntityFactory::branch()
        .with_name("feature/test")
        .build();
    mock_service.add_branch(branch);

    // 使用 mock
    let branches = mock_service.list_branches().unwrap();
    assert_eq!(branches.len(), 1);
}
```

---

## 🔒 测试隔离

### 使用临时目录

```rust
use tempfile::TempDir;

#[test]
fn test_file_operations() {
    // ✅ 使用临时目录
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // 执行文件操作
    std::fs::write(&file_path, "test").unwrap();

    // 断言
    assert!(file_path.exists());

    // temp_dir 会自动清理
}

// ❌ 不要使用固定目录
#[test]
fn test_file_operations_bad() {
    let file_path = "/tmp/test.txt";  // 可能与其他测试冲突
    std::fs::write(file_path, "test").unwrap();
    // 需要手动清理
}
```

### 避免共享状态

```rust
// ❌ 不好 - 共享全局状态
static mut COUNTER: i32 = 0;

#[test]
fn test1() {
    unsafe { COUNTER += 1; }
    // 可能与 test2 冲突
}

#[test]
fn test2() {
    unsafe { COUNTER += 1; }
    // 可能与 test1 冲突
}

// ✅ 好 - 每个测试独立
#[test]
fn test1() {
    let counter = 0;
    // 测试逻辑
}

#[test]
fn test2() {
    let counter = 0;
    // 测试逻辑
}
```

---

## 🎯 测试覆盖率策略

### 优先级排序

1. **核心业务逻辑** (目标: 90%+)
   - 领域模型
   - 业务规则
   - 关键算法

2. **错误处理** (目标: 85%+)
   - 各种错误场景
   - 边界条件

3. **API/集成层** (目标: 75%+)
   - HTTP 端点
   - 外部服务调用

4. **工具函数** (目标: 70%+)
   - 辅助函数
   - 格式化函数

### 排除不必要的代码

```rust
// 排除测试代码
#[cfg(test)]
mod tests { }

// 排除示例代码
// examples/

// 排除生成代码
// target/

// 在 Cargo.toml 中配置:
// [workspace.metadata.tarpaulin]
// exclude-files = ["tests/*", "benches/*"]
```

---

## ⚡ 测试性能优化

### 避免慢测试

```rust
// ❌ 慢测试
#[test]
fn test_slow() {
    std::thread::sleep(std::time::Duration::from_secs(5));
    // 这会拖慢整个测试套件
}

// ✅ 标记为忽略
#[test]
#[ignore]  // 默认不运行
fn test_slow_integration() {
    std::thread::sleep(std::time::Duration::from_secs(5));
    // 用 cargo test -- --ignored 运行
}
```

### 并行测试注意事项

```rust
use serial_test::serial;

// 需要串行运行的测试
#[test]
#[serial]
fn test_shared_resource() {
    // 访问共享资源（文件、数据库等）
}

#[test]
#[serial]
fn test_another_shared_resource() {
    // 也访问同样的共享资源
}
```

---

## 📊 断言最佳实践

### 使用描述性断言

```rust
// ❌ 不清晰
assert!(result);

// ✅ 清晰
assert!(result, "Expected result to be true, but got false");

// ✅ 更好 - 使用专门的断言
assert_eq!(actual, expected);
assert_ne!(actual, unexpected);
```

### 自定义错误消息

```rust
#[test]
fn test_parse_branch() {
    let result = parse_branch("feature/test");

    assert!(
        result.is_ok(),
        "Failed to parse branch: {:?}",
        result.err()
    );

    let (prefix, name) = result.unwrap();

    assert_eq!(
        prefix, "feature",
        "Expected prefix 'feature', got '{}'",
        prefix
    );
}
```

---

## 🐛 测试错误处理

### 测试 Result 类型

```rust
#[test]
fn test_successful_operation() {
    let result = some_operation();
    assert!(result.is_ok());
}

#[test]
fn test_error_case() {
    let result = some_operation_with_error();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.to_string(), "Expected error message");
}
```

### 测试 panic

```rust
#[test]
#[should_panic]
fn test_panic() {
    panic!("This should panic");
}

#[test]
#[should_panic(expected = "division by zero")]
fn test_panic_with_message() {
    let _ = 1 / 0;
}
```

---

## 📚 文档测试

### 在文档中编写测试

```rust
/// 解析分支名称
///
/// # Examples
///
/// ```
/// use workflow::parse_branch;
///
/// let (prefix, name) = parse_branch("feature/test").unwrap();
/// assert_eq!(prefix, "feature");
/// assert_eq!(name, "test");
/// ```
pub fn parse_branch(name: &str) -> Option<(&str, &str)> {
    // 实现
}
```

### 标记不运行的示例

```rust
/// # Examples
///
/// ```no_run
/// // 这段代码不会运行（例如需要网络连接）
/// let result = fetch_from_api();
/// ```
```

---

## 🔄 重构测试

### 提取测试辅助函数

```rust
// ❌ 重复代码
#[test]
fn test1() {
    let temp_dir = TempDir::new().unwrap();
    let repo = setup_git_repo(temp_dir.path());
    // 测试...
}

#[test]
fn test2() {
    let temp_dir = TempDir::new().unwrap();
    let repo = setup_git_repo(temp_dir.path());
    // 测试...
}

// ✅ 提取辅助函数
fn create_test_repo() -> (TempDir, GitRepository) {
    let temp_dir = TempDir::new().unwrap();
    let repo = setup_git_repo(temp_dir.path());
    (temp_dir, repo)
}

#[test]
fn test1() {
    let (_temp, repo) = create_test_repo();
    // 测试...
}

#[test]
fn test2() {
    let (_temp, repo) = create_test_repo();
    // 测试...
}
```

---

## ⚠️ 常见陷阱

### 1. 测试实现而非行为

```rust
// ❌ 测试实现细节
#[test]
fn test_internal_state() {
    let obj = MyObject::new();
    assert_eq!(obj.internal_field, 0);  // 内部细节
}

// ✅ 测试公共行为
#[test]
fn test_public_behavior() {
    let obj = MyObject::new();
    assert_eq!(obj.get_value(), 0);  // 公共 API
}
```

### 2. 测试太多东西

```rust
// ❌ 一个测试测太多
#[test]
fn test_everything() {
    test_parse();
    test_create();
    test_update();
    test_delete();
}

// ✅ 每个测试一个关注点
#[test]
fn test_parse() { }

#[test]
fn test_create() { }

#[test]
fn test_update() { }

#[test]
fn test_delete() { }
```

### 3. 过度使用 Mock

```rust
// ❌ Mock 太多
#[test]
fn test_with_too_many_mocks() {
    let mock1 = Mock::new();
    let mock2 = Mock::new();
    let mock3 = Mock::new();
    // 测试变得复杂且脆弱
}

// ✅ 只 Mock 必要的外部依赖
#[test]
fn test_with_minimal_mocks() {
    let mock = MockExternalService::new();
    // 测试真实的内部逻辑
}
```

---

## 📋 测试清单

编写测试时检查以下项目：

- [ ] 测试名称清晰描述被测试的内容
- [ ] 测试遵循 AAA 模式（Arrange-Act-Assert）
- [ ] 测试独立，不依赖其他测试
- [ ] 使用测试数据工厂而非硬编码
- [ ] 使用临时目录而非固定路径
- [ ] 断言有清晰的错误消息
- [ ] 慢测试标记为 `#[ignore]`
- [ ] 测试公共 API 而非实现细节
- [ ] 一个测试一个关注点
- [ ] 测试覆盖正常和错误路径

---

## 📚 相关资源

- [快速入门指南](./QUICK_START.md)
- [故障排查指南](./TROUBLESHOOTING.md)
- [Rust 测试文档](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [测试最佳实践 (Google)](https://google.github.io/eng-practices/review/reviewer/looking-for.html)

---

**最后更新**: 2025-02-11
```

创建此文件：

```bash
cat > docs/testing/BEST_PRACTICES.md << 'EOF'
# (粘贴上述内容)
EOF
```

---

### Step 5: 创建 FAQ 和故障排查文档 (30 分钟)

#### 5.1 创建 `docs/testing/FAQ.md`

```markdown
# 常见问题 (FAQ)

快速找到测试相关问题的答案。

---

## 🚀 运行测试

### Q: 如何运行所有测试？
```bash
cargo test
```

### Q: 如何运行特定 crate 的测试？
```bash
cargo test -p domain
cargo test -p http
```

### Q: 如何运行特定的测试？
```bash
# 按名称匹配
cargo test test_parse

# 精确匹配
cargo test test_parse_branch_name -- --exact
```

### Q: 如何查看测试输出？
```bash
# 显示 println! 等输出
cargo test -- --nocapture

# 显示成功的测试
cargo test -- --nocapture --show-output
```

### Q: 如何运行被忽略的测试？
```bash
# 只运行被忽略的测试
cargo test -- --ignored

# 运行所有测试（包括被忽略的）
cargo test -- --include-ignored
```

---

## 📊 覆盖率

### Q: 如何生成覆盖率报告？
```bash
make coverage
```

### Q: 如何查看 HTML 覆盖率报告？
```bash
make coverage-open
```

### Q: 覆盖率目标是多少？
- 整体目标: 80%
- 最低要求: 75%
- 核心模块 (domain, storage): 85%+

### Q: 如何提高覆盖率？
1. 运行 `make coverage-open` 查看未覆盖的代码
2. 为红色标记的代码行添加测试
3. 优先覆盖核心业务逻辑

---

## 🎨 测试数据

### Q: 如何创建测试数据？
使用测试数据工厂：

```rust
use http::testing::TestDataFactory;

let pr = TestDataFactory::github_pr()
    .with_title("My PR")
    .build();
```

### Q: 有哪些测试数据工厂？
- `http::testing::TestDataFactory` - HTTP 相关数据
- `domain::testing::TestEntityFactory` - 领域实体
- `services::testing::*` - Mock 服务
- `storage::testing::*` - Git 测试辅助

### Q: 如何创建自定义测试数据？
参考现有的数据工厂，使用构建器模式：

```rust
pub struct MyDataBuilder {
    field1: Option<String>,
    field2: Option<i32>,
}

impl MyDataBuilder {
    pub fn with_field1(mut self, value: impl Into<String>) -> Self {
        self.field1 = Some(value.into());
        self
    }

    pub fn build(self) -> MyData {
        MyData {
            field1: self.field1.unwrap_or_default(),
            field2: self.field2.unwrap_or(0),
        }
    }
}
```

---

## 🎭 Mock 测试

### Q: 如何 Mock HTTP 请求？
```rust
use http::testing::{MockServerManager, TestDataFactory};

let mut manager = MockServerManager::new();
let pr_data = TestDataFactory::github_pr().build();
let _mock = manager.setup_github_pr_list(vec![pr_data]);

let url = manager.url("github").unwrap();
// 使用 URL 进行测试
```

### Q: 如何 Mock 服务？
```rust
use services::testing::MockBranchService;

let mock = MockBranchService::new();
mock.add_branch(branch);

let branches = mock.list_branches().unwrap();
```

---

## ⚡ 性能测试

### Q: 如何运行性能基准测试？
```bash
# 所有基准测试
make bench

# 特定类型
make bench-cli
make bench-core
```

### Q: 如何查看性能报告？
```bash
open target/criterion/report/index.html
```

### Q: 如何对比性能？
```bash
# 建立基线
make bench-baseline

# 修改代码后对比
cargo bench -- --baseline initial
```

---

## 🪝 Git Hooks

### Q: 如何安装预提交钩子？
```bash
make install-hooks
```

### Q: 如何跳过预提交钩子？
```bash
git commit --no-verify -m "commit message"
```

### Q: 预提交钩子检查什么？
- 代码格式 (cargo fmt)
- Clippy 警告
- 单元测试

### Q: 如何卸载钩子？
```bash
make uninstall-hooks
```

---

## 🐛 调试测试

### Q: 测试失败如何调试？
```bash
# 1. 查看详细输出
cargo test test_name -- --nocapture

# 2. 使用 dbg! 宏
fn my_function() {
    let value = compute();
    dbg!(&value);  // 打印调试信息
}

# 3. 运行单个测试
cargo test test_name --exact
```

### Q: 如何在 IDE 中调试测试？
在 VSCode/RustRover 中：
1. 点击测试函数上的 "Debug" 按钮
2. 或使用 "Run and Debug" 面板

---

## 📦 CI/CD

### Q: CI 中运行什么测试？
- Pull Request: 单元测试 + 覆盖率
- Main 分支: 所有测试 + 性能基准

### Q: 如何查看 CI 测试结果？
访问 GitHub Actions: `https://github.com/USER/REPO/actions`

### Q: 覆盖率报告在哪里？
- Codecov: https://codecov.io/gh/USER/REPO
- PR 评论中会显示覆盖率变化

---

## 💡 最佳实践

### Q: 测试应该放在哪里？
- 单元测试: `src/` 文件中的 `#[cfg(test)] mod tests`
- 集成测试: `tests/` 目录
- 基准测试: `benches/` 目录

### Q: 测试命名有什么规范？
使用模板: `test_<function>_<scenario>_should_<result>`

例如:
- `test_parse_valid_branch_should_succeed`
- `test_create_pr_with_invalid_token_should_fail`

### Q: 如何组织测试代码？
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 按功能分组
    mod parsing {
        #[test]
        fn test_parse_valid() { }

        #[test]
        fn test_parse_invalid() { }
    }

    mod creation {
        #[test]
        fn test_create_success() { }

        #[test]
        fn test_create_error() { }
    }
}
```

---

## 🚫 常见错误

### Q: 为什么测试会随机失败？
可能原因：
1. 测试之间有依赖（共享状态）
2. 依赖时间/随机数
3. 并发问题

解决方法：
- 使用 `#[serial]` 串行运行
- 使用固定的随机种子
- 避免共享全局状态

### Q: 为什么覆盖率报告不准确？
可能原因：
1. 排除规则过于宽泛
2. 测试代码被包含
3. 生成代码被统计

解决方法：
- 检查 `coverage.toml` 配置
- 使用 `#[cfg(test)]` 标记测试代码

---

## 📚 更多资源

- [快速入门](./QUICK_START.md)
- [最佳实践](./BEST_PRACTICES.md)
- [故障排查](./TROUBLESHOOTING.md)

---

**最后更新**: 2025-02-11
```

#### 5.2 创建 `docs/testing/TROUBLESHOOTING.md`

```markdown
# 故障排查指南

解决常见测试问题的指南。

---

## 🐛 测试失败

### 问题：测试随机失败
**症状**: 同样的代码，有时通过，有时失败

**可能原因**:
1. 测试之间有共享状态
2. 依赖时间/随机数
3. 并发竞争条件

**解决方法**:
```rust
// 1. 使用 serial 串行运行
use serial_test::serial;

#[test]
#[serial]
fn test_shared_resource() {
    // 访问共享资源
}

// 2. 固定随机种子
use rand::SeedableRng;
let mut rng = rand::rngs::StdRng::seed_from_u64(42);

// 3. 避免全局状态
// 使用临时目录、Mock 等
```

---

### 问题：测试超时
**症状**: 测试运行很长时间后超时

**可能原因**:
1. 无限循环
2. 死锁
3. 等待外部资源

**解决方法**:
```bash
# 1. 增加超时时间
cargo test -- --test-threads=1 --nocapture

# 2. 使用 timeout
#[test]
#[timeout(Duration::from_secs(5))]
fn test_with_timeout() {
    // 最多运行 5 秒
}

# 3. 检查代码逻辑
# 添加 debug 输出查看卡在哪里
```

---

## 📊 覆盖率问题

### 问题：cargo-tarpaulin 安装失败
**症状**: `cargo install cargo-tarpaulin` 报错

**解决方法**:
```bash
# macOS
brew install cargo-tarpaulin

# 或使用 binary
curl -L https://github.com/xd009642/tarpaulin/releases/latest/download/cargo-tarpaulin-x86_64-unknown-linux-musl.tar.gz | tar xz
```

---

### 问题：覆盖率报告生成失败
**症状**: `make coverage` 失败

**可能原因**:
1. cargo-tarpaulin 未安装
2. 测试编译失败
3. 配置错误

**解决方法**:
```bash
# 1. 确认 tarpaulin 可用
cargo tarpaulin --version

# 2. 手动运行检查错误
cargo tarpaulin --out Html --output-dir coverage

# 3. 检查配置
cat coverage.toml
```

---

### 问题：覆盖率为 0% 或异常低
**可能原因**:
1. 排除规则过于宽泛
2. 测试未运行

**解决方法**:
```bash
# 1. 检查排除规则
grep exclude coverage.toml

# 2. 确认测试运行
cargo test

# 3. 使用 --verbose 查看详情
cargo tarpaulin --verbose
```

---

## ⚡ 性能测试问题

### 问题：基准测试运行很慢
**症状**: `cargo bench` 运行时间很长

**解决方法**:
```bash
# 1. 只运行特定基准
cargo bench --bench cli_performance

# 2. 减少样本数（在基准测试代码中）
group.sample_size(10);  // 默认 100

# 3. 减少测量时间
group.measurement_time(Duration::from_secs(5));  // 默认 10
```

---

### 问题：CLI 基准测试失败
**症状**: `bench-cli` 找不到二进制文件

**解决方法**:
```bash
# 1. 先构建 release 版本
cargo build --release

# 2. 确认二进制存在
ls -la target/release/

# 3. 检查二进制名称
# 在 benches/cli_performance.rs 中确认名称正确
```

---

## 🪝 Git Hooks 问题

### 问题：预提交钩子未运行
**可能原因**:
1. 钩子未安装
2. 权限问题

**解决方法**:
```bash
# 1. 确认钩子已安装
ls -la .git/hooks/pre-commit

# 2. 确认是符号链接
readlink .git/hooks/pre-commit

# 3. 重新安装
make uninstall-hooks
make install-hooks
```

---

### 问题：预提交钩子太慢
**症状**: 每次提交等待很久

**解决方法**:
```bash
# 1. 使用快速版钩子
make install-hooks-fast

# 2. 临时跳过
git commit --no-verify

# 3. 只在 CI 运行完整检查
# 本地只做快速检查
```

---

## 🎭 Mock 测试问题

### 问题：Mock 服务器端口冲突
**症状**: 测试失败，显示端口已被占用

**解决方法**:
```rust
// Mock 服务器会自动分配端口
let manager = MockServerManager::new();
// 不要手动指定端口

// 如果仍有问题，串行运行测试
#[test]
#[serial]
fn test_with_mock_server() {
    // ...
}
```

---

### 问题：Mock 数据未生效
**症状**: 测试调用了真实 API 而非 Mock

**解决方法**:
```rust
// 1. 确认使用了 Mock URL
let url = manager.url("github").unwrap();
let client = GitHubClient::new(&url);  // 使用 Mock URL

// 2. 确认 Mock 在请求前创建
let _mock = manager.setup_github_pr_list(...);
// Mock 必须在作用域内

// 3. 检查 Mock 匹配规则
// URL、方法、参数都要匹配
```

---

## 📦 CI/CD 问题

### 问题：CI 测试通过但本地失败
**可能原因**:
1. 环境差异
2. 依赖版本不同
3. 时区/locale 差异

**解决方法**:
```bash
# 1. 使用相同的 Rust 版本
rustup update stable

# 2. 清理并重新构建
cargo clean
cargo build
cargo test

# 3. 检查环境变量
printenv | grep RUST
```

---

### 问题：覆盖率报告未上传到 Codecov
**可能原因**:
1. Token 未配置
2. 文件路径错误
3. Codecov action 失败

**解决方法**:
```bash
# 1. 确认 Secret 已添加
# GitHub Settings > Secrets > CODECOV_TOKEN

# 2. 检查 CI 日志
# 查看 "Upload coverage to Codecov" 步骤

# 3. 手动上传测试
# 在本地运行：
bash <(curl -s https://codecov.io/bash) -t TOKEN
```

---

## 🔧 编译问题

### 问题：测试编译失败但代码编译成功
**可能原因**:
1. 测试依赖缺失
2. Feature 未启用

**解决方法**:
```bash
# 1. 检查测试编译
cargo test --no-run

# 2. 确认 dev-dependencies
grep -A 5 "dev-dependencies" Cargo.toml

# 3. 启用必要的 features
cargo test --features testing
```

---

## 💾 缓存问题

### 问题：测试结果不更新
**症状**: 修改代码后测试结果还是旧的

**解决方法**:
```bash
# 1. 清理缓存
cargo clean

# 2. 强制重新编译
cargo test --no-fail-fast

# 3. 清理增量编译
rm -rf target/debug/incremental
```

---

## 📚 获取更多帮助

如果以上方法都无法解决问题：

1. 查看详细日志:
```bash
RUST_LOG=debug cargo test -- --nocapture
```

2. 搜索类似问题:
   - [Rust 论坛](https://users.rust-lang.org/)
   - [Stack Overflow](https://stackoverflow.com/questions/tagged/rust)
   - 项目 Issue Tracker

3. 提交 Issue:
   - 包含完整的错误信息
   - 提供最小可复现示例
   - 说明环境信息 (OS, Rust 版本等)

---

**最后更新**: 2025-02-11
```

创建这两个文件：

```bash
# FAQ
cat > docs/testing/FAQ.md << 'EOF'
# (粘贴FAQ内容)
EOF

# Troubleshooting
cat > docs/testing/TROUBLESHOOTING.md << 'EOF'
# (粘贴Troubleshooting内容)
EOF
```

---

### Step 6: 为主要 Crate 创建 TESTING.md (30 分钟)

由于篇幅限制，我提供一个模板，你可以为每个 crate 复制和定制。

#### 6.1 创建 `crates/http/TESTING.md` (示例)

```markdown
# HTTP Crate 测试文档

HTTP crate 的测试指南和工具使用说明。

---

## 🎯 测试工具

### MockServerManager

管理 Mock HTTP 服务器，用于测试 HTTP 客户端。

```rust
use http::testing::{MockServerManager, TestDataFactory};

#[test]
fn test_github_api() {
    let mut manager = MockServerManager::new();

    // 设置 Mock 响应
    let pr = TestDataFactory::github_pr().build();
    let _mock = manager.setup_github_pr_list(vec![pr]);

    // 获取 Mock 服务器 URL
    let url = manager.url("github").unwrap();

    // 使用 URL 测试你的代码
    // let client = GitHubClient::new(&url);
    // let result = client.list_prs();
}
```

### TestDataFactory

创建测试数据的工厂。

```rust
use http::testing::TestDataFactory;

// GitHub PR
let pr = TestDataFactory::github_pr()
    .with_title("My PR")
    .with_head("feature/test")
    .build();

// Jira Issue
let issue = TestDataFactory::jira_issue()
    .with_summary("Bug fix")
    .with_issue_type("Bug")
    .build();
```

---

## 📝 常见测试场景

### 测试 GitHub API 调用

```rust
#[test]
fn test_list_pull_requests() {
    let mut manager = MockServerManager::new();

    let pr1 = TestDataFactory::github_pr()
        .with_title("PR 1")
        .build();
    let pr2 = TestDataFactory::github_pr()
        .with_title("PR 2")
        .build();

    let _mock = manager.setup_github_pr_list(vec![pr1, pr2]);

    // 测试逻辑...
}
```

### 测试错误处理

```rust
#[test]
fn test_api_auth_error() {
    let mut manager = MockServerManager::new();

    // 设置错误响应
    let _mock = manager.setup_error_response("github", 401, "Bad credentials");

    // 测试逻辑...
}
```

---

## 🔧 运行测试

```bash
# 运行所有测试
cargo test -p http

# 运行特定测试
cargo test -p http test_mock_server

# 带 testing feature
cargo test -p http --features testing
```

---

## 📚 API 参考

完整的 API 文档:
```bash
cargo doc -p http --features testing --open
```

---

**相关文档**: [测试快速入门](../../docs/testing/QUICK_START.md)
```

为其他 crate 创建类似的文档。

---

## ✅ 验证和测试

### 完整验证流程

```bash
# 1. 确认所有文档文件创建
ls -la docs/testing/
ls -la crates/*/TESTING.md

# 2. 检查 markdown 格式
# 使用 markdown linter 或在 GitHub 上预览

# 3. 验证链接有效
# 点击文档中的所有链接

# 4. 让团队成员审阅
# 确保文档易懂、准确

# 5. 更新主 README
# 添加测试章节链接
```

---

## 📋 检查清单

实施完成后，确认以下项目：

- [ ] `docs/testing/README.md` 已创建
- [ ] `docs/testing/QUICK_START.md` 已创建
- [ ] `docs/testing/BEST_PRACTICES.md` 已创建
- [ ] `docs/testing/FAQ.md` 已创建
- [ ] `docs/testing/TROUBLESHOOTING.md` 已创建
- [ ] 主要 crate 有 `TESTING.md` 文档
- [ ] 主 README 添加了测试章节
- [ ] 所有链接都是有效的
- [ ] 文档经过审阅
- [ ] 团队成员能理解文档

---

**文档版本**: 1.0
**创建日期**: 2025-02-11
**最后更新**: 2025-02-11
**完成**: 所有测试文档实施指南已完成
