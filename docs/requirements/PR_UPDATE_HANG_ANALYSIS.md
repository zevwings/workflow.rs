# `workflow pr update` 卡死问题分析

## 问题描述

运行 `workflow pr update` 命令时，程序会直接卡死，需要多次按 Ctrl+C 才能退出。

## 执行流程分析

### 命令执行流程

```
workflow pr update
  ↓
PullRequestUpdateCommand::update()
  ↓
1. ProxyManager::ensure_proxy_enabled()     # 步骤 0：检查并启用代理
  ↓
2. get_pull_request_title()                  # 步骤 1：获取 PR 标题
   ├─ get_current_branch_pr_id()            # 1.1：获取当前分支的 PR ID
   │   └─ provider.get_current_branch_pull_request()
   │       └─ GitHub API 调用（可能多次）
   └─ provider.get_pull_request_title()      # 1.2：获取 PR 标题
       └─ GitHub API 调用
  ↓
3. GitPreCommit::run_checks()                # 步骤 2：运行 pre-commit 检查
  ↓
4. GitCommit::commit()                       # 步骤 3：提交更改
  ↓
5. GitBranch::push()                         # 步骤 4：推送到远程
```

## 可能导致卡死的原因

### 原因 1：`ProxyManager::ensure_proxy_enabled()` 阻塞

**位置**：`src/commands/pr/update.rs:19`

**问题分析**：
- 调用 `SystemProxyReader::read()` 读取系统代理设置
- 使用 `duct::cmd("scutil", &["--proxy"]).read()` 执行系统命令
- **如果系统代理设置有问题或系统响应慢，`scutil` 命令可能会阻塞**

**代码路径**：
```rust
ProxyManager::ensure_proxy_enabled()
  ↓
SystemProxyReader::read()
  ↓
cmd("scutil", &["--proxy"]).read()  // 可能阻塞
```

**可能的情况**：
1. macOS 系统代理设置损坏或异常
2. `scutil` 命令响应慢（系统负载高）
3. 权限问题导致命令挂起

### 原因 2：`get_current_branch_pr_id()` 网络请求阻塞

**位置**：`src/commands/pr/update.rs:56`

**问题分析**：
- 调用 `get_current_branch_pr_id()` 获取当前分支的 PR ID
- 内部会调用 GitHub API，**可能执行多次网络请求**：
  1. 首先尝试查找 `state=open` 的 PR（一次 API 调用）
  2. 如果找不到，再查找 `state=all` 的 PR（第二次 API 调用）
  3. 如果还是找不到，从 work-history 文件查找（本地操作，不会阻塞）

**代码路径**：
```rust
get_current_branch_pr_id()
  ↓
provider.get_current_branch_pull_request()
  ↓
GitHub::get_current_branch_pull_request()
  ↓
1. GET /repos/{owner}/{repo}/pulls?head={owner}:{branch}&state=open    # 第一次 API 调用
2. GET /repos/{owner}/{repo}/pulls?head={owner}:{branch}&state=all     # 第二次 API 调用（如果第一次失败）
```

**可能的情况**：
1. **网络连接问题**：无法连接到 GitHub API
2. **代理配置错误**：虽然 `ensure_proxy_enabled()` 设置了代理，但代理配置可能不正确
3. **DNS 解析失败**：无法解析 `api.github.com`
4. **超时设置问题**：虽然 HTTP 客户端有 30 秒默认超时，但在某些情况下（如代理连接建立失败）可能不会触发超时

### 原因 3：`get_pull_request_title()` 网络请求阻塞

**位置**：`src/commands/pr/update.rs:66-68`

**问题分析**：
- 如果找到了 PR ID，会再次调用 GitHub API 获取 PR 标题
- 使用 `Spinner::with()` 显示加载动画，但**如果网络请求阻塞，Spinner 会一直显示，程序看起来像卡死**

**代码路径**：
```rust
provider.get_pull_request_title(&pr_id)
  ↓
GitHub::get_pull_request_title()
  ↓
fetch_pr_info_internal()
  ↓
GET /repos/{owner}/{repo}/pulls/{pr_number}  # API 调用
```

**可能的情况**：
1. 网络连接问题
2. 代理配置错误
3. GitHub API 响应慢或超时

### 原因 4：HTTP 客户端超时机制可能失效

**位置**：`src/lib/base/http/client.rs:143`

**问题分析**：
- HTTP 客户端默认超时为 30 秒
- 但在某些情况下，超时可能不会正确触发：
  1. **代理连接建立阶段**：如果代理服务器无法连接，可能在建立连接阶段就卡住，而不是在请求阶段
  2. **DNS 解析阶段**：DNS 解析失败可能不会触发 HTTP 客户端的超时机制
  3. **TCP 连接阶段**：TCP 连接建立失败可能不会触发超时

**代码**：
```rust
// src/lib/base/http/client.rs:143
let timeout_duration = config.timeout.unwrap_or_else(|| Duration::from_secs(30));
request = request.timeout(timeout_duration);
```

**注意**：`reqwest` 的 `timeout()` 主要针对请求阶段，对于连接建立阶段的超时可能不够有效。

### 原因 5：`duct::cmd` 没有超时设置

**位置**：`src/lib/proxy/system_reader.rs:29`

