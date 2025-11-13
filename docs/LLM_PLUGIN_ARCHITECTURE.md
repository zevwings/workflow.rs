# LLM 统一配置驱动架构方案

## 📋 概述

本文档描述了 Workflow CLI 中 LLM（大语言模型）客户端的统一配置驱动架构方案。该方案通过**统一客户端**和**配置文件驱动**，实现所有 LLM 提供商的统一调用，消除代码重复，支持通过配置文件持久化提供商配置。

### 为什么选择配置驱动方案？

基于 CURL 示例分析（参见 [LLM_PLUGIN_CURL.md](./LLM_PLUGIN_CURL.md)），所有 LLM 提供商都遵循 **OpenAI 兼容格式**：

- ✅ **请求格式完全相同**：都使用 POST 到 `/v1/chat/completions`，请求体结构相同
- ✅ **响应格式完全相同**：都从 `choices[0].message.content` 提取内容
- ✅ **唯一差异**：URL 和 API Key（配置差异，非代码差异）

**结论**：**不需要传统插件系统**（trait、registry、manager），只需要**配置驱动 + 统一客户端**方案。

### 设计目标

1. **统一客户端**：所有 LLM 提供商使用同一个客户端实现，消除代码重复
2. **配置驱动**：所有参数（URL、API Key、响应格式）从配置文件读取
3. **易于扩展**：添加新的 LLM 提供商只需配置，无需写代码
4. **持久化配置**：通过配置文件支持提供商配置的持久化存储
5. **向后兼容**：保持现有 API 不变，支持从环境变量创建默认配置

### 核心问题

当前实现中，三个客户端（OpenAI、DeepSeek、Proxy）的代码 **95% 相同**：
- **调用格式**：都使用 OpenAI 兼容格式
- **响应格式**：都假设返回 OpenAI 兼容格式
- **代码重复**：每个客户端都有独立的实现文件，但逻辑几乎完全相同
- **扩展性**：添加新的提供商需要创建新文件并实现相同逻辑

### 解决方案：统一配置驱动

**核心思想**：直接从配置文件读取所有参数（URL、API Key、响应格式），使用**统一客户端**处理所有调用，无需为每个提供商单独实现。

**核心优势**：
- ✅ **代码量少**：只需 ~250 行（统一客户端 + 配置加载）
- ✅ **易于扩展**：添加新提供商只需配置，无需写代码
- ✅ **维护成本低**：只需维护一个统一客户端
- ✅ **配置简单**：只需填写 URL 和 API Key
- ✅ **灵活性强**：通过配置支持自定义格式
- ✅ **向后兼容**：支持从环境变量创建默认配置

**核心组件**：
1. **LLMClient**：统一客户端（处理所有 LLM 调用）
2. **LLMConfig**：配置加载器（从 TOML 或环境变量读取配置）
3. **配置文件**：存储不同提供商的配置（URL、API Key 等）

**实现要点**：
- ✅ **统一请求格式**：所有提供商使用相同的请求体结构
- ✅ **统一响应解析**：所有提供商从 `choices[0].message.content` 提取内容
- ✅ **配置驱动**：所有差异通过配置文件解决
- ✅ **向后兼容**：支持从环境变量创建默认配置

---

## 🏗️ 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                  PullRequestLLM                         │
│  (统一入口)                                               │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│              LLMConfig                                   │
│  (配置文件加载和解析)                                      │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│              LLMClient                            │
│  (统一客户端，处理所有 LLM 调用)                           │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
            ┌────────────────┐
            │  HTTP Client   │
            │  (发送请求)     │
            └────────────────┘
```

### 核心组件

#### 1. LLMClient（统一客户端）

所有 LLM 提供商使用同一个客户端实现，通过配置区分不同的提供商：

```rust
pub struct LLMClient {
    name: String,
    config: LLMClientConfig,
}

#[derive(Debug, Clone)]
pub struct LLMClientConfig {
    pub url: String,
    pub api_key: String,
    pub response_format: ResponseFormat,
    pub timeout: Option<u64>,
    pub retry_count: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum ResponseFormat {
    /// OpenAI 标准格式：choices[0].message.content
    OpenAI,
    /// 自定义格式：通过 JSON path 提取
    Custom {
        content_path: String,
        error_path: Option<String>,
    },
}

impl LLMClient {
    /// 从配置创建客户端
    pub fn from_config(name: String, config: LLMClientConfig) -> Self {
        Self { name, config }
    }

