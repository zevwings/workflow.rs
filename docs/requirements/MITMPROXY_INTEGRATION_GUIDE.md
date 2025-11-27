# mitmproxy 集成指南

## 📋 概述

本文档介绍如何在项目中使用和集成 **mitmproxy**，实现从终端访问和记录 HTTP/HTTPS 请求。

## 🎯 为什么选择 mitmproxy

相比 Proxyman（GUI 工具），mitmproxy 的优势：

- ✅ **原生命令行工具**：完全在终端中运行
- ✅ **实时捕获**：可以实时查看和修改请求
- ✅ **Python 脚本扩展**：支持自定义脚本处理请求
- ✅ **多种接口**：提供 mitmproxy（交互式）、mitmdump（命令行）、mitmweb（Web 界面）
- ✅ **易于集成**：可以通过环境变量或代码配置代理
- ✅ **数据导出**：支持导出为 HAR、JSON 等格式

## 📦 安装

### macOS

```bash
# 使用 Homebrew 安装
brew install mitmproxy

# 或使用 pip
pip3 install mitmproxy
```

### 验证安装

```bash
mitmproxy --version
```

## 🚀 基本使用

### 1. 启动 mitmproxy

```bash
# 交互式界面（推荐用于调试）
mitmproxy -p 8080

# 命令行模式（适合脚本集成）
mitmdump -p 8080

# Web 界面
mitmweb -p 8080
```

### 2. 配置系统代理

mitmproxy 默认监听 `127.0.0.1:8080`，需要配置系统或应用使用该代理。

#### 方法一：使用项目的代理管理功能

```bash
# 1. 手动设置系统代理为 127.0.0.1:8080
# 2. 使用项目的代理命令启用
workflow proxy on
```

#### 方法二：直接设置环境变量

```bash
export http_proxy=http://127.0.0.1:8080
export https_proxy=http://127.0.0.1:8080
export all_proxy=socks5://127.0.0.1:8080
```

### 3. 安装证书（HTTPS 支持）

mitmproxy 需要安装 CA 证书才能解密 HTTPS 流量：

```bash
# 1. 启动 mitmproxy
mitmproxy -p 8080

# 2. 在浏览器中访问 http://mitm.it
# 3. 下载并安装对应平台的证书

# macOS 安装步骤：
# - 下载证书后，双击打开
# - 在"钥匙串访问"中找到 mitmproxy 证书
# - 双击证书，展开"信任"，选择"始终信任"
```

## 🔧 集成到项目

### 方案一：通过环境变量配置（推荐）

修改 `HttpClient` 以支持从环境变量读取代理配置：

```rust
// src/lib/base/http/client.rs

impl HttpClient {
    fn new() -> Result<Self> {
        let mut builder = Client::builder();

        // 从环境变量读取代理配置
        if let Ok(proxy_url) = std::env::var("http_proxy")
            .or_else(|_| std::env::var("https_proxy"))
            .or_else(|_| std::env::var("all_proxy"))
        {
            if let Ok(proxy) = reqwest::Proxy::http(&proxy_url) {
                builder = builder.proxy(proxy);
            }
        }

        let client = builder
            .build()
            .context("Failed to create HTTP client")?;
        Ok(Self { client })
    }
}
```

### 方案二：通过配置参数指定代理

扩展 `RequestConfig` 支持代理配置：

```rust
// src/lib/base/http/config.rs

pub struct RequestConfig<'a, B, Q: ?Sized> {
    // ... 现有字段
    pub proxy: Option<&'a str>,  // 新增：代理 URL
}

impl<'a, B, Q: ?Sized> RequestConfig<'a, B, Q> {
    pub fn proxy(mut self, proxy_url: &'a str) -> Self {
        self.proxy = Some(proxy_url);
        self
    }
}
```

然后在 `HttpClient::build_request` 中应用代理：

```rust
fn build_request<B, Q>(...) -> reqwest::blocking::RequestBuilder {
    let mut builder = Client::builder();

    if let Some(proxy_url) = config.proxy {
        if let Ok(proxy) = reqwest::Proxy::http(proxy_url) {
            builder = builder.proxy(proxy);
        }
    }

    // ... 其他配置
}
```

### 方案三：创建专用的 mitmproxy 客户端

创建一个专门用于捕获请求的 HTTP 客户端：

