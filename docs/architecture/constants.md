# Constants 常量模块架构文档

## 📋 概述

Constants 模块是 Workflow CLI 的基础设施模块之一，统一管理项目中使用的字符串常量、错误消息、API URL 等，提升代码一致性、维护性和可读性。该模块采用模块化设计，按功能领域分为多个子模块。

**模块统计：**
- 总代码行数：约 200+ 行
- 文件数量：5 个（`mod.rs`、`errors.rs`、`git.rs`、`messages.rs`、`network.rs`、`validation.rs`）
- 常量类别：4 类（错误消息、Git 常量、网络常量、验证常量、用户消息）

---

## 📁 Lib 层架构（核心业务逻辑）

### 核心模块文件

```
src/lib/base/constants/
├── mod.rs          # 模块导出和公共 API (16行)
├── errors.rs       # 通用错误消息常量 (101行)
├── git.rs          # Git 操作相关常量 (16行)
├── messages.rs     # 消息常量 (58行)
├── network.rs      # 网络相关常量 (16行)
└── validation.rs   # 验证相关常量 (43行)
```

### 依赖模块

- **无外部依赖**：所有常量都是编译时常量，不依赖外部库

### 模块集成

Constants 模块被整个项目广泛使用：

- **错误处理**：
  - 所有模块使用 `constants::errors` 中的错误消息常量
  - 提供统一的错误消息格式

- **Git 操作**：
  - `Git` 模块使用 `constants::git` 中的 Git 相关常量
  - `Check` 命令使用 Git 检查错误消息

- **网络操作**：
  - `HTTP` 模块使用 `constants::network` 中的网络错误消息
  - 提供统一的网络错误处理

- **验证**：
  - `Branch` 模块使用 `constants::validation::branch` 中的分支验证消息
  - `Config` 模块使用 `constants::validation::config` 中的配置验证消息

- **用户交互**：
  - 所有命令使用 `constants::messages` 中的用户消息常量
  - 提供统一的用户体验

---

## 🏗️ 架构设计

### 设计原则

1. **统一管理**：所有常量集中管理，避免重复定义
2. **分类组织**：按功能领域分类，便于查找和维护
3. **命名规范**：使用清晰的命名，提升可读性
4. **编译时常量**：所有常量都是编译时常量，无运行时开销

### 核心组件

#### 1. errors 模块（通用错误消息）

**位置**：`errors.rs`

**职责**：统一管理项目中使用的错误消息

**子模块**：

##### `file_operations` - 文件操作错误消息

- `CREATE_DIR_FAILED` - 创建目录失败
- `CREATE_TEMP_DIR_FAILED` - 创建临时目录失败
- `CREATE_PARENT_DIR_FAILED` - 创建父目录失败
- `CREATE_CONFIG_DIR_FAILED` - 创建配置目录失败
- `READ_FILE_FAILED` - 读取文件失败
- `READ_CONFIG_FAILED` - 读取配置文件失败
- `WRITE_FILE_FAILED` - 写入文件失败
- `WRITE_CONFIG_FAILED` - 写入配置失败
- 等等...

##### `http_client` - HTTP 客户端错误消息

- `CREATE_CLIENT_FAILED` - 创建 HTTP 客户端失败

##### `input_reading` - 输入读取错误消息

- `READ_JIRA_TICKET_ID_FAILED` - 读取 Jira 票据 ID 失败
- `READ_BRANCH_NAME_FAILED` - 读取分支名称失败

##### `generator_creation` - 生成器创建错误消息

- `CREATE_GENERATOR_FAILED_FORMAT` - 创建生成器失败（带格式化参数）
- `CREATE_ZSH_GENERATOR_FAILED` - 创建 zsh 生成器失败

##### `validation_errors` - 验证错误消息

- `INVALID_PR_NUMBER` - 无效的 PR 编号
- `INVALID_REPO_FORMAT` - 无效的仓库格式
- `INVALID_JIRA_ID_FORMAT` - 无效的 JIRA ID 格式
- `JIRA_ID_FORMAT_HELP` - JIRA ID 格式说明
- `JIRA_ID_EMPTY` - JIRA ID 不能为空
- `JIRA_ID_VALIDATION_ERROR_TEMPLATE` - JIRA ID 格式验证失败的完整消息模板

#### 2. git 模块（Git 操作相关常量）

**位置**：`git.rs`

**职责**：统一管理 Git 相关的错误消息和检查信息

**子模块**：

##### `check_errors` - Git 检查错误消息

- `NOT_GIT_REPO` - 不在 Git 仓库中
- `NETWORK_CHECK_FAILED` - 网络检查失败
- `LINT_CHECK_FAILED` - Lint 检查失败

#### 3. messages 模块（消息常量）

**位置**：`messages.rs`

**职责**：统一管理用户交互消息、日志消息等跨模块使用的消息常量

**子模块**：

##### `pull_requests` - GitHub PR 相关常量

- `APPROVE_EVENT` - PR 批准事件
- `REQUEST_CHANGES_EVENT` - PR 请求修改事件
- `COMMENT_EVENT` - PR 评论事件
- `APPROVE_EMOJI` - PR 批准 emoji

##### `user` - 用户交互消息

- `OPERATION_CANCELLED` - 操作已取消
- `NOT_EXISTS` - 不存在
- `NOT_SET` - 未设置
- `DOWNLOAD_COMPLETE` - 下载完成
- `UPDATE_CANCELLED` - 更新已取消
- `INSTALLATION_FAILED` - 安装失败
- `UPDATE_FAILED` - 更新失败
- `ROLLBACK_COMPLETED` - 回滚完成

