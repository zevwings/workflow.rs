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

#### [BRANCH_ARCHITECTURE.md](./architecture/lib/BRANCH_ARCHITECTURE.md)
**Branch 模块架构文档**

- 分支命名服务（从 JIRA ticket、标题、类型生成）
- 分支前缀管理（JIRA ticket 前缀、仓库前缀）
- 分支配置管理（仓库级别前缀、忽略列表）
- 分支类型定义（feature/bugfix/refactoring/hotfix/chore）
- 分支名生成（模板系统、LLM、简单回退）
- 非英文翻译功能

#### [TAG_ARCHITECTURE.md](./architecture/lib/TAG_ARCHITECTURE.md)
**Tag 模块架构文档**

- Tag 列表操作（本地、远程、全部）
- Tag 信息获取（名称、commit hash、存在位置）
- Tag 删除操作（本地、远程、同时删除）
- Tag 存在性检查
- 类型定义（TagInfo 结构体）

#### [COMMIT_ARCHITECTURE.md](./architecture/lib/COMMIT_ARCHITECTURE.md)
**Commit 模块架构文档**

- Commit amend 业务逻辑（预览、格式化、完成提示）
- Commit reword 业务逻辑（预览、格式化、完成提示）
- 历史 commit reword（rebase 交互式编辑）
- 预览信息生成和格式化显示
- 无状态设计，职责单一

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

- 分支创建功能（支持从 JIRA ticket 创建，使用 LLM 生成分支名）
- 分支清理功能（清理已合并分支，保留重要分支）
- 分支忽略列表管理（按仓库配置忽略分支）
- 分支前缀管理（按仓库配置分支前缀，用于自动生成分支名）
- 支持 dry-run 模式和确认机制
- 首次使用自动提示配置分支前缀

#### [TAG_COMMAND_ARCHITECTURE.md](./architecture/commands/TAG_COMMAND_ARCHITECTURE.md)
**Tag 管理命令层架构文档**

- Tag 删除功能（支持本地和远程 tag 删除）
- 模式匹配批量删除（支持 shell 通配符）
- 交互式选择多个 tag
- 预览和确认机制
- 支持 dry-run 模式和强制删除

#### [COMMIT_COMMAND_ARCHITECTURE.md](./architecture/commands/COMMIT_COMMAND_ARCHITECTURE.md)
**Commit 管理命令层架构文档**

- Commit amend 命令（修改最后一次提交的消息和文件）
- Commit reword 命令（修改指定提交的消息，不改变内容）
- Commit squash 命令（压缩多个 commits）
- 交互式工作流（选择、输入、确认等）
- 预览机制（在执行操作前生成预览信息）
- 分支保护（检查默认分支，防止误操作）
- 支持 HEAD 和历史 commit 的 reword

#### [STASH_COMMAND_ARCHITECTURE.md](./architecture/commands/STASH_COMMAND_ARCHITECTURE.md)
**Stash 管理命令层架构文档**

- Stash list 命令（列出所有 stash 条目，支持统计信息）
- Stash apply 命令（应用 stash，保留条目）
- Stash drop 命令（删除 stash 条目，支持多选）
- Stash pop 命令（应用并删除 stash 条目）
- 交互式选择界面（支持选择特定的 stash）
- 冲突检测和处理（自动检测冲突并提供解决提示）

#### [ALIAS_COMMAND_ARCHITECTURE.md](./architecture/commands/ALIAS_COMMAND_ARCHITECTURE.md)
**别名管理命令层架构文档**

- Alias list 命令（列出所有别名）
- Alias add 命令（添加别名，支持直接模式和交互式模式）
- Alias remove 命令（删除别名，支持直接模式和交互式多选模式）
- 别名展开功能（支持嵌套别名和循环检测）
- 命令行参数展开（在命令解析前自动展开别名）

#### [MIGRATE_COMMAND_ARCHITECTURE.md](./architecture/commands/MIGRATE_COMMAND_ARCHITECTURE.md)
**配置迁移命令层架构文档**

- 版本化迁移系统（迁移版本独立于软件版本）
- 自动检测待迁移版本
- 支持 dry-run 模式和保留旧配置文件选项
- 迁移历史记录管理
- 每个迁移版本独立实现，互不干扰

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

**注意**：分支配置已迁移到项目级配置（`.workflow/config.toml`）。请使用 `workflow repo setup` 设置项目级配置。

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

