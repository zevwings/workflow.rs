# LLM 模块架构文档

## 📋 概述

LLM 模块是 Workflow CLI 的核心模块，提供统一配置驱动的 LLM（大语言模型）客户端实现和配置管理功能。该模块采用分层架构设计，包括：

- **Lib 层**（`lib/base/llm/`）：提供统一配置驱动的 LLM 客户端实现，支持 OpenAI、DeepSeek 和代理 API。所有 LLM 提供商都遵循 OpenAI 兼容格式，使用相同的请求和响应处理逻辑
- **Commands 层**（`commands/llm/`）：提供 CLI 命令封装，处理用户交互，包括 LLM 配置的交互式设置和查看功能

LLM 模块通过**统一客户端**和**Settings 配置系统**，实现所有 LLM 提供商的统一调用，支持 OpenAI、DeepSeek 和代理 API。

**模块统计：**
- Lib 层代码行数：约 790 行
- Commands 层代码行数：约 406 行
- 命令数量：2 个（setup, show）
- 文件数量：Lib 层 4 个，Commands 层 3 个
- 支持提供商：OpenAI、DeepSeek、Proxy（代理 API）
- 主要结构体：`LLMClient`、`LLMRequestParams`、`PullRequestLLM`、`PullRequestContent`、`PullRequestSummary`

### 核心设计原则

1. **统一客户端**：所有 LLM 提供商使用同一个客户端实现
2. **配置驱动**：所有参数（URL、API Key、Model、Response Format）从 `Settings` 动态获取
3. **易于扩展**：添加新的 LLM 提供商只需配置，无需写代码
4. **向后兼容**：保持现有 API 不变，支持从 `workflow.toml` 配置

### 为什么选择统一配置驱动方案？

基于 API 调用分析，所有 LLM 提供商都遵循 **OpenAI 兼容格式**：

- ✅ **请求格式完全相同**：都使用 POST 到 `/v1/chat/completions` 或 `/chat/completions`，请求体结构相同
- ✅ **响应格式完全相同**：都从 `choices[0].message.content` 提取内容（或通过自定义 JSON path）
- ✅ **唯一差异**：URL 和 API Key（配置差异，非代码差异）

**结论**：**不需要传统插件系统**（trait、registry、manager），只需要**配置驱动 + 统一客户端**方案。

---

## 📁 Lib 层架构（核心业务逻辑）

LLM 模块（`lib/base/llm/`）是 Workflow CLI 的核心库模块，提供统一配置驱动的 LLM（大语言模型）客户端实现。

### 模块结构

```
src/lib/base/llm/
├── mod.rs          # LLM 模块声明和导出 (12行)
├── client.rs       # LLMClient 统一客户端 (503行)
└── types.rs        # LLMRequestParams 类型定义 (34行)
```

### 业务层封装

```
src/lib/pr/llm.rs   # PullRequestLLM 业务层封装 (253行)
```

**职责**：
- 提供 PR 专用的 LLM 服务（生成分支名、PR 标题、描述）
- 封装 LLM 调用逻辑，提供业务友好的接口

### 依赖模块

- **`lib/base/settings/`**：配置管理（从 `workflow.toml` 读取 LLM 配置）
- **`lib/base/http/`**：HTTP 响应处理（`HttpResponse`）
- **`lib/pr/helpers/`**：PR 辅助函数（分支名转换等）

### 模块集成

#### PR 模块集成

**使用场景**：
- 生成 PR 分支名
- 生成 PR 标题
- 生成 PR 描述
- 生成 PR 总结文档

**关键调用**：
```rust
// 生成分支名、PR 标题和描述
let llm = PullRequestLLM::new(commits, config);
let result = llm.generate()?;

// 生成 PR 总结
let summary = PullRequestLLM::summarize_pr(&pr_title, &pr_diff, language)?;
```

**位置**：`src/lib/pr/llm.rs`

---

## 🏗️ Lib 层架构设计

### 设计原则

1. **统一客户端**：所有 LLM 提供商使用同一个客户端实现
2. **配置驱动**：所有参数从 `Settings` 动态获取
3. **单例模式**：使用 `OnceLock` 实现线程安全的全局单例
4. **无状态设计**：客户端不存储配置，每次调用时从 `Settings` 获取
5. **统一错误处理**：所有提供商使用相同的错误处理逻辑

