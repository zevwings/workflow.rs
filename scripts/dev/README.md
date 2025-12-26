# 开发工具脚本

本目录包含用于开发和维护项目的辅助脚本。

## 📋 脚本列表

### Python 脚本

| 脚本 | 说明 | 状态 |
|------|------|------|
| `add-test-docs.py` | 为测试函数添加标准文档注释 | ✅ 已完成 |

### Bash 脚本（待迁移）

| 脚本 | 说明 | 迁移状态 |
|------|------|----------|
| `check-test-docs.sh` | 检查测试文档注释完成情况 | 📋 待迁移 |
| `check-docs.sh` | 文档检查（链接、架构文档、时间戳） | 📋 待迁移 |
| `check-links.sh` | 文档链接有效性检查 | 📋 待迁移 |
| `check-migration-status.sh` | 检查测试迁移状态 | 📋 待迁移 |
| `identify-migration-targets.sh` | 识别需要迁移的测试文件 | 📋 待迁移 |
| `check-coverage.sh` | 测试覆盖率检查 | 📋 待迁移 |

### Bash 脚本（保持）

| 脚本 | 说明 | 原因 |
|------|------|------|
| `verify-test-stability.sh` | 连续运行测试验证稳定性 | 主要是命令调用 |
| `check-doctests.sh` | 文档测试检查 | 极简单，无需重写 |

---

## 🔄 迁移计划

**迁移方案**: Python 统一（方案 A）

所有适合的脚本将统一迁移到 Python，以提高代码可维护性和开发效率。

**详细计划**: 参见 [docs/requirements/scripts-migration-plan.md](../../docs/requirements/scripts-migration-plan.md)

### 迁移进度

- [ ] 阶段 1: 高优先级脚本（`check-test-docs.sh`, `check-docs.sh`）
- [ ] 阶段 2: 中优先级脚本（`check-links.sh`, `check-migration-status.sh`, `identify-migration-targets.sh`, `check-coverage.sh`）
- [ ] 阶段 3: 优化和整合

---

## 📖 使用说明

### Python 脚本

#### add-test-docs.py

为测试函数添加标准文档注释。

```bash
python3 scripts/dev/add-test-docs.py <test_file>
```

**示例**:
```bash
python3 scripts/dev/add-test-docs.py tests/base/fs/directory.rs
```

### Bash 脚本

#### check-test-docs.sh

检查测试文件文档注释完成情况。

```bash
./scripts/dev/check-test-docs.sh
```

#### check-docs.sh

文档检查脚本，用于本地测试 document-check.yml 和 CI check-docs job 的逻辑。

```bash
./scripts/dev/check-docs.sh
```

#### check-links.sh

文档链接有效性检查。

```bash
./scripts/dev/check-links.sh
```

#### check-migration-status.sh

检查测试迁移状态。

```bash
./scripts/dev/check-migration-status.sh
```

#### identify-migration-targets.sh

识别需要迁移的测试文件。

```bash
./scripts/dev/identify-migration-targets.sh
```

#### check-coverage.sh

测试覆盖率检查。

```bash
./scripts/dev/check-coverage.sh
```

**前置要求**: 需要安装 `cargo-tarpaulin`
```bash
cargo install cargo-tarpaulin
```

#### verify-test-stability.sh

连续运行测试验证稳定性。

```bash
./scripts/dev/verify-test-stability.sh [运行次数]
```

**示例**:
```bash
./scripts/dev/verify-test-stability.sh 100  # 运行 100 次
```

#### check-doctests.sh

文档测试（doctest）检查。

```bash
./scripts/dev/check-doctests.sh
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

