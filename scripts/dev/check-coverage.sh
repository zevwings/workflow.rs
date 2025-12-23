#!/bin/bash
# 测试覆盖率检查脚本

set -e

echo "=========================================="
echo "测试覆盖率检查"
echo "=========================================="
echo ""

# 检查 cargo-tarpaulin 是否安装
if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
  echo "❌ cargo-tarpaulin 未安装"
  echo "   安装方法: cargo install cargo-tarpaulin"
  exit 1
fi

echo "✅ cargo-tarpaulin 已安装"
cargo tarpaulin --version
echo ""

# 生成覆盖率报告
echo "生成覆盖率报告..."
cargo tarpaulin --out Html --output-dir coverage

# 检查覆盖率是否达到阈值
echo ""
echo "检查覆盖率阈值..."

# 解析覆盖率报告（从 JSON 输出中提取覆盖率）
COVERAGE_JSON=$(cargo tarpaulin --out Json --output-dir coverage 2>/dev/null | tail -1)
if [ -n "$COVERAGE_JSON" ]; then
  # 使用 Python 或 jq 解析 JSON（如果可用）
  if command -v python3 >/dev/null 2>&1; then
    COVERAGE=$(echo "$COVERAGE_JSON" | python3 -c "import sys, json; data = json.load(sys.stdin); print(data.get('coverage_percent', 0))" 2>/dev/null || echo "0")
  elif command -v jq >/dev/null 2>&1; then
    COVERAGE=$(echo "$COVERAGE_JSON" | jq -r '.coverage_percent // 0' 2>/dev/null || echo "0")
  else
    COVERAGE="0"
  fi

  if [ "$COVERAGE" != "0" ]; then
    echo "当前覆盖率: ${COVERAGE}%"
    TARGET_COVERAGE=80.0

    # 比较覆盖率（使用 awk 进行浮点数比较）
    if awk "BEGIN {exit !($COVERAGE >= $TARGET_COVERAGE)}"; then
      echo "✅ 覆盖率已达到目标阈值 (${TARGET_COVERAGE}%)"
    else
      echo "⚠️  覆盖率未达到目标阈值 (${TARGET_COVERAGE}%)"
      echo "   当前覆盖率: ${COVERAGE}%"
      echo "   目标覆盖率: ${TARGET_COVERAGE}%"
    fi
  else
    echo "⚠️  无法解析覆盖率数据，请检查报告"
  fi
else
  echo "⚠️  无法获取覆盖率数据"
fi

echo ""
echo "✅ 覆盖率检查完成"
echo "   报告位置: coverage/tarpaulin-report.html"
echo ""
echo "查看报告:"
echo "  - HTML 报告: coverage/tarpaulin-report.html"
echo "  - 或运行: make coverage-open"

