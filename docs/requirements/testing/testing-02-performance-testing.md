# 系统化性能测试实施指南

> **目标**: 建立全面的性能基准测试体系，监控性能回归
> **优先级**: 🟡 P1 (中高)
> **预计时间**: 2-3 天
> **依赖**: 无

---

## 🎯 目标和范围

### 实施目标
1. ✅ 创建根级别 `benches/` 目录结构
2. ✅ 实现 CLI 性能基准测试
3. ✅ 实现核心操作性能测试
4. ✅ 实现网络操作性能测试
5. ✅ 配置 Criterion 基准测试框架
6. ✅ 创建性能回归检测机制
7. ✅ 更新 Makefile 性能测试命令

### 产出物
```
workflow.rs/
├── benches/
│   ├── cli_performance.rs          # CLI 性能测试
│   ├── core_operations.rs          # 核心操作测试
│   └── network_operations.rs       # 网络操作测试
├── scripts/
│   ├── check_performance_regression.py   # 性能回归检测
│   └── compare_benchmarks.py             # 基准对比
├── Cargo.toml                       # 添加 bench 配置
└── make/Makefile.bench.mk           # 更新性能测试命令
```

---

## 📊 当前状态

### ✅ 已有基础
- ✅ `crates/storage/benches/git_services_bench.rs` - Storage 层性能测试
- ✅ `make/Makefile.bench.mk` - 基础 bench 命令
- ✅ Criterion 已在 workspace 依赖中配置

### ❌ 缺失部分
- ❌ 没有根级别 `benches/` 目录
- ❌ 没有 CLI 启动时间测试
- ❌ 没有命令解析性能测试
- ❌ 没有核心业务操作性能测试
- ❌ 没有网络操作性能测试
- ❌ 没有性能回归检测机制
- ❌ 没有性能基准对比工具

---

## 📋 前置条件

### 系统要求
```bash
# 1. 确认 Criterion 可用
cargo bench --version

# 2. 确认 Python 3 可用（用于分析脚本）
python3 --version

# 3. 确认项目可以编译
cargo build --release
```

### 知识准备
- 了解 Criterion 基准测试框架
- 了解性能测试的基本原则
- 了解如何避免编译器优化影响测试结果

---

## 🔨 详细实施步骤

### Step 1: 配置项目基准测试 (15 分钟)

#### 1.1 创建 benches 目录

```bash
mkdir -p benches
```

#### 1.2 更新根 `Cargo.toml`

在根 `Cargo.toml` 中添加基准测试配置：

```toml
# 在文件末尾添加

#------------------------------------------------------------------------------
# 基准测试配置
#------------------------------------------------------------------------------

[[bench]]
name = "cli_performance"
harness = false
path = "benches/cli_performance.rs"

[[bench]]
name = "core_operations"
harness = false
path = "benches/core_operations.rs"

[[bench]]
name = "network_operations"
harness = false
path = "benches/network_operations.rs"

# 基准测试依赖
[dev-dependencies]
criterion = { workspace = true, features = ["html_reports"] }
```

#### 1.3 确认 workspace 中的 Criterion 配置

检查 `Cargo.toml` 的 `[workspace.dependencies]` 部分：

```toml
[workspace.dependencies]
# 其他依赖...
criterion = { version = "0.5", features = ["html_reports"] }
```

如果没有，添加上述配置。

#### ✅ **验证 Step 1**
```bash
# 验证目录存在
ls -la benches/

# 验证 Cargo.toml 配置
grep -A 5 "^\[\[bench\]\]" Cargo.toml

# 验证 Criterion 可用
cargo bench --no-run 2>&1 | head -5
```

---

### Step 2: 创建 CLI 性能基准测试 (45 分钟)

#### 2.1 创建 `benches/cli_performance.rs`

这个测试用于测量 CLI 启动时间、命令解析等性能：

