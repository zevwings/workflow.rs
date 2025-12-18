# 架构文档检查工具实现方案 TODO

> 阶段3自动化检查工具的详细分析和设计方案

**目标**：分析如何实现自动化检查工具，减少人工检查工作量，提升文档同步效率。

**优先级**：P2（可选，但建议实施以提升效率）

**时间估算**：2-3天

**状态**：📋 分析完成，待实施

---

## 📋 功能需求分析

### 3.1 文档路径验证脚本

#### 功能需求

1. **扫描架构文档**
   - 扫描 `docs/architecture/` 目录下的所有 Markdown 文件
   - 支持递归扫描子目录（`lib/`、`commands/`）

2. **提取文件路径**
   - 从文档中提取提到的文件路径（如 `src/lib/pr/github/platform.rs`）
   - 识别路径模式：
     - 代码块中的路径：`` `src/lib/pr/github/platform.rs` ``
     - 文本中的路径：`src/lib/pr/github/platform.rs`
     - 目录结构图中的路径

3. **验证路径存在性**
   - 验证路径是否存在（相对于项目根目录）
   - 区分文件路径和目录路径
   - 处理相对路径和绝对路径

4. **验证模块路径**
   - 提取文档中提到的模块路径（如 `crate::pr::github::Platform`）
   - 验证模块路径是否有效（检查 Rust 模块结构）

5. **生成验证报告**
   - Markdown 格式报告
   - 列出所有不匹配的路径
   - 包含文件路径、文档位置、问题类型

#### 技术难点

1. **路径提取**
   - 需要识别多种路径格式
   - 处理代码块、文本、列表等不同上下文
   - 区分代码路径和普通文本

2. **模块路径验证**
   - 需要解析 Rust 模块结构
   - 验证 `crate::` 路径是否有效
   - 处理模块重导出（`pub use`）

3. **路径规范化**
   - 统一路径格式（相对路径 vs 绝对路径）
   - 处理路径分隔符（`/` vs `\`）
   - 处理路径别名（如 `~`）

---

### 3.2 模块统计验证脚本

#### 功能需求

1. **统计实际代码**
   - 扫描 `src/lib/` 和 `src/commands/` 目录
   - 统计代码行数（排除空行、注释行，可选）
   - 统计文件数量（`.rs` 文件）

2. **解析文档统计**
   - 从架构文档中提取统计信息
   - 识别统计信息格式：
     - "总代码行数：约 XXX 行"
     - "文件数量：X 个"
     - "主要组件：X 个（...）"

3. **对比统计信息**
   - 与实际统计结果对比
   - 允许 ±10% 误差（代码行数）
   - 文件数量必须完全一致

4. **生成差异报告**
   - Markdown 格式报告
   - 列出所有统计差异
   - 包含模块名、文档统计、实际统计、差异百分比

#### 技术难点

1. **统计信息提取**
   - 需要识别多种统计信息格式
   - 处理"约"、"大约"等模糊词汇
   - 提取数字和单位（行、个、组件等）

2. **代码行数统计**
   - 需要决定是否排除空行和注释
   - 处理多行注释
   - 处理文档注释（`///`、`//!`）

3. **模块映射**
   - 需要将文档与代码模块对应
   - 处理 Lib 层和命令层的不同结构
   - 处理模块名称的映射关系

---

### 3.3 综合检查脚本（可选）

#### 功能需求

1. **整合多个检查**
   - 整合路径验证和统计验证
   - 支持选择性运行（全部/单个检查）

2. **生成综合报告**
   - 统一的报告格式
   - 包含所有检查结果
   - 问题优先级分类

3. **支持检查范围**
   - 全部模块检查
   - 单个模块检查
   - 指定模块列表检查

---

## 🛠️ 技术方案选择

### 方案对比

