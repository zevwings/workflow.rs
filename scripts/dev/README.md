# 开发工具脚本

本目录包含用于开发和维护项目的辅助脚本。

## 📋 脚本分类

### 🧪 测试相关

#### 测试文档管理

| 脚本 | 语言 | 说明 | 状态 |
|------|------|------|------|
| `check-test-docs.sh` | Bash | 检查测试文档注释完成情况 | 📋 待迁移 |

#### 测试执行与报告

| 脚本 | 语言 | 说明 | 状态 |
|------|------|------|------|
| `generate-test-report.py` | Python | 生成测试执行报告（HTML/JSON） | ✅ 已完成 |
| `generate-pr-comment.py` | Python | 从测试报告生成 PR 评论内容 | ✅ 已完成 |
| `verify-test-stability.sh` | Bash | 连续运行测试验证稳定性 | 🔧 保持 |

#### 测试指标与分析

| 脚本 | 语言 | 说明 | 状态 |
|------|------|------|------|
| `collect-test-metrics.py` | Python | 收集测试指标数据，用于趋势分析 | ✅ 已完成 |
| `analyze-test-trends.py` | Python | 分析测试指标历史数据，生成趋势报告 | ✅ 已完成 |
| `analyze-performance-regression.py` | Python | 对比性能数据，检测性能回归 | ✅ 已完成 |

#### 测试覆盖率

| 脚本 | 语言 | 说明 | 状态 |
|------|------|------|------|
| `analyze-coverage-trends.py` | Python | 分析覆盖率趋势，检测覆盖率变化和回归 | ✅ 已完成 |
| `check-coverage.sh` | Bash | 测试覆盖率检查 | 📋 待迁移 |

#### 测试迁移工具

> **注意**: 测试迁移工具已完成使用，相关脚本已删除。

### 📚 文档相关

| 脚本 | 语言 | 说明 | 状态 |
|------|------|------|------|
| `check-docs.sh` | Bash | 文档检查（链接、架构文档、时间戳） | 📋 待迁移 |
| `check-links.sh` | Bash | 文档链接有效性检查 | 📋 待迁移 |

---

## 🔄 迁移计划

**迁移方案**: Python 统一（方案 A）

所有适合的脚本将统一迁移到 Python，以提高代码可维护性和开发效率。

**详细计划**: 参见 [docs/requirements/scripts-migration-plan.md](../../docs/requirements/scripts-migration-plan.md)

### 迁移进度

- [ ] 阶段 1: 高优先级脚本（`check-test-docs.sh`, `check-docs.sh`）
- [ ] 阶段 2: 中优先级脚本（`check-links.sh`, `check-coverage.sh`）
- [ ] 阶段 3: 优化和整合

**状态说明**:
- ✅ 已完成：Python 脚本，功能完整
- 📋 待迁移：Bash 脚本，计划迁移到 Python
- 🔧 保持：Bash 脚本，保持现状（简单工具或一次性脚本）

---

## 📖 使用说明

### 🧪 测试相关脚本

#### 测试文档管理

##### check-test-docs.sh

检查测试文件文档注释完成情况。

```bash
./scripts/dev/check-test-docs.sh
```

#### 测试执行与报告

##### generate-test-report.py

生成测试执行报告（HTML 或 JSON 格式）。

```bash
python3 scripts/dev/generate-test-report.py [OPTIONS]
```

**选项**:
- `--format, -f <format>`: 报告格式，`html` 或 `json`（默认: `html`）
- `--output, -o <path>`: 输出文件路径（默认: `test-report.html`）
- `--help, -h`: 显示帮助信息

**示例**:
```bash
# 生成 HTML 报告
cargo test --message-format=json 2>&1 | \
    python3 scripts/dev/generate-test-report.py -f html -o report.html

# 生成 JSON 报告
cargo test --message-format=json 2>&1 | \
    python3 scripts/dev/generate-test-report.py -f json -o report.json

# 使用管道（推荐方式）
cargo test --message-format=json 2>&1 | \
    python3 scripts/dev/generate-test-report.py --format html --output test-report.html
```

**工作原理**:
1. 运行 `cargo test --message-format=json` 获取 JSON 格式的测试输出
2. 通过管道传递给 Python 脚本进行解析
3. 生成 HTML 或 JSON 格式的测试报告

**前置要求**:
- Python 3.8+

##### generate-pr-comment.py

从测试报告 JSON 生成 PR 评论的 Markdown 内容。

```bash
python3 scripts/dev/generate-pr-comment.py [OPTIONS]
```

**选项**:
- `--report, -r <path>`: 测试报告 JSON 文件路径（可指定多个文件进行合并）
- `--artifact-url, -a <url>`: Artifact 下载 URL（可选）
- `--output, -o <path>`: 输出文件路径（默认：输出到 stdout）
- `--help, -h`: 显示帮助信息