```rust
//! CLI 性能基准测试
//!
//! 测试内容:
//! - CLI 启动时间
//! - 命令解析性能
//! - 帮助信息生成速度
//! - 版本信息获取速度

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::process::Command;
use std::time::Duration;

/// 获取编译后的 CLI 二进制路径
fn get_cli_binary() -> String {
    // 假设二进制名称是 "workflow" 或 "wf"
    // 需要先构建: cargo build --release
    let binary_name = env!("CARGO_PKG_NAME");
    format!("target/release/{}", binary_name)
}

/// 测试 CLI 启动时间
///
/// 这是最重要的性能指标之一，用户体验直接受其影响
fn bench_cli_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("cli_startup");

    // 设置测量时间
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    // 测试基本启动（无参数）
    group.bench_function("no_args", |b| {
        b.iter(|| {
            let output = Command::new(get_cli_binary())
                .output()
                .expect("Failed to execute CLI");
            black_box(output);
        });
    });

    // 测试 --help 命令
    group.bench_function("help", |b| {
        b.iter(|| {
            let output = Command::new(get_cli_binary())
                .arg("--help")
                .output()
                .expect("Failed to execute CLI --help");
            black_box(output);
        });
    });

    // 测试 --version 命令
    group.bench_function("version", |b| {
        b.iter(|| {
            let output = Command::new(get_cli_binary())
                .arg("--version")
                .output()
                .expect("Failed to execute CLI --version");
            black_box(output);
        });
    });

    group.finish();
}

/// 测试子命令解析性能
fn bench_subcommand_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("subcommand_parsing");

    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    // 测试各个子命令的解析速度
    let subcommands = vec![
        "branch",
        "commit",
        "pr",
        "issue",
        "status",
    ];

    for subcommand in subcommands {
        group.bench_with_input(
            BenchmarkId::new("help", subcommand),
            &subcommand,
            |b, &cmd| {
                b.iter(|| {
                    let output = Command::new(get_cli_binary())
                        .arg(cmd)
                        .arg("--help")
                        .output()
                        .expect("Failed to execute subcommand --help");
                    black_box(output);
                });
            },
        );
    }

    group.finish();
}

/// 测试错误处理性能
///
/// 确保错误路径不会造成性能问题
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    group.measurement_time(Duration::from_secs(5));
    group.sample_size(30);

    // 测试无效命令
    group.bench_function("invalid_command", |b| {
        b.iter(|| {
            let output = Command::new(get_cli_binary())
                .arg("nonexistent-command")
                .output()
                .expect("Failed to execute CLI");
            black_box(output);
        });
    });

    // 测试无效参数
    group.bench_function("invalid_arg", |b| {
        b.iter(|| {
            let output = Command::new(get_cli_binary())
                .arg("branch")
                .arg("--nonexistent-flag")
                .output()
                .expect("Failed to execute CLI");
            black_box(output);
        });
    });

    group.finish();
}

/// 测试配置文件加载性能
fn bench_config_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_loading");

    group.measurement_time(Duration::from_secs(5));
    group.sample_size(30);

    // 注意: 这需要实际的配置文件加载代码
    // 如果当前没有配置文件，可以先跳过这个测试

    // 示例: 测试带配置文件的命令执行
    // group.bench_function("with_config", |b| {
    //     b.iter(|| {
    //         // 执行需要加载配置的命令
    //     });
    // });

    // 临时占位测试
    group.bench_function("placeholder", |b| {
        b.iter(|| {
            black_box(1 + 1);
        });
    });

    group.finish();
}

// 组合所有基准测试
criterion_group!(
    cli_benches,
    bench_cli_startup,
    bench_subcommand_parsing,
    bench_error_handling,
    bench_config_loading,
);

criterion_main!(cli_benches);
```

#### 2.2 准备测试前的构建

```bash
# 构建 release 版本（基准测试需要）
cargo build --release
```

#### 2.3 运行 CLI 性能测试

```bash
# 运行 CLI 性能测试
cargo bench --bench cli_performance

# 查看结果
open target/criterion/report/index.html
```

#### ✅ **验证 Step 2**
```bash
# 验证测试文件存在
ls -la benches/cli_performance.rs

# 验证可以编译
cargo bench --bench cli_performance --no-run

# 运行测试（可选，因为比较慢）
# cargo bench --bench cli_performance
```

---

### Step 3: 创建核心操作性能测试 (45 分钟)

#### 3.1 创建 `benches/core_operations.rs`

这个测试用于测量核心业务操作的性能：