| 方案 | 优点 | 缺点 | 推荐度 |
|------|------|------|--------|
| **Rust 实现** | 1. 与项目技术栈一致<br>2. 类型安全，错误处理完善<br>3. 性能好，适合复杂解析<br>4. 易于集成到项目<br>5. 可以复用项目现有工具 | 1. 开发时间较长<br>2. 需要添加依赖（Markdown 解析） | ⭐⭐⭐⭐⭐ 推荐 |
| **Shell 脚本** | 1. 开发快速<br>2. 无需编译<br>3. 易于调试 | 1. 路径解析复杂<br>2. 跨平台兼容性问题<br>3. 错误处理较弱<br>4. 难以处理复杂逻辑 | ⭐⭐⭐ 可选 |
| **Python 脚本** | 1. 文本处理能力强<br>2. 有丰富的 Markdown 解析库 | 1. 需要 Python 环境<br>2. 与项目技术栈不一致 | ⭐⭐ 不推荐 |

### 推荐方案：Rust 实现

**理由**：
1. 与项目技术栈一致，便于维护
2. 类型安全，减少错误
3. 性能好，适合批量处理
4. 可以复用项目现有工具（如路径管理、文件操作）
5. 易于集成到 CI/CD

---

## 🏗️ 实现方案设计

### 3.1 文档路径验证脚本

#### 技术选型

- **语言**：Rust
- **Markdown 解析**：`pulldown-cmark` 或 `comrak`（推荐 `pulldown-cmark`，更轻量）
- **路径处理**：标准库 `std::path::Path`
- **文件操作**：标准库 `std::fs`

#### 实现架构

```
scripts/check-doc-paths.rs
├── main() - 入口函数
├── scan_architecture_docs() - 扫描架构文档
├── extract_paths_from_doc() - 从文档提取路径
│   ├── parse_markdown() - 解析 Markdown
│   ├── extract_code_paths() - 提取代码块中的路径
│   ├── extract_text_paths() - 提取文本中的路径
│   └── extract_module_paths() - 提取模块路径
├── validate_paths() - 验证路径
│   ├── check_file_exists() - 检查文件是否存在
│   ├── check_module_exists() - 检查模块是否存在
│   └── normalize_path() - 规范化路径
└── generate_report() - 生成报告
```

#### 核心功能实现

**1. Markdown 解析**

```rust
use pulldown_cmark::{Parser, Event, Tag};

fn extract_paths_from_doc(content: &str) -> Vec<PathInfo> {
    let parser = Parser::new(content);
    let mut paths = Vec::new();

    for event in parser {
        match event {
            Event::Code(code) => {
                // 提取代码块中的路径
                extract_paths_from_code(&code, &mut paths);
            }
            Event::Text(text) => {
                // 提取文本中的路径
                extract_paths_from_text(&text, &mut paths);
            }
            _ => {}
        }
    }

    paths
}
```

**2. 路径提取正则**

```rust
// 文件路径模式
let file_path_re = Regex::new(r"src/(lib|commands)/[a-zA-Z0-9_/]+\.rs")?;

// 模块路径模式
let module_path_re = Regex::new(r"crate::[a-zA-Z0-9_:]+")?;
```

**3. 模块路径验证**

```rust
fn validate_module_path(module_path: &str) -> bool {
    // 解析 crate::pr::github::Platform
    let parts: Vec<&str> = module_path.split("::").collect();

    // 跳过 "crate"
    if parts.is_empty() || parts[0] != "crate" {
        return false;
    }

    // 检查模块文件是否存在
    let module_file = format!("src/{}.rs", parts[1..].join("/"));
    Path::new(&module_file).exists()
}
```

#### 输出格式

```markdown
# 文档路径验证报告

**检查日期**：2025-12-18 19:14:08
**检查范围**：docs/architecture/ 目录下的所有文档

## 检查结果

### ✅ 通过的路径
- `src/lib/pr/github/platform.rs` (docs/architecture/lib/PR_ARCHITECTURE.md)

### ❌ 不存在的路径
1. `src/lib/pr/old_module.rs`
   - **文档位置**：docs/architecture/lib/PR_ARCHITECTURE.md
   - **问题类型**：文件不存在
   - **建议**：检查文件是否已删除或重命名

### ⚠️ 无效的模块路径
1. `crate::pr::old::Module`
   - **文档位置**：docs/architecture/lib/PR_ARCHITECTURE.md
   - **问题类型**：模块不存在
   - **建议**：检查模块是否已重构或删除
```

