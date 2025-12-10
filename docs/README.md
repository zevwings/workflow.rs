# Workflow CLI 文档索引

## 📚 文档概览

本文档目录包含 Workflow CLI 的完整架构文档和使用说明。

---

## 🏗️ 核心架构文档

### [ARCHITECTURE.md](./architecture/ARCHITECTURE.md)
**总体架构设计文档**

- 项目概述和模块划分
- 三层架构设计（CLI 入口层、命令封装层、核心业务逻辑层）
- 核心模块设计（AI、日志处理等）
- 数据存储和配置管理
- 开发规范

---

## 📦 模块架构文档

> 所有架构文档位于 [`architecture/`](./architecture/) 目录下

### Lib 层架构文档（核心业务逻辑）

#### [PR_ARCHITECTURE.md](./architecture/lib/PR_ARCHITECTURE.md)
**Pull Request 模块架构文档**

- PR 创建、合并、关闭、查询等操作
- 支持 GitHub 和 Codeup 平台
- 平台抽象设计（PlatformProvider Trait）
- 工厂函数实现多态分发
- LLM 标题生成功能
- LLM PR 总结功能（支持多语言）

#### [JIRA_ARCHITECTURE.md](./architecture/lib/JIRA_ARCHITECTURE.md)
**Jira 模块架构文档**

- 分层架构设计（HTTP 客户端层、API 方法层、业务逻辑层）
- API 子模块（Issue、User、Project）
- 配置管理（ConfigManager）
- 业务功能（用户管理、Ticket 操作、状态管理、工作历史记录）
- 日志处理模块（JiraLogs）

#### [GIT_ARCHITECTURE.md](./architecture/lib/GIT_ARCHITECTURE.md)
**Git 模块架构文档**

- 分支管理操作（创建、切换、合并、删除）
- 提交管理（状态检查、暂存、提交）
- 仓库检测和类型识别
- 暂存管理（stash push/pop）
- Pre-commit hooks 支持
- Git 配置管理

#### [HTTP_ARCHITECTURE.md](./architecture/lib/HTTP_ARCHITECTURE.md)
**HTTP 模块架构文档**

- HTTP 客户端封装（单例模式）
- 请求配置（body、query、auth、headers、timeout）
- 响应处理（延迟解析、JSON/Text 解析）
- 重试机制（指数退避、智能错误判断）
- 认证支持（Basic Authentication）

#### [SETTINGS_ARCHITECTURE.md](./architecture/lib/SETTINGS_ARCHITECTURE.md)
**Settings 模块架构文档**

- TOML 配置文件加载和管理
- 配置结构体定义（Jira、GitHub、日志、LLM、Codeup）
- 路径管理（配置文件路径、安装路径、Shell 路径）
- 默认值管理
- 配置验证功能

#### [LLM_ARCHITECTURE.md](./architecture/lib/LLM_ARCHITECTURE.md)
**LLM 模块架构文档**

- 统一配置驱动的 LLM 客户端实现
- 支持 OpenAI、DeepSeek、Proxy 提供商
- 单例模式和配置驱动设计
- PR 标题和分支名生成功能
- PR 总结文档生成功能（支持多语言，自动生成文件名）

#### [SHELL_ARCHITECTURE.md](./architecture/lib/SHELL_ARCHITECTURE.md)
**Shell 检测与管理模块架构文档**

- Shell 类型检测（zsh、bash、fish、powershell、elvish）
- Shell 配置重新加载
- Shell 配置文件管理（环境变量、source 语句、配置块）
- 多 shell 支持策略
- 与 Completion 和 Proxy 模块的集成

#### [COMPLETION_ARCHITECTURE.md](./architecture/lib/COMPLETION_ARCHITECTURE.md)
**Shell Completion 模块架构文档**

- Completion 脚本生成（workflow 及其所有子命令）
- Completion 安装和卸载
- 多 Shell 支持（zsh、bash、fish、powershell、elvish）
- Shell 配置文件管理集成