### 核心组件

#### 1. LLMClient（统一客户端）

**职责**：提供所有 LLM 提供商的统一调用接口

**位置**：`src/lib/base/llm/client.rs`

**关键方法**：
- `global()` - 获取全局单例
- `call()` - 调用 LLM API
- `build_url()` - 构建 API URL（根据 provider 动态构建）
- `build_headers()` - 构建请求头（从 Settings 获取 API Key）
- `build_model()` - 构建模型名称（根据 provider 获取默认值）
- `build_payload()` - 构建请求体（统一格式）
- `extract_content()` - 提取响应内容（支持标准格式和自定义 JSON path）
- `extract_by_path()` - 通过 JSON path 提取内容
- `handle_error()` - 统一错误处理

**关键特性**：
- ✅ **单例模式**：使用 `OnceLock` 实现线程安全的全局单例
- ✅ **无状态**：不存储配置，每次调用时从 `Settings::get()` 获取
- ✅ **动态配置**：所有配置（URL、Key、Model）都从 `Settings` 动态获取
- ✅ **统一处理**：所有提供商使用相同的请求和响应处理逻辑
- ✅ **超时控制**：60 秒超时设置
- ✅ **自定义响应格式**：支持通过 JSON path 提取内容

#### 2. LLMRequestParams（请求参数）

**职责**：定义 LLM API 请求参数

**位置**：`src/lib/base/llm/types.rs`

**字段**：
- `system_prompt` - 系统提示词
- `user_prompt` - 用户提示词
- `max_tokens` - 最大 token 数
- `temperature` - 温度参数（控制输出的随机性）
- `model` - 模型名称（实际使用时从 Settings 获取）

#### 3. PullRequestLLM（业务层）

**职责**：提供 PR 专用的 LLM 服务

**位置**：`src/lib/pr/llm.rs`

**关键方法**：
- `generate()` - 生成分支名、PR 标题和描述
- `summarize_pr()` - 生成 PR 总结文档和文件名（支持多语言）
- `system_prompt()` - 生成系统提示词
- `user_prompt()` - 生成用户提示词
- `parse_llm_response()` - 解析 LLM 响应
- `parse_summary_response()` - 解析 PR 总结响应（JSON 格式）

**关键特性**：
- ✅ **业务封装**：封装 LLM 调用逻辑，提供业务友好的接口
- ✅ **智能生成**：根据 commit 标题和 Git diff 生成分支名和 PR 标题
- ✅ **PR 总结**：使用 LLM 生成 PR 的详细总结文档和文件名，支持多语言（en, zh, zh-CN, zh-TW, ja, ko, de 等）
- ✅ **多语言支持**：自动翻译非英文内容为英文，支持配置语言偏好
- ✅ **响应解析**：支持 JSON 和 Markdown 代码块格式
- ✅ **文件名生成**：LLM 根据 PR 内容自动生成合适的文件名

#### 4. Settings（配置系统）

**职责**：从 `workflow.toml` 读取 LLM 配置

**配置位置**：`workflow.toml` 文件的 `[llm]` 部分

**配置字段**：

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `provider` | String | ✅ | 提供商名称：`openai`、`deepseek`、`proxy` |
| `key` | String | ✅ | API Key |
| `url` | String | ⚠️ | API URL（仅 `proxy` 提供商需要） |
| `model` | String | ⚠️ | 模型名称（`openai`/`deepseek` 有默认值，`proxy` 必填） |
| `response_format` | String | ❌ | 响应格式路径（默认：`choices[0].message.content`） |
| `language` | String | ❌ | 输出语言（默认：`en`） |

**默认值**：
- `provider`: `"openai"`
- `model`:
  - `openai`: `"gpt-4.0"`
  - `deepseek`: `"deepseek-chat"`
  - `proxy`: 无默认值，必须配置
- `response_format`: `"choices[0].message.content"`
- `language`: `"en"`

### 设计模式

#### 1. 单例模式

通过 `OnceLock` 实现线程安全的全局单例：

```rust
pub fn global() -> &'static Self {
    static CLIENT: OnceLock<Self> = OnceLock::new();
    CLIENT.get_or_init(Self::new)
}
```