**示例**:
```bash
# 生成 PR 评论（输出到 stdout）
python3 scripts/dev/generate-pr-comment.py --report test-report.json

# 生成 PR 评论并保存到文件
python3 scripts/dev/generate-pr-comment.py --report test-report.json --output pr-comment.md

# 合并多个报告并生成评论
python3 scripts/dev/generate-pr-comment.py \
    --report unit-test-report.json integration-test-report.json \
    --artifact-url https://github.com/.../artifacts \
    --output pr-comment.md
```

#### 测试指标与分析

##### collect-test-metrics.py

从测试报告 JSON 中提取指标数据，用于趋势分析。

```bash
python3 scripts/dev/collect-test-metrics.py [OPTIONS]
```

**选项**:
- `--report, -r <path>`: 测试报告 JSON 文件（必需）
- `--output, -o <path>`: 输出指标文件（必需）
- `--test-type <type>`: 测试类型（unit/integration，可选）
- `--platform <platform>`: 平台（Linux/macOS/Windows，可选）
- `--help, -h`: 显示帮助信息

**示例**:
```bash
python3 scripts/dev/collect-test-metrics.py \
    --report test-report.json \
    --output metrics/2024-01-01-unit-linux.json \
    --test-type unit \
    --platform Linux
```

##### analyze-test-trends.py

分析测试指标的历史数据，生成趋势报告。

```bash
python3 scripts/dev/analyze-test-trends.py [OPTIONS]
```

**选项**:
- `--metrics-dir, -d <path>`: 指标数据目录（必需）
- `--output, -o <path>`: 输出报告文件（必需）
- `--help, -h`: 显示帮助信息

**示例**:
```bash
python3 scripts/dev/analyze-test-trends.py \
    --metrics-dir metrics/ \
    --output trends-report.md
```

##### analyze-performance-regression.py

对比当前性能与基准性能，检测性能回归。

```bash
python3 scripts/dev/analyze-performance-regression.py [OPTIONS]
```

**选项**:
- `--current, -c <path>`: 当前性能指标 JSON 文件（必需）
- `--baseline, -b <path>`: 基准性能指标 JSON 文件（可选）
- `--output, -o <path>`: 输出报告文件（必需）
- `--threshold, -t <value>`: 回归阈值（默认: 0.2，即 20%）
- `--help, -h`: 显示帮助信息

**示例**:
```bash
python3 scripts/dev/analyze-performance-regression.py \
    --current metrics/current.json \
    --baseline metrics/baseline.json \
    --output performance-report.md \
    --threshold 0.2
```

#### 测试覆盖率

##### analyze-coverage-trends.py

分析覆盖率的历史数据，检测覆盖率变化和回归。

```bash
python3 scripts/dev/analyze-coverage-trends.py [OPTIONS]
```

**选项**:
- `--current, -c <path>`: 当前覆盖率 JSON 文件（必需）
- `--baseline, -b <path>`: 基准覆盖率 JSON 文件（可选）
- `--output, -o <path>`: 输出报告文件（必需）
- `--threshold, -t <value>`: 回归阈值（%，默认: 1.0）
- `--help, -h`: 显示帮助信息

**示例**:
```bash
python3 scripts/dev/analyze-coverage-trends.py \
    --current coverage.json \
    --baseline baseline-coverage.json \
    --output coverage-report.md \
    --threshold 1.0
```

##### check-coverage.sh

测试覆盖率检查。

```bash
./scripts/dev/check-coverage.sh
```

**前置要求**: 需要安装 `cargo-tarpaulin`
```bash
cargo install cargo-tarpaulin
```

### 📚 文档相关脚本

##### check-docs.sh

文档检查脚本，用于本地测试 document-check.yml 和 CI check-docs job 的逻辑。

```bash
./scripts/dev/check-docs.sh
```

##### check-links.sh

文档链接有效性检查。

```bash
./scripts/dev/check-links.sh
```

---

## 🔧 依赖要求

### Python 脚本
- Python 3.8+
- 标准库（优先使用）

### Bash 脚本
- Bash 4.0+
- 常用 Unix 工具（`grep`, `awk`, `find`, `sed` 等）
- `cargo-tarpaulin`（用于 `check-coverage.sh`）
- `lychee`（可选，用于 `check-links.sh` 的外部链接检查）
- `bc`（用于数学计算）

---

## 📝 注意事项

1. **迁移进行中**: 部分脚本正在迁移到 Python，请关注迁移进度
2. **兼容性**: 所有脚本应在 macOS 和 Linux 上正常工作
3. **错误处理**: 脚本使用 `set -e` 确保错误时退出
4. **路径**: 脚本应在项目根目录运行

---

## 🤝 贡献

如需添加新脚本或改进现有脚本，请：

1. 遵循代码风格规范
2. 添加适当的文档注释
3. 测试脚本功能
4. 更新本文档

---

## 📚 相关文档

- [脚本迁移分析报告](../../docs/requirements/scripts-migration-analysis.md)
- [脚本迁移实施计划](../../docs/requirements/scripts-migration-plan.md)