#### [PROXY_ARCHITECTURE.md](./architecture/lib/PROXY_ARCHITECTURE.md)
**代理管理模块架构文档**

- 代理开启/关闭/检查功能
- 从 macOS 系统设置读取代理配置
- 环境变量代理配置管理
- 临时模式和持久化模式

#### [ROLLBACK_ARCHITECTURE.md](./architecture/lib/ROLLBACK_ARCHITECTURE.md)
**回滚模块架构文档**

- 更新失败时的备份和恢复机制
- 二进制文件和补全脚本的备份
- 自动回滚功能

#### [TOOLS_ARCHITECTURE.md](./architecture/lib/TOOLS_ARCHITECTURE.md)
**工具函数模块架构文档**

- 日志输出系统（带颜色的日志宏和日志级别管理）
- 字符串处理工具（敏感值隐藏）
- 浏览器和剪贴板操作
- 文件解压和校验和验证
- 用户确认对话框

#### [PROMPT_ARCHITECTURE.md](./architecture/lib/PROMPT_ARCHITECTURE.md)
**Prompt 管理模块架构文档**

- Prompt 文件的加载和管理
- 文件缓存机制（避免重复读取）
- 线程安全的 Prompt 管理
- 扁平化文件结构设计

### 命令层架构文档（CLI 命令封装）

#### [PR_COMMAND_ARCHITECTURE.md](./architecture/commands/PR_COMMAND_ARCHITECTURE.md)
**PR 命令层架构文档**

- PR 创建、合并、关闭、查询等命令
- PR Pick 命令（跨分支移植代码并创建新 PR）
- PR 总结命令（使用 LLM 生成详细总结文档）
- 命令层设计（CLI 入口层、命令封装层）
- 与 lib/pr 模块的集成
- 命令使用示例

#### [LOG_COMMAND_ARCHITECTURE.md](./architecture/commands/LOG_COMMAND_ARCHITECTURE.md)
**日志操作命令层架构文档**

#### [JIRA_COMMAND_ARCHITECTURE.md](./architecture/commands/JIRA_COMMAND_ARCHITECTURE.md)
**Jira 操作命令层架构文档**

- 日志下载、查找、搜索功能（`workflow log` 子命令）
- Jira ticket 信息显示（`workflow jira` 子命令）
- 命令层设计（CLI 入口层、命令封装层）
- 与 Jira 日志处理模块的集成
- 命令使用示例

#### [CONFIG_COMMAND_ARCHITECTURE.md](./architecture/commands/CONFIG_COMMAND_ARCHITECTURE.md)
**配置管理命令层架构文档**

- 初始化设置命令（setup）
- 配置查看命令（config）
- GitHub 账号管理（多账号支持）
- 日志级别管理
- 环境检查功能（Git 仓库状态、网络连接）
- Completion 管理命令

#### [LIFECYCLE_COMMAND_ARCHITECTURE.md](./architecture/commands/LIFECYCLE_COMMAND_ARCHITECTURE.md)
**生命周期管理命令层架构文档**

- 安装功能（二进制文件和 shell completion 脚本）
- 卸载功能（清理所有相关文件和配置）
- 更新功能（从 GitHub Releases 更新到新版本）
- GitHub Actions 发布流程
- HOMEBREW_TAP_TOKEN 配置说明

#### [BRANCH_COMMAND_ARCHITECTURE.md](./architecture/commands/BRANCH_COMMAND_ARCHITECTURE.md)
**分支管理命令层架构文档**

- 分支清理功能（清理已合并分支，保留重要分支）
- 分支忽略列表管理（按仓库配置忽略分支）
- 分支前缀管理（按仓库配置分支前缀，用于自动生成分支名）
- 支持 dry-run 模式和确认机制
- 首次使用自动提示配置分支前缀

#### [CHECK_COMMAND_ARCHITECTURE.md](./architecture/commands/CHECK_COMMAND_ARCHITECTURE.md)
**环境检查命令层架构文档**