```rust
//! 核心操作性能基准测试
//!
//! 测试内容:
//! - 字符串操作性能
//! - 数据结构操作性能
//! - 解析和序列化性能
//! - 算法性能

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// 测试字符串操作性能
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    // 测试分支名称格式化
    group.bench_function("branch_format", |b| {
        b.iter(|| {
            let branch = black_box("feature/PROJ-123-add-new-feature");
            let _result = format!("refs/heads/{}", branch);
        });
    });

    // 测试分支名称解析
    group.bench_function("branch_parse", |b| {
        b.iter(|| {
            let branch = black_box("feature/PROJ-123-add-new-feature");
            let parts: Vec<&str> = branch.split('/').collect();
            black_box(parts);
        });
    });

    // 测试提取 Jira Issue Key
    group.bench_function("extract_jira_key", |b| {
        b.iter(|| {
            let branch = black_box("feature/PROJ-123-add-new-feature");
            let parts: Vec<&str> = branch.split('/').collect();
            if parts.len() >= 2 {
                let key_part = parts[1];
                let key: Vec<&str> = key_part.split('-').take(2).collect();
                black_box(key.join("-"));
            }
        });
    });

    // 测试 PR 标题格式化
    group.bench_function("pr_title_format", |b| {
        b.iter(|| {
            let issue_key = black_box("PROJ-123");
            let description = black_box("Add new feature");
            let _result = format!("[{}] {}", issue_key, description);
        });
    });

    group.finish();
}

/// 测试数据结构操作性能
fn bench_data_structure_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_structures");

    // 测试 Vec 操作
    group.bench_function("vec_push_100", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..100 {
                vec.push(black_box(i));
            }
            black_box(vec);
        });
    });

    // 测试 HashMap 操作
    group.bench_function("hashmap_insert_100", |b| {
        b.iter(|| {
            use std::collections::HashMap;
            let mut map = HashMap::new();
            for i in 0..100 {
                map.insert(black_box(format!("key_{}", i)), black_box(i));
            }
            black_box(map);
        });
    });

    // 测试字符串拼接（不同方法对比）
    group.bench_function("string_concat_format", |b| {
        b.iter(|| {
            let mut result = String::new();
            for i in 0..10 {
                result = format!("{}{}", result, black_box(i));
            }
            black_box(result);
        });
    });

    group.bench_function("string_concat_push_str", |b| {
        b.iter(|| {
            let mut result = String::new();
            for i in 0..10 {
                result.push_str(&black_box(i).to_string());
            }
            black_box(result);
        });
    });

    group.finish();
}

/// 测试序列化/反序列化性能
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    use serde_json::{json, Value};

    // 测试 JSON 序列化
    let data = json!({
        "title": "Test PR",
        "body": "This is a test pull request",
        "head": "feature/test",
        "base": "main",
        "state": "open",
        "number": 123,
        "user": {
            "login": "testuser",
            "id": 1
        }
    });

    group.bench_function("json_serialize", |b| {
        b.iter(|| {
            let s = serde_json::to_string(black_box(&data)).unwrap();
            black_box(s);
        });
    });

    // 测试 JSON 反序列化
    let json_str = serde_json::to_string(&data).unwrap();

    group.bench_function("json_deserialize", |b| {
        b.iter(|| {
            let v: Value = serde_json::from_str(black_box(&json_str)).unwrap();
            black_box(v);
        });
    });

    group.finish();
}

/// 测试算法性能
fn bench_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("algorithms");

    // 测试排序
    group.bench_function("sort_100_strings", |b| {
        let mut data: Vec<String> = (0..100)
            .map(|i| format!("branch_{}", i))
            .collect();

        b.iter(|| {
            let mut vec = data.clone();
            vec.sort();
            black_box(vec);
        });
    });

    // 测试过滤
    group.bench_function("filter_100_strings", |b| {
        let data: Vec<String> = (0..100)
            .map(|i| format!("feature_{}", i))
            .collect();

        b.iter(|| {
            let filtered: Vec<_> = data
                .iter()
                .filter(|s| black_box(s.starts_with("feature_1")))
                .collect();
            black_box(filtered);
        });
    });

    // 测试映射
    group.bench_function("map_100_strings", |b| {
        let data: Vec<String> = (0..100)
            .map(|i| format!("branch_{}", i))
            .collect();

        b.iter(|| {
            let mapped: Vec<_> = data
                .iter()
                .map(|s| black_box(s.to_uppercase()))
                .collect();
            black_box(mapped);
        });
    });

    group.finish();
}

criterion_group!(
    core_benches,
    bench_string_operations,
    bench_data_structure_operations,
    bench_serialization,
    bench_algorithms,
);

criterion_main!(core_benches);
```