    /// 调用 LLM API
    pub fn call(&self, params: LLMRequestParams) -> Result<String> {
        // 构建请求体、请求头，发送请求，解析响应
        // 所有逻辑都在这里统一处理
    }
}
```

#### 2. LLMConfig（配置文件管理）

负责配置文件的加载、解析和验证：

- **配置文件查找**：按优先级查找配置文件
- **环境变量解析**：支持 `${VAR}` 和 `${VAR:default}` 格式
- **配置验证**：验证必填字段和格式
- **默认配置**：如果配置文件不存在，从环境变量创建默认配置

#### 3. PullRequestLLM（业务层）

支持从配置文件加载提供商配置：

- **自动加载**：程序启动时自动查找并加载配置文件
- **优先级支持**：支持命令行参数、配置文件、环境变量的优先级覆盖
- **向后兼容**：保持现有环境变量配置方式，配置文件作为增强功能

---

## 🔌 配置方式

### 配置文件（持久化配置）⭐

**适用场景**：需要持久化配置、支持多个 LLM 提供商

#### 配置文件位置（按优先级）

```
1. ~/.workflow/llm.toml          # 用户主目录（推荐）
2. $XDG_CONFIG_HOME/workflow/llm.toml  # XDG 标准
3. ./llm.toml           # 当前目录（项目级配置）
4. $WORKFLOW_LLM_CONFIG          # 环境变量指定路径
```

#### 配置文件格式

```toml
# Workflow CLI LLM Configuration
version = "1.0"

# 默认使用的提供商名称（可选，覆盖 LLM_PROVIDER 环境变量）
default_provider = "my-proxy-1"

# 提供商列表
[[providers]]
name = "openai-default"
enabled = true

[providers.config]
url = "https://api.openai.com/v1/chat/completions"
api_key = "${LLM_OPENAI_KEY}"  # 支持环境变量引用
response_format = "openai"

[providers.advanced]
timeout = 30
retry_count = 3

# DeepSeek 提供商
[[providers]]
name = "deepseek-default"
enabled = true

[providers.config]
url = "https://api.deepseek.com/v1/chat/completions"
api_key = "${LLM_DEEPSEEK_KEY}"
response_format = "openai"

# Proxy 提供商 - OpenAI 兼容格式
[[providers]]
name = "my-proxy-1"
enabled = true

[providers.config]
url = "https://proxy1.example.com/api/chat/completions"
api_key = "${LLM_PROXY_KEY_1}"
response_format = "openai"

# Proxy 提供商 - 自定义格式
[[providers]]
name = "my-proxy-2"
enabled = true

[providers.config]
url = "https://proxy2.example.com/api/v1/chat"
api_key = "${LLM_PROXY_KEY_2}"
response_format = "custom"

[providers.config.custom_format]
content_path = "data.result.text"  # JSON path
error_path = "error.message"
```

#### 加载流程

```
1. 检查环境变量 WORKFLOW_LLM_CONFIG
   ↓ (如果存在，直接使用)
   加载指定路径的配置文件

2. 检查当前目录的 llm.toml
   ↓ (如果存在，使用项目级配置)
   加载项目级配置

3. 检查 XDG_CONFIG_HOME/workflow/llm.toml
   ↓ (如果存在)
   加载用户配置

4. 检查 ~/.workflow/llm.toml
   ↓ (如果存在)
   加载主目录配置

5. 如果都不存在，使用默认配置（从环境变量创建）
```

#### 配置文件持久化机制

配置文件采用**文件系统持久化**，与现有的环境变量配置系统（`~/.zshrc`、`~/.bashrc`）并行工作：

1. **持久化存储**：
   - 配置文件保存在文件系统中，不会因为终端会话结束而丢失
   - 支持版本控制（可以提交到 Git 仓库）
   - 支持多环境配置（开发、测试、生产）

2. **自动加载**：
   - 程序启动时自动查找并加载配置文件
   - 按优先级顺序查找，找到第一个有效配置即停止
   - 加载失败时静默回退到环境变量配置

3. **配置加载**：
   - 配置文件中的提供商配置会被加载
   - 与环境变量配置共存（配置文件优先级更高）
   - 支持多个配置文件（按优先级顺序加载）

#### 配置文件管理

##### 创建配置文件

```bash
# 方式 1：手动创建（推荐）
mkdir -p ~/.workflow
cat > ~/.workflow/llm.toml << 'EOF'
version = "1.0"
default_provider = "my-proxy"

[[providers]]
name = "my-proxy"
enabled = true

[providers.config]
url = "https://my-proxy.com/api/chat/completions"
api_key = "${LLM_PROXY_KEY}"
response_format = "openai"
EOF

# 方式 2：通过 CLI 命令创建（未来支持）
# workflow llm init
```

##### 验证配置文件

```bash
# 验证配置文件格式和内容
workflow llm validate

# 或指定配置文件路径
workflow llm validate --config ~/.workflow/llm.toml
```

##### 查看已加载的提供商

```bash
# 列出所有已配置的提供商
workflow llm list

# 查看特定提供商的详细信息
workflow llm info my-proxy
```

##### 测试提供商配置

```bash
# 测试提供商是否能正常工作
workflow llm test my-proxy

# 或使用测试命令
workflow pr create --dry-run --llm-provider my-proxy
```

#### 与现有配置系统的集成

配置文件系统与现有的环境变量配置系统（参考 [CONFIG_ARCHITECTURE.md](./CONFIG_ARCHITECTURE.md)）并行工作：

**配置优先级**（从高到低）：
```
1. 命令行参数（--llm-provider）
2. 配置文件（llm.toml）
3. 环境变量（LLM_PROVIDER, LLM_PROXY_URL 等）
4. 内置默认值（openai）
```

**配置来源对比**：

| 特性 | 配置文件 | 环境变量 |
|------|---------|---------|
| 持久化 | ✅ 文件系统 | ✅ Shell 配置文件 |
| 版本控制 | ✅ 可提交到 Git | ❌ 通常不提交 |
| 多提供商支持 | ✅ 支持多个提供商 | ❌ 单一配置 |
| 复杂配置 | ✅ 支持嵌套配置 | ❌ 扁平结构 |
| 环境变量引用 | ✅ 支持 `${VAR}` | ✅ 直接使用 |
| 项目级配置 | ✅ 支持 | ❌ 全局配置 |

**迁移建议**：
- **简单配置**：继续使用环境变量（适合单一提供商、简单场景）
- **复杂配置**：迁移到配置文件（适合多提供商、自定义格式、项目级配置）

#### 使用示例

##### 示例 1：自动加载配置文件

```rust
// 在程序启动时自动加载（PullRequestLLM::generate 内部实现）
let content = PullRequestLLM::generate(title, None, None)?;
// 会自动查找并加载配置文件，使用 default_provider 指定的提供商
```

##### 示例 2：手动加载配置文件

```rust
// 手动指定配置文件路径
let config_path = Path::new("~/.workflow/llm.toml");
let config = LLMConfig::load(&config_path)?;

// 获取指定提供商
let client = config.get_client("my-proxy")?;
let response = client.call(params)?;
```

##### 示例 3：项目级配置

```bash
# 在项目根目录创建配置文件
cat > ./llm.toml << 'EOF'
version = "1.0"
default_provider = "project-proxy"

[[providers]]
name = "project-proxy"
enabled = true

[providers.config]
url = "https://project-proxy.example.com/api/chat/completions"
api_key = "${PROJECT_LLM_KEY}"
response_format = "openai"
EOF

# 提交到 Git（可选）
git add llm.toml
git commit -m "Add LLM configuration"
```

#### 配置文件最佳实践

1. **敏感信息管理**：
   - ✅ 使用环境变量引用：`api_key = "${LLM_PROXY_KEY}"`
   - ❌ 不要直接写入密钥：`api_key = "sk-xxx"`（会被提交到 Git）

2. **版本控制**：
   - ✅ 项目级配置可以提交到 Git（使用环境变量引用）
   - ❌ 用户级配置（`~/.workflow/llm.toml`）不要提交

3. **多环境配置**：
   - 开发环境：使用项目级配置
   - 生产环境：使用用户级配置或环境变量

4. **配置验证**：
   - 定期运行 `workflow llm validate` 验证配置
   - 在 CI/CD 中验证项目级配置文件

5. **配置文档**：
   - 在配置文件中添加注释说明每个提供商的用途
   - 记录环境变量的含义和获取方式

**优点**：
- ✅ **持久化配置**：配置文件保存在文件系统中，不会丢失
- ✅ **支持版本控制**：项目级配置可以提交到 Git
- ✅ **多环境支持**：支持项目级和用户级配置
- ✅ **灵活配置**：支持复杂嵌套配置和自定义格式
- ✅ **环境变量集成**：支持引用环境变量，保持安全性

**缺点**：
- ⚠️ **需要配置文件管理**：需要创建和维护配置文件
- ⚠️ **配置验证复杂**：需要验证配置格式和内容
- ⚠️ **学习成本**：需要了解 TOML 格式和配置结构

#### 配置优先级

配置文件系统与现有的环境变量配置系统并行工作，配置优先级（从高到低）：

1. **命令行参数**：`--llm-provider <name>`（临时覆盖）
2. **配置文件**：`llm.toml` 中的 `default_provider`
3. **环境变量**：`LLM_PROVIDER`（保持向后兼容）
4. **内置默认值**：`"openai"`

---

## 📁 文件结构

```
src/lib/llm/
├── mod.rs                    # 模块导出
├── pr_llm.rs                 # PullRequestLLM（业务层）
└── client/
    ├── mod.rs                 # 客户端模块导出
    ├── client.rs              # LLMClient（统一客户端）
    ├── config.rs              # 配置文件加载和解析
    └── common.rs              # 共享类型和工具（LLMRequestParams 等）
```

---

## 🔧 实现细节

### 1. 统一客户端实现

统一客户端处理所有 LLM 调用，根据配置选择不同的响应解析方式：

```rust
impl LLMClient {
    /// 调用 LLM API
    pub fn call(&self, params: LLMRequestParams) -> Result<String> {
        let client = HttpClient::new()?;

        // 构建请求体（统一格式）
        let payload = self.build_payload(&params);

        // 构建请求头（统一格式）
        let headers = self.build_headers()?;

        // 发送请求
        let response: HttpResponse<serde_json::Value> = client
            .post(&self.config.url, &payload, None, Some(&headers))
            .context("Failed to send LLM request")?;

        // 检查错误
        if !response.is_success() {
            return self.handle_error(&response);
        }

        // 根据配置的响应格式提取内容
        self.extract_content(&response.data)
    }

