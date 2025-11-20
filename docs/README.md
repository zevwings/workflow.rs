# Workflow CLI 文档索引

## 📚 文档概览

本文档目录包含 Workflow CLI 的完整架构文档和使用说明。

---

## 🏗️ 核心架构文档

### [ARCHITECTURE.md](./ARCHITECTURE.md)
**总体架构设计文档**

- 项目概述和模块划分
- 三层架构设计（CLI 入口层、命令封装层、核心业务逻辑层）
- 核心模块设计（AI、日志处理等）
- 数据存储和配置管理
- 开发规范

---

## 📦 模块架构文档

### [PR_ARCHITECTURE.md](./PR_ARCHITECTURE.md)
**Pull Request 模块架构文档**

- PR 创建、合并、关闭、查询等操作
- 支持 GitHub 和 Codeup 平台
- 平台抽象设计（PlatformProvider Trait）
- 工厂函数实现多态分发
- Jira 状态管理集成
- LLM 标题生成功能

### [QK_ARCHITECTURE.md](./QK_ARCHITECTURE.md)
**快速日志操作模块架构文档**

- 日志下载、查找、搜索功能
- 三层架构设计
- 与 Jira 和 Streamock 的集成
- 命令使用示例

### [JIRA_ARCHITECTURE.md](./JIRA_ARCHITECTURE.md)
**Jira 模块架构文档**

- 分层架构设计（HTTP 客户端层、API 方法层、业务逻辑层）
- API 子模块（Issue、User、Project）
- 配置管理（ConfigManager）
- 业务功能（用户管理、Ticket 操作、状态管理、工作历史记录）
- 日志处理模块（JiraLogs）

### [INSTALL_ARCHITECTURE.md](./INSTALL_ARCHITECTURE.md)
**安装/卸载模块架构文档**

- Shell completion 脚本生成和安装
- 二进制文件安装
- 卸载功能实现
- GitHub Actions 发布流程
- HOMEBREW_TAP_TOKEN 配置说明

### [CONFIG_ARCHITECTURE.md](./CONFIG_ARCHITECTURE.md)
**配置管理模块架构文档**

- 初始化设置命令（setup）
- 配置查看命令（config）
- 环境变量管理

### [PROXY_ARCHITECTURE.md](./PROXY_ARCHITECTURE.md)
**代理管理模块架构文档**

- 代理开启/关闭/检查功能
- 环境变量代理配置

### [CHECK_ARCHITECTURE.md](./CHECK_ARCHITECTURE.md)
**环境检查模块架构文档**

- Git 仓库状态检查
- 网络连接检查

### [LLM_ARCHITECTURE.md](./LLM_ARCHITECTURE.md)
**LLM 模块架构文档**

- 统一配置驱动的 LLM 客户端实现
- 支持 OpenAI、DeepSeek、Proxy 提供商
- 单例模式和配置驱动设计
- PR 标题和分支名生成功能

---

## 📋 设计文档

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

---

## 📖 快速导航

- 想了解整体架构？ → [ARCHITECTURE.md](./ARCHITECTURE.md)
- 想了解 PR 功能？ → [PR_ARCHITECTURE.md](./PR_ARCHITECTURE.md)
- 想了解日志操作？ → [QK_ARCHITECTURE.md](./QK_ARCHITECTURE.md)
- 想了解 Jira 集成？ → [JIRA_ARCHITECTURE.md](./JIRA_ARCHITECTURE.md)
- 想了解 LLM/AI 功能？ → [LLM_ARCHITECTURE.md](./LLM_ARCHITECTURE.md)
- 想了解配置管理？ → [CONFIG_ARCHITECTURE.md](./CONFIG_ARCHITECTURE.md)
- 想了解安装配置？ → [INSTALL_ARCHITECTURE.md](./INSTALL_ARCHITECTURE.md)
- 想了解代理管理？ → [PROXY_ARCHITECTURE.md](./PROXY_ARCHITECTURE.md)
- 想了解环境检查？ → [CHECK_ARCHITECTURE.md](./CHECK_ARCHITECTURE.md)

