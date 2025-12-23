# 测试覆盖检查机制指南

> 🔍 **测试覆盖检查**：建立系统化的测试覆盖检查机制，确保新增功能有对应的测试用例

## 📋 概述

本指南提供了系统化的测试覆盖检查方法，用于：
- 检查新增功能的测试覆盖情况
- 识别缺失的测试用例
- 建立定期检查机制
- 确保关键功能有足够的测试保护

---

## 🎯 检查目标

### 主要目标

1. **新增功能测试覆盖**：确保每个新增功能都有对应的测试用例
2. **关键路径测试**：确保关键业务逻辑有充分的测试覆盖
3. **边界情况测试**：确保边界情况和错误处理有测试覆盖
4. **回归测试**：确保重构不会破坏现有功能

### 检查范围

- **单元测试**：`#[cfg(test)]` 模块中的测试
- **集成测试**：`tests/` 目录中的测试文件
- **文档测试**：文档中的代码示例（doctest）

---

## 🔄 检查流程

### 步骤 1：功能变更识别

#### 1.1 识别新增功能

```bash
# 查看最近的提交，识别新增功能
git log --oneline --since="1 week ago" | grep -E "(feat|add|新增)"

# 查看新增的文件
git diff --name-status main...HEAD | grep "^A"

# 查看修改的文件
git diff --name-status main...HEAD | grep "^M"
```

#### 1.2 分析代码变更

```bash
# 查看新增的公共 API
git diff main...HEAD | grep -E "^\+.*pub fn|^\+.*pub struct|^\+.*pub enum"

# 查看新增的命令
git diff main...HEAD | grep -E "^\+.*Commands::|^\+.*Subcommand"
```

### 步骤 2：测试覆盖检查

#### 2.1 检查单元测试

**检查方法**：
```bash
# 查找新增功能的测试文件
find src -name "*.rs" -exec grep -l "新增功能名称" {} \;

# 检查测试模块是否存在
grep -r "#\[cfg(test)\]" src/ | grep "新增模块名"

# 运行特定模块的测试
cargo test --lib 模块名
```

**检查清单**：
- [ ] 新增的公共函数是否有单元测试？
- [ ] 新增的结构体/枚举是否有测试？
- [ ] 错误处理路径是否有测试？
- [ ] 边界情况是否有测试？

#### 2.2 检查集成测试

**检查方法**：
```bash
# 查找对应的集成测试文件
find tests -name "*.rs" -exec grep -l "新增功能名称" {} \;

# 检查测试文件是否存在
ls tests/新增模块名/mod.rs

# 运行集成测试
cargo test --test 测试文件名
```

**检查清单**：
- [ ] 新增的 CLI 命令是否有集成测试？
- [ ] 新增的业务逻辑是否有集成测试？
- [ ] 端到端流程是否有测试？

#### 2.3 检查文档测试

**检查方法**：
```bash
# 运行文档测试
cargo test --doc

# 检查文档中的代码示例
grep -r "```rust" docs/ | grep "新增功能"
```

**检查清单**：
- [ ] 文档中的代码示例是否可以编译？
- [ ] 文档中的代码示例是否可以运行？
- [ ] 新增功能的文档是否有示例？

### 步骤 3：测试覆盖分析

#### 3.1 运行测试覆盖率工具

**使用 cargo-tarpaulin**（如果已安装）：
```bash
# 安装 cargo-tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html --output-dir coverage

# 查看覆盖率报告
open coverage/tarpaulin-report.html
```

**使用 cargo-llvm-cov**（如果已安装）：
```bash
# 安装 cargo-llvm-cov
cargo install cargo-llvm-cov

# 生成覆盖率报告
cargo llvm-cov --html --output-dir coverage

# 查看覆盖率报告
open coverage/index.html
```

#### 3.2 手动覆盖率检查

**检查方法**：
```bash
# 列出所有新增的公共函数
git diff main...HEAD | grep -E "^\+.*pub fn" | sed 's/^+//' | awk '{print $2}'

# 检查每个函数是否有测试
for func in $(git diff main...HEAD | grep -E "^\+.*pub fn" | sed 's/^+//' | awk '{print $2}'); do
    echo "检查函数: $func"
    grep -r "test.*$func\|$func.*test" tests/ src/ || echo "  ⚠️  未找到测试"
done
```

### 步骤 4：缺失测试识别

#### 4.1 识别缺失的测试

**检查清单**：
- [ ] 新增的公共 API 是否有测试？
- [ ] 新增的命令是否有测试？
- [ ] 新增的错误处理是否有测试？
- [ ] 新增的边界情况是否有测试？

#### 4.2 生成缺失测试报告

**创建检查脚本**：
```bash
cat > check-_missing-_tests.sh << 'EOF'
#!/bin/bash
echo "=== 测试覆盖检查报告 ==="
echo "生成时间: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# 检查新增的公共函数
echo "## 新增公共函数测试覆盖"
echo ""
git diff main...HEAD | grep -E "^\+.*pub fn" | while read line; do
    func=$(echo "$line" | sed 's/^+//' | awk '{print $2}')
    file=$(echo "$line" | awk '{print $NF}')
    if grep -r "test.*$func\|$func.*test" tests/ src/ > /dev/null 2>&1; then
        echo "✅ $func (在 $file)"
    else
        echo "❌ $func (在 $file) - 缺少测试"
    fi
done

