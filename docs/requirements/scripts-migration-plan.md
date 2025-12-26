# scripts/dev 目录脚本迁移实施计划

## 概述

本文档详细说明了将 `scripts/dev` 目录下的 Bash 脚本迁移到 Python 的实施计划。

**迁移方案**: 方案 A - Python 统一

**目标**: 将所有适合的脚本统一迁移到 Python，提高代码可维护性和开发效率。

---

## 迁移范围

### ✅ 需要迁移的脚本 (6个)

| 脚本 | 优先级 | 状态 | 预计工作量 |
|------|--------|------|-----------|
| `check-test-docs.sh` | 🔴 高 | 📋 待开始 | 2-3 天 |
| `check-docs.sh` | 🔴 高 | 📋 待开始 | 2-3 天 |
| `check-links.sh` | 🟡 中 | 📋 待开始 | 1-2 天 |
| `check-migration-status.sh` | 🟡 中 | 📋 待开始 | 1 天 |
| `identify-migration-targets.sh` | 🟡 中 | 📋 待开始 | 0.5 天 (合并) |
| `check-coverage.sh` | 🟡 中 | 📋 待开始 | 1 天 |

### ⚠️ 保持 Bash 的脚本 (2个)

| 脚本 | 原因 |
|------|------|
| `verify-test-stability.sh` | 主要是命令调用，Bash 足够 |
| `check-doctests.sh` | 极简单，只是调用 `cargo test --doc` |

### ✅ 已是 Python 的脚本 (1个)

| 脚本 | 状态 |
|------|------|
| `add-test-docs.py` | ✅ 无需修改 |

---

## 技术规范

### Python 版本要求
- **Python 3.8+** (确保兼容性)
- 使用标准库优先，减少外部依赖

