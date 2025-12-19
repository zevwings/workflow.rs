# GitHub OAuth 认证功能 TODO

## 📋 概述

本文档记录了为 Workflow CLI 添加 GitHub OAuth Device Flow 认证功能的计划。该功能将允许用户通过 OAuth 2.0 Device Authorization Flow 自动获取 GitHub Personal Access Token，无需手动创建 Token。

---

## 🎯 功能目标

### 当前实现方式
- 用户需要手动在 GitHub 网站创建 Personal Access Token (PAT)
- 通过 `workflow github add` 命令手动输入 Token
- 简单直接，但需要手动操作

### 目标实现方式
- 通过 GitHub OAuth Device Flow 自动获取 Token
- 提供更好的用户体验，无需手动创建 Token
- 支持 Token 刷新（如果使用 refresh token）

---

## 📝 待实现功能

### 1. OAuth Device Flow 实现

#### 1.1 注册 OAuth App
- ❌ 在 GitHub Settings → Developer settings → OAuth Apps 注册应用
- ❌ 获取 Client ID
- ❌ 启用 Device Flow（默认关闭，需手动开启）
- ❌ 配置回调 URL（虽然 Device Flow 不使用，但必须指定）

**实现建议**：
- 提供文档说明如何注册 OAuth App
- 考虑是否需要在项目中预配置一个公共的 Client ID（需要评估安全性）

#### 1.2 Device Flow 流程实现
- ❌ 请求设备码：调用 `POST https://github.com/login/device/code`
- ❌ 用户授权：显示 `user_code` 和 `verification_uri`，引导用户完成授权
- ❌ 轮询获取 Token：定期调用 `POST https://github.com/login/oauth/access_token`
- ❌ 处理授权状态：处理 `authorization_pending`、`slow_down` 等状态
- ❌ 错误处理：处理超时、用户拒绝等错误情况

**API 端点**：
- `POST https://github.com/login/device/code` - 获取设备码和用户码
- `POST https://github.com/login/oauth/access_token` - 轮询获取访问令牌

**请求参数**：
```rust
// 请求设备码
{
    "client_id": "your_client_id",
    "scope": "repo workflow"
}

// 轮询获取 Token
{
    "client_id": "your_client_id",
    "device_code": "device_code_from_step1",
    "grant_type": "urn:ietf:params:oauth:grant-type:device_code"
}
```

**响应格式**：
```rust
// 设备码响应
{
    "device_code": "xxx",
    "user_code": "ABCD-1234",
    "verification_uri": "https://github.com/login/device",
    "expires_in": 900,
    "interval": 5
}

// Token 响应
{
    "access_token": "xxx",
    "token_type": "bearer",
    "scope": "repo workflow"
}
```

#### 1.3 用户交互优化
- ❌ 自动打开浏览器（可选）
- ❌ 显示清晰的授权指引
- ❌ 显示轮询进度（spinner）
- ❌ 支持取消操作

**用户体验流程**：
1. 用户运行 `workflow github add --oauth` 或选择 OAuth 方式
2. 显示授权码和验证 URL
3. 自动或手动打开浏览器
4. 用户输入授权码完成授权
5. 显示轮询进度
6. 获取 Token 后自动保存到配置

---

## 🔧 实现细节

### 2. 代码结构

#### 2.1 新增模块
- ❌ `src/lib/github/oauth.rs` - OAuth Device Flow 实现
- ❌ `src/lib/github/mod.rs` - GitHub 模块（如果不存在）

**模块结构**：
```rust
// src/lib/github/oauth.rs
pub struct OAuthDeviceFlow {
    client_id: String,
    client: reqwest::Client,
}

impl OAuthDeviceFlow {
    pub async fn request_device_code(&self, scope: &str) -> Result<DeviceCodeResponse>;
    pub async fn poll_for_token(&self, device_code: &str) -> Result<TokenResponse>;
    pub async fn authenticate(&self, scope: &str) -> Result<String>;
}
```

#### 2.2 修改现有命令
- ❌ 修改 `src/commands/github/github.rs::add()` - 添加 OAuth 选项
- ❌ 修改 `src/commands/github/helpers.rs::collect_github_account()` - 支持 OAuth 流程

**命令选项**：
```rust
// 在 workflow github add 中添加选项
workflow github add                    # 手动输入 Token（当前方式）
workflow github add --oauth            # 使用 OAuth Device Flow
workflow github add --oauth --scope "repo workflow"  # 指定 scope
```