- Git 仓库状态检查
- 网络连接检查（到 GitHub）
- 作为其他命令的前置检查步骤

#### [GITHUB_COMMAND_ARCHITECTURE.md](./architecture/commands/GITHUB_COMMAND_ARCHITECTURE.md)
**GitHub 账号管理命令层架构文档**

- 多账号管理（支持配置多个 GitHub 账号）
- 账号切换功能（自动更新 Git 配置）
- 账号配置管理（添加、删除、更新）

#### [PROXY_COMMAND_ARCHITECTURE.md](./architecture/commands/PROXY_COMMAND_ARCHITECTURE.md)
**代理管理命令层架构文档**

- 代理启用/禁用功能（临时模式和持久模式）
- 代理状态检查（系统设置、环境变量、配置文件）
- 自动读取系统代理设置

---

## 🔄 迁移文档

> 迁移文档位于 [`migration/`](./migration/) 目录下

### [迁移文档索引](./migration/README.md)
**版本迁移指南**

- 各版本之间的配置迁移说明
- 迁移步骤和注意事项
- 迁移前后配置对比
- 回滚说明

### [1.4.8 → 1.4.9 迁移指南](./migration/1.4.8-to-1.4.9.md)
**从 1.4.8 升级到 1.4.9 的迁移说明**

- 配置迁移系统（v1.0.0）迁移说明
- 从 `branch.toml` 迁移到 `repositories.toml`
- 详细的迁移步骤和验证方法

---

## 📋 设计文档和指南

> 设计文档和指南位于 [`guidelines/`](./guidelines/) 目录下

### [DEVELOPMENT_GUIDELINES.md](./guidelines/DEVELOPMENT_GUIDELINES.md)
**开发规范文档**

- 代码风格规范（格式化、Lint、命名约定）
- 错误处理规范（anyhow、上下文信息）
- 文档规范（公共 API 文档、注释格式）
- 命名规范（文件、函数、结构体、常量）
- 模块组织规范（目录结构、模块职责）
- Git 工作流（分支策略、工作流程）
- 提交规范（Conventional Commits）
- 测试规范（单元测试、集成测试）
- 代码审查（审查清单、审查重点）
- 依赖管理（添加依赖、版本管理）
- 开发工具（必需工具、常用命令）

### [DOCUMENT_GUIDELINES.md](./guidelines/DOCUMENT_GUIDELINES.md)
**文档编写指南**

- 文档模板使用说明
- 章节检查清单
- 文档编写规范

### [WHY_BOTH_ZSH_BASH.md](./WHY_BOTH_ZSH_BASH.md)
**为什么需要同时生成 zsh 和 bash 的补全脚本**

- 配置文件设计说明
- 多 shell 环境支持场景
- 设计决策和实现方案

### [FUTURE_IMPROVEMENTS.md](./FUTURE_IMPROVEMENTS.md)
**未来改进计划**

- 未实现功能列表
- 改进建议和优先级
- 贡献指南

### [FEATURE_EXTENSIONS.md](./FEATURE_EXTENSIONS.md)
**功能拓展分析文档**

- 基于代码库分析的功能拓展方向
- 10 个主要功能模块的详细拓展建议
- 每个功能的命令示例和实现建议
- 优先级建议和开发顺序
- 技术考虑和贡献指南

---

## 📝 文档结构说明

所有模块架构文档遵循统一的结构：

1. **📋 概述** - 模块功能和定位
2. **📁 相关文件** - 文件组织结构
3. **🔄 调用流程** - 整体架构流程
4. **功能说明** - 各命令的详细说明
5. **📊 数据流** - 数据流向图
6. **🔗 与其他模块的集成** - 模块间关系
7. **🎯 设计模式** - 使用的设计模式
8. **🔍 错误处理** - 错误处理机制
9. **📝 扩展性** - 如何扩展功能
10. **📚 相关文档** - 相关文档链接