#### 3.2 运行核心操作性能测试

```bash
# 运行核心操作测试
cargo bench --bench core_operations
```

#### ✅ **验证 Step 3**
```bash
# 验证测试文件存在
ls -la benches/core_operations.rs

# 验证可以编译
cargo bench --bench core_operations --no-run
```

---

### Step 4: 创建网络操作性能测试 (45 分钟)

#### 4.1 创建 `benches/network_operations.rs`

这个测试用于测量网络相关操作的性能：

```rust
//! 网络操作性能基准测试
//!
//! 测试内容:
//! - HTTP 请求性能
//! - Mock 服务器响应速度
//! - 重试机制性能
//! - 超时处理性能

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// 注意: 这些测试需要 http crate 的 testing feature
#[cfg(feature = "http-testing")]
use http::testing::{MockServerManager, TestDataFactory};

/// 测试 Mock 服务器性能
#[cfg(feature = "http-testing")]
fn bench_mock_server(c: &mut Criterion) {
    let mut group = c.benchmark_group("mock_server");

    // 测试 Mock 服务器创建速度
    group.bench_function("create_server", |b| {
        b.iter(|| {
            let manager = MockServerManager::new();
            black_box(manager);
        });
    });

    // 测试 Mock 配置速度
    group.bench_function("setup_mock", |b| {
        let mut manager = MockServerManager::new();
        let pr_data = TestDataFactory::github_pr().build();

        b.iter(|| {
            let _mock = manager.setup_github_pr_list(vec![pr_data.clone()]);
            black_box(_mock);
        });
    });

    group.finish();
}

/// 测试 HTTP 客户端性能
fn bench_http_client(c: &mut Criterion) {
    let mut group = c.benchmark_group("http_client");

    // 测试简单的 HTTP GET（需要真实的服务器或 mock）
    // 这里使用 httpbin.org 作为测试服务器

    // 注意: 实际基准测试不应该依赖外部服务
    // 这里仅作为示例，生产环境应该使用 mock

    group.bench_function("simple_get_request", |b| {
        b.iter(|| {
            // 使用 reqwest 的阻塞客户端
            // 注意: 这需要网络连接，不适合 CI 环境
            // 在实际使用中应该替换为 mock 服务器

            // 临时占位测试
            black_box(1 + 1);
        });
    });

    group.finish();
}

/// 测试数据工厂性能
#[cfg(feature = "http-testing")]
fn bench_test_data_factory(c: &mut Criterion) {
    let mut group = c.benchmark_group("test_data_factory");

    // 测试 GitHub PR 构建速度
    group.bench_function("build_github_pr", |b| {
        b.iter(|| {
            let pr = TestDataFactory::github_pr()
                .with_title("Test PR")
                .with_head("feature")
                .with_base("main")
                .build();
            black_box(pr);
        });
    });

    // 测试 Jira Issue 构建速度
    group.bench_function("build_jira_issue", |b| {
        b.iter(|| {
            let issue = TestDataFactory::jira_issue()
                .with_summary("Test Issue")
                .with_issue_type("Task")
                .build();
            black_box(issue);
        });
    });

    group.finish();
}

/// 测试重试逻辑性能
fn bench_retry_logic(c: &mut Criterion) {
    let mut group = c.benchmark_group("retry_logic");

    // 模拟重试逻辑（立即成功）
    group.bench_function("immediate_success", |b| {
        b.iter(|| {
            let result = (|| -> Result<(), ()> {
                Ok(())
            })();
            black_box(result);
        });
    });

    // 模拟重试逻辑（第二次成功）
    group.bench_function("retry_once", |b| {
        b.iter(|| {
            let mut attempt = 0;
            let result = loop {
                attempt += 1;
                if attempt >= 2 {
                    break Ok(());
                }
            };
            black_box(result);
        });
    });

    group.finish();
}

// 根据 feature 有条件地组合基准测试
#[cfg(feature = "http-testing")]
criterion_group!(
    network_benches,
    bench_mock_server,
    bench_http_client,
    bench_test_data_factory,
    bench_retry_logic,
);

#[cfg(not(feature = "http-testing"))]
criterion_group!(
    network_benches,
    bench_http_client,
    bench_retry_logic,
);

criterion_main!(network_benches);
```

#### 4.2 运行网络操作性能测试

