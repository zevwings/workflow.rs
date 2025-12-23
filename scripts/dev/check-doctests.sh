#!/bin/bash
# 文档测试（doctest）检查脚本

set -e

echo "=========================================="
echo "文档测试（doctest）检查"
echo "=========================================="
echo ""

# 运行文档测试
echo "运行文档测试..."
if cargo test --doc; then
  echo ""
  echo "✅ 所有文档测试通过"
else
  echo ""
  echo "❌ 文档测试失败"
  echo "   请检查文档中的代码示例是否正确"
  echo "   确保代码示例可以编译和运行"
  echo ""
  echo "提示:"
  echo "  - 检查代码示例中的导入语句是否正确"
  echo "  - 检查代码示例中的函数调用是否正确"
  echo "  - 确保代码示例使用正确的 doctest 格式"
  exit 1
fi

echo ""
echo "✅ 文档测试检查完成"

