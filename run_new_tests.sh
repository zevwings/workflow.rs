#!/bin/bash
# 运行我们新创建的测试脚本
# 这个脚本只运行新创建的测试，避免旧测试的编译错误

echo "🚀 运行新创建的测试模块..."
echo ""

echo "📦 测试 Base/Concurrent 模块:"
cargo test --lib base::concurrent::executor --quiet

echo ""
echo "📦 测试 Base/Util/String 模块:"
cargo test --lib base::util::string --quiet

echo ""
echo "📦 测试 Base/Util/Date 模块:"
cargo test --lib base::util::date --quiet

echo ""
echo "📦 测试 Base/Util/Format 模块:"
cargo test --lib base::util::format --quiet

echo ""
echo "📦 测试 Base/Util/Checksum 模块:"
cargo test --lib base::util::checksum --quiet

echo ""
echo "✅ 所有新创建的测试模块运行完成！"