```bash
# 不带 http-testing feature（基础测试）
cargo bench --bench network_operations

# 带 http-testing feature（完整测试）
cargo bench --bench network_operations --features http/testing
```

#### ✅ **验证 Step 4**
```bash
# 验证测试文件存在
ls -la benches/network_operations.rs

# 验证可以编译
cargo bench --bench network_operations --no-run
```

---

### Step 5: 更新 Makefile 性能测试命令 (30 分钟)

#### 5.1 备份现有 Makefile

```bash
cp make/Makefile.bench.mk make/Makefile.bench.mk.backup
```

#### 5.2 增强 `make/Makefile.bench.mk`

添加详细的性能测试命令：

```makefile
# 在文件末尾添加以下内容

#------------------------------------------------------------------------------
# 性能基准测试增强命令
#------------------------------------------------------------------------------

# 运行所有基准测试（已有命令保持不变）
# bench: ...

# 运行 CLI 性能测试
bench-cli:
	@echo "⚡ 运行 CLI 性能基准测试..."
	@echo "提示: 确保已构建 release 版本 (cargo build --release)"
	cargo bench --bench cli_performance

# 运行核心操作测试
bench-core:
	@echo "⚡ 运行核心操作基准测试..."
	cargo bench --bench core_operations

# 运行网络操作测试
bench-network:
	@echo "⚡ 运行网络操作基准测试..."
	cargo bench --bench network_operations

# 运行网络操作测试（带 testing feature）
bench-network-full:
	@echo "⚡ 运行网络操作基准测试 (完整)..."
	cargo bench --bench network_operations --features http/testing

# 运行特定 crate 的性能测试
bench-storage:
	@echo "⚡ 运行 Storage 性能基准测试..."
	cargo bench -p storage

# 性能回归检测
bench-regression:
	@echo "🔍 运行性能回归检测..."
	@if [ ! -d "target/criterion" ]; then \
		echo "❌ 错误: 没有基线数据，请先运行 make bench"; \
		exit 1; \
	fi
	@echo "保存当前基准为 regression..."
	cargo bench -- --save-baseline regression
	@echo "运行回归检测脚本..."
	@python3 scripts/check_performance_regression.py

# 性能对比（对比两个基线）
bench-compare:
	@echo "📊 对比性能基准..."
	@if [ ! -d "target/criterion" ]; then \
		echo "❌ 错误: 没有基线数据，请先运行 make bench"; \
		exit 1; \
	fi
	@echo "保存当前基准为 current..."
	cargo bench -- --save-baseline current
	@echo "运行对比脚本..."
	@python3 scripts/compare_benchmarks.py target/criterion/

# 建立性能基线
bench-baseline:
	@echo "📝 建立性能基线..."
	cargo bench -- --save-baseline initial
	@echo "✅ 基线已保存为 'initial'"
	@echo "后续可以使用 'cargo bench -- --baseline initial' 对比"

# 清理性能测试数据
bench-clean:
	@echo "🧹 清理性能测试数据..."
	@rm -rf target/criterion/
	@echo "✅ 性能测试数据已清理"

# 性能测试帮助
bench-help:
	@echo "📚 性能测试命令说明:"
	@echo ""
	@echo "  make bench                 - 运行所有基准测试"
	@echo "  make bench-cli             - 运行 CLI 性能测试"
	@echo "  make bench-core            - 运行核心操作测试"
	@echo "  make bench-network         - 运行网络操作测试"
	@echo "  make bench-network-full    - 运行网络操作测试 (带 testing feature)"
	@echo "  make bench-storage         - 运行 Storage 性能测试"
	@echo "  make bench-baseline        - 建立性能基线"
	@echo "  make bench-regression      - 检测性能回归"
	@echo "  make bench-compare         - 对比两个性能基线"
	@echo "  make bench-clean           - 清理性能测试数据"
	@echo "  make bench-help            - 显示此帮助信息"
	@echo ""
	@echo "📊 查看结果:"
	@echo "  open target/criterion/report/index.html"
	@echo ""

# CI 性能监控（不失败）
bench-ci:
	@echo "📊 CI 性能监控..."
	@mkdir -p target/criterion-history
	cargo bench --no-fail-fast -- --save-baseline ci-$$(date +%Y%m%d)
	@echo "✅ CI 性能基准已保存"

.PHONY: bench-cli bench-core bench-network bench-network-full bench-storage \
        bench-regression bench-compare bench-baseline bench-clean bench-help bench-ci
```