**问题分析**：
- `SystemProxyReader::read()` 使用 `duct::cmd("scutil", &["--proxy"]).read()` 执行系统命令
- **`duct` 库默认没有超时设置**，如果 `scutil` 命令阻塞，会一直等待

**代码**：
```rust
let output = cmd("scutil", &["--proxy"])
    .read()
    .context("Failed to get system proxy settings")?;
```

**可能的情况**：
1. `scutil` 命令在某些系统状态下可能响应很慢
2. 系统代理设置损坏导致命令挂起
3. 权限问题导致命令无法正常返回

## 最可能的卡死点

根据代码执行流程和常见问题，**最可能的卡死点按优先级排序**：

### 1. **`get_current_branch_pr_id()` 中的 GitHub API 调用**（最可能）

**原因**：
- 这是第一个需要网络访问的操作
- 如果网络有问题或代理配置错误，会在这里卡住
- 可能执行两次 API 调用（open 状态 + all 状态），增加了卡死的概率

**证据**：
- 用户需要多次按 Ctrl+C 才能退出，说明程序可能在等待网络响应
- 没有看到任何输出，说明可能在 `get_pull_request_title()` 之前就卡住了

### 2. **`ProxyManager::ensure_proxy_enabled()` 中的 `scutil` 命令**（次可能）

**原因**：
- 这是命令执行的第一个步骤
- `scutil` 命令没有超时设置
- 如果系统代理设置有问题，可能在这里卡住

**证据**：
- 如果在这里卡住，用户会看到程序启动后立即卡死，没有任何输出

### 3. **`get_pull_request_title()` 中的 GitHub API 调用**（如果前面的步骤成功）

**原因**：
- 如果前面的步骤都成功，但在这里卡住，用户会看到 "Fetching PR #xxx title..." 的 Spinner 动画一直转
- 但根据用户描述（直接卡死，没有输出），这种情况可能性较小

## 诊断建议

### 1. 检查网络连接

```bash
# 测试 GitHub API 连接
curl -I https://api.github.com

# 测试代理配置（如果使用代理）
curl -I --proxy http://your-proxy:port https://api.github.com
```

### 2. 检查系统代理设置

```bash
# 检查系统代理设置
scutil --proxy

# 如果命令响应慢或卡住，说明系统代理设置可能有问题
```

### 3. 检查环境变量

```bash
# 检查代理环境变量
echo $http_proxy
echo $https_proxy
echo $all_proxy
```

### 4. 添加调试输出

在代码中添加日志，确定卡死的确切位置：

```rust
// 在 ProxyManager::ensure_proxy_enabled() 前后添加日志
log_info!("Before ensure_proxy_enabled");
ProxyManager::ensure_proxy_enabled()?;
log_info!("After ensure_proxy_enabled");

// 在 get_current_branch_pr_id() 前后添加日志
log_info!("Before get_current_branch_pr_id");
let pr_id = get_current_branch_pr_id()?;
log_info!("After get_current_branch_pr_id");
```

## 解决方案建议（暂不实现）

### 1. 为 `scutil` 命令添加超时

```rust
// 使用 timeout 命令包装 scutil
let output = cmd("timeout", &["5", "scutil", "--proxy"])
    .read()
    .context("Failed to get system proxy settings")?;
```

### 2. 为 HTTP 请求添加连接超时

```rust
// 在创建 reqwest Client 时添加连接超时
let client = Client::builder()
    .timeout(Duration::from_secs(30))
    .connect_timeout(Duration::from_secs(10))  // 添加连接超时
    .build()?;
```

### 3. 为 `get_current_branch_pr_id()` 添加超时包装

```rust
// 使用 tokio::time::timeout 或类似机制包装网络请求
let pr_id = timeout(Duration::from_secs(30), get_current_branch_pr_id())?;
```

### 4. 添加重试机制

```rust
// 使用指数退避重试网络请求
let pr_id = retry_with_backoff(|| get_current_branch_pr_id(), max_retries: 3)?;
```

### 5. 添加更详细的错误处理和日志

```rust
// 在关键步骤添加详细的错误信息
match get_current_branch_pr_id() {
    Ok(Some(id)) => id,
    Ok(None) => {
        log_warning!("No PR found for current branch");
        return Ok(None);
    }
    Err(e) => {
        log_error!("Failed to get PR ID: {}", e);
        // 提供更详细的错误信息，帮助用户诊断问题
        return Err(e);
    }
}
```

## 总结

`workflow pr update` 命令卡死的主要原因是：

1. **网络请求阻塞**：GitHub API 调用可能因为网络问题、代理配置错误或 DNS 解析失败而阻塞
2. **系统命令阻塞**：`scutil --proxy` 命令可能因为系统代理设置问题而阻塞
3. **超时机制不完善**：HTTP 客户端的超时主要针对请求阶段，对连接建立阶段的超时支持不够；`duct::cmd` 没有超时设置

**最可能的卡死点**是 `get_current_branch_pr_id()` 中的 GitHub API 调用，因为这是第一个需要网络访问的操作，且可能执行多次 API 调用。

**建议的修复方向**：
1. 为所有网络请求添加连接超时
2. 为系统命令添加超时
3. 添加重试机制和更详细的错误处理
4. 添加调试日志，帮助定位问题
