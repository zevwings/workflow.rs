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
- Jira 状态管理集成
- 平台抽象设计（PlatformProvider Trait）

### [QK_ARCHITECTURE.md](./QK_ARCHITECTURE.md)
**快速日志操作模块架构文档**

- 日志下载、查找、搜索功能
- 三层架构设计
- 与 Jira 和 Streamock 的集成
- 命令使用示例

### [INSTALL_ARCHITECTURE.md](./INSTALL_ARCHITECTURE.md)
**安装/卸载模块架构文档**

- Shell completion 脚本生成和安装
- 二进制文件安装
- 卸载功能实现

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

---

## 📖 快速导航

- 想了解整体架构？ → [ARCHITECTURE.md](./ARCHITECTURE.md)
- 想了解 PR 功能？ → [PR_ARCHITECTURE.md](./PR_ARCHITECTURE.md)
- 想了解日志操作？ → [QK_ARCHITECTURE.md](./QK_ARCHITECTURE.md)
- 想了解安装配置？ → [INSTALL_ARCHITECTURE.md](./INSTALL_ARCHITECTURE.md)