---

### 3.2 模块统计验证脚本

#### 技术选型

- **语言**：Rust
- **Markdown 解析**：`pulldown-cmark`
- **代码统计**：自定义实现（读取文件，统计行数）
- **正则表达式**：`regex` crate

#### 实现架构

```
scripts/check-doc-stats.rs
├── main() - 入口函数
├── scan_code_modules() - 扫描代码模块
│   ├── count_lines() - 统计代码行数
│   └── count_files() - 统计文件数量
├── parse_doc_stats() - 解析文档统计
│   ├── extract_line_count() - 提取代码行数
│   ├── extract_file_count() - 提取文件数量
│   └── extract_component_count() - 提取组件数量
├── compare_stats() - 对比统计信息
└── generate_report() - 生成差异报告
```

#### 核心功能实现

**1. 代码行数统计**

```rust
fn count_lines(module_path: &Path) -> Result<usize> {
    let mut total_lines = 0;

    for entry in WalkDir::new(module_path)
        .into_iter()
        .filter_entry(|e| {
            // 只统计 .rs 文件
            e.path().extension().map_or(false, |ext| ext == "rs")
        })
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            let content = fs::read_to_string(entry.path())?;
            total_lines += content.lines().count();
        }
    }

    Ok(total_lines)
}
```

**2. 统计信息提取**

```rust
fn extract_line_count(doc_content: &str) -> Option<usize> {
    // 匹配 "总代码行数：约 XXX 行" 或 "总代码行数：XXX 行"
    let re = Regex::new(r"总代码行数[：:]\s*(?:约\s*)?(\d+)\s*行")?;

    re.captures(doc_content)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
}
```

**3. 差异计算**

```rust
fn calculate_difference(doc_value: usize, actual_value: usize) -> f64 {
    if doc_value == 0 {
        return 0.0;
    }

    let diff = actual_value as f64 - doc_value as f64;
    (diff / doc_value as f64) * 100.0
}

fn is_within_tolerance(doc_value: usize, actual_value: usize, tolerance: f64) -> bool {
    let diff_percent = calculate_difference(doc_value, actual_value).abs();
    diff_percent <= tolerance
}
```

#### 输出格式

```markdown
# 模块统计验证报告

**检查日期**：2025-12-18 19:14:08
**检查范围**：docs/architecture/ 目录下的所有文档

## 检查结果

### ✅ 统计一致
- **PR 模块**：文档 891 行，实际 891 行，差异 0%

### ⚠️ 统计差异（在允许范围内）
- **Jira 模块**：文档 1200 行，实际 1250 行，差异 +4.2%（允许 ±10%）

### ❌ 统计差异（超出允许范围）
- **Git 模块**：文档 500 行，实际 800 行，差异 +60%（超出 ±10% 范围）
  - **建议**：更新文档中的统计信息

### ❌ 文件数量不一致
- **Template 模块**：文档 3 个文件，实际 5 个文件
  - **建议**：更新文档中的文件数量统计
```

---

### 3.3 综合检查脚本（可选）

#### 实现架构

```
scripts/check-architecture-docs.rs
├── main() - 入口函数，解析命令行参数
├── run_all_checks() - 运行所有检查
├── run_path_check() - 运行路径验证
├── run_stats_check() - 运行统计验证
└── generate_combined_report() - 生成综合报告
```

#### 命令行接口

```rust
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// 检查类型：all, paths, stats
    #[arg(long, default_value = "all")]
    check_type: String,

    /// 检查范围：all, module:pr, module:jira
    #[arg(long, default_value = "all")]
    scope: String,

    /// 输出报告路径
    #[arg(long, default_value = "report/")]
    output: PathBuf,
}
```

---

## 🔧 技术难点和解决方案

### 难点1：Markdown 路径提取

**问题**：需要从 Markdown 文档中准确提取文件路径，区分代码路径和普通文本。

