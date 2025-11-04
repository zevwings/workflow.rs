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
│   ├── lib/             # 核心库模块
│   ├── commands/        # 命令实现
│   └── types/           # 类型定义
└── docs/                # 文档目录
    └── ARCHITECTURE.md  # 架构设计（包含 AI 模块）
```

## 📋 命令清单

### 检查工具
```bash
workflow check                    # 综合检查
workflow check --git-only         # 只检查 Git
workflow check --network-only     # 只检查网络
workflow check --proxy-only       # 只检查代理
workflow check --pre-commit       # Pre-commit 检查
```

### Jira 操作
```bash
workflow jira status PROJ-123     # 配置状态映射
workflow jira show PROJ-123        # 显示 ticket
```

### PR 操作
```bash
workflow pr create PROJ-123       # 创建 PR（AI 生成标题）
workflow pr create --title "..."  # 手动指定标题
workflow pr merge                 # 合并当前分支的 PR
workflow pr merge 123             # 合并指定 PR
workflow pr show                  # 显示当前分支的 PR
workflow pr show 123              # 显示指定 PR
workflow pr list                  # 列出所有 PR
workflow pr list --state open     # 列出打开的 PR
```

### 日志操作
```bash
workflow logs download PROJ-123   # 下载日志
workflow logs find <file> <id>    # 查找请求 ID
workflow logs search <file> <term> # 搜索关键词
```

### 辅助功能
```bash
workflow update                    # 快速更新（使用 PR 标题）
workflow proxy                     # 检查并设置代理
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

