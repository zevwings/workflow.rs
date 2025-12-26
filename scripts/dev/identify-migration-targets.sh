#!/bin/bash
# 识别需要迁移的测试文件

set -e

echo "=== 查找使用 set_current_dir 的测试 ==="
grep -rn "set_current_dir" tests/ --include="*.rs" | \
  awk -F: '{print $1}' | sort -u

echo -e "\n=== 查找使用 TempDir 但未使用隔离工具的测试 ==="
grep -rn "tempfile::tempdir\|TempDir" tests/ --include="*.rs" | \
  grep -v "TestIsolation\|CliTestEnv\|GitTestEnv\|CurrentDirGuard\|common/isolation.rs\|common/environments" | \
  awk -F: '{print $1}' | sort -u

echo -e "\n=== 查找手动设置环境变量的测试 ==="
grep -rn "env::set_var\|std::env::set_var" tests/ --include="*.rs" | \
  grep -v "EnvGuard\|MockServer\|common/guards\|common/http_helpers" | \
  awk -F: '{print $1}' | sort -u

echo -e "\n=== 统计待迁移文件 ==="
ALL_FILES=$(cat <(grep -rn "set_current_dir" tests/ --include="*.rs" | awk -F: '{print $1}') \
  <(grep -rn "tempfile::tempdir\|TempDir" tests/ --include="*.rs" | grep -v "TestIsolation\|CliTestEnv\|GitTestEnv\|CurrentDirGuard\|common/isolation.rs\|common/environments" | awk -F: '{print $1}') \
  <(grep -rn "env::set_var\|std::env::set_var" tests/ --include="*.rs" | grep -v "EnvGuard\|MockServer\|common/guards\|common/http_helpers" | awk -F: '{print $1}') \
  | sort -u)

TOTAL=$(echo "$ALL_FILES" | wc -l | tr -d ' ')
echo "总计: $TOTAL 个文件需要迁移"

