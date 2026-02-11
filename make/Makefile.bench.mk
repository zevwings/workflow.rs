# make/Makefile.bench.mk
# 性能测试模块（轻量 CLI：CLI 启动 + Storage）

# Help 信息
define HELP_BENCH
	@echo "性能测试相关："
	@echo "  make bench             - 运行所有基准测试（CLI + Storage）"
	@echo "  make bench-cli         - CLI 启动性能测试"
	@echo "  make bench-storage     - Storage/Git 操作性能测试"
	@echo "  make bench-report      - 生成性能报告"
	@echo "  make bench-compare     - 性能对比"
	@echo "  make bench-regression  - 性能回归检测"
	@echo "  make bench-open        - 打开性能报告"
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

# 运行所有基准测试（CLI + Storage）
bench: check-criterion
	@echo "运行所有基准测试..."
	cargo build --release -p app
	cargo bench -p app --bench cli_startup
	cargo bench -p storage
	@echo ""
	@echo "✓ 基准测试完成"
	@echo "性能报告位置: target/criterion/"

# CLI 启动性能测试
bench-cli: check-criterion
	@echo "运行 CLI 性能基准测试..."
	cargo build --release -p app
	cargo bench -p app --bench cli_startup
	@echo ""
	@echo "✓ CLI 性能测试完成"
	@echo "性能报告位置: target/criterion/cli_startup/"

# Storage/Git 操作性能测试
bench-storage: check-criterion
	@echo "运行 Storage 性能基准测试..."
	cargo bench -p storage
	@echo ""
	@echo "✓ Storage 性能测试完成"
	@echo "性能报告位置: target/criterion/"

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
	@echo "  - CLI 启动: target/criterion/cli_startup/"
	@echo "  - Storage: target/criterion/"
	@echo ""
	@echo "使用 'make bench-open' 打开报告"

# 打开性能报告
bench-open:
	@if [ -d "target/criterion/cli_startup" ]; then \
		find target/criterion/cli_startup -name "index.html" -path "*/report/*" | head -1 | xargs -I {} open {} 2>/dev/null || open target/criterion/; \
	elif [ -d "target/criterion" ]; then \
		find target/criterion -name "index.html" -path "*/report/*" | head -1 | xargs -I {} open {} 2>/dev/null || open target/criterion/; \
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
