# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- 新增功能将在这里记录

### Changed
- 变更将在这里记录

### Fixed
- 修复将在这里记录

---

## [1.6.4] - 2025-12-17

### Added
- **仓库清理命令**：新增 `workflow repo clean` 命令
  - `workflow repo clean` - 清理本地分支和本地 tag（保留 main/master、develop、当前分支和忽略列表中的分支）
  - `workflow repo clean --dry-run` - 预览将要删除的分支和 tag，不实际删除
  - 支持自动分类已合并和未合并分支
  - 支持清理只存在于本地但不在远程的 tag
- **Tag 删除命令**：新增 `workflow tag delete` 命令
  - `workflow tag delete [TAG_NAME]` - 删除指定 tag（本地和远程）
  - `workflow tag delete [TAG_NAME] --local` - 只删除本地 tag
  - `workflow tag delete [TAG_NAME] --remote` - 只删除远程 tag
  - `workflow tag delete --pattern "v1.*"` - 删除匹配模式的 tag
  - `workflow tag delete [TAG_NAME] --dry-run` - 预览模式
  - `workflow tag delete [TAG_NAME] --force` - 强制删除（跳过确认）
  - 支持交互式选择多个 tag 进行删除
- **分支删除命令**：新增 `workflow branch delete` 命令
  - `workflow branch delete [BRANCH_NAME]` - 删除指定分支（本地和远程）
  - `workflow branch delete [BRANCH_NAME] --local-only` - 只删除本地分支
  - `workflow branch delete [BRANCH_NAME] --remote-only` - 只删除远程分支
  - `workflow branch delete [BRANCH_NAME] --dry-run` - 预览模式
  - `workflow branch delete [BRANCH_NAME] --force` - 强制删除（跳过确认）
  - 支持交互式选择多个分支进行删除
  - 自动检测受保护分支（默认分支、develop、忽略列表中的分支）
  - 自动检测未合并分支并提示确认

---

## [1.5.10] - 2025-12-14

### Changed
- 优化 CI 流程，提升构建效率

---

## [1.5.9] - 2025-12-14

### Changed
- 更新代码生成规则

---

## [1.5.8] - 2025-12-14

### Added
- **别名管理系统**：新增 `workflow alias` 命令，支持为常用命令创建简短别名
  - `workflow alias add <name> <command>` - 添加别名
  - `workflow alias list` - 列出所有别名
  - `workflow alias remove <name>` - 删除别名
  - 支持交互式添加和删除别名

---

## [1.5.7] - 2025-12-14

### Fixed
- 配置优化，提升配置加载和验证性能

---

## [1.5.6] - 2025-12-14

### Fixed
- 修复分支类型和 PR 标题不一致的问题

---

## [1.5.5] - 2025-12-14

### Changed
- 重构测试用例，提升测试覆盖率和代码质量

---

## [1.5.4] - 2025-12-14

### Changed
- **错误处理改进**：将 `anyhow` 替换为 `color-eyre`，提供更好的错误报告和堆栈跟踪

---

## [1.5.3] - 2025-12-14

### Changed
- 使用 `serde_with` 优化序列化/反序列化代码

---

## [1.5.2] - 2025-12-14

### Changed
- **代码重构**：重构 PR/分支/提交管理模块，提升代码可维护性

---

## [1.5.1] - 2025-12-14

### Changed
- **代码重构**：重构 Jira 附件下载功能，优化下载流程和错误处理

---

## [1.5.0] - 2025-12-14

### Added
- **配置管理命令**：新增 `workflow config` 命令集
  - `workflow config show` - 查看当前配置
  - `workflow config validate` - 验证配置文件
  - `workflow config export` - 导出配置文件
  - `workflow config import` - 导入配置文件

---

## 版本历史说明

本项目采用语义化版本（Semantic Versioning）：
- **Major** (X.0.0): 重大变更，可能不向后兼容
- **Minor** (x.X.0): 新功能，向后兼容
- **Patch** (x.x.X):  bug 修复和小改进，向后兼容

版本号遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范，根据 commit messages 自动确定版本更新类型。

---

## 链接

- [GitHub Releases](https://github.com/zevwings/workflow.rs/releases)
- [项目文档](https://github.com/zevwings/workflow.rs/blob/master/README.md)
- [架构文档](https://github.com/zevwings/workflow.rs/tree/master/docs)

---

**最后更新**: 2025-12-16