#### ✅ **验证 Step 5**
```bash
# 验证 Makefile 语法
make -n bench-cli

# 测试帮助命令
make bench-help

# 测试清理命令
make bench-clean
```

---

### Step 6: 创建性能回归检测脚本 (45 分钟)

#### 6.1 创建 `scripts/check_performance_regression.py`

```python
#!/usr/bin/env python3
"""
性能回归检测脚本

用法:
    python3 scripts/check_performance_regression.py

功能:
    - 对比当前性能与基线
    - 检测性能下降 (> 10%)
    - 生成回归报告
"""

import sys
import json
from pathlib import Path
from typing import Dict, List, Tuple


def load_criterion_estimates(benchmark_dir: Path) -> Dict[str, float]:
    """加载 Criterion 基准测试估计值"""
    estimates = {}

    # Criterion 的数据结构: target/criterion/<benchmark_name>/base/estimates.json
    for benchmark_path in benchmark_dir.glob("*/*/estimates.json"):
        parts = benchmark_path.parts
        # parts[-3] 是 benchmark 名称
        benchmark_name = parts[-3]

        try:
            with open(benchmark_path, 'r') as f:
                data = json.load(f)
                # 使用中位数作为性能指标
                mean_estimate = data.get('mean', {}).get('point_estimate', 0)
                estimates[benchmark_name] = mean_estimate
        except Exception as e:
            print(f"⚠️  警告: 无法读取 {benchmark_path}: {e}")

    return estimates


def compare_baselines(
    current: Dict[str, float],
    baseline: Dict[str, float],
    threshold: float = 10.0
) -> Tuple[List[str], List[str], List[str]]:
    """
    对比两个基线

    返回: (regressions, improvements, unchanged)
    """
    regressions = []
    improvements = []
    unchanged = []

    for name, current_time in current.items():
        if name not in baseline:
            continue

        baseline_time = baseline[name]

        # 计算变化百分比
        change_percent = ((current_time - baseline_time) / baseline_time) * 100

        if change_percent > threshold:
            regressions.append((name, baseline_time, current_time, change_percent))
        elif change_percent < -threshold:
            improvements.append((name, baseline_time, current_time, change_percent))
        else:
            unchanged.append((name, baseline_time, current_time, change_percent))

    return regressions, improvements, unchanged


def format_time(nanoseconds: float) -> str:
    """格式化时间显示"""
    if nanoseconds < 1000:
        return f"{nanoseconds:.2f} ns"
    elif nanoseconds < 1_000_000:
        return f"{nanoseconds / 1000:.2f} µs"
    elif nanoseconds < 1_000_000_000:
        return f"{nanoseconds / 1_000_000:.2f} ms"
    else:
        return f"{nanoseconds / 1_000_000_000:.2f} s"


def print_regression_report(
    regressions: List[Tuple],
    improvements: List[Tuple],
    unchanged: List[Tuple]
):
    """打印回归报告"""
    print("\n" + "="*70)
    print("🔍 性能回归检测报告")
    print("="*70)

    # 性能回归
    if regressions:
        print(f"\n❌ 性能回归 ({len(regressions)} 个):")
        print("-" * 70)
        for name, baseline, current, change in sorted(regressions, key=lambda x: x[3], reverse=True):
            print(f"  🔴 {name}")
            print(f"     基线: {format_time(baseline)}")
            print(f"     当前: {format_time(current)}")
            print(f"     变化: {change:+.2f}% (慢了)")
            print()
    else:
        print("\n✅ 没有检测到性能回归")

    # 性能改进
    if improvements:
        print(f"\n📈 性能改进 ({len(improvements)} 个):")
        print("-" * 70)
        for name, baseline, current, change in sorted(improvements, key=lambda x: x[3]):
            print(f"  🟢 {name}")
            print(f"     基线: {format_time(baseline)}")
            print(f"     当前: {format_time(current)}")
            print(f"     变化: {change:+.2f}% (快了)")
            print()

    # 保持不变
    if unchanged:
        print(f"\nℹ️  性能保持稳定 ({len(unchanged)} 个)")

    print("="*70 + "\n")


def main():
    criterion_dir = Path("target/criterion")

    if not criterion_dir.exists():
        print("❌ 错误: target/criterion 目录不存在")
        print("请先运行基准测试: cargo bench")
        sys.exit(1)

    # 检查是否有 regression baseline
    regression_dir = criterion_dir / "regression"
    if not regression_dir.exists():
        print("ℹ️  没有 'regression' 基线，使用当前数据作为基线")
        print("运行: cargo bench -- --save-baseline regression")
        sys.exit(0)

    # 加载基线数据
    print("📊 加载基线数据...")
    baseline_estimates = load_criterion_estimates(regression_dir)

    # 加载当前数据
    print("📊 加载当前数据...")
    current_estimates = load_criterion_estimates(criterion_dir / "base")

    if not baseline_estimates or not current_estimates:
        print("❌ 错误: 无法加载性能数据")
        sys.exit(1)

    # 对比
    print(f"🔍 对比 {len(current_estimates)} 个基准测试...")
    regressions, improvements, unchanged = compare_baselines(
        current_estimates,
        baseline_estimates,
        threshold=10.0  # 10% 阈值
    )

    # 打印报告
    print_regression_report(regressions, improvements, unchanged)

    # 如果有回归，退出码为 1
    if regressions:
        print("❌ 检测到性能回归！")
        sys.exit(1)
    else:
        print("✅ 性能检查通过！")
        sys.exit(0)


if __name__ == '__main__':
    main()
```

