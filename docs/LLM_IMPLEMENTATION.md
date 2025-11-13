# LLM 统一配置驱动实现指南

## 📋 概述

本文档提供了 LLM 统一配置驱动方案的详细实现指南，包含完整的代码结构、API 定义、实现步骤和测试策略。

**参考文档**：
- [LLM_PLUGIN_ARCHITECTURE.md](./LLM_PLUGIN_ARCHITECTURE.md) - 架构设计文档
- [LLM_PLUGIN_CURL.md](./LLM_PLUGIN_CURL.md) - API 调用示例

---

## 📦 依赖项

### 需要添加的依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
# ... 现有依赖 ...
toml = "0.8"  # TOML 配置文件解析
dirs = "5.0"  # 用于查找配置文件路径（可选，如果已有则不需要）
```

**注意**：检查项目中是否已有 `dirs` 依赖，如果没有则添加。

---

## 📁 文件结构

```
src/lib/llm/
├── mod.rs                    # 模块导出
├── pr_llm.rs                 # PullRequestLLM（业务层，需要更新）
└── client/
    ├── mod.rs                 # 客户端模块导出
    ├── client.rs              # LLMClient（统一客户端）⭐ 新建
    ├── config.rs              # 配置文件加载和解析 ⭐ 新建
    └── common.rs              # 共享类型和工具 ⭐ 新建
```

**迁移计划**：
- 保留 `openai.rs`、`deepseek.rs`、`proxy.rs` 作为向后兼容（标记为 deprecated）
- 新建 `client.rs`、`config.rs`、`common.rs`
- 更新 `pr_llm.rs` 使用统一客户端

**命名说明**：
- `client.rs`：统一客户端实现（参考项目中的 `src/lib/http/client.rs` 命名习惯）
- `config.rs`：配置文件加载和解析
- `common.rs`：共享类型和工具

---

## 🔧 实现步骤

### 步骤 1：创建共享类型（common.rs）

**文件**：`src/lib/llm/client/common.rs`

```rust
//! LLM 客户端共享类型和工具

use serde::{Deserialize, Serialize};

/// LLM 请求参数
///
/// 包含调用 LLM API 所需的所有参数。
#[derive(Debug, Clone, Serialize)]
pub struct LLMRequestParams {
    /// 系统提示词
    pub system_prompt: String,
    /// 用户提示词
    pub user_prompt: String,
    /// 最大 token 数
    pub max_tokens: u32,
    /// 温度参数（控制输出的随机性）
    pub temperature: f32,
    /// 模型名称（如 "gpt-3.5-turbo"）
    pub model: String,
}

impl Default for LLMRequestParams {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            user_prompt: String::new(),
            max_tokens: 100,
            temperature: 0.5,
            model: "gpt-3.5-turbo".to_string(),
        }
    }
}
```

---

### 步骤 2：实现统一客户端（client.rs）

**文件**：`src/lib/llm/client/client.rs`

```rust
//! LLM 客户端
//!
//! 本模块提供了 LLM 客户端实现，支持所有遵循 OpenAI 兼容格式的提供商。

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

use crate::http::{HttpClient, HttpResponse};
use super::common::LLMRequestParams;

/// 响应格式
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

/// LLM 客户端配置
#[derive(Debug, Clone)]
pub struct LLMClientConfig {
    pub url: String,
    pub api_key: String,
    pub response_format: ResponseFormat,
    pub timeout: Option<u64>,
    pub retry_count: Option<u32>,
}

/// LLM 客户端
///
/// 所有 LLM 提供商使用同一个客户端实现，通过配置区分不同的提供商。
pub struct LLMClient {
    name: String,
    config: LLMClientConfig,
}

impl LLMClient {
    /// 从配置创建客户端
    pub fn from_config(name: String, config: LLMClientConfig) -> Self {
        Self { name, config }
    }

    /// 调用 LLM API
    ///
    /// # 参数
    ///
    /// * `params` - LLM 请求参数
    ///
    /// # 返回
    ///
    /// 返回 LLM 生成的文本内容（去除首尾空白）。
    ///
    /// # 错误
    ///
    /// 如果 API 调用失败或响应格式不正确，返回相应的错误信息。
    pub fn call(&self, params: &LLMRequestParams) -> Result<String> {
        let client = HttpClient::new()?;

        // 构建请求体（统一格式）
        let payload = self.build_payload(params);

        // 构建请求头（统一格式）
        let headers = self.build_headers()?;

        // 发送请求
        let response: HttpResponse<serde_json::Value> = client
            .post(&self.config.url, &payload, None, Some(&headers))
            .with_context(|| format!("Failed to send LLM request to {}", self.name))?;

        // 检查错误
        if !response.is_success() {
            return self.handle_error(&response);
        }

        // 根据配置的响应格式提取内容
        self.extract_content(&response.data)
    }

