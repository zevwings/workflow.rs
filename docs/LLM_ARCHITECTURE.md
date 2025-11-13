# LLM 统一配置驱动架构

## 📋 概述

本文档描述了 Workflow CLI 中 LLM（大语言模型）客户端的统一配置驱动架构。该架构通过**统一客户端**和**Settings 配置系统**，实现所有 LLM 提供商的统一调用，消除代码重复，支持通过 `workflow.toml` 配置文件持久化提供商配置。

### 核心设计原则

1. **统一客户端**：所有 LLM 提供商使用同一个客户端实现
2. **配置驱动**：所有参数（URL、API Key、Model、Response Format）从 `Settings` 动态获取
3. **易于扩展**：添加新的 LLM 提供商只需配置，无需写代码
4. **向后兼容**：保持现有 API 不变，支持从 `workflow.toml` 配置

### 为什么选择统一配置驱动方案？

基于 API 调用分析（参见 [LLM_PLUGIN_CURL.md](./LLM_PLUGIN_CURL.md)），所有 LLM 提供商都遵循 **OpenAI 兼容格式**：

- ✅ **请求格式完全相同**：都使用 POST 到 `/v1/chat/completions`，请求体结构相同
- ✅ **响应格式完全相同**：都从 `choices[0].message.content` 提取内容
- ✅ **唯一差异**：URL 和 API Key（配置差异，非代码差异）

**结论**：**不需要传统插件系统**（trait、registry、manager），只需要**配置驱动 + 统一客户端**方案。

---

## 🏗️ 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                  PullRequestLLM                         │
│  (业务层：生成 PR 内容)                                   │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│              LLMClient                                  │
│  (统一客户端，处理所有 LLM 调用)                          │
│  - build_url()     从 Settings 获取 URL                  │
│  - build_model()   从 Settings 获取 Model                │
│  - build_headers() 从 Settings 获取 API Key              │
│  - build_payload() 构建请求体                             │
│  - extract_content() 根据 response_format 提取内容       │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│              Settings                                   │
│  (配置系统：从 workflow.toml 读取)                         │
│  - llm.provider     提供商名称 (openai/deepseek/proxy)   │
│  - llm.url          API URL (仅 proxy 需要)               │
│  - llm.key          API Key                               │
│  - llm.model        模型名称                               │
│  - llm.response_format 响应格式路径                       │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
            ┌────────────────┐
            │  HTTP Client   │
            │  (reqwest)     │
            └────────────────┘
```

### 核心组件

#### 1. LLMClient（统一客户端）

所有 LLM 提供商使用同一个客户端实现，通过 `Settings` 配置区分不同的提供商：

```rust
pub struct LLMClient;

impl LLMClient {
    pub fn new() -> Self {
        Self
    }

    pub fn call(&self, params: &LLMRequestParams) -> Result<String> {
        // 1. 从 Settings 获取配置
        // 2. 构建 URL、Headers、Payload
        // 3. 发送 HTTP 请求
        // 4. 根据 response_format 提取内容
    }
}
```

**关键特性**：
- ✅ **无状态**：不存储配置，每次调用时从 `Settings::get()` 获取
- ✅ **动态配置**：所有配置（URL、Key、Model）都从 `Settings` 动态获取
- ✅ **统一处理**：所有提供商使用相同的请求和响应处理逻辑
- ✅ **超时控制**：60 秒超时设置

#### 2. Settings（配置系统）

配置存储在 `workflow.toml` 文件的 `[llm]` 部分：

```toml
[llm]
provider = "openai"  # 或 "deepseek" 或 "proxy"
key = "sk-xxx"        # API Key
model = "gpt-4.0"     # 可选，openai/deepseek 有默认值
url = "https://..."   # 仅 proxy 需要
response_format = "choices[0].message.content"  # 可选，有默认值
```

**配置字段说明**：

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `provider` | String | ✅ | 提供商名称：`openai`、`deepseek`、`proxy` |
| `key` | String | ✅ | API Key |
| `url` | String | ⚠️ | API URL（仅 `proxy` 提供商需要） |
| `model` | String | ⚠️ | 模型名称（`openai`/`deepseek` 有默认值，`proxy` 必填） |
| `response_format` | String | ❌ | 响应格式路径（默认：`choices[0].message.content`） |

**默认值**：

- `provider`: `"openai"`
- `model`:
  - `openai`: `"gpt-4.0"`
  - `deepseek`: `"deepseek-chat"`
  - `proxy`: 无默认值，必须配置
- `response_format`: `"choices[0].message.content"`

#### 3. PullRequestLLM（业务层）

使用统一客户端生成 PR 内容：

```rust
impl PullRequestLLM {
    pub fn generate(...) -> Result<PullRequestContent> {
        let client = LLMClient::new();
        let params = LLMRequestParams { ... };
        let response = client.call(&params)?;
        Self::parse_llm_response(response)
    }
}
```

---

## 🔌 配置方式

### 配置文件（推荐）

通过 `workflow setup` 命令交互式配置，或直接编辑 `workflow.toml`：

```toml
[llm]
provider = "openai"
key = "sk-xxx"
model = "gpt-4.0"
```

### 支持的提供商

#### OpenAI

```toml
[llm]
provider = "openai"
key = "sk-xxx"
model = "gpt-4.0"  # 可选，默认 "gpt-4.0"
```

**自动配置**：
- URL: `https://api.openai.com/v1/chat/completions`（自动设置，无需配置）

