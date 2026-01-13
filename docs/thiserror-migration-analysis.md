# thiserror 迁移分析

本文档分析项目中可以使用 `thiserror` 代替 `color-eyre` 的地方。

## 说明

`thiserror` 和 `color-eyre` 的定位不同：
- **`thiserror`**: 用于定义结构化的自定义错误类型，自动实现 `Display` 和 `Error` trait
- **`color-eyre`**: 用于错误传播和报告，在应用顶层使用

它们可以配合使用：
- 使用 `thiserror` 定义模块/库级别的错误类型
- 使用 `color-eyre` 在应用顶层进行错误传播和报告

## 可以迁移的地方

### 1. ✅ `PromptError` - 高优先级

**位置**: `src/lib/base/interactive/error.rs`

**当前实现**: 手动实现了 `Display` 和 `Error` trait，以及 `From<std::io::Error>`

**建议**: 使用 `thiserror` 简化代码

**当前代码**:
```rust
#[derive(Debug)]
pub enum PromptError {
    Io(std::io::Error),
    Terminal(String),
    Validation(String),
    Cancelled,
    InvalidInput(String),
    TerminalNotSupported,
}

impl std::fmt::Display for PromptError { ... }
impl std::error::Error for PromptError { ... }
impl From<std::io::Error> for PromptError { ... }
```

**使用 thiserror 后**:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PromptError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Terminal error: {0}")]
    Terminal(String),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("User cancelled")]
    Cancelled,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Terminal not supported")]
    TerminalNotSupported,
}
```

**收益**:
- 减少约 30 行样板代码
- 自动实现 `Display` 和 `Error` trait
- 自动实现 `From<std::io::Error>`（通过 `#[from]` 属性）

---

### 2. ✅ HTTP 方法解析错误 - 中优先级

**位置**: `src/lib/base/http/method.rs`

**当前实现**: `FromStr` 返回 `color_eyre::eyre::Report`

**建议**: 定义自定义错误类型

**当前代码**:
```rust
impl FromStr for HttpMethod {
    type Err = color_eyre::eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            // ...
            _ => color_eyre::eyre::bail!("Invalid HTTP method: {}", s),
        }
    }
}
```

**使用 thiserror 后**:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpMethodError {
    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),
}

impl FromStr for HttpMethod {
    type Err = HttpMethodError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            // ...
            _ => Err(HttpMethodError::InvalidMethod(s.to_string())),
        }
    }
}
```

**收益**:
- 类型安全：错误类型明确
- 更好的错误处理：可以匹配特定错误类型
- 库级别的错误定义，不依赖应用级别的 `color-eyre`

---

### 3. ⚠️ GitHub 错误处理 - 中优先级（需要评估）

**位置**: `src/lib/pr/github/errors.rs`

**当前实现**: 函数返回 `color_eyre::eyre::Report`

**建议**: 可以考虑定义自定义错误类型，但需要评估是否值得

**当前代码**:
```rust
pub fn format_error(error: &GitHubErrorResponse, response: &HttpResponse) -> Report {
    // ...
    eyre!(msg)
}

pub fn handle_github_error(response: &HttpResponse) -> Report {
    // ...
    eyre!(msg)
}
```

**使用 thiserror 后**:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("GitHub API error: {message} (Status: {status})")]
    ApiError {
        message: String,
        status: u16,
        details: Option<String>,
    },
    #[error("Failed to parse GitHub error response: {0}")]
    ParseError(String),
}

pub fn format_error(error: &GitHubErrorResponse, response: &HttpResponse) -> GitHubError {
    // ...
    GitHubError::ApiError { ... }
}
```

**收益**:
- 类型安全的错误处理
- 可以匹配特定错误类型
- 但需要修改调用方的错误处理代码

**权衡**:
- 如果这些函数主要在应用层使用，保持 `Report` 可能更合适
- 如果这些函数在库层使用，定义错误类型更好

---

### 4. ⚠️ 其他使用 `eyre!` 的地方 - 低优先级

**统计**: 项目中约有 139 处使用 `eyre!` 或 `eyre::eyre!`

**建议**: 对于以下情况，可以考虑定义错误类型：
- 重复出现的错误模式
- 需要程序化处理的错误（如重试逻辑）
- 库/模块级别的公共 API

**示例位置**:
- `src/lib/base/http/retry.rs` - 重试相关的错误
- `src/lib/base/http/response.rs` - HTTP 响应错误
- `src/lib/base/http/client.rs` - HTTP 客户端错误
- `src/lib/jira/` - Jira 相关的错误
- `src/lib/git/` - Git 操作相关的错误

**评估标准**:
1. 错误是否在多个地方重复出现？
2. 是否需要程序化处理（如重试、分类）？
3. 是否是公共 API 的一部分？
4. 是否需要在库层使用（而非应用层）？

如果满足以上条件，建议定义错误类型。

---

## 迁移建议

### 阶段 1: 高优先级（立即迁移）
1. ✅ `PromptError` - 明显的收益，代码简化

### 阶段 2: 中优先级（评估后迁移）
2. ✅ HTTP 方法解析错误 - 库级别的错误定义
3. ⚠️ GitHub 错误处理 - 需要评估调用方影响

### 阶段 3: 低优先级（按需迁移）
4. ⚠️ 其他使用 `eyre!` 的地方 - 根据实际需求决定

---

## 实施步骤

### 1. 添加 thiserror 依赖

在 `Cargo.toml` 中添加：
```toml
[dependencies]
thiserror = "1.0"
```

### 2. 迁移 PromptError

1. 修改 `src/lib/base/interactive/error.rs`
2. 使用 `#[derive(Error)]` 和 `#[error(...)]` 属性
3. 移除手动的 `Display` 和 `Error` 实现
4. 使用 `#[from]` 属性自动实现 `From` trait

### 3. 迁移 HTTP 方法错误

1. 在 `src/lib/base/http/method.rs` 中定义 `HttpMethodError`
2. 修改 `FromStr` 实现
3. 更新调用方（如果有）

### 4. 测试

确保所有测试通过，特别是：
- `src/lib/base/interactive/` 的测试
- `src/lib/base/http/` 的测试

---

## 注意事项

1. **不要完全替换 color-eyre**: `color-eyre` 仍然需要在应用顶层使用（`main` 函数、错误报告等）
2. **保持兼容性**: 确保错误类型可以转换为 `eyre::Report`（通过实现 `std::error::Error`）
3. **渐进式迁移**: 不需要一次性迁移所有地方，可以逐步进行
4. **评估收益**: 对于简单的临时错误（如 `eyre!("message")`），可能不需要定义错误类型

---

## 总结

**推荐立即迁移**:
- ✅ `PromptError` - 明显的代码简化收益

**推荐评估后迁移**:
- ✅ HTTP 方法解析错误 - 库级别的错误定义
- ⚠️ GitHub 错误处理 - 需要评估影响

**按需迁移**:
- ⚠️ 其他使用 `eyre!` 的地方 - 根据实际需求决定