    fn extract_content(&self, response: &serde_json::Value) -> Result<String> {
        match &self.config.response_format {
            ResponseFormat::OpenAI => {
                // 标准 OpenAI 格式
                response
                    .get("choices")
                    .and_then(|c| c.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|choice| choice.get("message"))
                    .and_then(|msg| msg.get("content"))
                    .and_then(|c| c.as_str())
                    .context("Failed to extract content from OpenAI format response")
                    .map(|s| s.trim().to_string())
            }
            ResponseFormat::Custom { content_path, .. } => {
                // 通过 JSON path 提取
                self.extract_by_path(response, content_path)
            }
        }
    }

    fn extract_by_path(&self, json: &serde_json::Value, path: &str) -> Result<String> {
        // 例如: "data.result.text" -> json["data"]["result"]["text"]
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = json;

        for part in parts {
            current = current
                .get(part)
                .with_context(|| format!("Path '{}' not found in response", path))?;
        }

        current
            .as_str()
            .with_context(|| format!("Value at path '{}' is not a string", path))
            .map(|s| s.trim().to_string())
    }
}
```

### 2. 环境变量解析

配置文件支持环境变量引用：

```toml
# 基本引用
api_key = "${LLM_PROXY_KEY}"

# 带默认值
api_key = "${LLM_PROXY_KEY:default-key}"

# 嵌套引用
url = "${LLM_PROXY_BASE_URL}/api"
```

解析规则：
- `${VAR_NAME}` → 从环境变量读取，如果不存在则报错
- `${VAR_NAME:default}` → 从环境变量读取，如果不存在则使用默认值
- 支持在字符串中混合使用

### 3. 配置文件加载和解析

#### 配置文件查找逻辑

```rust
pub fn find_config_path() -> Option<PathBuf> {
    // 1. 检查环境变量 WORKFLOW_LLM_CONFIG
    if let Ok(path) = env::var("WORKFLOW_LLM_CONFIG") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }

