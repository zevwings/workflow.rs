# 安全性规则

> 本文档定义了 Workflow CLI 项目的安全性规则和最佳实践，所有贡献者都应遵循这些规范。

---

## 📋 目录

- [概述](#-概述)
- [API Token 和敏感信息处理](#-api-token-和敏感信息处理)
- [输入验证和清理](#-输入验证和清理)
- [依赖安全检查](#-依赖安全检查)
- [安全性检查清单](#-安全性检查清单)
- [相关文档](#-相关文档)

---

## 📋 概述

本文档定义了安全性规则，包括 API Token 处理、输入验证和依赖安全检查。

### 核心原则

- **禁止硬编码**：禁止在代码中硬编码敏感信息
- **输入验证**：所有用户输入必须验证和清理
- **日志过滤**：日志中自动过滤敏感信息
- **依赖安全**：定期检查依赖安全漏洞

### 使用场景

- 编写涉及敏感信息的代码时参考
- 处理用户输入时使用
- 添加新依赖时检查

---

## API Token 和敏感信息处理

### 禁止硬编码敏感信息

**规则**：禁止在代码中硬编码 API Token、密码等敏感信息。

```rust
// ❌ 禁止：硬编码 API Token
const GITHUB_TOKEN: &str = "ghp_xxxxxxxxxxxxxxxxxxxx";

// ✅ 正确：从配置文件或环境变量读取
let token = Settings::get().github.token
    .ok_or_else(|| anyhow!("GitHub token not configured"))?;
```

### 使用配置文件或环境变量

**规则**：敏感信息应存储在配置文件或环境变量中。

1. **配置文件存储**：
   - 配置文件位置：`~/.workflow/config/workflow.toml`（macOS/Linux）或 `%APPDATA%\workflow\config\workflow.toml`（Windows）
   - 配置文件不应提交到代码仓库
   - 配置文件应设置适当的文件权限（仅所有者可读）

2. **环境变量存储**：
   - 敏感信息可以通过环境变量传递
   - 环境变量名称应清晰明确（如 `GITHUB_TOKEN`、`JIRA_API_TOKEN`）
   - 在文档中说明环境变量的使用方法

### 日志中自动过滤敏感信息

**规则**：所有日志输出前必须过滤敏感信息。

**使用 `mask_sensitive_value` 函数**：

```rust
use crate::base::util::string::mask_sensitive_value;

// ❌ 不安全：直接输出敏感信息
trace_info!("API token: {}", token);

// ✅ 安全：使用 mask_sensitive_value 过滤
trace_info!("API token: {}", mask_sensitive_value(token));
```

**使用 `Sensitive` trait**：

```rust
use crate::base::util::string::Sensitive;

// ❌ 不安全：直接输出敏感信息
log_info!("API token: {}", token);

// ✅ 安全：使用 mask() 方法过滤
log_info!("API token: {}", token.mask());
```

**敏感信息类型**：
- API Token（GitHub、Jira、LLM 等）
- 密码和密钥
- 用户输入中的敏感信息
- 配置文件路径中的敏感信息（如包含用户名的路径）

**参考**：详细的敏感信息过滤规则见 [日志和调试规范](./logging.md) 中的"敏感信息过滤规则"部分。

### 配置导出时过滤敏感信息

**规则**：配置导出时必须过滤敏感信息。

**实现方式**：
- 使用 `filter_secrets` 函数过滤敏感字段
- 敏感字段应显示为 `***` 或 `[REDACTED]`
- 导出配置前必须验证敏感信息已过滤

**参考**：`src/commands/config/export.rs` 中的 `filter_secrets` 实现。

---

## 输入验证和清理

### 所有用户输入必须验证

**规则**：所有用户输入必须验证和清理，防止注入攻击。

**使用 `InputDialog` 的验证器**：

```rust
use crate::base::dialog::InputDialog;

// ✅ 正确：使用验证器验证输入
let input = InputDialog::new("Enter JIRA ID")
    .validator(|input: &str| {
        if input.is_empty() {
            Err("JIRA ID cannot be empty".into())
        } else if !input.contains('-') {
            Err("Invalid JIRA ID format (expected: PROJ-123)".into())
        } else {
            Ok(())
        }
    })
    .interact()?;
```

**验证规则**：
- 检查输入是否为空
- 检查输入格式是否正确（如 JIRA ID 格式、URL 格式）
- 检查输入长度是否在合理范围内
- 检查输入是否包含非法字符

### 防止注入攻击

**规则**：防止命令注入、路径遍历等攻击。

1. **命令注入防护**：
   - 避免直接拼接用户输入到命令中
   - 使用参数化命令执行（如 `duct::cmd`）
   - 验证命令参数的安全性

```rust
// ❌ 不安全：直接拼接用户输入
let command = format!("git checkout {}", user_input);
std::process::Command::new("sh")
    .arg("-c")
    .arg(&command)
    .output()?;

// ✅ 安全：使用参数化命令
duct::cmd("git", &["checkout", &user_input])
    .run()?;
```

2. **路径遍历防护**：
   - 文件路径必须规范化
   - 检查路径是否在允许的目录范围内
   - 使用 `Path::canonicalize()` 规范化路径

```rust
use std::path::Path;

// ✅ 安全：规范化路径并检查
let path = Path::new(&user_input)
    .canonicalize()
    .context("Invalid path")?;

// 检查路径是否在允许的目录内
let allowed_dir = Path::new("/allowed/directory")
    .canonicalize()?;

if !path.starts_with(&allowed_dir) {
    bail!("Path outside allowed directory");
}
```

3. **URL 验证**：
   - 验证 URL 格式是否正确
   - 检查 URL 协议（只允许 `http://` 或 `https://`）
   - 验证 URL 主机名

```rust
// ✅ 安全：验证 URL 格式
fn validate_url(url: &str) -> Result<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        bail!("URL must start with http:// or https://");
    }

    // 进一步验证 URL 格式
    url::Url::parse(url)
        .context("Invalid URL format")?;

    Ok(())
}
```

---

## 依赖安全检查

### 使用 cargo audit 检查安全漏洞

**规则**：定期使用 `cargo audit` 检查依赖安全漏洞。

**检查频率**：
- 每次添加新依赖前
- 每月定期检查
- 发布前必须检查

**检查命令**：
```bash
# 安装 cargo-audit
cargo install cargo-audit

# 检查安全漏洞
cargo audit
```

**参考**：详细的依赖安全检查流程见 [依赖管理规范](./dependency-management.md) 中的"依赖安全漏洞处理流程"部分。

### 定期更新依赖

**规则**：定期更新依赖以修复安全漏洞。

**更新优先级**：
- **安全补丁**：最高优先级，应立即更新
- **补丁版本更新**：可以自动更新
- **次版本更新**：需要测试验证
- **主版本更新**：需要全面评估和测试

**参考**：详细的依赖更新规则见 [依赖管理规范](./dependency-management.md) 中的"依赖更新频率要求"部分。

### 新增依赖前检查

**规则**：新增依赖前检查是否有已知安全漏洞。

**检查步骤**：
1. 使用 `cargo audit` 检查依赖是否有已知漏洞
2. 检查依赖的维护状态（是否活跃维护）
3. 检查依赖的许可证是否兼容
4. 评估依赖的安全记录

---

## 安全性检查清单

在进行代码开发时，使用以下检查清单确保安全性：

### API Token 和敏感信息检查

- [ ] 没有硬编码 API Token、密码等敏感信息
- [ ] 敏感信息从配置文件或环境变量读取
- [ ] 日志输出前已过滤敏感信息（使用 `mask_sensitive_value` 或 `Sensitive` trait）
- [ ] 配置导出时已过滤敏感信息
- [ ] 配置文件设置了适当的文件权限

### 输入验证检查

- [ ] 所有用户输入都经过验证和清理
- [ ] 使用 `InputDialog` 的验证器验证输入
- [ ] 防止命令注入（使用参数化命令执行）
- [ ] 防止路径遍历（规范化路径并检查范围）
- [ ] URL 格式验证（只允许 `http://` 或 `https://`）

### 依赖安全检查

- [ ] 新增依赖前已检查安全漏洞（`cargo audit`）
- [ ] 定期更新依赖以修复安全漏洞
- [ ] 安全补丁已及时更新
- [ ] 依赖的维护状态良好

---

## 🔍 故障排除

### 问题 1：敏感信息泄露

**症状**：日志或配置导出中包含敏感信息

**解决方案**：

1. 使用 `mask_sensitive_value()` 或 `Sensitive` trait 过滤敏感信息
2. 检查所有日志输出点
3. 检查配置导出功能

### 问题 2：输入验证不足

**症状**：用户输入未经过验证导致安全问题

**解决方案**：

1. 使用 `InputDialog` 的验证器验证输入
2. 防止命令注入（使用参数化命令执行）
3. 防止路径遍历（规范化路径并检查范围）

---

## 📚 相关文档

### 开发规范

- [日志和调试规范](./logging.md) - 敏感信息过滤规则
- [依赖管理规范](./dependency-management.md) - 依赖安全漏洞处理流程和依赖更新频率要求

### 代码实现

- `src/lib/base/util/string.rs` - 敏感信息处理工具（`mask_sensitive_value`、`Sensitive` trait）
- `src/commands/config/export.rs` - 配置导出时的敏感信息过滤（`filter_secrets`）
- `src/lib/base/dialog/input.rs` - 输入验证机制（`InputDialog` 验证器）

---

## ✅ 检查清单

使用本规范时，请确保：

- [ ] 没有硬编码敏感信息
- [ ] 敏感信息从配置文件或环境变量读取
- [ ] 日志输出前已过滤敏感信息
- [ ] 所有用户输入都经过验证和清理
- [ ] 新增依赖前已检查安全漏洞

---

**最后更新**: 2025-12-23

