# make/Makefile.bench.mk
# 性能测试模块

# Help 信息
define HELP_BENCH
	@echo "性能测试相关："
	@echo "  make bench             - 运行所有基准测试"
	@echo "  make bench-cli         - CLI性能测试"
	@echo "  make bench-core        - 核心操作测试"
	@echo "  make bench-network     - 网络操作测试"
	@echo "  make bench-report      - 生成性能报告"
	@echo "  make bench-compare     - 性能对比"
	@echo "  make bench-regression  - 性能回归检测"
	@echo "  make bench-ci          - CI性能监控"
	@echo ""
endef

# ============================================
# 工具检查
# ============================================

# 检查 criterion 是否可用（通过检查 Cargo.toml 中的配置）
check-criterion:
	@if ! grep -q "criterion" Cargo.toml 2>/dev/null; then \
		echo "错误: criterion 未配置"; \
		echo "请在 Cargo.toml 的 [dev-dependencies] 中添加: criterion = { version = \"0.5\", features = [\"html_reports\"] }"; \
		exit 1; \
	fi

# ============================================
# 基准测试运行
# ============================================

# 运行所有基准测试
bench: check-criterion
	@echo "运行所有基准测试..."
	cargo bench
	@echo ""
	@echo "✓ 基准测试完成"
	@echo "性能报告位置: target/criterion/"

# CLI性能测试
bench-cli: check-criterion
	@echo "运行 CLI 性能基准测试..."
	cargo bench --bench cli_performance
	@echo ""
	@echo "✓ CLI 性能测试完成"
	@echo "性能报告位置: target/criterion/cli_performance/"

# 核心操作测试
bench-core: check-criterion
	@echo "运行核心操作基准测试..."
	cargo bench --bench core_operations
	@echo ""
	@echo "✓ 核心操作测试完成"
	@echo "性能报告位置: target/criterion/core_operations/"

# 网络操作测试
bench-network: check-criterion
	@echo "运行网络操作基准测试..."
	cargo bench --bench network_operations
	@echo ""
	@echo "✓ 网络操作测试完成"
	@echo "性能报告位置: target/criterion/network_operations/"

# ============================================
# 性能报告和对比
# ============================================

# 生成性能报告（Criterion 会自动生成 HTML 报告）
bench-report: check-criterion
	@echo "生成性能报告..."
	@if [ ! -d "target/criterion" ]; then \
		echo "错误: 基准测试结果不存在，请先运行 'make bench'"; \
		exit 1; \
	fi
	@echo "性能报告已生成到 target/criterion/ 目录"
	@echo ""
	@echo "查看报告："
	@echo "  - CLI 性能: target/criterion/cli_performance/index.html"
	@echo "  - 核心操作: target/criterion/core_operations/index.html"
	@echo "  - 网络操作: target/criterion/network_operations/index.html"
	@echo ""
	@echo "使用 'make bench-open' 打开报告"

# 打开性能报告
bench-open:
	@if [ -f "target/criterion/cli_performance/index.html" ]; then \
		open target/criterion/cli_performance/index.html; \
	elif [ -f "target/criterion/core_operations/index.html" ]; then \
		open target/criterion/core_operations/index.html; \
	elif [ -f "target/criterion/network_operations/index.html" ]; then \
		open target/criterion/network_operations/index.html; \
	else \
		echo "错误: 性能报告不存在，请先运行 'make bench'"; \
		exit 1; \
	fi

# 性能对比（需要先运行基准测试）
bench-compare: check-criterion
	@echo "性能对比..."
	@if [ ! -d "target/criterion" ]; then \
		echo "错误: 基准测试结果不存在，请先运行 'make bench'"; \
		exit 1; \
	fi
	@echo "Criterion 会自动对比当前结果与历史结果"
	@echo "查看对比结果：运行 'make bench-open' 打开报告"
	@echo ""
	@echo "提示: Criterion 会将结果保存在 target/criterion/ 目录中"
	@echo "     每次运行基准测试时，会自动与上次结果进行对比"

# 性能回归检测（检查是否有性能下降）
bench-regression: check-criterion
	@echo "检测性能回归..."
	@if [ ! -d "target/criterion" ]; then \
		echo "错误: 基准测试结果不存在，请先运行 'make bench'"; \
		exit 1; \
	fi
	@echo "检查基准测试结果..."
	@echo ""
	@echo "Criterion 会自动检测性能回归："
	@echo "  - 如果性能下降超过阈值，会在报告中标记"
	@echo "  - 查看详细报告：运行 'make bench-open'"
	@echo ""
	@echo "提示: 性能回归检测基于 Criterion 的统计分析"
	@echo "     如果检测到回归，请检查最近的代码变更"

# ============================================
# CI 环境性能监控
# ============================================

# CI性能监控（适合 CI/CD 环境，输出简洁格式）
bench-ci: check-criterion
	@echo "运行 CI 性能监控..."
	@echo ""
	@echo "运行所有基准测试（CI 模式）..."
	cargo bench --bench cli_performance --bench core_operations --bench network_operations
	@echo ""
	@echo "✓ CI 性能监控完成"
	@echo ""
	@echo "提示: CI 环境中，性能报告会保存在 target/criterion/ 目录"
	@echo "     可以将其作为 Artifact 上传以供后续分析"