```rust
// src/lib/base/http/mitm_client.rs

use crate::base::http::{HttpClient, RequestConfig};
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::Value;

/// mitmproxy HTTP 客户端
///
/// 专门用于通过 mitmproxy 捕获请求的客户端
pub struct MitmHttpClient {
    client: Client,
    proxy_url: String,
}

impl MitmHttpClient {
    /// 创建新的 mitmproxy 客户端
    ///
    /// # 参数
    ///
    /// * `proxy_url` - mitmproxy 代理地址，默认为 `http://127.0.0.1:8080`
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use crate::base::http::MitmHttpClient;
    ///
    /// let client = MitmHttpClient::new(Some("http://127.0.0.1:8080"))?;
    /// ```
    pub fn new(proxy_url: Option<&str>) -> Result<Self> {
        let proxy_url = proxy_url.unwrap_or("http://127.0.0.1:8080").to_string();

        let proxy = reqwest::Proxy::http(&proxy_url)
            .context("Failed to create proxy")?;

        let client = Client::builder()
            .proxy(proxy)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, proxy_url })
    }

    /// 执行 GET 请求（通过 mitmproxy）
    pub fn get<Q>(&self, url: &str, config: RequestConfig<Value, Q>) -> Result<HttpResponse>
    where
        Q: Serialize + ?Sized,
    {
        // 使用与 HttpClient 相同的逻辑，但通过 mitmproxy
        // ...
    }
}
```

## 📝 捕获和记录请求

### 方法一：使用 mitmdump 脚本

创建 Python 脚本来记录请求：

```python
# scripts/mitm_record.py

import json
from datetime import datetime
from mitmproxy import http
from pathlib import Path

# 请求记录存储目录
RECORD_DIR = Path.home() / ".workflow" / "mitm_records"
RECORD_DIR.mkdir(parents=True, exist_ok=True)

def request(flow: http.HTTPFlow) -> None:
    """记录请求"""
    record = {
        "timestamp": datetime.now().isoformat(),
        "method": flow.request.method,
        "url": flow.request.pretty_url,
        "headers": dict(flow.request.headers),
        "content": flow.request.content.decode("utf-8", errors="ignore"),
    }

    # 保存到文件
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    filename = RECORD_DIR / f"request_{timestamp}.json"

    with open(filename, "w") as f:
        json.dump(record, f, indent=2)

    print(f"Recorded: {flow.request.method} {flow.request.pretty_url}")

def response(flow: http.HTTPFlow) -> None:
    """记录响应"""
    record = {
        "timestamp": datetime.now().isoformat(),
        "status_code": flow.response.status_code,
        "headers": dict(flow.response.headers),
        "content": flow.response.content.decode("utf-8", errors="ignore"),
    }

    # 保存到文件
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    filename = RECORD_DIR / f"response_{timestamp}.json"

    with open(filename, "w") as f:
        json.dump(record, f, indent=2)
```

使用脚本：

```bash
mitmdump -p 8080 -s scripts/mitm_record.py
```

### 方法二：使用 mitmdump 导出 HAR

```bash
# 导出为 HAR 格式
mitmdump -p 8080 -w requests.har

# 导出为流文件（可以重放）
mitmdump -p 8080 -w requests.flow
```

### 方法三：从 Rust 代码中读取记录

创建模块来读取 mitmproxy 记录：

```rust
// src/lib/base/http/mitm_records.rs

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestRecord {
    pub timestamp: String,
    pub method: String,
    pub url: String,
    pub headers: serde_json::Value,
    pub content: String,
}

pub struct MitmRecords {
    record_dir: PathBuf,
}

impl MitmRecords {
    pub fn new() -> Result<Self> {
        let record_dir = dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".workflow")
            .join("mitm_records");

        // 确保目录存在
        fs::create_dir_all(&record_dir)
            .context("Failed to create mitm records directory")?;

        Ok(Self { record_dir })
    }

    /// 列出所有请求记录
    pub fn list_requests(&self) -> Result<Vec<PathBuf>> {
        let mut records = Vec::new();

        for entry in fs::read_dir(&self.record_dir)
            .context("Failed to read records directory")?
        {
            let entry = entry?;
            let path = entry.path();

            if path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("request_"))
                .unwrap_or(false)
            {
                records.push(path);
            }
        }

        // 按时间排序（最新的在前）
        records.sort_by(|a, b| {
            b.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .cmp(
                    &a.metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                )
        });

        Ok(records)
    }

    /// 读取请求记录
    pub fn read_request(&self, path: &Path) -> Result<RequestRecord> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read request record: {:?}", path))?;

        let record: RequestRecord = serde_json::from_str(&content)
            .context("Failed to parse request record")?;

        Ok(record)
    }

    /// 搜索请求记录
    pub fn search(&self, query: &str) -> Result<Vec<RequestRecord>> {
        let mut results = Vec::new();

        for record_path in self.list_requests()? {
            let record = self.read_request(&record_path)?;

            if record.url.contains(query)
                || record.method.contains(query)
                || record.content.contains(query)
            {
                results.push(record);
            }
        }

        Ok(results)
    }
}
```

## 🛠️ 命令行工具集成

### 创建 mitmproxy 管理命令

```rust
// src/commands/mitm/mod.rs

pub mod mitm;

// src/commands/mitm/mitm.rs

use anyhow::{Context, Result};
use crate::base::http::MitmRecords;
use crate::{log_info, log_success, log_message};

pub struct MitmCommand;