#### 2.3 配置管理
- ❌ 在配置文件中存储 OAuth Client ID（可选）
- ❌ 支持从环境变量读取 Client ID
- ❌ 支持从配置文件读取 Client ID

**配置示例**：
```toml
[github.oauth]
client_id = "your_client_id"  # 可选，也可以从环境变量读取
```

**环境变量**：
```bash
GITHUB_OAUTH_CLIENT_ID=your_client_id
```

---

## 📚 技术实现

### 3. 依赖库

#### 3.1 HTTP 客户端
- ✅ 已有 `reqwest` - 用于 HTTP 请求
- ❌ 可能需要添加 `url` crate（如果未包含）- 用于 URL 处理

#### 3.2 异步处理
- ✅ 已有 `tokio` - 用于异步运行时
- ❌ 需要实现轮询逻辑（使用 `tokio::time::sleep`）

#### 3.3 浏览器打开
- ❌ 添加 `open` crate - 用于自动打开浏览器
- 或使用系统命令（`open` on macOS, `xdg-open` on Linux, `start` on Windows）

### 4. 实现示例

#### 4.1 Device Flow 实现
```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    scope: String,
}

pub struct OAuthDeviceFlow {
    client_id: String,
    client: Client,
}

impl OAuthDeviceFlow {
    pub async fn request_device_code(&self, scope: &str) -> Result<DeviceCodeResponse> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("scope", scope),
        ];

        let response = self.client
            .post("https://github.com/login/device/code")
            .form(&params)
            .send()
            .await?;

        let device_code: DeviceCodeResponse = response.json().await?;
        Ok(device_code)
    }

    pub async fn poll_for_token(
        &self,
        device_code: &str,
        interval: u64,
    ) -> Result<TokenResponse> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("device_code", device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ];

        loop {
            let response = self.client
                .post("https://github.com/login/oauth/access_token")
                .form(&params)
                .header("Accept", "application/json")
                .send()
                .await?;

            if response.status().is_success() {
                let token: TokenResponse = response.json().await?;
                return Ok(token);
            } else if response.status() == 400 {
                let error: serde_json::Value = response.json().await?;
                let error_type = error.get("error").and_then(|e| e.as_str());

                match error_type {
                    Some("authorization_pending") => {
                        // 继续轮询
                        sleep(Duration::from_secs(interval)).await;
                        continue;
                    }
                    Some("slow_down") => {
                        // 增加轮询间隔
                        sleep(Duration::from_secs(interval + 5)).await;
                        continue;
                    }
                    Some("expired_token") => {
                        return Err(eyre!("Device code expired"));
                    }
                    Some("access_denied") => {
                        return Err(eyre!("User denied authorization"));
                    }
                    _ => {
                        return Err(eyre!("Authorization failed: {:?}", error));
                    }
                }
            } else {
                return Err(eyre!("HTTP error: {}", response.status()));
            }
        }
    }

    pub async fn authenticate(&self, scope: &str) -> Result<String> {
        // 1. 请求设备码
        let device_code_resp = self.request_device_code(scope).await?;

        // 2. 显示授权信息
        println!("Please visit: {}", device_code_resp.verification_uri);
        println!("Enter code: {}", device_code_resp.user_code);

        // 3. 打开浏览器（可选）
        if let Err(e) = open::that(&device_code_resp.verification_uri) {
            eprintln!("Failed to open browser: {}", e);
        }

        // 4. 轮询获取 Token
        let token_resp = self.poll_for_token(
            &device_code_resp.device_code,
            device_code_resp.interval,
        ).await?;

        Ok(token_resp.access_token)
    }
}
```

#### 4.2 集成到现有命令
```rust
// src/commands/github/helpers.rs
pub fn collect_github_account_with_oauth() -> Result<GitHubAccount> {
    // 询问用户选择认证方式
    let use_oauth = ConfirmDialog::new("Use OAuth Device Flow? (otherwise manual token input)")
        .with_default(true)
        .prompt()?;

    if use_oauth {
        // 使用 OAuth Device Flow
        let client_id = get_oauth_client_id()?; // 从配置或环境变量获取
        let oauth = OAuthDeviceFlow::new(client_id);
        let scope = "repo workflow"; // 默认 scope
        let token = oauth.authenticate(scope).await?;

        // 获取用户信息（使用 token 调用 GitHub API）
        let user_info = get_github_user_info(&token).await?;

        Ok(GitHubAccount {
            name: user_info.login,
            email: user_info.email,
            api_token: token,
        })
    } else {
        // 使用现有的手动输入方式
        collect_github_account()
    }
}
```

---

## 🧪 测试计划

