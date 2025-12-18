# 🚀 快速参考指南

> 提取两个主要检查指南的核心信息，提供快速查找和执行的参考。

## 📋 检查类型选择

| 检查类型 | 时间 | 使用场景 | 参考文档 |
|---------|------|----------|----------|
| **快速检查** | 5-15分钟 | 日常提交前 | [提交前检查指南](./PRE_COMMIT_GUIDELINES.md) |
| **深入检查** | 2-4小时 | 功能完成后、定期审查 | [检查指南](./REVIEW_GUIDELINES.md) |

---

## ⚡ 快速检查命令

### 必须执行（P0）

```bash
# 1. 代码质量检查
make lint

# 2. 运行所有测试
make test

# 3. 补全脚本完整性测试
cargo test --test completeness

# 4. 编译检查
cargo check
```

### 版本检查（如有功能性变更）

```bash
# 检查是否需要更新版本号
git diff master -- Cargo.toml | grep "^+version"

# 验证 CHANGELOG.md 是否已更新
head -30 CHANGELOG.md
```

---

## 📊 检查清单速览

### ✅ 提交前必检项（5-15分钟）

- [ ] **代码质量**：`make lint` 通过
- [ ] **测试通过**：`make test` 通过
- [ ] **补全完整**：`cargo test --test completeness` 通过
- [ ] **编译成功**：`cargo check` 通过
- [ ] **版本管理**：如有功能变更，检查版本号更新

### 🔍 深入检查项（2-4小时）

- [ ] **CLI 检查**：命令结构、补全脚本、参数复用
- [ ] **代码检查**：重复代码、工具复用、第三方库
- [ ] **测试检查**：覆盖情况、合理性、缺失测试
- [ ] **文档检查**：README、架构文档、CHANGELOG

---

## 🎯 问题优先级

| 优先级 | 描述 | 处理方式 |
|--------|------|----------|
| **P0** | 必须修复 | 阻止提交，立即修复 |
| **P1** | 建议修复 | 本次或下次提交修复 |
| **P2** | 可选修复 | 记录到技术债务，择机修复 |

---

## 📁 报告文件位置

| 检查类型 | 报告位置 | 文件名格式 |
|---------|----------|------------|
| 快速检查汇总 | `report/` | `CHECK_REPORT_{timestamp}.md` |
| CLI 检查 | `report/` | `REVIEW_CLI_{timestamp}.md` |
| 代码检查 | `report/` | `REVIEW_CODE_{timestamp}.md` |
| 测试检查 | `report/` | `REVIEW_TEST_{timestamp}.md` |
| 文档检查 | `report/` | `REVIEW_DOCUMENT_{timestamp}.md` |
| 综合检查 | `report/` | `REVIEW_REPORT_{timestamp}.md` |

**时间戳格式**：`YYYY-MM-DD_HH-MM-SS`（如：`2024-12-19_14-30-00`）

---

## 🔧 常用工具函数

### 生成时间戳

```rust
use workflow::base::util::date::format_filename_timestamp;

let timestamp = format_filename_timestamp();
let report_path = format!("report/CHECK_REPORT_{}.md", timestamp);
```

### 命令行生成时间戳

```bash
# Unix/macOS/Linux
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
echo "report/CHECK_REPORT_${TIMESTAMP}.md"

# Windows PowerShell
$timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
Write-Host "report/CHECK_REPORT_${timestamp}.md"
```

---

## 🆘 故障排除

### 常见问题快速解决

| 问题 | 快速解决方案 |
|------|-------------|
| 格式化失败 | `cargo fmt` |
| Clippy 警告 | `cargo clippy --fix --allow-dirty --allow-staged` |
| 测试失败 | 检查具体失败的测试，修复后重新运行 |
| 补全测试失败 | 检查 CLI 命令注册，运行 `cargo run -- completion generate` |
| 版本号不同步 | `cargo update -p workflow --precise <版本号>` |

### 获取详细帮助

- **代码问题**：参考 [代码检查指南](./reviews/REVIEW_CODE_GUIDELINES.md)
- **测试问题**：参考 [测试用例检查指南](./reviews/REVIEW_TEST_CASE_GUIDELINES.md)
- **文档问题**：参考 [文档检查指南](./reviews/REVIEW_DOCUMENT_GUIDELINES.md)
- **CLI 问题**：参考 [CLI 检查指南](./reviews/REVIEW_CLI_GUIDELINES.md)

---

## 📞 联系方式

如需更详细的检查指南，请查看：
- [提交前检查指南](./PRE_COMMIT_GUIDELINES.md) - 完整的快速检查流程
- [检查指南](./REVIEW_GUIDELINES.md) - 完整的深入检查流程

---

**最后更新**: 2025-12-18