#### 6.2 创建 `scripts/compare_benchmarks.py`

```python
#!/usr/bin/env python3
"""
基准测试对比脚本

用法:
    python3 scripts/compare_benchmarks.py target/criterion/

功能:
    - 对比不同基线的性能
    - 生成对比报告
"""

import sys
import json
from pathlib import Path
from typing import Dict, List


def load_all_baselines(criterion_dir: Path) -> Dict[str, Dict[str, float]]:
    """加载所有基线数据"""
    baselines = {}

    # 查找所有 baseline 目录
    for baseline_dir in criterion_dir.glob("*"):
        if not baseline_dir.is_dir():
            continue
        if baseline_dir.name in ["report", "base"]:
            continue

        baseline_name = baseline_dir.name
        estimates = {}

        # 加载此 baseline 的所有估计值
        for estimates_file in baseline_dir.glob("*/estimates.json"):
            benchmark_name = estimates_file.parent.name

            try:
                with open(estimates_file, 'r') as f:
                    data = json.load(f)
                    mean_estimate = data.get('mean', {}).get('point_estimate', 0)
                    estimates[benchmark_name] = mean_estimate
            except Exception:
                pass

        if estimates:
            baselines[baseline_name] = estimates

    return baselines


def print_comparison_table(baselines: Dict[str, Dict[str, float]]):
    """打印对比表格"""
    if not baselines:
        print("ℹ️  没有找到基线数据")
        return

    # 获取所有 benchmark 名称
    all_benchmarks = set()
    for estimates in baselines.values():
        all_benchmarks.update(estimates.keys())

    baseline_names = sorted(baselines.keys())

    print("\n" + "="*80)
    print("📊 性能基线对比")
    print("="*80)
    print(f"\n找到 {len(baseline_names)} 个基线:")
    for name in baseline_names:
        count = len(baselines[name])
        print(f"  - {name} ({count} 个基准测试)")

    print(f"\n对比表格:")
    print("-"*80)

    # 表头
    header = f"{'Benchmark':<40}"
    for name in baseline_names[:3]:  # 最多显示 3 个基线
        header += f" | {name:>12}"
    print(header)
    print("-"*80)

    # 数据行
    for benchmark in sorted(all_benchmarks):
        row = f"{benchmark:<40}"
        for baseline_name in baseline_names[:3]:
            if benchmark in baselines[baseline_name]:
                time_ns = baselines[baseline_name][benchmark]
                if time_ns < 1000:
                    row += f" | {time_ns:>10.2f} ns"
                elif time_ns < 1_000_000:
                    row += f" | {time_ns/1000:>10.2f} µs"
                else:
                    row += f" | {time_ns/1_000_000:>10.2f} ms"
            else:
                row += f" | {'N/A':>12}"
        print(row)

    print("="*80 + "\n")


def main():
    if len(sys.argv) != 2:
        print("用法: python3 scripts/compare_benchmarks.py <criterion_dir>")
        print("示例: python3 scripts/compare_benchmarks.py target/criterion/")
        sys.exit(1)

    criterion_dir = Path(sys.argv[1])

    if not criterion_dir.exists():
        print(f"❌ 错误: 目录不存在: {criterion_dir}")
        sys.exit(1)

    # 加载所有基线
    baselines = load_all_baselines(criterion_dir)

    # 打印对比表格
    print_comparison_table(baselines)


if __name__ == '__main__':
    main()
```

