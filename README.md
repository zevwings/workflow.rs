# Workflow - Rust CLI 工具

工作流自动化工具的 Rust 实现版本。

## 📋 项目状态

✅ **生产就绪** - 重构完成度 98%，所有核心功能已完整实现并可以投入使用

## 📚 文档

- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) - 架构设计文档（包含 AI 模块设计）

## 🚀 快速开始

### 编译项目

```bash
cargo build --release
```

### 运行测试

```bash
cargo test
```

### 运行 CLI

```bash
cargo run -- --help
```

## 📦 项目结构

```
workflow/
├── Cargo.toml           # 项目配置
├── src/
│   ├── main.rs          # CLI 入口
│   ├── lib.rs           # 库入口
│   ├── lib/             # 核心库模块
│   │   ├── git/         # Git 操作
│   │   ├── jira/        # Jira API 集成
│   │   ├── pr/          # PR 相关功能
│   │   ├── llm/         # LLM 集成（AI）
│   │   ├── log/         # 日志处理
│   │   ├── settings/    # 配置管理
│   │   └── utils/       # 工具函数
│   └── commands/        # 命令实现
│       ├── pr/          # PR 命令
│       ├── jira/        # Jira 命令
│       ├── logs/        # 日志命令
│       └── ...
└── docs/                # 文档目录
    └── ARCHITECTURE.md  # 架构设计（包含 AI 模块和数据存储）
```

## 📋 命令清单

### 检查工具
```bash
workflow check git_status         # 检查 Git 状态
workflow check network             # 检查网络连接（GitHub）
workflow check pre_commit          # 运行 Pre-commit 检查
```

### 代理管理
```bash
workflow proxy on                  # 开启代理
workflow proxy off                 # 关闭代理
workflow proxy check               # 检查代理状态和配置
```

### 配置管理
```bash
workflow setup                     # 初始化或更新配置
workflow config                    # 查看当前配置
```

### Jira 操作
```bash
jira status PROJ-123     # 配置状态映射（交互式）
jira show PROJ-123        # 显示 ticket 信息
```

### PR 操作
```bash
pr create [PROJ-123]     # 创建 PR（可选 Jira ticket，AI 生成标题）
pr create --title "..."  # 手动指定标题
pr create --description "..." # 指定简短描述
pr create --dry-run      # 干运行（不实际创建）
pr merge [PR_ID]         # 合并 PR（可选指定 PR ID，否则自动检测当前分支）
pr merge --force         # 强制合并
pr show [PR_ID_OR_BRANCH] # 显示 PR 信息（可选参数）
pr list                   # 列出所有 PR
pr list --state open     # 按状态过滤（open/closed/merged）
pr list --limit 10       # 限制结果数量
pr update                 # 更新代码（使用 PR 标题作为提交信息）
```

### 日志操作
```bash
logs download PROJ-123   # 下载日志文件
logs find <file> <id> [PROJ-123] # 查找请求 ID（可选 Jira ID 用于域名）
logs search <file> <term> # 搜索关键词
```

### 快捷日志操作 (qk)
```bash
qk PROJ-123 download      # 下载日志（等价于 logs download）
qk PROJ-123 find [id]     # 查找请求 ID（可选，不提供会提示）
qk PROJ-123 search [term] # 搜索关键词（可选，不提供会提示）
```

### 辅助功能
```bash
update                    # 快速更新（使用 PR 标题作为提交信息）
```

> **注意**：Codeup 仓库的 PR 查看和合并功能正在开发中，GitHub 仓库已完整支持。

## 🔧 开发

### 添加依赖

```bash
cargo add <package-name>
```

### 代码格式化

```bash
cargo fmt
```

### Lint 检查

```bash
cargo clippy
```

## 📝 贡献

请参考以下文档了解更多信息：
- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) - 了解架构设计和核心模块详情