**优势**：
- 客户端只初始化一次，提高性能
- 线程安全，可以在多线程环境中安全使用
- 统一管理，所有模块使用同一个客户端实例

#### 2. 配置驱动模式

所有配置从 `Settings` 动态获取，无需硬编码：

```rust
let settings = Settings::get();
let url = self.build_url(&settings.llm.provider, &settings.llm.url)?;
let model = self.build_model(&settings.llm)?;
```

**优势**：
- 易于配置，无需修改代码
- 支持运行时切换提供商
- 配置集中管理

#### 3. 统一客户端模式

所有 LLM 提供商使用同一个客户端实现：

```rust
impl LLMClient {
    pub fn call(&self, params: &LLMRequestParams) -> Result<String> {
        // 统一的请求和响应处理逻辑
    }
}
```

**优势**：
- 代码复用，减少重复
- 易于维护，修改一处即可
- 统一的错误处理和日志记录

### 错误处理

#### 分层错误处理

1. **配置错误**：
   - Provider 不支持：返回错误
   - API Key 未配置：返回错误
   - URL 未配置（proxy 提供商）：返回错误

2. **网络错误**：
   - 连接失败：返回错误
   - 超时（60 秒）：返回错误

3. **API 错误**：
   - HTTP 状态码非 200：返回错误
   - 响应格式错误：返回错误

4. **解析错误**：
   - JSON 解析失败：返回错误
   - 响应内容提取失败：返回错误

#### 容错机制

- **配置验证**：在调用前验证配置是否完整
- **超时控制**：60 秒超时设置，避免长时间等待
- **详细错误信息**：提供详细的错误信息，包含 HTTP 状态码和响应体
- **统一错误处理**：所有提供商使用相同的错误处理逻辑

---

## 📁 Commands 层架构（命令封装）

LLM 命令层是 Workflow CLI 的命令接口，提供 LLM 配置的交互式设置和查看功能。该层采用命令模式设计，通过调用 `lib/base/settings/` 模块提供的 API 实现配置管理。

### 相关文件

#### CLI 入口层

```
src/bin/workflow.rs
```

#### 命令封装层

```
src/commands/llm/
├── mod.rs        # 模块声明和导出（~11 行）
├── setup.rs      # LLM 配置设置命令（~210 行）
└── show.rs       # LLM 配置查看命令（~80 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（输入、选择等）
- 格式化输出
- 调用核心业务逻辑层 (`lib/base/settings/`) 的功能

### 依赖模块

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/base/settings/`**：配置管理（`Settings`、`Paths`、`defaults`）
  - `Settings::load()` - 加载配置
  - `Paths::workflow_config()` - 获取配置文件路径
  - `default_llm_model()` - 获取默认模型
- **`lib/jira/config.rs`**：配置管理器（`ConfigManager`）
  - `ConfigManager::update()` - 更新配置
- **`lib/base/llm/`**：LLM 客户端（配置读取，不直接调用）
- **`lib/commands/config/helpers.rs`**：配置辅助函数
  - `select_language()` - 选择语言

---

## 🔄 集成关系

### Lib 层和 Commands 层的协作

LLM 模块采用清晰的分层架构，Lib 层和 Commands 层通过以下方式协作：

1. **配置管理**：Commands 层通过 `ConfigManager` 更新配置，Lib 层从 `Settings` 读取配置
2. **配置展示**：Commands 层读取配置并格式化输出，Lib 层不直接参与展示
3. **配置验证**：Commands 层在设置时验证配置，Lib 层在使用时验证配置

### 调用流程

#### 整体架构流程

```
用户输入（commit 标题、Git diff）
  ↓
PullRequestLLM::generate()
  ↓
LLMClient::global() (获取全局单例)
  ↓
LLMClient::call() (调用 LLM API)
  ├─ build_url() (从 Settings 获取 URL)
  ├─ build_headers() (从 Settings 获取 API Key)
  ├─ build_model() (从 Settings 获取 Model)
  ├─ build_payload() (构建请求体)
  └─ extract_content() (提取响应内容)
  ↓
reqwest HTTP Client (发送请求)
  ↓
LLM API (OpenAI/DeepSeek/Proxy)
  ↓
解析响应并返回
```

#### 命令分发流程