    // 2. 检查当前目录的 llm.toml
    let current_dir = env::current_dir().ok()?;
    let project_config = current_dir.join("llm.toml");
    if project_config.exists() {
        return Some(project_config);
    }

    // 3. 检查 XDG_CONFIG_HOME/workflow/llm.toml
    if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
        let xdg_path = PathBuf::from(xdg_config)
            .join("workflow")
            .join("llm.toml");
        if xdg_path.exists() {
            return Some(xdg_path);
        }
    }

    // 4. 检查 ~/.workflow/llm.toml
    if let Some(home) = dirs::home_dir() {
        let home_config = home.join(".workflow").join("llm.toml");
        if home_config.exists() {
            return Some(home_config);
        }
    }

    None
}
```

#### 配置文件解析

```rust
use serde::{Deserialize, Serialize};
use toml::Value;

#[derive(Debug, Deserialize)]
pub struct LLMConfig {
    pub version: String,
    pub default_provider: Option<String>,
    pub providers: Vec<ProviderEntry>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderEntry {
    pub name: String,
    pub enabled: bool,
    pub config: Value,  // 使用 Value 支持灵活配置
    #[serde(default)]
    pub advanced: Option<Value>,
}

impl LLMConfig {
    /// 从文件加载配置
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        // 解析环境变量引用
        let content = Self::resolve_env_vars(&content)?;