---

## 🔄 文档更新记录

- **2024-01** - 移除 QuickCommand 包装器，统一为直接调用方式
- **2024-01** - 创建 PR helpers 模块，提取重复逻辑
- **2024-01** - 统一文档结构和命名规范
- **2024-01** - 统一 LLM 架构文档，移除旧的实现文档（`LLM_IMPLEMENTATION.md`、`LLM_PLUGIN_ARCHITECTURE.md`、`LLM_PLUGIN_CURL.md`），保留统一的架构设计文档 `LLM_ARCHITECTURE.md`
- **2024-01** - 添加 GitHub Actions 发布流程文档，包括 HOMEBREW_TAP_TOKEN 配置说明和自动验证机制
- **2024-01** - 合并未实现功能到 `FUTURE_IMPROVEMENTS.md`，删除已实现的改进文档（`UPDATE_IMPROVEMENTS.md`、`COMPLETION_FILE_NAMING_ANALYSIS.md`、`INSTALL_COMPLETION_IMPROVEMENT.md`）
- **2024-11** - 整合 Jira 模块架构文档，合并 `JIRA_LOGS_ARCHITECTURE.md` 和 `JIRA_MODULE_REFACTOR_ANALYSIS.md` 为统一的 `JIRA_ARCHITECTURE.md`
- **2024-11** - 重构 PR 模块架构文档，删除分析文档（`PR_HTTP_CLIENT_ANALYSIS.md`、`PR_REFACTOR_STATUS.md`、`PR_REFACTOR_GUIDE.md`），统一为 `PR_ARCHITECTURE.md`，参考 JIRA 和 GIT 架构文档结构
- **2024-11** - 重构 LLM 模块架构文档，参考 JIRA、PR、GIT 架构文档结构，统一文档格式
- **2024-12** - 合并环境检查模块文档到配置管理模块文档，`CHECK_ARCHITECTURE.md` 已合并到 `CONFIG_ARCHITECTURE.md`
- **2024-12** - 更新生命周期管理模块文档，将 `INSTALL_ARCHITECTURE.md` 扩展为包含安装、卸载、更新三个功能的完整文档，并重命名为 `LIFECYCLE_COMMAND_ARCHITECTURE.md`
- **2024-12** - 创建 Shell 检测与管理模块架构文档 `SHELL_ARCHITECTURE.md`
- **2024-12** - 创建 Settings 模块架构文档 `SETTINGS_ARCHITECTURE.md`，删除重复的 `CONFIG_ARCHITECTURE.md`
- **2024-12** - 更新所有 lib 层架构文档，移除命令层详细内容，确保只描述 lib/ 模块
- **2024-12** - 重组文档结构，分为 ARCHITECTURE 和 GUIDELINES 两个目录
- **2024-12** - 为所有 log 和 jira 命令添加交互式输入支持（JIRA_ID 参数可选，包括 info、attachments、download、find、search）
- **2024-12** - 实现同时搜索 api.log 和 flutter-api.log 功能，更新搜索命令以支持两个文件
- **2024-12** - 删除废弃的 QK_COMMAND_ARCHITECTURE.md 文档（已拆分为 LOG_COMMAND_ARCHITECTURE.md 和 JIRA_COMMAND_ARCHITECTURE.md）
- **2024-12** - 创建开发规范文档 `DEVELOPMENT_GUIDELINES.md`，包含代码风格、错误处理、文档、命名、模块组织、Git 工作流、提交、测试、代码审查、依赖管理等规范
- **2024-12** - 创建功能拓展分析文档 `FEATURE_EXTENSIONS.md`，基于代码库分析提出 10 个主要功能模块的拓展建议，包含详细的功能说明、命令示例、实现建议和优先级建议
- **2024-12** - 创建迁移文档目录 `migration/`，包含迁移文档索引和 1.4.8 → 1.4.9 迁移指南

---

## 📖 快速导航