```
bin/workflow.rs::main()
  ↓
Cli::parse() (clap 解析)
  ↓
match Commands::Llm { subcommand }
  ↓
  ├─ LLMSubcommand::Setup → LLMSetupCommand::setup()
  └─ LLMSubcommand::Show → LLMShowCommand::show()
```

---

## 📋 Commands 层命令详情

### 1. Setup 命令 (`setup.rs`)

Setup 命令提供交互式的 LLM 配置设置功能：

1. **交互式配置收集**：
   - 提供友好的交互式界面
   - 显示当前配置值（如果存在）
   - 支持保留现有配置（按 Enter 跳过）

2. **Provider 选择**：
   - 支持三种 Provider：`openai`、`deepseek`、`proxy`
   - 根据选择的 Provider 动态调整配置项

3. **配置项验证**：
   - Proxy Provider 要求 Model 必须配置
   - 其他 Provider 的 Model 为可选

4. **配置保存**：
   - 使用 `ConfigManager` 原子性更新配置
   - 保存到 `workflow.toml` 文件

### 2. Show 命令 (`show.rs`)

Show 命令提供 LLM 配置的查看和展示功能：

1. **配置展示**：
   - 显示所有 LLM 配置项
   - 根据 Provider 类型显示不同的配置项
   - 掩码显示敏感信息（API Key）

2. **空配置检测**：
   - 检测配置是否为空（使用默认值）
   - 如果为空，提示用户运行 `workflow llm setup`

3. **格式化输出**：
   - 使用日志宏格式化输出
   - 区分已配置和未配置的项
   - 显示默认值信息

---

## 🔄 调用流程与数据流

### PR 总结流程

```
用户输入（PR 标题、PR diff、语言）
  ↓
PullRequestLLM::summarize_pr()
  ├─ 确定语言（命令行参数 > 配置文件 > 默认值 "en"）
  ├─ 构建 user prompt（包含 PR 标题和完整 diff）
  ├─ 根据语言生成 system prompt（generate_summarize_pr_system_prompt()）
  │   └─ 使用语言系统获取多语言指令
  └─ 构建请求参数（max_tokens: 2000, temperature: 0.3）
  ↓
LLMClient::global() (获取全局单例)
  ↓
LLMClient::call() (调用 LLM API)
  ├─ build_url() (从 Settings 获取 URL)
  ├─ build_headers() (从 Settings 获取 API Key)
  ├─ build_model() (从 Settings 获取 Model)
  ├─ build_payload() (构建请求体)
  └─ extract_content() (提取响应内容)
  ↓
reqwest HTTP Client (发送请求)
  ↓
LLM API (OpenAI/DeepSeek/Proxy)
  ↓
parse_summary_response() (解析响应)
  ├─ 提取 JSON（支持 markdown 代码块格式）
  ├─ 解析 JSON 获取 summary 和 filename
  ├─ 清理文件名（移除特殊字符，限制长度）
  └─ 返回 PullRequestSummary
  ↓
返回 PullRequestSummary（summary 和 filename）
```

**关键说明**：
- **语言优先级**：命令行参数 > 配置文件（`llm.language`）> 默认值（"en"）
- **System Prompt**：根据语言动态生成，包含详细的要求分析、功能说明、用户场景等指导
- **请求参数**：
  - `max_tokens: 2000` - 确保有足够空间返回完整的总结文档
  - `temperature: 0.3` - 降低温度，使输出更稳定和一致
- **响应格式**：LLM 返回 JSON 格式，包含 `summary`（Markdown 文档）和 `filename`（文件名）
- **文件名处理**：自动清理文件名，移除特殊字符，限制长度，确保文件名安全可用

---

## 📋 使用示例

### Setup 命令

```bash
# 交互式设置 LLM 配置
workflow llm setup

# 执行流程：
# 1. 选择 Provider（openai/deepseek/proxy）
# 2. 输入 URL（仅 proxy 需要）
# 3. 输入 API Key
# 4. 输入 Model（proxy 必须，其他可选）
# 5. 选择输出语言
# 6. 保存配置
```

### Show 命令

```bash
# 查看当前 LLM 配置
workflow llm show
```

### 基本使用（业务层）