### 5. 单元测试
- ❌ 测试 `request_device_code()` - 模拟 GitHub API 响应
- ❌ 测试 `poll_for_token()` - 模拟各种响应状态
- ❌ 测试错误处理 - 超时、拒绝、过期等场景

### 6. 集成测试
- ❌ 测试完整的 OAuth 流程（需要真实的 OAuth App）
- ❌ 测试与现有 `workflow github add` 命令的集成
- ❌ 测试配置保存和加载

### 7. 手动测试
- ❌ 测试用户授权流程
- ❌ 测试浏览器自动打开
- ❌ 测试取消操作
- ❌ 测试错误场景（网络错误、用户拒绝等）

---

## 📊 优先级

### 高优先级
1. **OAuth Device Flow 核心实现**
   - 请求设备码
   - 轮询获取 Token
   - 错误处理

2. **用户交互优化**
   - 显示授权信息
   - 轮询进度显示
   - 浏览器自动打开（可选）

### 中优先级
1. **配置管理**
   - 支持从配置文件读取 Client ID
   - 支持从环境变量读取 Client ID

2. **命令集成**
   - 修改 `workflow github add` 支持 OAuth 选项
   - 保持向后兼容（默认手动输入）

### 低优先级
1. **增强功能**
   - Token 刷新支持（如果 GitHub 支持）
   - 多 scope 选择
   - 授权状态缓存

---

## 🔒 安全考虑

### 8. 安全最佳实践
- ✅ 使用 HTTPS 进行所有 API 调用
- ✅ Token 存储在配置文件中（已加密或权限保护）
- ⚠️ **Client ID 管理**：考虑是否在项目中预配置公共 Client ID
  - 优点：用户无需注册 OAuth App
  - 缺点：安全性较低，可能被滥用
  - 建议：提供文档说明如何注册自己的 OAuth App，同时可选支持公共 Client ID
- ⚠️ **Token 安全**：确保 Token 不会泄露到日志或错误消息中
- ⚠️ **Scope 最小化**：只请求必要的权限（`repo`、`workflow`）

---

## 📚 相关文档

### GitHub 官方文档
- [GitHub OAuth Device Flow](https://docs.github.com/apps/building-oauth-apps/authorizing-oauth-apps#device-flow)
- [GitHub OAuth Apps](https://docs.github.com/apps/oauth-apps/building-oauth-apps)
- [GitHub API Authentication](https://docs.github.com/en/rest/authentication)

### 项目相关文档
- [GitHub 命令架构文档](../architecture/commands/GITHUB_COMMAND_ARCHITECTURE.md)
- [GitHub 配置指南](../guidelines/GITHUB_SETUP_GUIDELINES.md)

---

## 📝 实现建议

### 开发顺序
1. **第一阶段**：核心 OAuth Device Flow 实现
   - 实现 `OAuthDeviceFlow` 结构体和基本方法
   - 实现设备码请求和 Token 轮询
   - 添加单元测试

2. **第二阶段**：用户交互和集成
   - 集成到 `workflow github add` 命令
   - 添加用户交互（显示授权信息、进度显示）
   - 添加浏览器自动打开功能

3. **第三阶段**：配置和优化
   - 支持从配置文件/环境变量读取 Client ID
   - 优化错误处理和用户体验
   - 添加集成测试

### 技术考虑
1. **异步处理**：使用 `tokio` 进行异步轮询
2. **错误处理**：提供清晰的错误消息和恢复建议
3. **用户体验**：提供清晰的授权指引和进度反馈
4. **向后兼容**：保持现有的手动输入方式作为默认或备选
5. **测试**：添加充分的单元测试和集成测试
6. **文档**：更新用户文档，说明如何使用 OAuth 认证

---

## ✅ 验收标准

### 功能验收
- [ ] 用户可以通过 `workflow github add --oauth` 使用 OAuth 认证
- [ ] 能够成功获取 GitHub Personal Access Token
- [ ] Token 正确保存到配置文件
- [ ] 支持取消操作
- [ ] 错误场景有清晰的错误提示

### 用户体验验收
- [ ] 授权指引清晰易懂
- [ ] 轮询进度有反馈
- [ ] 浏览器自动打开（可选）
- [ ] 与手动输入方式无缝切换

### 代码质量验收
- [ ] 代码通过 `cargo clippy` 检查
- [ ] 代码通过 `cargo fmt` 格式化
- [ ] 单元测试覆盖率 > 80%
- [ ] 集成测试通过
- [ ] 文档完整更新

---

**最后更新**: 2025-01-XX