echo ""
echo "## 新增命令测试覆盖"
echo ""
git diff main...HEAD | grep -E "^\+.*Commands::|^\+.*Subcommand" | while read line; do
    cmd=$(echo "$line" | sed 's/^+//')
    if grep -r "$cmd" tests/cli/ > /dev/null 2>&1; then
        echo "✅ $cmd"
    else
        echo "❌ $cmd - 缺少测试"
    fi
done
EOF

chmod +x check-_missing-_tests.sh
./check-_missing-_tests.sh
```

---

## 📊 检查报告模板

### 测试覆盖检查报告

```markdown
# 测试覆盖检查报告

**检查日期**：YYYY-MM-DD HH:MM:SS
**检查范围**：[本次检查涉及的功能/模块]
**检查类型**：[功能完成 / 定期审查]

---

## 检查概览

[简要描述本次检查的范围和目的]

---

## 新增功能测试覆盖

### ✅ 已覆盖的功能

- [功能1] - [测试文件位置]
- [功能2] - [测试文件位置]

### ❌ 缺失测试的功能

- [功能1] - [原因]
- [功能2] - [原因]

---

## 测试覆盖统计

- **单元测试**：X 个新增功能，Y 个有测试（Z%）
- **集成测试**：X 个新增命令，Y 个有测试（Z%）
- **文档测试**：X 个新增示例，Y 个通过（Z%）

---

## 改进建议

### 高优先级

- [ ] [需要添加的测试1]
- [ ] [需要添加的测试2]

### 中优先级

- [ ] [需要改进的测试1]
- [ ] [需要改进的测试2]

---

## 总结

**测试覆盖情况**：[优秀 / 良好 / 需要改进]

**关键发现**：
- [发现1]
- [发现2]

**下一步行动**：
- [行动1]
- [行动2]
```

---

## 🔄 定期检查机制

### 每周检查

**时间**：每周一次（建议周五下午）

**检查范围**：
- 本周新增功能的测试覆盖
- 本周修改功能的测试更新

**检查流程**：
1. 识别本周新增/修改的功能
2. 检查对应的测试覆盖情况
3. 记录缺失的测试
4. 制定下周测试计划

**时间估算**：30 分钟-1 小时

### 每月检查

**时间**：每月一次（建议月末）

**检查范围**：
- 本月所有新增功能的测试覆盖
- 测试覆盖趋势分析
- 测试质量评估

**检查流程**：
1. 运行测试覆盖率工具
2. 生成测试覆盖报告
3. 分析测试覆盖趋势
4. 制定下月测试改进计划

**时间估算**：1-2 小时

### 功能完成后检查

**时间**：每次功能完成后

**检查范围**：
- 本次新增功能的测试覆盖
- 相关功能的回归测试

**检查流程**：
1. 运行所有测试确保通过
2. 检查新增功能的测试覆盖
3. 运行测试覆盖率工具（如已安装）
4. 记录测试覆盖情况

**时间估算**：15-30 分钟

---

## 🛠️ 工具和命令

### 基本检查命令

```bash
# 运行所有测试
cargo test

# 运行单元测试
cargo test --lib

# 运行集成测试
cargo test --test 测试文件名

# 运行文档测试
cargo test --doc

# 运行特定模块的测试
cargo test 模块名
```

### 覆盖率工具命令

```bash
# 使用 cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage

# 使用 cargo-llvm-cov
cargo llvm-cov --html --output-dir coverage

# 查看覆盖率报告
open coverage/index.html
```

### 检查脚本

```bash
# 检查新增功能的测试覆盖
./scripts/check-_test-_coverage.sh

# 生成测试覆盖报告
./scripts/generate-_test-_coverage-_report.sh
```

---

## 📚 相关文档

- [测试规范指南](../../testing.md) - 完整的测试规范
- [测试用例检查指南](./review-test-case.md) - 详细的测试检查方法
- [提交前检查指南](../workflows/pre-commit.md) - 提交前的测试检查清单

---

## 🎯 最佳实践

### 1. 测试驱动开发（TDD）

- 先编写测试，再实现功能
- 确保测试通过后再提交代码

### 2. 及时添加测试

- 新增功能时立即添加测试
- 不要等到功能完成后再补测试

### 3. 测试覆盖目标

- **整体覆盖率**：> 80%
- **关键业务逻辑**：> 90%
- **新增功能**：100%（理想情况）

### 4. 定期检查

- 每周检查新增功能的测试覆盖
- 每月进行全面的测试覆盖分析

---

## 📋 检查清单

### 功能变更识别检查清单

- [ ] 已识别所有新增功能
- [ ] 已识别所有修改的功能
- [ ] 已分析代码变更（公共 API、命令等）

### 测试覆盖检查清单

- [ ] 新增的公共函数是否有单元测试？
- [ ] 新增的结构体/枚举是否有测试？
- [ ] 新增的 CLI 命令是否有集成测试？
- [ ] 新增的业务逻辑是否有集成测试？
- [ ] 错误处理路径是否有测试？
- [ ] 边界情况是否有测试？
- [ ] 文档中的代码示例是否可以编译运行？

### 测试覆盖分析检查清单

- [ ] 已运行测试覆盖率工具（如已安装）
- [ ] 已手动检查新增功能的测试覆盖
- [ ] 已识别缺失的测试用例
- [ ] 已生成测试覆盖报告

### 定期检查机制检查清单

- [ ] 已建立每周检查机制
- [ ] 已建立每月检查机制
- [ ] 功能完成后已进行检查
- [ ] 已记录测试覆盖情况

---

**最后更新**: 2025-12-23

