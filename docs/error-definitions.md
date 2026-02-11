# Domain 错误类型定义

> 按服务/接口划分，各模块使用独立错误类型，替代已删除的 `ServiceError`。

---

## 1. AliasError

**模块**: `domain::alias`  
**文件**: `alias/error.rs`

| 变体 | 说明 |
|------|------|
| `InvalidInput(String)` | 参数校验失败（空名、非法字符、别名不存在等） |
| `CircularReference(String)` | 循环引用 |
| `MaxDepthExceeded` | 展开深度超限 |
| `Config(String)` | 配置操作失败（加载/保存） |

---

## 2. CommitMessageError

**模块**: `domain::commit`  
**文件**: `commit/error.rs`

| 变体 | 说明 |
|------|------|
| `LLMError(String)` | LLM 调用失败 |
| `ParseFailed(String)` | 解析失败 |
| `Git(GitError)` | Git 操作失败 |

---

## 3. CompletionError

**模块**: `domain::completion`  
**文件**: `completion/error.rs`

| 变体 | 说明 |
|------|------|
| `InvalidInput(String)` | 不支持的 shell 类型 |
| `Shell(ShellError)` | Shell 检测失败 |
| `Path(PathError)` | 路径相关错误 |

---

## 4. CommitSummaryError

**模块**: `domain::summary`  
**文件**: `summary/error.rs`

| 变体 | 说明 |
|------|------|
| `LLMError(String)` | LLM 调用失败 |
| `ParseFailed(String)` | 解析失败 |
| `SerializeFailed(String)` | 序列化失败 |
| `NoChangesToAnalyze` | 无变更可分析 |
| `Git(GitError)` | Git 操作失败 |

---

## 5. PullRequestError

**模块**: `domain::pr`  
**文件**: `pr/error.rs`

| 变体 | 说明 |
|------|------|
| `Git(String)` | Git 操作失败 |
| `GitHub(GitHubError)` | GitHub API 失败 |
| `NotFound(String)` | PR 不存在 |
| `InvalidInput(String)` | 无效参数 |
| `UnsupportedOperation(String)` | 不支持的操作（如非 GitHub 平台） |

---

## 6. ConfigError

**模块**: `domain::config`  
**文件**: `config/error.rs`  
**适用**: `RepoConfigRepository`、`GlobalConfigRepository`、`VerificationService`

| 变体 | 说明 |
|------|------|
| `Io(#[from] std::io::Error)` | I/O 错误 |
| `Toml(#[from] toml::de::Error)` | TOML 解析错误 |
| `Path(PathError)` | 路径相关错误 |
| `LockFailed(String)` | 获取锁失败 |
| `OperationFailed(String)` | 操作失败 |

---

## 7. 已有错误（保持不变）

| 模块 | 错误类型 | 文件 |
|------|----------|------|
| branch | `BranchServiceError` | `branch/service.rs` |
| git | `GitError` | `git/error.rs` |
| path | `PathError` | `path/error.rs` |
| github | `GitHubError` | `github/error.rs` |
| jira | `JiraError` | `jira/error.rs` |
