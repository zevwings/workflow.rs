#!/bin/bash
# 检查测试迁移状态

set -e

echo "=== 测试迁移状态检查 ==="
echo ""

# 统计已迁移的测试（使用隔离工具）
MIGRATED=$(grep -rn "TestIsolation\|CliTestEnv\|GitTestEnv" tests/ --include="*.rs" | \
  grep -v "common/isolation.rs\|common/environments" | \
  wc -l | tr -d ' ')

# 统计待迁移的测试
PENDING_TEMPDIR=$(grep -rn "tempfile::tempdir\|TempDir" tests/ --include="*.rs" | \
  grep -v "TestIsolation\|CliTestEnv\|GitTestEnv\|CurrentDirGuard\|common/isolation.rs\|common/environments" | \
  wc -l | tr -d ' ')

PENDING_SETDIR=$(grep -rn "set_current_dir" tests/ --include="*.rs" | \
  grep -v "CurrentDirGuard\|common/helpers.rs" | \
  wc -l | tr -d ' ')

PENDING_ENVVAR=$(grep -rn "env::set_var\|std::env::set_var" tests/ --include="*.rs" | \
  grep -v "EnvGuard\|MockServer\|common/guards\|common/http_helpers" | \
  wc -l | tr -d ' ')

PENDING=$((PENDING_TEMPDIR + PENDING_SETDIR + PENDING_ENVVAR))
TOTAL=$((MIGRATED + PENDING))

if [ $TOTAL -gt 0 ]; then
    PERCENTAGE=$(echo "scale=2; $MIGRATED * 100 / $TOTAL" | bc)
else
    PERCENTAGE=0
fi

echo "已迁移（使用隔离工具）: $MIGRATED"
echo "待迁移（TempDir）: $PENDING_TEMPDIR"
echo "待迁移（set_current_dir）: $PENDING_SETDIR"
echo "待迁移（env::set_var）: $PENDING_ENVVAR"
echo "待迁移总计: $PENDING"
echo "总计: $TOTAL"
echo "完成度: $PERCENTAGE%"
echo ""

# 列出待迁移的文件
echo "=== 待迁移文件列表 ==="
ALL_FILES=$(cat <(grep -rn "set_current_dir" tests/ --include="*.rs" | grep -v "CurrentDirGuard\|common/helpers.rs" | awk -F: '{print $1}') \
  <(grep -rn "tempfile::tempdir\|TempDir" tests/ --include="*.rs" | grep -v "TestIsolation\|CliTestEnv\|GitTestEnv\|CurrentDirGuard\|common/isolation.rs\|common/environments" | awk -F: '{print $1}') \
  <(grep -rn "env::set_var\|std::env::set_var" tests/ --include="*.rs" | grep -v "EnvGuard\|MockServer\|common/guards\|common/http_helpers" | awk -F: '{print $1}') \
  | sort -u)

echo "$ALL_FILES" | head -20
if [ $(echo "$ALL_FILES" | wc -l) -gt 20 ]; then
    echo "... (还有 $(($(echo "$ALL_FILES" | wc -l) - 20)) 个文件)"
fi

