# 命令模块文档缺失分析

## 📋 现有文档情况

### 已有文档的命令模块

1. **PR_COMMAND_ARCHITECTURE.md** ✅
   - 模块：`commands/pr/`
   - 命令：create, merge, close, status, list, update, integrate

2. **QK_COMMAND_ARCHITECTURE.md** ✅
   - 模块：`commands/qk/`
   - 命令：download, find, search, clean, info（用于 `workflow log` 和 `workflow jira`）

3. **CONFIG_COMMAND_ARCHITECTURE.md** ✅
   - 模块：`commands/config/`
   - 命令：setup, show, log, completion

4. **LIFECYCLE_COMMAND_ARCHITECTURE.md** ✅
   - 模块：`commands/lifecycle/`
   - 命令：install, uninstall, update

---

## ❌ 缺失文档的命令模块

### 1. BRANCH_COMMAND_ARCHITECTURE.md

**模块路径**：`src/commands/branch/`

**文件结构**：
- `mod.rs` - 模块声明和导出
- `clean.rs` - 分支清理命令（~195 行）
- `ignore.rs` - 分支忽略列表管理命令（~94 行）
- `helpers.rs` - 辅助函数（BranchConfig 管理）

**命令**：
- `workflow branch clean` - 清理本地分支
- `workflow branch ignore add` - 添加分支到忽略列表
- `workflow branch ignore remove` - 从忽略列表移除分支
- `workflow branch ignore list` - 列出忽略的分支

**功能说明**：
- 清理本地分支，保留 main/master、develop、当前分支和忽略列表中的分支
- 管理分支忽略列表（基于仓库的配置文件 `branch.toml`）
- 支持 dry-run 模式预览清理操作

**依赖模块**：
- `lib/git/` - GitBranch, GitRepo
- `commands/check/` - CheckCommand::run_all()
- `lib/base/util/` - confirm

---

### 2. CHECK_COMMAND_ARCHITECTURE.md

**模块路径**：`src/commands/check/`

**文件结构**：
- `mod.rs` - 模块声明
- `check.rs` - 环境检查命令（~68 行）

**命令**：
- `workflow check` - 运行环境检查

**功能说明**：
- 检查 Git 仓库状态（是否在 Git 仓库中，是否有未提交的更改）
- 检查网络连接（到 GitHub 的连接）
- 提供清晰的错误提示和解决建议

**依赖模块**：
- `lib/git/` - GitRepo, GitCommit
- `lib/base/http/` - HttpClient

**注意**：`CONFIG_COMMAND_ARCHITECTURE.md` 中提到"详见独立的 Check 命令文档"，但该文档不存在。

---

### 3. GITHUB_COMMAND_ARCHITECTURE.md

**模块路径**：`src/commands/github/`

**文件结构**：
- `mod.rs` - 模块声明
- `github.rs` - GitHub 账号管理命令（~444 行）
- `helpers.rs` - 辅助函数（账号收集、验证等）

**命令**：
- `workflow github list` - 列出所有 GitHub 账号
- `workflow github current` - 显示当前 GitHub 账号
- `workflow github add` - 添加新的 GitHub 账号
- `workflow github remove` - 移除 GitHub 账号
- `workflow github switch` - 切换 GitHub 账号
- `workflow github update` - 更新 GitHub 账号配置

**功能说明**：
- 多账号管理（支持多个 GitHub 账号配置）
- 账号切换（更新 Git 配置和 SSH 配置）
- 账号验证（验证 Token 有效性）
- 配置文件管理（`workflow.toml` 中的 `github.accounts` 数组）

**依赖模块**：
- `lib/base/settings/` - Settings, Paths
- `lib/git/` - GitConfig
- `lib/jira/config.rs` - ConfigManager
- `lib/base/util/` - confirm, mask_sensitive_value

**注意**：`CONFIG_COMMAND_ARCHITECTURE.md` 中提到"详见独立的 GitHub 命令文档"，但该文档不存在。