        // 解析 TOML
        let config: LLMConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        // 验证配置
        config.validate()?;

        Ok(config)
    }

    /// 解析环境变量引用
    fn resolve_env_vars(content: &str) -> Result<String> {
        use regex::Regex;

        // 匹配 ${VAR_NAME} 或 ${VAR_NAME:default}
        let re = Regex::new(r"\$\{([^}:]+)(?::([^}]*))?\}")?;

        let result = re.replace_all(content, |caps: &regex::Captures| {
            let var_name = caps.get(1).unwrap().as_str();
            let default = caps.get(2).map(|m| m.as_str());

            match env::var(var_name) {
                Ok(value) => value,
                Err(_) => {
                    if let Some(default_val) = default {
                        default_val.to_string()
                    } else {
                        // 如果环境变量不存在且没有默认值，保持原样（后续验证会报错）
                        caps.get(0).unwrap().as_str().to_string()
                    }
                }
            }
        });

        Ok(result.to_string())
    }

    /// 验证配置
    fn validate(&self) -> Result<()> {
        // 验证版本
        if self.version != "1.0" {
            return Err(anyhow::anyhow!("Unsupported config version: {}", self.version));
        }

        // 验证提供商配置
        for provider in &self.providers {
            if !provider.enabled {
                continue;
            }

            // 验证必填字段（所有提供商都需要 url 和 api_key）
            Self::require_field(&provider.config, "url")?;
            Self::require_field(&provider.config, "api_key")?;
        }

        // 验证 default_provider 是否存在
        if let Some(ref default) = self.default_provider {
            if !self.providers.iter().any(|p| p.name == *default && p.enabled) {
                return Err(anyhow::anyhow!(
                    "Default provider '{}' not found or disabled",
                    default
                ));
            }
        }

        Ok(())
    }

    fn require_field(config: &Value, field: &str) -> Result<()> {
        if !config.get(field).is_some() {
            return Err(anyhow::anyhow!("Missing required field: {}", field));
        }
        Ok(())
    }

    /// 获取指定名称的客户端
    pub fn get_client(&self, name: &str) -> Result<LLMClient> {
        let provider = self.providers
            .iter()
            .find(|p| p.name == name && p.enabled)
            .with_context(|| format!("Provider '{}' not found or disabled", name))?;

        let config = LLMClientConfig::from_toml_value(&provider.config)?;
        Ok(LLMClient::from_config(provider.name.clone(), config))
    }

    /// 获取默认客户端
    pub fn get_default_client(&self) -> Result<LLMClient> {
        if let Some(ref default_name) = self.default_provider {
            return self.get_client(default_name);
        }

        // 如果没有指定默认提供商，返回第一个启用的提供商
        let provider = self.providers
            .iter()
            .find(|p| p.enabled)
            .context("No enabled providers found")?;

        let config = LLMClientConfig::from_toml_value(&provider.config)?;
        Ok(LLMClient::from_config(provider.name.clone(), config))
    }
}
```

#### 配置文件自动加载

在程序启动时自动加载配置文件：

```rust
// 在 PullRequestLLM 初始化时自动加载
impl PullRequestLLM {
    pub fn new() -> Self {
        // 自动加载配置文件
        Self::load_config_file();
        Self
    }

