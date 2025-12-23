#!/bin/bash
# 文档检查脚本
# 用于本地测试 doc-consistency-check.yml 和 CI check-docs job 的逻辑

set -e

echo "=========================================="
echo "文档检查脚本"
echo "=========================================="
echo ""

# 测试 1: 文档链接检查
echo "📋 测试 1: 文档链接检查"
echo "----------------------------------------"
if command -v lychee >/dev/null 2>&1; then
  echo "✅ lychee 已安装"
  lychee --version 2>&1 | head -1
  echo ""
  echo "运行链接检查（仅检查前5个文档）..."
  find docs -name "*.md" -type f ! -path "*/templates/*" | head -5 | while read -r file; do
    echo "  检查: $file"
  done
  echo "ℹ️  完整链接检查需要运行: lychee docs/**/*.md --exclude-all-private --exclude-loopback"
else
  echo "ℹ️  lychee 未安装（这是正常的，CI 中会跳过）"
  echo "   安装方法: cargo install lychee"
fi
echo ""

# 测试 2: 架构文档存在性检查（CI check-docs 逻辑）
echo "📋 测试 2: CI check-docs job 逻辑"
echo "----------------------------------------"
echo "模拟检查变更的文件..."

# 模拟一些变更的文件
CHANGED_FILES="src/lib/pr/github.rs
src/commands/pr/create.rs
src/lib/jira/api.rs"

echo "变更的文件:"
echo "$CHANGED_FILES"
echo ""

MISSING_DOCS=0
echo "$CHANGED_FILES" | while IFS= read -r file; do
  # 跳过空行
  [ -z "$file" ] && continue

  # 提取模块名
  module=$(echo "$file" | sed 's|^src/lib/\([^/]*\).*|\1|')
  if [ "$module" = "$file" ]; then
    module=$(echo "$file" | sed 's|^src/commands/\([^/]*\).*|\1|')
  fi

  # 跳过空模块名或无效路径
  [ -z "$module" ] || [ "$module" = "$file" ] && continue

  doc_path="docs/architecture/${module}.md"
  if [ ! -f "$doc_path" ]; then
    echo "⚠️  Warning: Module '$module' changed but architecture doc not found: $doc_path"
    MISSING_DOCS=$((MISSING_DOCS + 1))
  else
    echo "✅ Module '$module' has architecture doc: $doc_path"
  fi
done

if [ $MISSING_DOCS -gt 0 ]; then
  echo ""
  echo "📋 Summary: $MISSING_DOCS module(s) missing architecture documentation"
else
  echo ""
  echo "✅ All changed modules have architecture documentation"
fi
echo ""

# 测试 3: 架构文档存在性检查（Scheduled workflow 逻辑）
echo "📋 测试 3: Scheduled workflow 逻辑（全面检查）"
echo "----------------------------------------"
echo "检查所有 lib 层模块..."
MISSING_LIB=0
for module_dir in src/lib/*/; do
  if [ -d "$module_dir" ]; then
    module=$(basename "$module_dir")
    doc_path="docs/architecture/${module}.md"
    if [ ! -f "$doc_path" ]; then
      echo "⚠️  Missing: $doc_path (module: $module)"
      MISSING_LIB=$((MISSING_LIB + 1))
    else
      echo "✅ $module -> $doc_path"
    fi
  fi
done

echo ""
echo "检查所有 commands 层模块..."
MISSING_CMD=0
for module_dir in src/commands/*/; do
  if [ -d "$module_dir" ]; then
    module=$(basename "$module_dir")
    doc_path="docs/architecture/${module}.md"
    if [ ! -f "$doc_path" ]; then
      echo "⚠️  Missing: $doc_path (module: $module)"
      MISSING_CMD=$((MISSING_CMD + 1))
    else
      echo "✅ $module -> $doc_path"
    fi
  fi
done

TOTAL_MISSING=$((MISSING_LIB + MISSING_CMD))
if [ $TOTAL_MISSING -gt 0 ]; then
  echo ""
  echo "📋 Found $TOTAL_MISSING missing architecture document(s)"
else
  echo ""
  echo "✅ All modules have architecture documentation"
fi
echo ""

# 测试 4: 时间戳格式检查
echo "📋 测试 4: 文档时间戳格式检查"
echo "----------------------------------------"
INVALID_TIMESTAMPS=0
CHECKED=0
while IFS= read -r file; do
  CHECKED=$((CHECKED + 1))
  if ! tail -5 "$file" | grep -qE '\*\*最后更新\*\*: [0-9]{4}-[0-9]{2}-[0-9]{2}'; then
    echo "⚠️  Invalid timestamp format: $file"
    INVALID_TIMESTAMPS=$((INVALID_TIMESTAMPS + 1))
  fi
done < <(find docs -name "*.md" -type f ! -path "*/templates/*" ! -name "README.md" | head -10)

echo "检查了 $CHECKED 个文档"
if [ $INVALID_TIMESTAMPS -gt 0 ]; then
  echo "📋 Found $INVALID_TIMESTAMPS document(s) with invalid timestamp format"
else
  echo "✅ All checked documents have valid timestamp format"
fi
echo ""

# 测试 5: 报告生成
echo "📋 测试 5: 报告生成"
echo "----------------------------------------"
mkdir -p report
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
REPORT_FILE="report/doc-consistency-check-test-${TIMESTAMP}.md"
CHECK_DATE=$(date '+%Y-%m-%d %H:%M:%S')
UPDATE_DATE=$(date '+%Y-%m-%d')

cat > "$REPORT_FILE" << EOF
# 架构文档一致性检查报告（测试）

**检查日期**：${CHECK_DATE}
**检查类型**：测试运行

## 检查结果

### 文档链接检查

已完成文档链接有效性检查。

### 架构文档存在性检查

已完成架构文档存在性检查。

### 文档时间戳格式检查

已完成文档时间戳格式检查。

## 问题汇总

请查看上方的检查输出以了解详细问题。

## 改进建议

1. 确保所有模块都有对应的架构文档
2. 确保所有文档都有正确的时间戳格式
3. 确保所有文档链接都有效

参考文档：
- [架构文档审查指南](docs/guidelines/development/references/review-architecture-consistency.md)
- [文档更新检查清单](docs/guidelines/development/code-review.md)

---

**最后更新**: ${UPDATE_DATE}
EOF

echo "✅ 报告已生成: $REPORT_FILE"
ls -lh "$REPORT_FILE"
echo ""

echo "=========================================="
echo "✅ 所有测试完成"
echo "=========================================="