    /// 构建请求体
    fn build_payload(&self, params: &LLMRequestParams) -> serde_json::Value {
        json!({
            "model": params.model,
            "messages": [
                {
                    "role": "system",
                    "content": params.system_prompt
                },
                {
                    "role": "user",
                    "content": params.user_prompt
                }
            ],
            "max_tokens": params.max_tokens,
            "temperature": params.temperature
        })
    }

    /// 构建请求头
    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.config.api_key))
                .context("Failed to create Authorization header")?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    /// 从响应中提取内容
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

    /// 通过 JSON path 提取内容
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

    /// 处理错误响应
    fn handle_error(&self, response: &HttpResponse<serde_json::Value>) -> Result<String> {
        let error_text = serde_json::to_string(&response.data).unwrap_or_default();
        anyhow::bail!(
            "LLM API request failed ({}): {} - {}",
            self.name,
            response.status,
            error_text
        );
    }
}
```

---

### 步骤 3：实现配置加载（config.rs）

**文件**：`src/lib/llm/client/config.rs`

```rust
//! LLM 配置文件加载和解析

use anyhow::{Context, Result};
use std::env;
use std::path::{Path, PathBuf};
use toml::Value;

use super::client::{LLMClientConfig, ResponseFormat, LLMClient};

/// 插件配置条目
#[derive(Debug)]
pub struct PluginEntry {
    pub name: String,
    pub enabled: bool,
    pub config: Value,  // 使用 Value 支持灵活配置
    pub advanced: Option<Value>,
}

/// LLM 插件配置
///
/// 从 TOML 配置文件加载的 LLM 提供商配置。
#[derive(Debug)]
pub struct LLMConfig {
    pub version: String,
    pub default_plugin: Option<String>,
    pub plugins: Vec<PluginEntry>,
}

impl LLMConfig {
    /// 查找配置文件路径（按优先级）
    pub fn find_config_path() -> Option<PathBuf> {
        // 1. 检查环境变量 WORKFLOW_LLM_PLUGINS_CONFIG
        if let Ok(path) = env::var("WORKFLOW_LLM_PLUGINS_CONFIG") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }

        // 2. 检查当前目录的 llm.toml
        if let Ok(current_dir) = env::current_dir() {
            let project_config = current_dir.join("llm.toml");
            if project_config.exists() {
                return Some(project_config);
            }
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

    /// 从文件加载配置
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        // 解析环境变量引用
        let content = Self::resolve_env_vars(&content)?;

        // 解析 TOML
        let value: Value = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        // 转换为 LLMConfig
        let config = Self::from_toml_value(value)?;

        // 验证配置
        config.validate()?;

        Ok(config)
    }

    /// 从 TOML Value 创建 LLMConfig
    fn from_toml_value(value: Value) -> Result<Self> {
        let version = value
            .get("version")
            .and_then(|v| v.as_str())
            .context("Missing 'version' field")?
            .to_string();

        let default_plugin = value
            .get("default_plugin")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let plugins_array = value
            .get("plugins")
            .and_then(|v| v.as_array())
            .context("Missing 'plugins' array")?;

        let mut plugins = Vec::new();
        for plugin_value in plugins_array {
            let name = plugin_value
                .get("name")
                .and_then(|v| v.as_str())
                .context("Missing 'name' field in plugin")?
                .to_string();

            let enabled = plugin_value
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            let config = plugin_value
                .get("config")
                .context("Missing 'config' field in plugin")?
                .clone();

            let advanced = plugin_value.get("advanced").cloned();

            plugins.push(PluginEntry {
                name,
                enabled,
                config,
                advanced,
            });
        }

        Ok(LLMConfig {
            version,
            default_plugin,
            plugins,
        })
    }

    /// 解析环境变量引用
    ///
    /// 支持 `${VAR_NAME}` 和 `${VAR_NAME:default}` 格式。
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

        // 验证插件配置
        for plugin in &self.plugins {
            if !plugin.enabled {
                continue;
            }

            // 验证必填字段（所有插件都需要 url 和 api_key）
            Self::require_field(&plugin.config, "url")?;
            Self::require_field(&plugin.config, "api_key")?;
        }