### 代码风格
- 遵循 [PEP 8](https://www.python.org/dev/peps/pep-0008/)
- 使用类型提示 (Type Hints)
- 添加文档字符串 (docstrings)
- 使用 `pathlib` 处理路径
- 使用 `argparse` 或 `click` 处理命令行参数

### 依赖管理
- 优先使用 Python 标准库
- 如需第三方库，使用 `requirements.txt` 管理
- 避免引入过多依赖

### 错误处理
- 使用 `try-except` 进行错误处理
- 提供清晰的错误信息
- 适当的退出码 (0 = 成功, 1 = 失败)

### 输出格式
- 使用颜色输出提高可读性 (可选: `colorama` 或 `rich`)
- 统一的日志格式
- 支持 `--verbose` 和 `--quiet` 选项

---

## 实施阶段

### 阶段 1: 高优先级脚本 (预计 4-6 天)

#### 1.1 check-test-docs.py
**目标**: 重写 `check-test-docs.sh`

**功能**:
- 遍历 `tests/` 目录下的所有 `.rs` 文件
- 统计测试函数数量
- 检查文档注释完整性（包含"测试目的"、"测试场景"、"预期结果"）
- 按优先级分类（高/中/低）
- 生成统计报告

**技术要点**:
- 使用 `pathlib` 遍历文件
- 使用 `re` 模块解析 Rust 代码
- 使用 `collections.Counter` 统计
- 输出格式化的报告

**验收标准**:
- [ ] 功能与原始脚本一致
- [ ] 输出格式清晰易读
- [ ] 错误处理完善
- [ ] 性能不低于原始脚本
- [ ] 添加单元测试

---

#### 1.2 check-docs.py
**目标**: 重写 `check-docs.sh`

**功能**:
- 检查文档链接有效性
- 检查架构文档存在性
- 检查文档时间戳格式
- 生成检查报告

**技术要点**:
- 使用 `pathlib` 处理文件路径
- 使用 `re` 模块解析 Markdown
- 检查文件存在性
- 生成 Markdown 格式报告

**验收标准**:
- [ ] 所有检查功能正常工作
- [ ] 报告格式清晰
- [ ] 错误处理完善
- [ ] 支持增量检查（仅检查变更文件）

---

### 阶段 2: 中优先级脚本 (预计 3-4 天)

#### 2.1 check-links.py
**目标**: 重写 `check-links.sh`

**功能**:
- 解析 Markdown 文件中的链接
- 检查内部链接有效性
- 调用 `lychee` 检查外部链接（如果可用）
- 报告断链

**技术要点**:
- 使用正则表达式或 Markdown 解析库解析链接
- 处理相对路径和绝对路径
- 规范化路径
- 调用外部工具 (`lychee`)

**验收标准**:
- [ ] 正确解析所有链接格式
- [ ] 路径处理准确
- [ ] 错误信息清晰
- [ ] 支持排除特定路径

---

#### 2.2 check-migration.py
**目标**: 合并并重写 `check-migration-status.sh` 和 `identify-migration-targets.sh`

**功能**:
- 检查测试迁移状态
- 识别需要迁移的测试文件
- 统计迁移进度
- 列出待迁移文件

**技术要点**:
- 使用 `grep` 功能搜索模式（或使用 `re`）
- 统计匹配数量
- 分类输出结果

**验收标准**:
- [ ] 合并两个脚本的功能
- [ ] 提供统一的 CLI 接口
- [ ] 输出格式清晰
- [ ] 性能良好

---

#### 2.3 check-coverage.py
**目标**: 重写 `check-coverage.sh`

**功能**:
- 调用 `cargo-tarpaulin`
- 解析 JSON 输出
- 检查覆盖率阈值
- 生成报告

**技术要点**:
- 使用 `subprocess` 调用外部命令
- 使用 `json` 模块解析输出
- 计算覆盖率百分比
- 比较阈值

**验收标准**:
- [ ] 正确解析 JSON
- [ ] 覆盖率计算准确
- [ ] 阈值检查正确
- [ ] 错误处理完善

---

### 阶段 3: 优化和整合 (预计 2-3 天)

#### 3.1 统一 CLI 接口
**目标**: 创建统一的命令行工具

**方案**: 创建 `dev-tools.py` 或 `workflow-dev.py`

```python
workflow-dev.py check test-docs
workflow-dev.py check docs
workflow-dev.py check links
workflow-dev.py check migration
workflow-dev.py check coverage
```

**技术要点**:
- 使用 `argparse` 或 `click` 创建子命令
- 统一的错误处理
- 统一的输出格式
- 配置文件支持

---

#### 3.2 改进和优化
- 添加单元测试
- 改进错误处理
- 优化性能
- 添加进度条（可选）
- 添加配置文件支持

---

## 文件结构

### 迁移后的目录结构

```
scripts/dev/
├── add-test-docs.py          # ✅ 已有
├── check-test-docs.py        # 🔄 新增
├── check-docs.py              # 🔄 新增
├── check-links.py             # 🔄 新增
├── check-migration.py         # 🔄 新增（合并）
├── check-coverage.py          # 🔄 新增
├── verify-test-stability.sh   # ⚠️ 保持 Bash
├── check-doctests.sh          # ⚠️ 保持 Bash
├── workflow-dev.py            # 🔄 新增（统一 CLI，可选）
├── requirements.txt           # 🔄 新增（如有第三方依赖）
└── README.md                  # 🔄 新增（使用说明）
```

---

## 实施步骤

### 步骤 1: 准备工作
1. [ ] 创建 `requirements.txt`（如有需要）
2. [ ] 创建 `scripts/dev/README.md` 文档
3. [ ] 设置 Python 开发环境

### 步骤 2: 阶段 1 实施
1. [ ] 实现 `check-test-docs.py`
2. [ ] 测试并验证功能
3. [ ] 实现 `check-docs.py`
4. [ ] 测试并验证功能
5. [ ] 更新文档

### 步骤 3: 阶段 2 实施
1. [ ] 实现 `check-links.py`
2. [ ] 实现 `check-migration.py`（合并两个脚本）
3. [ ] 实现 `check-coverage.py`
4. [ ] 测试并验证所有功能
5. [ ] 更新文档

### 步骤 4: 阶段 3 优化
1. [ ] 创建统一 CLI（可选）
2. [ ] 添加单元测试
3. [ ] 性能优化
4. [ ] 完善文档

### 步骤 5: 迁移和清理
1. [ ] 备份原始 Bash 脚本（移动到 `scripts/dev/archive/`）
2. [ ] 更新所有引用（Makefile、文档等）
3. [ ] 更新 CI/CD 配置（如有）
4. [ ] 验证所有功能正常
5. [ ] 删除旧脚本（确认无误后）

---

## 测试策略

### 单元测试
- 为每个 Python 脚本编写单元测试
- 使用 `pytest` 或 `unittest`
- 测试覆盖率 > 80%

### 集成测试
- 在真实项目环境中测试
- 对比新旧脚本的输出结果
- 确保功能一致性

### 性能测试
- 确保性能不低于原始脚本
- 对于大文件处理，优化性能

---

## 文档要求

### 每个脚本应包含
1. **文件头注释**: 说明脚本用途、用法、依赖
2. **函数文档**: 所有函数都有 docstring
3. **使用示例**: 在 README 中提供使用示例
4. **错误处理说明**: 说明可能的错误和解决方案

### README.md 应包含
1. 脚本列表和说明
2. 安装和依赖说明
3. 使用示例
4. 常见问题解答

---

## 风险评估

### 风险 1: 功能不一致
**影响**: 高
**缓解措施**:
- 详细对比新旧脚本的输出
- 编写测试用例验证功能
- 保留旧脚本作为备份

### 风险 2: 性能下降
**影响**: 中
**缓解措施**:
- 性能测试和优化
- 使用生成器处理大文件
- 必要时使用多进程

### 风险 3: 依赖问题
**影响**: 低
**缓解措施**:
- 优先使用标准库
- 明确记录所有依赖
- 提供 `requirements.txt`

---

## 时间估算

| 阶段 | 任务 | 预计时间 |
|------|------|----------|
| 阶段 1 | check-test-docs.py | 2-3 天 |
| 阶段 1 | check-docs.py | 2-3 天 |
| 阶段 2 | check-links.py | 1-2 天 |
| 阶段 2 | check-migration.py | 1 天 |
| 阶段 2 | check-coverage.py | 1 天 |
| 阶段 3 | 统一 CLI | 1-2 天 |
| 阶段 3 | 测试和优化 | 1 天 |
| **总计** | | **9-13 天** |

---

## 成功标准

### 功能完整性
- [ ] 所有脚本功能与原始版本一致
- [ ] 输出格式清晰易读
- [ ] 错误处理完善

### 代码质量
- [ ] 遵循 PEP 8 规范
- [ ] 有完整的文档字符串
- [ ] 有单元测试（覆盖率 > 80%）

### 可维护性
- [ ] 代码结构清晰
- [ ] 易于扩展和修改
- [ ] 有完整的使用文档

### 性能
- [ ] 性能不低于原始脚本
- [ ] 内存使用合理

---

## 后续优化

### 短期优化（1-2 个月）
- 添加更多配置选项
- 改进错误信息
- 优化性能

### 长期优化（3-6 个月）
- 考虑整合为单一工具
- 添加 GUI 界面（可选）
- 添加 CI/CD 集成

---

## 更新日志

| 日期 | 更新内容 | 作者 |
|------|----------|------|
| 2024-XX-XX | 创建迁移计划 | - |

---

## 参考资源

- [PEP 8 Style Guide](https://www.python.org/dev/peps/pep-0008/)
- [Python Type Hints](https://docs.python.org/3/library/typing.html)
- [pathlib Documentation](https://docs.python.org/3/library/pathlib.html)
- [argparse Tutorial](https://docs.python.org/3/howto/argparse.html)

