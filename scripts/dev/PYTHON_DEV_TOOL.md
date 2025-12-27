# Python Dev 工具使用说明

## 概述

Python 版本的 dev 工具，用于替代 Rust 二进制版本，实现零编译时间、快速启动。

**要求**: Python 3.13+

## 使用方法

### 基本用法

```bash
python3 scripts/dev/dev.py <command> [options]
```

脚本会自动检查 Python 版本，如果版本不符合要求会报错退出。

### CI Check Skip

检查是否应该跳过 CI（版本更新分支检查）：

```bash
# 基本用法（非 CI 模式）
python3 scripts/dev/dev.py ci check-skip --branch "feature/testing"

# CI 模式（输出到 GITHUB_OUTPUT）
python3 scripts/dev/dev.py ci check-skip \
    --branch "bump-version-1.6.10" \
    --pr-creator "workflow-bot" \
    --expected-user "workflow-bot" \
    --ci
```

#### 参数说明

- `--branch`: 分支名称（可选，会尝试从环境变量获取）
- `--pr-creator`: PR 创建者用户名（可选，会尝试从环境变量获取）
- `--expected-user`: 期望的用户名（可选，会尝试从环境变量获取）
- `--ci`: CI 模式，输出结果到 `GITHUB_OUTPUT`

#### 环境变量

脚本会自动从以下环境变量读取值（如果命令行参数未提供）：

- `GITHUB_HEAD_REF`: PR 分支名
- `GITHUB_REF_NAME`: 当前分支名
- `GITHUB_EVENT_NAME`: GitHub 事件类型
- `GITHUB_PR_CREATOR`: PR 创建者
- `WORKFLOW_USER_NAME`: 期望的用户名
- `GITHUB_OUTPUT`: GitHub Actions 输出文件路径（CI 模式需要）

#### 行为说明

1. **非 bump-version-* 分支**: 返回 `should_skip=false`，CI 正常运行
2. **bump-version-* 分支**:
   - 非 PR 事件: 返回 `should_skip=true`
   - PR 事件: 验证 PR 创建者是否为授权用户
     - 如果匹配: 返回 `should_skip=true`
     - 如果不匹配: 报错并退出（退出码 1）

## 项目结构

```
scripts/dev/
├── dev.py              # 主入口脚本
├── utils/
│   ├── __init__.py
│   └── logger.py       # 日志工具（零依赖）
├── ci/
│   ├── __init__.py
│   └── check_skip.py   # CI check-skip 实现
└── PYTHON_DEV_TOOL.md  # 本文档
```

## 特性

- ✅ **零依赖**: 完全使用 Python 标准库
- ✅ **快速启动**: 无需编译，直接运行
- ✅ **颜色输出**: 支持 ANSI 颜色码（GitHub Actions 自动支持）
- ✅ **错误处理**: 完善的错误处理和退出码
- ✅ **版本检查**: 自动检查 Python 版本（要求 3.13+）

## 开发

### 添加新命令

1. 在对应的模块目录创建新的 Python 文件
2. 在 `dev.py` 中添加命令解析和路由
3. 实现命令处理函数

### 日志使用

```python
from utils.logger import log_info, log_error, log_success, log_warning

log_info("信息消息")
log_success("成功消息")
log_warning("警告消息")
log_error("错误消息")
```

## 迁移计划

- [x] 基础结构和日志工具
- [x] `ci check-skip` 命令
- [ ] `ci verify` 命令
- [ ] `checksum calculate` 命令
- [ ] 其他命令...

## 与 Rust 版本的对比

| 特性 | Rust 版本 | Python 版本 |
|------|----------|------------|
| 启动时间 | ~3 分钟（编译） | 0 秒（直接运行） |
| 依赖 | Rust 工具链 | Python 3.8+ |
| 维护性 | 需要编译 | 直接修改 |
| 性能 | 编译后快速 | 运行时快速 |

