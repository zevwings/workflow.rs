#!/bin/bash
# 连续运行测试N次，验证稳定性

set -e

RUNS=${1:-100}
FAILED_RUNS=0
PASSED_RUNS=0
LOG_DIR="test_runs_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$LOG_DIR"

echo "=== 开始连续运行测试 $RUNS 次 ==="
echo "日志目录: $LOG_DIR"
echo ""

for i in $(seq 1 $RUNS); do
    echo "[$i/$RUNS] 运行测试..."

    if cargo test --all --no-fail-fast > "$LOG_DIR/run_$i.log" 2>&1; then
        PASSED_RUNS=$((PASSED_RUNS + 1))
        echo "  ✅ 通过"
    else
        FAILED_RUNS=$((FAILED_RUNS + 1))
        echo "  ❌ 失败"
        echo "=== Run $i Failed ===" >> "$LOG_DIR/failures.log"
        tail -50 "$LOG_DIR/run_$i.log" >> "$LOG_DIR/failures.log"
        echo "" >> "$LOG_DIR/failures.log"
    fi

    if [ $((i % 10)) -eq 0 ]; then
        echo "  进度: $PASSED_RUNS 通过, $FAILED_RUNS 失败"
    fi
done

echo ""
echo "=== 测试完成 ==="
echo "总运行次数: $RUNS"
echo "通过: $PASSED_RUNS"
echo "失败: $FAILED_RUNS"

if [ $RUNS -gt 0 ]; then
    SUCCESS_RATE=$(echo "scale=2; $PASSED_RUNS * 100 / $RUNS" | bc)
    echo "成功率: $SUCCESS_RATE%"
fi

if [ $FAILED_RUNS -eq 0 ]; then
    echo "✅ 所有测试运行都通过！"
    exit 0
else
    echo "❌ 有 $FAILED_RUNS 次运行失败，请查看 $LOG_DIR/failures.log"
    exit 1
fi