**解决方案**：
1. 使用 `pulldown-cmark` 解析 Markdown，区分代码块和文本
2. 使用正则表达式匹配路径模式
3. 上下文分析：代码块中的路径优先级更高

**实现示例**：
```rust
fn extract_paths_from_code(code: &str) -> Vec<String> {
    // 匹配 src/ 开头的路径
    let re = Regex::new(r"src/(lib|commands)/[a-zA-Z0-9_/]+\.rs")?;
    re.find_iter(code)
        .map(|m| m.as_str().to_string())
        .collect()
}
```

---

### 难点2：模块路径验证

**问题**：需要验证 `crate::pr::github::Platform` 这样的模块路径是否有效。

**解决方案**：
1. 解析模块路径，转换为文件路径
2. 检查文件是否存在
3. 处理模块重导出（`pub use`）的情况

**实现示例**：
```rust
fn module_path_to_file_path(module_path: &str) -> Option<PathBuf> {
    // crate::pr::github::Platform -> src/pr/github.rs 或 src/pr/github/mod.rs
    let parts: Vec<&str> = module_path.split("::").collect();
    if parts.is_empty() || parts[0] != "crate" {
        return None;
    }

    // 尝试多种路径格式
    let base = parts[1..].join("/");
    let paths = vec![
        format!("src/{}.rs", base),
        format!("src/{}/mod.rs", base),
    ];

    paths.iter()
        .find(|p| Path::new(p).exists())
        .map(|p| PathBuf::from(p))
}
```

---

### 难点3：统计信息提取

**问题**：需要从文档中提取各种格式的统计信息（"约 XXX 行"、"X 个文件"等）。

**解决方案**：
1. 使用正则表达式匹配多种格式
2. 处理模糊词汇（"约"、"大约"、"约"）
3. 提取数字和单位

**实现示例**：
```rust
fn extract_statistics(doc_content: &str) -> DocStats {
    let line_count_re = Regex::new(
        r"总代码行数[：:]\s*(?:约\s*|大约\s*)?(\d+)\s*行"
    )?;

    let file_count_re = Regex::new(
        r"文件数量[：:]\s*(\d+)\s*个"
    )?;

    DocStats {
        line_count: extract_number(&line_count_re, doc_content),
        file_count: extract_number(&file_count_re, doc_content),
    }
}
```

---

### 难点4：模块映射

**问题**：需要将文档与代码模块对应（如 `PR_ARCHITECTURE.md` 对应 `src/lib/pr/`）。

**解决方案**：
1. 从文档文件名推断模块名（`PR_ARCHITECTURE.md` → `pr`）
2. 支持 Lib 层和命令层的不同结构
3. 使用配置文件定义映射关系（如需要）

**实现示例**：
```rust
fn doc_to_module_path(doc_path: &Path) -> Option<PathBuf> {
    let file_name = doc_path.file_stem()?.to_str()?;

    // PR_ARCHITECTURE.md -> pr
    let module_name = file_name
        .strip_suffix("_ARCHITECTURE")?
        .strip_suffix("_COMMAND_ARCHITECTURE")?
        .to_lowercase();

    // 判断是 Lib 层还是命令层
    if doc_path.parent()?.ends_with("lib") {
        Some(PathBuf::from(format!("src/lib/{}", module_name)))
    } else if doc_path.parent()?.ends_with("commands") {
        Some(PathBuf::from(format!("src/commands/{}", module_name)))
    } else {
        None
    }
}
```

---

## 📝 实施步骤

### 阶段1：基础功能（1天）

1. **创建项目结构**
   - 创建 `scripts/` 目录（如不存在）
   - 创建 `scripts/check-doc-paths.rs`
   - 添加依赖到 `Cargo.toml`（如需要）

2. **实现 Markdown 解析**
   - 添加 `pulldown-cmark` 依赖
   - 实现基本的 Markdown 解析
   - 提取代码块和文本内容

3. **实现路径提取**
   - 实现文件路径提取（正则表达式）
   - 实现模块路径提取
   - 测试路径提取准确性