### 整体架构
- 想了解整体架构？ → [ARCHITECTURE.md](./architecture/ARCHITECTURE.md)

### 版本迁移
- 需要升级版本？ → [迁移文档索引](./migration/README.md)
- 从 1.4.8 升级到 1.4.9？ → [1.4.8 → 1.4.9 迁移指南](./migration/1.4.8-to-1.4.9.md)

### Lib 层模块（核心业务逻辑）
- 想了解 PR 功能？ → [PR_ARCHITECTURE.md](./architecture/lib/PR_ARCHITECTURE.md)
- 想了解 Jira 集成？ → [JIRA_ARCHITECTURE.md](./architecture/lib/JIRA_ARCHITECTURE.md)
- 想了解 Git 操作？ → [GIT_ARCHITECTURE.md](./architecture/lib/GIT_ARCHITECTURE.md)
- 想了解 HTTP 客户端？ → [HTTP_ARCHITECTURE.md](./architecture/lib/HTTP_ARCHITECTURE.md)
- 想了解配置管理？ → [SETTINGS_ARCHITECTURE.md](./architecture/lib/SETTINGS_ARCHITECTURE.md)
- 想了解 LLM/AI 功能？ → [LLM_ARCHITECTURE.md](./architecture/lib/LLM_ARCHITECTURE.md)
- 想了解 Shell 检测与管理？ → [SHELL_ARCHITECTURE.md](./architecture/lib/SHELL_ARCHITECTURE.md)
- 想了解 Completion 功能？ → [COMPLETION_ARCHITECTURE.md](./architecture/lib/COMPLETION_ARCHITECTURE.md)
- 想了解代理管理？ → [PROXY_ARCHITECTURE.md](./architecture/lib/PROXY_ARCHITECTURE.md)
- 想了解回滚机制？ → [ROLLBACK_ARCHITECTURE.md](./architecture/lib/ROLLBACK_ARCHITECTURE.md)
- 想了解工具函数？ → [TOOLS_ARCHITECTURE.md](./architecture/lib/TOOLS_ARCHITECTURE.md)
- 想了解 Prompt 管理？ → [PROMPT_ARCHITECTURE.md](./architecture/lib/PROMPT_ARCHITECTURE.md)

### 命令层模块（CLI 命令封装）
- 想了解 PR 命令？ → [PR_COMMAND_ARCHITECTURE.md](./architecture/commands/PR_COMMAND_ARCHITECTURE.md)
- 想了解日志操作命令？ → [LOG_COMMAND_ARCHITECTURE.md](./architecture/commands/LOG_COMMAND_ARCHITECTURE.md)
- 想了解 Jira 操作命令？ → [JIRA_COMMAND_ARCHITECTURE.md](./architecture/commands/JIRA_COMMAND_ARCHITECTURE.md)
- 想了解配置管理命令？ → [CONFIG_COMMAND_ARCHITECTURE.md](./architecture/commands/CONFIG_COMMAND_ARCHITECTURE.md)
- 想了解生命周期管理命令（安装/卸载/更新）？ → [LIFECYCLE_COMMAND_ARCHITECTURE.md](./architecture/commands/LIFECYCLE_COMMAND_ARCHITECTURE.md)
- 想了解分支管理命令？ → [BRANCH_COMMAND_ARCHITECTURE.md](./architecture/commands/BRANCH_COMMAND_ARCHITECTURE.md)
- 想了解环境检查命令？ → [CHECK_COMMAND_ARCHITECTURE.md](./architecture/commands/CHECK_COMMAND_ARCHITECTURE.md)
- 想了解 GitHub 账号管理命令？ → [GITHUB_COMMAND_ARCHITECTURE.md](./architecture/commands/GITHUB_COMMAND_ARCHITECTURE.md)
- 想了解代理管理命令？ → [PROXY_COMMAND_ARCHITECTURE.md](./architecture/commands/PROXY_COMMAND_ARCHITECTURE.md)