---

### 4. PROXY_COMMAND_ARCHITECTURE.md

**模块路径**：`src/commands/proxy/`

**文件结构**：
- `mod.rs` - 模块声明
- `proxy.rs` - 代理管理命令（~190 行）

**命令**：
- `workflow proxy on` - 启用代理
- `workflow proxy off` - 禁用代理
- `workflow proxy check` - 检查代理状态
- `--temporary` / `-t` - 临时模式（仅当前 shell，不写入配置文件）

**功能说明**：
- 代理启用/禁用（通过环境变量 HTTP_PROXY, HTTPS_PROXY, NO_PROXY）
- 代理状态检查（系统代理设置、环境变量、shell 配置）
- 临时模式（仅当前 shell 生效，不持久化）
- 持久模式（写入 shell 配置文件，持久化）

**依赖模块**：
- `lib/proxy/` - ProxyManager, SystemProxyReader, ProxyConfigGenerator
- `lib/base/shell/` - ShellConfigManager
- `lib/base/util/` - Clipboard

**注意**：虽然 `lib/proxy/` 有 `PROXY_ARCHITECTURE.md` 文档，但命令层的文档缺失。

---

## 📊 文档完整性统计

| 命令模块 | 文档状态 | 优先级 | 备注 |
|---------|---------|--------|------|
| `branch/` | ❌ 缺失 | 中 | 功能相对独立，使用场景明确 |
| `check/` | ❌ 缺失 | 高 | 被其他命令依赖，CONFIG 文档中已引用 |
| `github/` | ❌ 缺失 | 高 | CONFIG 文档中已引用，功能复杂 |
| `proxy/` | ❌ 缺失 | 中 | lib 层有文档，但命令层文档缺失 |
| `config/` | ✅ 已有 | - | - |
| `lifecycle/` | ✅ 已有 | - | - |
| `pr/` | ✅ 已有 | - | - |
| `qk/` | ✅ 已有 | - | - |

---

## 🎯 建议的文档创建顺序

1. **CHECK_COMMAND_ARCHITECTURE.md** - 高优先级
   - 被多个命令依赖（branch, pr 等）
   - CONFIG 文档中已引用但文档不存在

2. **GITHUB_COMMAND_ARCHITECTURE.md** - 高优先级
   - CONFIG 文档中已引用但文档不存在
   - 功能复杂（多账号管理、切换、验证）

3. **BRANCH_COMMAND_ARCHITECTURE.md** - 中优先级
   - 功能相对独立
   - 使用场景明确

4. **PROXY_COMMAND_ARCHITECTURE.md** - 中优先级
   - lib 层已有文档，命令层相对简单
   - 可以引用 lib 层文档

---

## 📝 文档模板建议

每个命令架构文档应包含：

1. **📋 概述** - 模块功能和定位
2. **📁 相关文件** - 文件组织结构
3. **🔄 调用流程** - 整体架构流程
4. **功能说明** - 各命令的详细说明
5. **📊 数据流** - 数据流向图
6. **🔗 与其他模块的集成** - 模块间关系
7. **📋 使用示例** - 命令使用示例
8. **📝 扩展性** - 如何添加新功能
9. **📚 相关文档** - 相关模块文档链接
10. **✅ 总结** - 设计优势和特点

---

## 🔍 需要检查的其他事项

1. **文档引用更新**：
   - `CONFIG_COMMAND_ARCHITECTURE.md` 中引用了不存在的文档，需要创建或更新引用
   - `docs/README.md` 中需要添加新文档的索引

2. **文档一致性**：
   - 确保所有命令文档遵循相同的结构和格式
   - 确保命令示例使用 `workflow` 前缀（而不是旧的 `pr`、`qk` 独立命令）

3. **交叉引用**：
   - 确保相关文档之间有正确的交叉引用
   - 确保 lib 层文档和命令层文档的引用关系正确