#### 6.3 设置脚本权限

```bash
chmod +x scripts/check_performance_regression.py
chmod +x scripts/compare_benchmarks.py
```

#### ✅ **验证 Step 6**
```bash
# 验证脚本存在
ls -la scripts/check_performance_regression.py scripts/compare_benchmarks.py

# 验证 Python 语法
python3 -m py_compile scripts/check_performance_regression.py
python3 -m py_compile scripts/compare_benchmarks.py
```

---

## ✅ 验证和测试

### 完整验证流程

```bash
# 1. 构建 release 版本
cargo build --release

# 2. 运行所有基准测试
make bench

# 3. 运行分类测试
make bench-cli
make bench-core
make bench-network

# 4. 建立基线
make bench-baseline

# 5. 对比基准
make bench-compare

# 6. 检测回归
make bench-regression

# 7. 查看结果
open target/criterion/report/index.html

# 8. 查看帮助
make bench-help
```

### 预期结果

✅ **成功标准**:
- [ ] 所有基准测试可以成功运行
- [ ] HTML 报告可以正常生成和查看
- [ ] 性能回归检测脚本正常工作
- [ ] 基准对比脚本正常工作
- [ ] Makefile 命令都可以正常执行

---

## 📝 最佳实践

### 1. 基准测试编写原则
- 使用 `black_box()` 防止编译器优化
- 测试实际使用场景，而不是微小操作
- 避免依赖外部服务（使用 mock）
- 设置合适的样本大小和测量时间

### 2. 性能基线管理
```bash
# 重要版本发布前建立基线
git checkout main
cargo bench -- --save-baseline release-v1.0

# 功能开发完成后对比
git checkout feature-branch
cargo bench -- --baseline release-v1.0
```

### 3. 性能回归阈值
- **严格** (5%): 核心路径，用户直接感知
- **正常** (10%): 一般功能
- **宽松** (20%): 不频繁使用的功能

### 4. CI 集成建议
- 不要在每次 PR 都运行基准测试（太慢）
- 可以在合并到 main 分支时运行
- 保存历史基准数据用于趋势分析

---

## ⚠️ 注意事项

### 1. 基准测试环境
- 在稳定的环境中运行（关闭后台程序）
- 使用相同的硬件配置
- 多次运行取平均值

### 2. CLI 基准测试特殊性
- 需要先构建 release 版本
- 测试的是整个进程启动
- 受系统状态影响较大

### 3. 性能优化建议
- 先建立基线，再优化
- 优化前后都要测量
- 关注 90th/99th 百分位数，不仅是平均值

---

## 🔗 相关资源

### 内部文档
- [测试覆盖率监控实施指南](./testing-01-coverage-monitoring.md)
- [CI/CD 集成实施指南](./testing-03-cicd-integration.md)

### 外部资源
- [Criterion.rs 文档](https://bheisler.github.io/criterion.rs/book/)
- [Rust 性能优化指南](https://nnethercote.github.io/perf-book/)
- [性能测试最佳实践](https://pyperf.readthedocs.io/)

---

## 📋 检查清单

实施完成后，确认以下项目：

- [ ] `benches/` 目录已创建
- [ ] `benches/cli_performance.rs` 已创建
- [ ] `benches/core_operations.rs` 已创建
- [ ] `benches/network_operations.rs` 已创建
- [ ] 根 `Cargo.toml` 添加了 bench 配置
- [ ] `scripts/check_performance_regression.py` 已创建
- [ ] `scripts/compare_benchmarks.py` 已创建
- [ ] `make/Makefile.bench.mk` 添加了增强命令
- [ ] 所有基准测试可以成功运行
- [ ] HTML 报告可以正常查看
- [ ] 性能回归检测脚本正常工作
- [ ] 基准对比脚本正常工作
- [ ] 团队成员了解如何使用这些命令

---

**文档版本**: 1.0
**创建日期**: 2025-02-11
**最后更新**: 2025-02-11
**下一步**: [CI/CD 集成实施指南](./testing-03-cicd-integration.md)