```rust
use workflow::pr::PullRequestLLM;

// 生成分支名和 PR 标题
let content = PullRequestLLM::generate(
    "Fix login bug",
    Some(vec!["feature-1".to_string(), "feature-2".to_string()]),
    Some(git_diff),
)?;

log_message!("Branch: {}", content.branch_name);
log_message!("PR Title: {}", content.pr_title);
if let Some(desc) = content.description {
    log_message!("Description: {}", desc);
}
```

### 配置示例

#### 配置 OpenAI

```toml
# workflow.toml
[llm]
provider = "openai"
key = "sk-xxx"
model = "gpt-4.0"  # 可选，默认 "gpt-4.0"
language = "en"    # 可选，默认 "en"
```

#### 配置 DeepSeek

```toml
# workflow.toml
[llm]
provider = "deepseek"
key = "sk-xxx"
model = "deepseek-chat"  # 可选，默认 "deepseek-chat"
language = "en"
```

#### 配置 Proxy（代理 API）

```toml
# workflow.toml
[llm]
provider = "proxy"
url = "https://proxy.example.com"  # 必需
key = "your-api-key"                # 必需
model = "qwen-3-235b"               # 必需
response_format = "choices[0].message.content"  # 可选，默认值
language = "en"
```

---

## 📝 扩展性

### 添加新的 LLM 提供商

1. 在 `Settings` 中添加新的 provider 名称
2. 在 `LLMClient::build_url()` 中添加新 provider 的 URL 构建逻辑
3. 在 `LLMClient::build_model()` 中添加新 provider 的默认模型（如需要）
4. 在 `workflow.toml` 中配置新 provider 的 URL 和 API Key
5. 在 `setup.rs` 的 `llm_providers` 列表中添加新 Provider

### 添加新的业务功能

1. 在 `lib/pr/llm.rs` 或新建业务模块中添加新的业务方法
2. 使用 `LLMClient::global()` 调用 LLM API
3. 实现业务特定的 prompt 构建和响应解析逻辑

### 自定义响应格式

通过 `response_format` 配置支持自定义 JSON path：

```toml
[llm]
provider = "proxy"
url = "https://api.example.com"
key = "your-api-key"
model = "custom-model"
response_format = "candidates[0].content.parts[0].text"  # 自定义路径
```

---

## 📚 相关文档

- [主架构文档](./architecture.md)
- [PR 模块架构文档](./pr.md) - PR 模块如何使用 LLM 功能
- [Settings 模块架构文档](./settings.md) - Settings 配置系统架构

---

## ✅ 总结

LLM 模块采用统一配置驱动架构设计：

1. **统一客户端**：所有 LLM 提供商使用同一个 `LLMClient` 实现
2. **配置驱动**：所有参数从 `Settings` 动态获取，支持通过 `workflow.toml` 配置
3. **单例模式**：使用 `OnceLock` 实现线程安全的全局单例
4. **业务封装**：`PullRequestLLM` 提供业务友好的接口
5. **灵活扩展**：添加新提供商只需配置，无需写代码
6. **命令封装**：提供交互式配置设置和查看功能

**设计优势**：
- ✅ **代码复用**：消除代码重复，从多个独立客户端减少到一个统一客户端
- ✅ **易于扩展**：添加新提供商只需配置，无需写代码
- ✅ **统一管理**：所有配置集中在 `workflow.toml`
- ✅ **灵活配置**：支持自定义响应格式（JSON path）
- ✅ **向后兼容**：保持现有 API 不变
- ✅ **维护成本低**：只需维护一个统一客户端
- ✅ **用户友好**：交互式配置，支持默认值和验证

**当前实现状态**：

✅ **已实现**：
- 统一 `LLMClient` 实现
- 基于 `Settings` 的配置系统
- 支持 OpenAI、DeepSeek、Proxy 提供商
- 自定义响应格式支持（JSON path）
- PR 业务层封装（`PullRequestLLM`）
- 单例模式实现
- Setup 命令（交互式配置）
- Show 命令（配置展示）

**配置说明**：
- 当前实现使用 `workflow.toml` 的 `[llm]` 部分进行配置
- 所有 LLM 相关配置统一存储在 `workflow.toml` 中，与项目配置统一管理
- 通过 `workflow llm setup` 命令可以交互式配置 LLM 提供商

---

**最后更新**: 2025-12-16