### [PRE_COMMIT_GUIDELINES.md](./guidelines/PRE_COMMIT_GUIDELINES.md)
**提交前检查指南**

- 文档检查清单
- CLI 和 Completion 检查
- 代码优化检查（包含如何提取共用代码的具体示例）
- 测试用例检查
- 代码质量检查（make lint/fix）
- 其他检查项（版本管理、Git、依赖、平台兼容性等）
- 快速检查清单和常见问题

### [REVIEW_DOCUMENT_GUIDELINES.md](./guidelines/reviews/REVIEW_DOCUMENT_GUIDELINES.md)
**文档检查指南**

- 检查概述和检查步骤
- README.md 检查（基本结构、命令清单、配置说明、文档链接、版本号、架构总览）
- docs/ 目录检查（目录结构、架构文档、指南文档、迁移文档、文档索引）
- CHANGELOG.md 检查（格式、内容完整性、版本一致性、变更分类）
- 重复内容检查（跨文档重复、文档内部重复、链接和引用）
- 文档位置检查（文档分类、文档命名、临时文档）
- 文档优化和补全检查（内容完整性、准确性、格式规范性、可读性、链接有效性、文档更新）
- 检查报告生成和快速检查清单

### [REVIEW_TEST_CASE_GUIDELINES.md](./guidelines/reviews/REVIEW_TEST_CASE_GUIDELINES.md)
**测试用例检查指南**

- 检查目标（测试覆盖、合理性、缺失测试）
- 检查步骤（收集信息、覆盖检查、合理性检查、缺失识别、生成报告）
- 检查内容（模块覆盖、功能覆盖、测试工具使用、测试结构、测试内容）
- 检查方法（自动化工具、手动检查）
- 检查报告格式

### [CARGO_BLOAT_GUIDELINES.md](./guidelines/CARGO_BLOAT_GUIDELINES.md)
**cargo-bloat 使用指南**

- cargo-bloat 工具安装和基本使用
- 常用命令和参数说明
- 输出结果解读
- 二进制大小优化最佳实践
- 常见问题解答
- 相关工具和资源

### [REVIEW_CODE_GUIDELINES.md](./guidelines/reviews/REVIEW_CODE_GUIDELINES.md)
**代码检查指南**

- 系统化的代码检查方法
- 重复代码识别和抽取方法
- 已封装工具函数的使用检查
- 第三方库替换机会识别
- 详细的检查清单和示例分析
- 适用于 AI 辅助代码审查和人工代码审查

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
- 想了解 Tag 管理？ → [TAG_ARCHITECTURE.md](./architecture/lib/TAG_ARCHITECTURE.md)
- 想了解 Commit 管理？ → [COMMIT_ARCHITECTURE.md](./architecture/lib/COMMIT_ARCHITECTURE.md)
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
- 想了解 Tag 管理命令？ → [TAG_COMMAND_ARCHITECTURE.md](./architecture/commands/TAG_COMMAND_ARCHITECTURE.md)
- 想了解 Repo 管理命令？ → [REPO_COMMAND_ARCHITECTURE.md](./architecture/commands/REPO_COMMAND_ARCHITECTURE.md)
- 想了解 Commit 管理命令？ → [COMMIT_COMMAND_ARCHITECTURE.md](./architecture/commands/COMMIT_COMMAND_ARCHITECTURE.md)
- 想了解 Stash 管理命令？ → [STASH_COMMAND_ARCHITECTURE.md](./architecture/commands/STASH_COMMAND_ARCHITECTURE.md)
- 想了解别名管理命令？ → [ALIAS_COMMAND_ARCHITECTURE.md](./architecture/commands/ALIAS_COMMAND_ARCHITECTURE.md)
- 想了解配置迁移命令？ → [MIGRATE_COMMAND_ARCHITECTURE.md](./architecture/commands/MIGRATE_COMMAND_ARCHITECTURE.md)
- 想了解环境检查命令？ → [CHECK_COMMAND_ARCHITECTURE.md](./architecture/commands/CHECK_COMMAND_ARCHITECTURE.md)
- 想了解 GitHub 账号管理命令？ → [GITHUB_COMMAND_ARCHITECTURE.md](./architecture/commands/GITHUB_COMMAND_ARCHITECTURE.md)
- 想了解代理管理命令？ → [PROXY_COMMAND_ARCHITECTURE.md](./architecture/commands/PROXY_COMMAND_ARCHITECTURE.md)

---

**最后更新**: 2025-12-16