#### DeepSeek

```toml
[llm]
provider = "deepseek"
key = "sk-xxx"
model = "deepseek-chat"  # 可选，默认 "deepseek-chat"
```

**自动配置**：
- URL: `https://api.deepseek.com/chat/completions`（自动设置，无需配置）

#### Proxy（代理 API）

```toml
[llm]
provider = "proxy"
url = "https://proxy.example.com"  # 必需
key = "your-api-key"                # 必需
model = "qwen-3-235b"               # 必需
```

**自动配置**：
- URL: `{url}/chat/completions`（自动拼接 `/chat/completions`）

---

## 📁 文件结构

```
src/lib/llm/
├── mod.rs                    # 模块导出
├── pr_llm.rs                 # PullRequestLLM（业务层）
└── client/
    ├── mod.rs                 # 客户端模块导出
    ├── llm_client.rs         # LLMClient（统一客户端）
    └── types.rs              # 共享类型（LLMRequestParams）
```

---

## 🔧 实现细节

### 1. URL 构建

根据 `provider` 动态构建 URL：

```rust
fn build_url(&self) -> Result<String> {
    let settings = Settings::get();
    match settings.llm.provider.as_str() {
        "openai" => Ok("https://api.openai.com/v1/chat/completions".to_string()),
        "deepseek" => Ok("https://api.deepseek.com/chat/completions".to_string()),
        "proxy" => {
            let base_url = settings.llm.url.as_ref()
                .context("LLM proxy URL is not configured")?;
            Ok(format!("{}/chat/completions", base_url.trim_end_matches('/')))
        }
        _ => Err(anyhow::anyhow!("Unsupported LLM provider: {}", provider)),
    }
}
```

### 2. Model 构建

根据 `provider` 获取模型名称：

```rust
fn build_model(&self) -> Result<String> {
    let settings = Settings::get();
    match settings.llm.provider.as_str() {
        "openai" | "deepseek" => {
            Ok(settings.llm.model.clone()
                .unwrap_or_else(|| default_llm_model(&settings.llm.provider)))
        }
        "proxy" => {
            settings.llm.model.clone()
                .context("Model is required for proxy provider")
        }
        _ => Err(anyhow::anyhow!("Unsupported LLM provider")),
    }
}
```

### 3. 响应内容提取

根据 `response_format` 配置提取内容：

```rust
fn extract_content(&self, response: &serde_json::Value) -> Result<String> {
    let settings = Settings::get();
    let response_format = &settings.llm.response_format;

    if response_format.is_empty() || *response_format == default_response_format() {
        // 标准 OpenAI 格式：choices[0].message.content
        response
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|msg| msg.get("content"))
            .and_then(|c| c.as_str())
            .context("Failed to extract content")
            .map(|s| s.trim().to_string())
    } else {
        // 自定义 JSON path 提取
        self.extract_by_path(response, response_format)
    }
}
```

**支持的响应格式**：
- **标准格式**：`choices[0].message.content`（默认）
- **自定义格式**：支持 JSON path，如 `candidates[0].content.parts[0].text`