        // 验证 default_plugin 是否存在
        if let Some(ref default) = self.default_plugin {
            if !self.plugins.iter().any(|p| p.name == *default && p.enabled) {
                return Err(anyhow::anyhow!(
                    "Default plugin '{}' not found or disabled",
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
        let plugin = self.plugins
            .iter()
            .find(|p| p.name == name && p.enabled)
            .with_context(|| format!("Plugin '{}' not found or disabled", name))?;

        let config = Self::parse_client_config(&plugin.config)?;
        Ok(LLMClient::from_config(plugin.name.clone(), config))
    }

    /// 获取默认客户端
    pub fn get_default_client(&self) -> Result<LLMClient> {
        if let Some(ref default_name) = self.default_plugin {
            return self.get_client(default_name);
        }

        // 如果没有指定默认插件，返回第一个启用的插件
        let plugin = self.plugins
            .iter()
            .find(|p| p.enabled)
            .context("No enabled plugins found")?;

        let config = Self::parse_client_config(&plugin.config)?;
        Ok(LLMClient::from_config(plugin.name.clone(), config))
    }

    /// 从环境变量创建默认配置（向后兼容）
    pub fn from_env() -> Result<Option<LLMClient>> {
        use crate::settings::Settings;

        let settings = Settings::load();
        let provider = settings.llm_provider.as_str();

        let config = match provider {
            "openai" => {
                let api_key = settings.openai_key
                    .context("LLM_OPENAI_KEY not set")?;
                LLMClientConfig {
                    url: "https://api.openai.com/v1/chat/completions".to_string(),
                    api_key,
                    response_format: ResponseFormat::OpenAI,
                    timeout: None,
                    retry_count: None,
                }
            }
            "deepseek" => {
                let api_key = settings.deepseek_key
                    .context("LLM_DEEPSEEK_KEY not set")?;
                LLMClientConfig {
                    url: "https://api.deepseek.com/v1/chat/completions".to_string(),
                    api_key,
                    response_format: ResponseFormat::OpenAI,
                    timeout: None,
                    retry_count: None,
                }
            }
            "proxy" => {
                let api_key = settings.llm_proxy_key
                    .context("LLM_PROXY_KEY not set")?;
                let base_url = settings.llm_proxy_url
                    .context("LLM_PROXY_URL not set")?;
                let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
                LLMClientConfig {
                    url,
                    api_key,
                    response_format: ResponseFormat::OpenAI,
                    timeout: None,
                    retry_count: None,
                }
            }
            _ => return Ok(None),
        };

        Ok(Some(LLMClient::from_config(
            provider.to_string(),
            config,
        )))
    }

    /// 解析客户端配置
    fn parse_client_config(config: &Value) -> Result<LLMClientConfig> {
        let url = config
            .get("url")
            .and_then(|v| v.as_str())
            .context("Missing 'url' field")?
            .to_string();

        let api_key = config
            .get("api_key")
            .and_then(|v| v.as_str())
            .context("Missing 'api_key' field")?
            .to_string();

        let response_format = match config.get("response_format") {
            Some(Value::String(format)) if format == "openai" => ResponseFormat::OpenAI,
            Some(Value::String(format)) if format == "custom" => {
                let custom_format = config.get("custom_format")
                    .context("Missing 'custom_format' for custom response format")?;
                let content_path = custom_format
                    .get("content_path")
                    .and_then(|v| v.as_str())
                    .context("Missing 'content_path' in custom_format")?
                    .to_string();
                let error_path = custom_format
                    .get("error_path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                ResponseFormat::Custom {
                    content_path,
                    error_path,
                }
            }
            _ => ResponseFormat::OpenAI, // 默认使用 OpenAI 格式
        };

        let timeout = config.get("timeout")
            .and_then(|v| v.as_integer())
            .map(|i| i as u64);

        let retry_count = config.get("retry_count")
            .and_then(|v| v.as_integer())
            .map(|i| i as u32);

        Ok(LLMClientConfig {
            url,
            api_key,
            response_format,
            timeout,
            retry_count,
        })
    }
}
```

---

### 步骤 4：更新模块导出（mod.rs）

**文件**：`src/lib/llm/client/mod.rs`

```rust
//! LLM 客户端模块
//!
//! 本模块提供了统一配置驱动的 LLM 客户端实现。

pub mod client;
pub mod common;
pub mod config;

// 向后兼容：保留旧的客户端（标记为 deprecated）
#[deprecated(note = "Use client::LLMClient instead")]
pub mod deepseek;
#[deprecated(note = "Use client::LLMClient instead")]
pub mod openai;
#[deprecated(note = "Use client::LLMClient instead")]
pub mod proxy;

pub use common::LLMRequestParams;
pub use config::LLMConfig;
pub use client::{LLMClientConfig, ResponseFormat, LLMClient};
```

---

### 步骤 5：更新 PullRequestLLM（pr_llm.rs）

**关键更改**：

```rust
use super::client::{LLMConfig, LLMRequestParams, LLMClient};

impl PullRequestLLM {
    pub fn generate(
        commit_title: &str,
        exists_branches: Option<Vec<String>>,
        git_diff: Option<String>,
    ) -> Result<PullRequestContent> {
        // 1. 尝试从配置文件加载
        let client = if let Some(config_path) = LLMConfig::find_config_path() {
            match LLMConfig::load(&config_path) {
                Ok(config) => {
                    // 从配置文件获取客户端
                    config.get_default_client()?
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load config file: {}", e);
                    // 回退到环境变量
                    LLMConfig::from_env()?
                        .context("No LLM configuration found")?
                }
            }
        } else {
            // 从环境变量创建默认配置
            LLMConfig::from_env()?
                .context("No LLM configuration found")?
        };

        // 2. 构建请求参数
        let params = LLMRequestParams {
            system_prompt: Self::system_prompt(),
            user_prompt: Self::user_prompt(commit_title, exists_branches, git_diff),
            max_tokens: 100,
            temperature: 0.5,
            model: "gpt-3.5-turbo".to_string(),
        };

        // 3. 调用统一客户端
        let response = client.call(&params)?;

        // 4. 解析响应
        Self::parse_llm_response(response)
    }

    // ... 其他方法保持不变 ...
}
```

---

## 🧪 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_client_openai_format() {
        // 测试 OpenAI 格式响应解析
    }

    #[test]
    fn test_unified_client_custom_format() {
        // 测试自定义格式响应解析
    }

    #[test]
    fn test_config_loading() {
        // 测试配置文件加载
    }

    #[test]
    fn test_env_var_resolution() {
        // 测试环境变量解析
    }
}
```

### 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    #[ignore] // 需要真实的 API key
    fn test_openai_integration() {
        // 测试真实的 OpenAI API 调用
    }
}
```

---

## 🔄 迁移计划

### 阶段 1：实现新代码（不破坏现有功能）

1. ✅ 创建 `common.rs`、`client.rs`、`config.rs`
2. ✅ 更新 `client/mod.rs` 导出新模块
3. ✅ 保持 `openai.rs`、`deepseek.rs`、`proxy.rs` 不变

### 阶段 2：更新 PullRequestLLM（向后兼容）

1. ✅ 更新 `pr_llm.rs` 优先使用统一客户端
2. ✅ 如果配置文件不存在，回退到环境变量
3. ✅ 如果环境变量配置失败，回退到旧的客户端实现

### 阶段 3：移除旧代码（可选）

1. ⚠️ 标记旧客户端为 deprecated
2. ⚠️ 等待一段时间确保用户迁移完成
3. ⚠️ 移除 `openai.rs`、`deepseek.rs`、`proxy.rs`

---

## ✅ 实现检查清单

- [ ] 添加 `toml` 依赖到 `Cargo.toml`
- [ ] 创建 `src/lib/llm/client/common.rs`
- [ ] 创建 `src/lib/llm/client/client.rs`
- [ ] 创建 `src/lib/llm/client/config.rs`
- [ ] 更新 `src/lib/llm/client/mod.rs`
- [ ] 更新 `src/lib/llm/pr_llm.rs`
- [ ] 添加单元测试
- [ ] 测试配置文件加载
- [ ] 测试环境变量回退
- [ ] 测试向后兼容性

---

## 📝 注意事项

1. **错误处理**：配置文件不存在时应该静默回退到环境变量
2. **环境变量解析**：确保 `${VAR}` 和 `${VAR:default}` 格式正确解析
3. **配置验证**：加载配置后必须验证必填字段
4. **向后兼容**：保持现有 API 不变，确保现有代码继续工作
5. **测试覆盖**：确保所有代码路径都有测试覆盖

---

## 🔗 相关文档

- [LLM_PLUGIN_ARCHITECTURE.md](./LLM_PLUGIN_ARCHITECTURE.md) - 架构设计
- [LLM_PLUGIN_CURL.md](./LLM_PLUGIN_CURL.md) - API 调用示例