##### `log` - 日志消息

- `BRANCH_RENAME` - 分支重命名
- `TESTS_FAILED` - 测试失败
- `CONFIG_SAVED_PREFIX` - 配置保存消息前缀

#### 4. network 模块（网络相关常量）

**位置**：`network.rs`

**职责**：统一管理网络操作相关的错误消息和配置

**子模块**：

##### `errors` - 网络错误消息

- `TIMEOUT` - 网络超时
- `CONNECTION_FAILED` - 连接失败
- `RATE_LIMIT_EXCEEDED` - 速率限制超出

#### 5. validation 模块（验证相关常量）

**位置**：`validation.rs`

**职责**：统一管理各种验证场景的错误消息和规则

**子模块**：

##### `branch` - 分支名称验证错误消息

- `EMPTY_NAME` - 分支名称不能为空
- `INVALID_DOT_POSITION` - 分支名称不能以 '.' 开头或结尾
- `DOUBLE_DOT` - 分支名称不能包含 '..'
- `CONTAINS_SPACES` - 分支名称不能包含空格
- `INVALID_SPECIAL_CHAR` - 分支名称不能包含特殊字符
- `TRAILING_SLASH` - 分支名称不能以 '/' 结尾
- `DOUBLE_SLASH` - 分支名称不能包含连续的斜杠 '//'
- `RESERVED_NAME` - 分支名称不能是保留名称

##### `config` - 配置验证消息

- `VALIDATION_FAILED` - 配置验证失败
- `HEADER` - 配置标题
- `UNSUPPORTED_SHELL` - 不支持的 shell 类型

---

## 🔄 调用流程与数据流

### 典型使用流程

```
模块需要错误消息
  ↓
导入 constants 模块
  ↓
使用相应的常量
  ├─ errors::file_operations::READ_FILE_FAILED
  ├─ git::check_errors::NOT_GIT_REPO
  ├─ network::errors::TIMEOUT
  ├─ validation::branch::EMPTY_NAME
  └─ messages::user::OPERATION_CANCELLED
```

---

## 📋 使用示例

### 错误消息使用

```rust
use workflow::base::constants::errors::file_operations;
use workflow::base::constants::git::check_errors;
use workflow::base::constants::network::errors;

// 文件操作错误
if file_read_failed {
    return Err(eyre!(file_operations::READ_FILE_FAILED));
}

// Git 检查错误
if !is_git_repo {
    return Err(eyre!(check_errors::NOT_GIT_REPO));
}

// 网络错误
if network_timeout {
    return Err(eyre!(errors::TIMEOUT));
}
```

### 验证消息使用

```rust
use workflow::base::constants::validation::branch;
use workflow::base::constants::validation::config;

// 分支名称验证
if branch_name.is_empty() {
    return Err(eyre!(branch::EMPTY_NAME));
}

if branch_name.contains("..") {
    return Err(eyre!(branch::DOUBLE_DOT));
}

// 配置验证
if !is_valid_config {
    return Err(eyre!(config::VALIDATION_FAILED));
}
```

### 用户消息使用

```rust
use workflow::base::constants::messages::user;
use workflow::base::constants::messages::pull_requests;

// 用户交互消息
if user_cancelled {
    log_message!("{}", user::OPERATION_CANCELLED);
}

if download_complete {
    log_success!("{}", user::DOWNLOAD_COMPLETE);
}

// PR 相关消息
if pr_approved {
    log_success!("PR {} {}", pull_requests::APPROVE_EVENT, pull_requests::APPROVE_EMOJI);
}
```

---

## 🔍 错误处理

### 常量使用规范

1. **统一使用**：所有错误消息应使用 constants 模块中的常量
2. **避免硬编码**：不要在代码中硬编码错误消息字符串
3. **分类使用**：根据错误类型使用相应的常量模块

### 添加新常量

1. 确定常量所属的分类（errors、git、messages、network、validation）
2. 在相应的模块文件中添加常量
3. 使用清晰的命名（全大写，下划线分隔）
4. 添加注释说明常量的用途

---

## 📝 扩展性

### 添加新的常量分类

1. 创建新的模块文件（如 `api.rs`）
2. 在 `mod.rs` 中声明模块并重新导出
3. 添加相应的常量定义

### 添加新的常量

1. 确定常量所属的分类和子模块
2. 在相应的模块文件中添加常量
3. 使用清晰的命名和注释

---

## 📚 相关文档

- [主架构文档](./architecture.md)
- [错误处理规范](../guidelines/development/error-handling.md) - 错误处理规范
- [Git 模块架构文档](./git.md) - 使用 Git 常量
- [HTTP 模块架构文档](./http.md) - 使用网络常量

---

## ✅ 总结

Constants 模块采用清晰的分类组织设计：

1. **统一管理**：所有常量集中管理，避免重复定义
2. **分类组织**：按功能领域分类，便于查找和维护
3. **命名规范**：使用清晰的命名，提升可读性
4. **编译时常量**：所有常量都是编译时常量，无运行时开销

**设计优势**：
- ✅ 统一管理，避免重复定义
- ✅ 分类组织，便于查找和维护
- ✅ 命名规范，提升可读性
- ✅ 编译时常量，无运行时开销

**当前实现状态**：
- ✅ 错误消息常量完整实现
- ✅ Git 常量完整实现
- ✅ 网络常量完整实现
- ✅ 验证常量完整实现
- ✅ 用户消息常量完整实现
- ✅ 已在整个项目中广泛使用

---

**最后更新**: 2025-12-27

