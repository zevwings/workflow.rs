# make/Makefile.build.mk
# 构建和安装模块

# 项目名称
BINARY_NAME = workflow

# Help 信息
define HELP_BUILD
	@echo "构建相关："
	@echo "  make dev              - 构建 debug 版本"
	@echo "  make release          - 构建 release 版本（打包）"
	@echo "  make clean            - 清理构建产物"
	@echo ""
	@echo "安装相关："
	@echo "  make install          - 一次性安装全部（构建 + 安装二进制 + 安装 completion）"
	@echo "  make update           - 更新 Workflow CLI（重新构建 + 更新二进制 + 更新 completion）"
	@echo "  make uninstall        - 卸载二进制文件和 shell completion 脚本"
	@echo ""
endef

# ============================================
# 构建相关目标
# ============================================

# 构建 debug 版本
dev:
	@echo "构建 debug 版本..."
	cargo build
	@echo "构建完成: target/debug/$(BINARY_NAME)"

# 构建 release 版本（打包）
release:
	@echo "构建 release 版本..."
	cargo build --release
	@echo "打包完成: target/release/$(BINARY_NAME)"
	@echo "二进制文件:"
	@ls -lh target/release/{workflow,install} 2>/dev/null || ls -lh target/release/$(BINARY_NAME)

# 清理构建产物
clean:
	@echo "清理构建产物..."
	cargo clean
	@echo "清理完成"

# ============================================
# 安装和部署相关目标
# ============================================

# 安装到系统（默认 /usr/local/bin，一次性安装全部）
# 直接调用 cargo build --release 确保总是重新构建并安装最新代码
# 清理增量编译缓存以确保使用最新代码
install:
	@echo "构建 release 版本（确保使用最新代码）..."
	@echo "清理增量编译缓存以确保重新编译..."
	@rm -rf target/release/incremental/*/$(BINARY_NAME)-* 2>/dev/null || true
	@CARGO_INCREMENTAL=0 cargo build --release || cargo build --release
	@echo ""
	@echo "安装 Workflow CLI (二进制文件 + shell completion)..."
	@./target/release/install

# 更新 Workflow CLI（重新构建 + 更新二进制 + 更新 completion）
# 使用 workflow update 命令
update:
	@echo "更新 Workflow CLI..."
	@cargo build --release --bin $(BINARY_NAME)
	@./target/release/$(BINARY_NAME) update

# 卸载二进制文件和 shell completion 脚本（一次性清理全部）
uninstall:
	@cargo run --bin $(BINARY_NAME) -- uninstall

