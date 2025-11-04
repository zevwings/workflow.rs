.PHONY: help build release clean install test lint setup generate-completions install-completions

# 项目名称
BINARY_NAME = workflow

# 默认目标
.DEFAULT_GOAL := help

# 检查 rustfmt 是否安装
check-rustfmt:
	@command -v cargo-fmt >/dev/null 2>&1 || rustup component list --toolchain stable | grep -q "rustfmt.*installed" || (echo "错误: rustfmt 未安装。运行 'make setup' 或 'rustup component add rustfmt'" && exit 1)

# 检查 clippy 是否安装
check-clippy:
	@command -v cargo-clippy >/dev/null 2>&1 || rustup component list --toolchain stable | grep -q "clippy.*installed" || (echo "错误: clippy 未安装。运行 'make setup' 或 'rustup component add clippy'" && exit 1)

# 显示帮助信息
help:
	@echo "可用的 Make 目标："
	@echo "  make build              - 构建 debug 版本"
	@echo "  make release            - 构建 release 版本（打包）"
	@echo "  make clean              - 清理构建产物"
	@echo "  make install            - 安装到系统 (默认: /usr/local/bin)"
	@echo "  make test               - 运行测试"
	@echo "  make lint               - 运行完整的代码检查（格式化 + Clippy + Check）"
	@echo "  make setup              - 安装所需的开发工具（rustfmt, clippy）"
	@echo "  make generate-completions - 生成 shell completion 脚本"
	@echo "  make install-completions  - 安装 shell completion 脚本（自动检测 shell）"

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
	@ls -lh target/release/{workflow,pr,qk} 2>/dev/null || ls -lh target/release/$(BINARY_NAME)

# 清理构建产物
clean:
	@echo "清理构建产物..."
	cargo clean
	@echo "清理完成"

# 安装到系统（默认 /usr/local/bin）
install: release
	@echo "安装所有二进制文件到 /usr/local/bin..."
	@sudo cp target/release/workflow /usr/local/bin/ || echo "警告: workflow 安装失败"
	@sudo cp target/release/pr /usr/local/bin/ || echo "警告: pr 安装失败"
	@sudo cp target/release/qk /usr/local/bin/ || echo "警告: qk 安装失败"
	@echo "安装完成"
	@echo ""
	@echo "已安装的命令:"
	@echo "  - workflow (主命令)"
	@echo "  - pr (PR 操作命令)"
	@echo "  - qk (快速日志操作命令)"
	@echo ""
	@echo "⚠️  重要提示："
	@echo "  如果命令无法识别，请运行 'hash -r' 清除 shell 缓存，或重启终端"
	@echo "  验证安装: which pr (应该显示 /usr/local/bin/pr)"

# 运行测试
test:
	@echo "运行测试..."
	cargo test

# 安装所需的开发工具
setup:
	@echo "安装开发工具..."
	@rustup component add rustfmt 2>/dev/null || echo "rustfmt 已安装或安装失败"
	@rustup component add clippy 2>/dev/null || echo "clippy 已安装或安装失败"
	@echo "开发工具安装完成"

# 运行完整的代码检查（格式化检查 + Clippy + Check）
lint: check-rustfmt check-clippy
	@echo "运行完整的代码检查..."
	@echo ""
	@echo "1/3 检查代码格式..."
	@cargo fmt --check || (echo "✗ 代码格式不正确，运行 'cargo fmt' 自动修复" && exit 1)
	@echo "✓ 代码格式正确"
	@echo ""
	@echo "2/3 运行 Clippy 检查..."
	@cargo clippy -- -D warnings
	@echo "✓ Clippy 检查通过"
	@echo ""
	@echo "3/3 运行 cargo check..."
	@cargo check
	@echo "✓ Check 通过"
	@echo ""
	@echo "✓ 所有检查通过！"

# 生成 shell completion 脚本
generate-completions:
	@echo "生成 shell completion 脚本..."
	@if [ -z "$(SHELL_TYPE)" ]; then \
		echo "用法: make generate-completions SHELL_TYPE=<shell> OUTPUT_DIR=<dir>"; \
		echo "支持的 shell: zsh, bash, fish, powershell, elvish"; \
		echo ""; \
		echo "示例:"; \
		echo "  make generate-completions SHELL_TYPE=zsh OUTPUT_DIR=~/.zsh/completions"; \
		echo "  make generate-completions SHELL_TYPE=bash OUTPUT_DIR=~/.bash_completion.d"; \
		exit 1; \
	fi
	@if [ -z "$(OUTPUT_DIR)" ]; then \
		echo "错误: 请指定 OUTPUT_DIR"; \
		exit 1; \
	fi
	@cargo run --bin generate-completions -- $(SHELL_TYPE) $(OUTPUT_DIR)

# 安装 shell completion 脚本（自动检测 shell）
install-completions:
	@echo "检测 shell 类型..."
	@SHELL_TYPE=$$(basename $$SHELL); \
	if [ "$$SHELL_TYPE" = "zsh" ]; then \
		COMPLETION_DIR=$$HOME/.zsh/completions; \
		CONFIG_FILE=$$HOME/.zshrc; \
		echo "检测到 zsh，completion 目录: $$COMPLETION_DIR"; \
		mkdir -p $$COMPLETION_DIR; \
		cargo run --bin generate-completions -- zsh $$COMPLETION_DIR || exit 1; \
		if ! grep -q "fpath=($$COMPLETION_DIR \$$fpath)" $$CONFIG_FILE 2>/dev/null; then \
			echo "" >> $$CONFIG_FILE; \
			echo "# Workflow CLI completions" >> $$CONFIG_FILE; \
			echo "fpath=($$COMPLETION_DIR \$$fpath)" >> $$CONFIG_FILE; \
			echo "autoload -Uz compinit && compinit" >> $$CONFIG_FILE; \
			echo "✅ 已将 completion 配置添加到 $$CONFIG_FILE"; \
		else \
			echo "✅ completion 配置已存在于 $$CONFIG_FILE"; \
		fi; \
		echo ""; \
		echo "请运行以下命令重新加载配置:"; \
		echo "  source $$CONFIG_FILE"; \
	elif [ "$$SHELL_TYPE" = "bash" ]; then \
		COMPLETION_DIR=$$HOME/.bash_completion.d; \
		CONFIG_FILE=$$HOME/.bashrc; \
		echo "检测到 bash，completion 目录: $$COMPLETION_DIR"; \
		mkdir -p $$COMPLETION_DIR; \
		cargo run --bin generate-completions -- bash $$COMPLETION_DIR || exit 1; \
		if ! grep -q "source $$COMPLETION_DIR" $$CONFIG_FILE 2>/dev/null; then \
			echo "" >> $$CONFIG_FILE; \
			echo "# Workflow CLI completions" >> $$CONFIG_FILE; \
			echo "for f in $$COMPLETION_DIR/*.bash; do source \"\$$f\"; done" >> $$CONFIG_FILE; \
			echo "✅ 已将 completion 配置添加到 $$CONFIG_FILE"; \
		else \
			echo "✅ completion 配置已存在于 $$CONFIG_FILE"; \
		fi; \
		echo ""; \
		echo "请运行以下命令重新加载配置:"; \
		echo "  source $$CONFIG_FILE"; \
	else \
		echo "未支持的 shell: $$SHELL_TYPE"; \
		echo "请手动运行: make generate-completions SHELL_TYPE=<shell> OUTPUT_DIR=<dir>"; \
		exit 1; \
	fi