### 阶段2：验证功能（1天）

1. **实现路径验证**
   - 实现文件存在性检查
   - 实现模块路径验证
   - 处理路径规范化

2. **实现报告生成**
   - 实现 Markdown 报告生成
   - 格式化输出
   - 添加问题分类和优先级

3. **测试和优化**
   - 测试各种路径格式
   - 测试边界情况
   - 优化性能

### 阶段3：统计验证（1天）

1. **实现代码统计**
   - 实现代码行数统计
   - 实现文件数量统计
   - 处理空行和注释（可选）

2. **实现统计提取**
   - 实现文档统计信息提取
   - 处理多种格式
   - 处理模糊词汇

3. **实现对比和报告**
   - 实现统计对比
   - 实现差异计算
   - 生成差异报告

### 阶段4：综合脚本和 CI 集成（可选，1天）

1. **创建综合脚本**
   - 整合路径验证和统计验证
   - 实现命令行接口
   - 支持选择性运行

2. **CI 集成**
   - 在 `.github/workflows/ci.yml` 中添加检查步骤
   - 配置非阻塞模式
   - 生成检查报告

---

## 📦 依赖管理

### 需要添加的依赖

```toml
[dependencies]
# Markdown 解析
pulldown-cmark = "0.9"  # 或 comrak = "0.18"

# 正则表达式（已有）
regex = "1.10"

# 命令行参数解析（如果创建独立工具，已有）
clap = { version = "4", features = ["derive"] }

# 文件遍历（统计脚本，已有）
walkdir = "2.4"
```

**注意**：项目中已有 `regex` 和 `walkdir` 依赖，只需要添加 `pulldown-cmark`。

### 依赖原则

- **最小化依赖**：只添加必要的依赖
- **版本管理**：使用稳定版本，避免使用 `*` 通配符
- **功能标志**：使用 feature flags 控制可选功能

---

## 🎯 验收标准

### 路径验证脚本

- [ ] 可以扫描所有架构文档
- [ ] 可以准确提取文件路径和模块路径
- [ ] 可以验证路径是否存在
- [ ] 可以生成清晰的验证报告
- [ ] 报告包含问题位置、类型和建议

### 统计验证脚本

- [ ] 可以统计代码行数和文件数量
- [ ] 可以解析文档中的统计信息
- [ ] 可以对比统计差异
- [ ] 可以生成差异报告
- [ ] 正确处理 ±10% 误差范围

### 综合脚本（可选）

- [ ] 可以整合多个检查
- [ ] 支持选择性运行
- [ ] 生成统一的综合报告

### CI 集成（可选）

- [ ] CI 流程中包含文档检查步骤
- [ ] 检查失败时不阻塞构建
- [ ] 检查报告可以附加到 PR

---

## ⚠️ 注意事项

1. **性能考虑**
   - 批量处理时注意性能
   - 考虑缓存解析结果
   - 避免重复扫描

2. **错误处理**
   - 处理文件读取错误
   - 处理路径解析错误
   - 提供清晰的错误信息

3. **跨平台兼容**
   - 处理路径分隔符差异（`/` vs `\`）
   - 处理大小写敏感性问题（Linux vs Windows）

4. **可维护性**
   - 代码结构清晰
   - 添加文档注释
   - 提供使用说明

---

## 📚 相关文档

- [架构文档审查指南](../docs/guidelines/workflows/references/REVIEW_ARCHITECTURE_DOC_GUIDELINES.md) - 详细的检查方法和流程
- [开发规范](../docs/guidelines/DEVELOPMENT_GUIDELINES.md) - 开发规范和最佳实践

---

## 🔗 参考资源

- [pulldown-cmark 文档](https://docs.rs/pulldown-cmark/) - Markdown 解析库
- [regex 文档](https://docs.rs/regex/) - 正则表达式库
- [walkdir 文档](https://docs.rs/walkdir/) - 目录遍历库

---

**最后更新**: 2025-12-18
**状态**: 📋 分析完成，待实施