    fn load_config_file() -> Option<LLMConfig> {
        if let Some(path) = find_config_path() {
            match LLMConfig::load(&path) {
                Ok(config) => {
                    return Some(config);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load config file: {}", e);
                }
            }
        }
        None
    }
}
```

### 4. 错误处理

- **配置文件不存在**：静默忽略，使用默认配置
- **格式错误**：显示详细错误信息，指出行号和问题
- **版本不兼容**：警告并尝试兼容，或提示升级
- **提供商配置错误**：明确指出缺少的字段或无效的值
- **提供商加载失败**：记录错误但继续使用其他提供商
- **提供商调用失败**：回退到默认提供商或报错

---

## ✅ 优势分析

### 1. 统一客户端
- 所有 LLM 提供商使用同一个客户端实现
- **消除代码重复**：从 ~300 行减少到 ~250 行
- 统一的错误处理和请求逻辑

### 2. 配置驱动
- 所有参数从配置文件读取（URL、API Key、响应格式）
- **易于扩展**：添加新提供商只需配置，无需写代码
- 支持自定义响应格式（通过 JSON path）

### 3. 持久化配置
- 配置文件支持复杂配置
- 支持项目级和用户级配置
- 环境变量支持临时覆盖

### 4. 向后兼容
- 保持现有 API 不变
- 如果配置文件不存在，从环境变量创建默认配置
- 平滑迁移路径

### 5. 类型安全
- 编译时检查
- 避免运行时错误
- 更好的 IDE 支持

### 6. 可测试性
- 可以轻松注入 mock 客户端
- 统一客户端可独立测试
- 支持单元测试和集成测试

### 7. 维护成本低
- **单一代码路径**：所有客户端都走统一客户端
- **无需维护多套实现**：添加多个客户端时，只需配置
- **代码一致性高**：所有客户端使用相同的逻辑

---

## 🔄 迁移指南

### 从环境变量迁移到配置文件

**迁移前**（环境变量）：
```bash
export LLM_PROVIDER=proxy
export LLM_PROXY_URL=https://proxy.example.com/api
export LLM_PROXY_KEY=my-key
```

**迁移后**（配置文件）：
```toml
# ~/.workflow/llm.toml
default_provider = "proxy-default"

[[providers]]
name = "proxy-default"
enabled = true

[providers.config]
url = "https://proxy.example.com/api/chat/completions"
api_key = "${LLM_PROXY_KEY}"
response_format = "openai"
```

---

## 📝 使用示例

### 示例 1：使用配置文件（推荐）

```toml
# ~/.workflow/llm.toml
default_provider = "my-proxy"

[[providers]]
name = "my-proxy"
enabled = true

[providers.config]
url = "https://my-proxy.com/api/chat/completions"
api_key = "${LLM_PROXY_KEY}"
response_format = "custom"

[providers.config.custom_format]
content_path = "data.text"
```

```rust
// 自动加载配置文件
let content = PullRequestLLM::generate(title, None, None)?;
```

### 示例 2：项目级配置

在项目根目录创建 `llm.toml`，可以提交到 Git（使用环境变量引用敏感信息）。

---

## 📚 相关文档

- [LLM_IMPLEMENTATION.md](./LLM_IMPLEMENTATION.md) - **实现指南**（详细的代码实现步骤）
- [LLM_PLUGIN_CURL.md](./LLM_PLUGIN_CURL.md) - API 调用示例
- [配置管理架构](./CONFIG_ARCHITECTURE.md)
- [PR 创建流程](./PR_ARCHITECTURE.md)
- [整体架构](./ARCHITECTURE.md)

---

## 🔍 总结

该统一配置驱动方案提供了：

1. **统一客户端**：所有 LLM 提供商使用同一个客户端实现，消除代码重复
2. **配置驱动**：所有参数从配置文件读取，添加新提供商只需配置
3. **持久化配置**：通过配置文件支持提供商配置的持久化存储
4. **易于扩展**：添加新提供商只需配置，无需写代码
5. **向后兼容**：保持现有 API 不变，支持从环境变量创建默认配置
6. **类型安全**：利用 Rust 类型系统

通过该方案，可以：
- **统一不同 LLM 提供商的调用和返回格式**
- **支持通过配置文件持久化提供商配置**
- **提供灵活的配置管理（项目级和用户级）**
- **保持代码的可维护性和可扩展性**
- **降低维护成本**：只需维护一个统一客户端，添加多个客户端时只需配置

### 核心原则

- **配置驱动**：所有差异通过配置解决
- **统一实现**：所有提供商使用同一个客户端
- **简单高效**：无需复杂的插件系统

### 实现步骤

1. **实现 LLMClient**：统一客户端，处理所有 LLM 调用
2. **实现 LLMConfig**：配置加载器，从 TOML 或环境变量读取
3. **更新 PullRequestLLM**：使用统一客户端替代多个独立客户端
4. **保持向后兼容**：支持从环境变量创建默认配置