### 4. 错误处理

- **API Key 为空**：返回错误 `"LLM key is empty in settings"`
- **Proxy URL 未配置**：返回错误 `"LLM proxy URL is not configured"`
- **Proxy Model 未配置**：返回错误 `"Model is required for proxy provider"`
- **HTTP 请求失败**：返回详细的错误信息，包含状态码和响应体

---

## ✅ 优势分析

### 1. 统一客户端
- ✅ 所有 LLM 提供商使用同一个客户端实现
- ✅ **消除代码重复**：从 ~300 行（3 个独立客户端）减少到 ~450 行（1 个统一客户端）
- ✅ 统一的错误处理和请求逻辑

### 2. 配置驱动
- ✅ 所有参数从 `Settings` 读取（URL、API Key、Model、Response Format）
- ✅ **易于扩展**：添加新提供商只需配置，无需写代码
- ✅ 支持自定义响应格式（通过 JSON path）

### 3. 持久化配置
- ✅ 配置存储在 `workflow.toml`，与项目配置统一管理
- ✅ 通过 `workflow setup` 交互式配置
- ✅ 支持版本控制（可以提交到 Git 仓库）

### 4. 向后兼容
- ✅ 保持现有 API 不变
- ✅ 平滑迁移路径
- ✅ 配置简单直观

### 5. 维护成本低
- ✅ **单一代码路径**：所有客户端都走统一客户端
- ✅ **无需维护多套实现**：添加多个客户端时，只需配置
- ✅ **代码一致性高**：所有客户端使用相同的逻辑

---

## 📝 使用示例

### 示例 1：基本使用

```rust
use crate::llm::pr_llm::PullRequestLLM;

let content = PullRequestLLM::generate(
    "Fix login bug",
    None,
    None
)?;

println!("Branch: {}", content.branch_name);
println!("PR Title: {}", content.pr_title);
```

### 示例 2：配置 OpenAI

```toml
# workflow.toml
[llm]
provider = "openai"
key = "sk-xxx"
model = "gpt-4.0"
```

### 示例 3：配置 Proxy

```toml
# workflow.toml
[llm]
provider = "proxy"
url = "https://proxy.example.com"
key = "your-api-key"
model = "qwen-3-235b"
```

---

## 📚 相关文档

- [LLM_PLUGIN_CURL.md](./LLM_PLUGIN_CURL.md) - API 调用示例和响应格式分析
- [LLM_PLUGIN_ARCHITECTURE.md](./LLM_PLUGIN_ARCHITECTURE.md) - 详细架构设计（包含 TOML 配置文件系统设计，仅供参考）
- [LLM_IMPLEMENTATION.md](./LLM_IMPLEMENTATION.md) - 实现指南（包含 TOML 配置文件系统实现，仅供参考）
- [CONFIG_ARCHITECTURE.md](./CONFIG_ARCHITECTURE.md) - Settings 配置系统架构

---

## 🔍 总结

该统一配置驱动方案提供了：

1. **统一客户端**：所有 LLM 提供商使用同一个客户端实现，消除代码重复
2. **配置驱动**：所有参数从 `Settings` 读取，添加新提供商只需配置
3. **持久化配置**：通过 `workflow.toml` 支持提供商配置的持久化存储
4. **易于扩展**：添加新提供商只需配置，无需写代码
5. **向后兼容**：保持现有 API 不变
6. **维护成本低**：只需维护一个统一客户端

### 核心原则

- **配置驱动**：所有差异通过配置解决
- **统一实现**：所有提供商使用同一个客户端
- **简单高效**：无需复杂的插件系统

### 当前实现状态

✅ **已实现**：
- 统一 `LLMClient` 实现
- 基于 `Settings` 的配置系统
- 支持 OpenAI、DeepSeek、Proxy 提供商
- 自定义响应格式支持（JSON path）

⚠️ **文档说明**：
- `LLM_PLUGIN_ARCHITECTURE.md` 和 `LLM_IMPLEMENTATION.md` 中描述的独立 `llm.toml` 配置文件系统**尚未实现**
- 当前实现使用 `workflow.toml` 的 `[llm]` 部分进行配置
- 未来可能会实现独立的 `llm.toml` 配置文件系统以支持多提供商配置