impl MitmCommand {
    /// 列出所有请求记录
    pub fn list() -> Result<()> {
        let records = MitmRecords::new()?;
        let paths = records.list_requests()?;

        log_success!("Found {} request records", paths.len());

        for path in paths {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                log_info!("  {}", filename);
            }
        }

        Ok(())
    }

    /// 搜索请求记录
    pub fn search(query: &str) -> Result<()> {
        let records = MitmRecords::new()?;
        let results = records.search(query)?;

        log_success!("Found {} matching requests", results.len());

        for record in results {
            log_break!();
            log_info!("Method: {}", record.method);
            log_info!("URL: {}", record.url);
            log_info!("Time: {}", record.timestamp);
        }

        Ok(())
    }

    /// 显示请求详情
    pub fn show(path: &str) -> Result<()> {
        let records = MitmRecords::new()?;
        let record = records.read_request(Path::new(path))?;

        log_success!("Request Details:");
        log_info!("  Method: {}", record.method);
        log_info!("  URL: {}", record.url);
        log_info!("  Time: {}", record.timestamp);
        log_message!("  Headers: {}", serde_json::to_string_pretty(&record.headers)?);
        log_message!("  Content: {}", record.content);

        Ok(())
    }
}
```

### 添加到 CLI

```rust
// src/main.rs

#[derive(Parser)]
#[command(name = "workflow")]
pub struct Cli {
    // ... 现有字段

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    // ... 现有命令

    /// mitmproxy 相关命令
    #[command(subcommand)]
    Mitm(MitmSubcommand),
}

#[derive(Subcommand)]
pub enum MitmSubcommand {
    /// 列出所有请求记录
    List,
    /// 搜索请求记录
    Search {
        /// 搜索关键词
        query: String,
    },
    /// 显示请求详情
    Show {
        /// 请求记录文件路径
        path: String,
    },
}
```

## 📊 使用示例

### 1. 启动 mitmproxy 并记录请求

```bash
# 终端 1：启动 mitmproxy
mitmdump -p 8080 -s scripts/mitm_record.py

# 终端 2：设置代理并执行请求
export http_proxy=http://127.0.0.1:8080
export https_proxy=http://127.0.0.1:8080
workflow pr test-api 123
```

### 2. 查看记录的请求

```bash
# 列出所有请求
workflow mitm list

# 搜索特定请求
workflow mitm search "api.github.com"

# 查看请求详情
workflow mitm show ~/.workflow/mitm_records/request_20240101_120000.json
```

### 3. 在代码中使用 mitmproxy 客户端

```rust
use crate::base::http::MitmHttpClient;

let client = MitmHttpClient::new(Some("http://127.0.0.1:8080"))?;
let response = client.get("https://api.example.com", RequestConfig::new())?;
```

## ⚙️ 配置项

在 `Settings` 中添加 mitmproxy 配置：

```toml
[mitmproxy]
# mitmproxy 代理地址
proxy_url = "http://127.0.0.1:8080"

# 请求记录目录
record_dir = "~/.workflow/mitm_records"

# 是否自动启用 mitmproxy（如果检测到 mitmproxy 运行）
auto_enable = true

# 记录脚本路径
record_script = "scripts/mitm_record.py"
```

## 🔍 高级功能

### 1. 过滤特定请求

在 Python 脚本中添加过滤：

```python
def request(flow: http.HTTPFlow) -> None:
    # 只记录特定域名的请求
    if "api.github.com" not in flow.request.pretty_url:
        return

    # 记录请求
    # ...
```

### 2. 修改请求/响应

```python
def request(flow: http.HTTPFlow) -> None:
    # 添加自定义 header
    flow.request.headers["X-Custom-Header"] = "value"

    # 修改请求体
    if flow.request.content:
        content = flow.request.content.decode()
        # 修改内容
        flow.request.content = modified_content.encode()

def response(flow: http.HTTPFlow) -> None:
    # 修改响应
    if flow.response.status_code == 200:
        # 处理响应
        pass
```

### 3. 重放请求

```bash
# 使用保存的流文件重放
mitmdump -p 8080 -r requests.flow
```

## 📚 参考资源

- [mitmproxy 官方文档](https://docs.mitmproxy.org/)
- [mitmproxy Python API](https://docs.mitmproxy.org/stable/api/)
- [mitmproxy 脚本示例](https://github.com/mitmproxy/mitmproxy/tree/main/examples)

## ✅ 总结

通过集成 mitmproxy，我们可以：

1. ✅ **从终端访问请求记录**：通过命令行工具查看和搜索请求
2. ✅ **实时捕获请求**：在测试过程中自动记录所有请求
3. ✅ **灵活的数据处理**：支持 Python 脚本自定义处理逻辑
4. ✅ **易于集成**：通过环境变量或代码配置即可使用
5. ✅ **多种导出格式**：支持 HAR、JSON、流文件等格式

相比 Proxyman，mitmproxy 更适合命令行工作流和自动化测试场景。


