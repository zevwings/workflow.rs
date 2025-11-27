# 代码库访问策略文档

## 📋 问题分析

### 核心问题

1. **如何获取整个仓库的代码？**
   - PR diff 只包含修改的文件，但查找调用点需要访问完整代码库
   - 需要能够搜索整个代码库来找到接口/组件的调用点

2. **大代码库的性能问题**
   - 如果代码库很大（几 GB、几万文件），全量读取效率会很低
   - 需要优化策略，避免性能瓶颈

## 🎯 解决方案

### 方案一：基于 Git 命令的增量访问（推荐）

**核心思想**：不读取整个代码库，而是使用 Git 命令按需访问文件

#### 实现方式

1. **使用 Git 命令搜索**
   ```bash
   # 搜索接口路径
   git grep -n "POST /api/users" --all
   git grep -n "GET /api/users" --all

   # 搜索函数名
   git grep -n "getUser" --all
   git grep -n "createUser" --all

   # 搜索组件名
   git grep -n "UserCreate" --all
   git grep -n "<UserDetail" --all
   ```

2. **使用 Git 列出文件**
   ```bash
   # 列出所有文件（不读取内容）
   git ls-tree -r --name-only HEAD
   git ls-tree -r --name-only origin/main
   ```

3. **按需读取文件**
   ```bash
   # 只读取特定文件
   git show HEAD:path/to/file.rs
   git show origin/main:path/to/file.tsx
   ```

**优点**：
- ✅ 不需要 checkout 整个代码库
- ✅ 可以搜索所有分支和提交
- ✅ 性能好（Git 内部优化）
- ✅ 不占用大量磁盘空间

**缺点**：
- ⚠️ 需要 Git 仓库可用
- ⚠️ 需要理解 Git 命令输出格式

#### Rust 实现示例

```rust
use std::process::Command;

/// 使用 git grep 搜索代码
fn search_codebase(pattern: &str) -> Result<Vec<SearchResult>> {
    let output = Command::new("git")
        .args(&["grep", "-n", "--all", pattern])
        .output()?;

    // 解析输出：file:line:content
    parse_git_grep_output(&output.stdout)
}

/// 列出所有文件（不读取内容）
fn list_all_files(branch: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(&["ls-tree", "-r", "--name-only", branch])
        .output()?;

    Ok(String::from_utf8(output.stdout)?
        .lines()
        .map(|s| s.to_string())
        .collect())
}

/// 读取特定文件内容
fn read_file_from_git(branch: &str, path: &str) -> Result<String> {
    let output = Command::new("git")
        .args(&["show", &format!("{}:{}", branch, path)])
        .output()?;

    Ok(String::from_utf8(output.stdout)?)
}
```

### 方案二：智能过滤 + 增量读取

**核心思想**：只读取可能相关的文件，而不是整个代码库

#### 实现策略

1. **基于文件路径过滤**
   ```rust
   // 只搜索相关目录
   let search_dirs = vec![
       "src/api/",      // API 相关
       "src/pages/",   // 前端页面
       "src/components/", // 前端组件
       "src/services/",   // Service 层
   ];

   // 排除不相关的目录
   let exclude_dirs = vec![
       "node_modules/",
       "target/",
       ".git/",
       "dist/",
       "build/",
   ];
   ```

2. **基于文件类型过滤**
   ```rust
   // 只搜索特定类型的文件
   let relevant_extensions = vec![
       ".rs", ".go", ".java", ".py",  // 后端
       ".ts", ".tsx", ".js", ".jsx",  // 前端
   ];
   ```

3. **缓存搜索结果**
   ```rust
   // 缓存搜索结果，避免重复搜索
   use std::collections::HashMap;

   struct SearchCache {
       pattern_results: HashMap<String, Vec<SearchResult>>,
   }
   ```

**优点**：
- ✅ 大幅减少需要处理的文件数量
- ✅ 可以并行处理多个文件
- ✅ 可以缓存结果

**缺点**：
- ⚠️ 可能遗漏某些调用点（如果不在预期目录中）
- ⚠️ 需要维护过滤规则

#### Rust 实现示例

```rust
use std::path::Path;
use std::fs;

/// 智能过滤文件列表
fn filter_relevant_files(
    all_files: Vec<String>,
    include_dirs: &[&str],
    exclude_dirs: &[&str],
    extensions: &[&str],
) -> Vec<String> {
    all_files
        .into_iter()
        .filter(|file| {
            // 检查扩展名
            let has_valid_ext = extensions.iter()
                .any(|ext| file.ends_with(ext));
            if !has_valid_ext {
                return false;
            }

            // 检查包含目录
            let in_include_dir = include_dirs.is_empty() ||
                include_dirs.iter().any(|dir| file.starts_with(dir));
            if !in_include_dir {
                return false;
            }

            // 检查排除目录
            let in_exclude_dir = exclude_dirs.iter()
                .any(|dir| file.starts_with(dir));
            !in_exclude_dir
        })
        .collect()
}

/// 并行搜索多个文件
use rayon::prelude::*;

fn search_files_parallel(
    files: Vec<String>,
    pattern: &str,
) -> Vec<SearchResult> {
    files
        .par_iter()
        .filter_map(|file| {
            // 只读取匹配的文件
            if let Ok(content) = fs::read_to_string(file) {
                if content.contains(pattern) {
                    return Some(search_in_file(file, &content, pattern));
                }
            }
            None
        })
        .flatten()
        .collect()
}
```

### 方案三：使用 ripgrep（rg）进行快速搜索

**核心思想**：使用专业的代码搜索工具，而不是自己实现

#### 实现方式

```rust
use std::process::Command;

/// 使用 ripgrep 搜索（如果系统已安装）
fn search_with_ripgrep(
    pattern: &str,
    include_types: &[&str],
    exclude_dirs: &[&str],
) -> Result<Vec<SearchResult>> {
    let mut cmd = Command::new("rg");

    // 基本参数
    cmd.args(&["--line-number", "--no-heading", pattern]);

    // 指定文件类型
    for file_type in include_types {
        cmd.args(&["--type", file_type]);
    }

    // 排除目录
    for dir in exclude_dirs {
        cmd.args(&["--glob", &format!("!{}", dir)]);
    }

    let output = cmd.output()?;
    parse_ripgrep_output(&output.stdout)
}
```

**优点**：
- ✅ 性能极佳（专门优化的搜索工具）
- ✅ 支持正则表达式
- ✅ 支持文件类型过滤
- ✅ 支持排除目录

**缺点**：
- ⚠️ 需要系统安装 ripgrep（可选依赖）
- ⚠️ 需要处理命令不存在的情况

### 方案四：使用 GitHub MCP（适用于 GitHub 仓库）

**核心思想**：通过 GitHub MCP（Model Context Protocol）直接访问 GitHub 仓库内容，无需本地 Git 仓库

#### 实现方式

1. **获取文件内容**
   ```rust
   // 使用 GitHub MCP 获取文件内容
   use mcp_github_get_file_contents;

   let content = mcp_github_get_file_contents(
       owner: "owner",
       repo: "repo",
       path: "src/api/users.rs",
       branch: Some("main"),
   )?;
   ```

2. **搜索代码**
   ```rust
   // 使用 GitHub MCP 搜索代码
   use mcp_github_search_code;

   let results = mcp_github_search_code(
       q: "POST /api/users language:rust",
       per_page: Some(100),
   )?;
   ```

3. **获取 PR 文件列表**
   ```rust
   // 获取 PR 修改的文件列表
   use mcp_github_get_pull_request_files;

   let files = mcp_github_get_pull_request_files(
       owner: "owner",
       repo: "repo",
       pull_number: 123,
   )?;
   ```

**优点**：
- ✅ 不需要本地 Git 仓库
- ✅ 可以直接访问远程仓库内容
- ✅ 支持搜索整个代码库
- ✅ 可以获取特定分支的内容
- ✅ 可以获取 PR 的文件列表
- ✅ 不占用本地磁盘空间

**缺点**：
- ⚠️ 仅适用于 GitHub 仓库（不适用于 Codeup 等其他平台）
- ⚠️ 需要 GitHub MCP 服务可用
- ⚠️ 可能有 API 速率限制
- ⚠️ 需要网络连接

#### Rust 实现示例

```rust
/// 使用 GitHub MCP 获取文件内容
fn get_file_from_github(
    owner: &str,
    repo: &str,
    path: &str,
    branch: Option<&str>,
) -> Result<String> {
    // 调用 GitHub MCP
    let content = mcp_github_get_file_contents(
        owner,
        repo,
        path,
        branch,
    )?;

    Ok(content)
}

/// 使用 GitHub MCP 搜索代码
fn search_codebase_via_github(
    owner: &str,
    repo: &str,
    query: &str,
) -> Result<Vec<SearchResult>> {
    // 构建搜索查询（限定到特定仓库）
    let full_query = format!("repo:{} {} {}", owner, repo, query);

    // 调用 GitHub MCP
    let results = mcp_github_search_code(
        q: &full_query,
        per_page: Some(100),
    )?;

    // 解析结果
    parse_github_search_results(results)
}

/// 获取 PR 修改的文件列表
fn get_pr_files(
    owner: &str,
    repo: &str,
    pr_number: u64,
) -> Result<Vec<String>> {
    let files = mcp_github_get_pull_request_files(
        owner,
        repo,
        pr_number,
    )?;

    Ok(files.iter().map(|f| f.path.clone()).collect())
}
```

### 方案五：混合策略（最佳实践）

**核心思想**：结合多种方法，根据场景选择最优方案

#### 策略选择

```rust
enum SearchStrategy {
    /// 使用 GitHub MCP（如果可用且是 GitHub 仓库）
    GitHubMCP,
    /// 使用 git grep（默认，最可靠）
    GitGrep,
    /// 使用 ripgrep（如果可用，性能最好）
    RipGrep,
    /// 使用文件系统搜索（fallback）
    FileSystem,
}

impl SearchStrategy {
    fn detect() -> Self {
        // 检查是否是 GitHub 仓库且 MCP 可用
        if Self::is_github_repo() && Self::is_mcp_available() {
            return SearchStrategy::GitHubMCP;
        }

        // 检查 ripgrep 是否可用
        if Command::new("rg").output().is_ok() {
            return SearchStrategy::RipGrep;
        }

        // 检查是否在 Git 仓库中
        if Path::new(".git").exists() {
            return SearchStrategy::GitGrep;
        }

        // Fallback 到文件系统
        SearchStrategy::FileSystem
    }

    fn search(&self, pattern: &str) -> Result<Vec<SearchResult>> {
        match self {
            SearchStrategy::GitHubMCP => search_with_github_mcp(pattern),
            SearchStrategy::RipGrep => search_with_ripgrep(pattern),
            SearchStrategy::GitGrep => search_with_git_grep(pattern),
            SearchStrategy::FileSystem => search_with_filesystem(pattern),
        }
    }

    fn is_github_repo() -> bool {
        // 检查 Git remote URL 是否是 GitHub
        GitRepo::get_repo_type() == RepoType::GitHub
    }

    fn is_mcp_available() -> bool {
        // 检查 GitHub MCP 服务是否可用
        // 可以通过尝试调用 MCP 函数来判断
        true // 简化实现
    }
}
```

## ⚡ 性能优化策略

### 1. 延迟加载（Lazy Loading）

**策略**：只在需要时读取文件，而不是一次性加载所有文件

```rust
/// 延迟读取文件内容
struct LazyFileReader {
    file_path: String,
    content: Option<String>,
}

impl LazyFileReader {
    fn get_content(&mut self) -> Result<&str> {
        if self.content.is_none() {
            self.content = Some(fs::read_to_string(&self.file_path)?);
        }
        Ok(self.content.as_ref().unwrap())
    }
}
```

### 2. 并行处理

**策略**：使用多线程并行搜索和处理文件

```rust
use rayon::prelude::*;

/// 并行搜索多个模式
fn search_multiple_patterns_parallel(
    patterns: Vec<String>,
    files: Vec<String>,
) -> HashMap<String, Vec<SearchResult>> {
    patterns
        .par_iter()
        .map(|pattern| {
            let results = search_pattern_in_files(pattern, &files);
            (pattern.clone(), results)
        })
        .collect()
}
```

### 3. 缓存机制

**策略**：缓存搜索结果，避免重复搜索

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct SearchCache {
    cache: Arc<Mutex<HashMap<String, Vec<SearchResult>>>>,
}

impl SearchCache {
    fn get_or_search(
        &self,
        pattern: &str,
        search_fn: impl FnOnce() -> Result<Vec<SearchResult>>,
    ) -> Result<Vec<SearchResult>> {
        // 检查缓存
        let mut cache = self.cache.lock().unwrap();
        if let Some(results) = cache.get(pattern) {
            return Ok(results.clone());
        }

        // 执行搜索
        let results = search_fn()?;
        cache.insert(pattern.to_string(), results.clone());
        Ok(results)
    }
}
```

### 4. 增量搜索

**策略**：先搜索 PR diff，再扩展到相关文件

```rust
/// 增量搜索策略
fn incremental_search(
    pr_diff: &str,
    codebase: &Codebase,
) -> Result<Vec<SearchResult>> {
    // 第一步：从 PR diff 中提取接口/组件名
    let interfaces = extract_interfaces_from_diff(pr_diff)?;

    // 第二步：只搜索这些接口的调用点
    let mut all_results = Vec::new();
    for interface in interfaces {
        let results = codebase.search(&interface.name)?;
        all_results.extend(results);
    }

    Ok(all_results)
}
```

### 5. 索引机制（高级）

**策略**：为代码库建立索引，加速搜索

```rust
/// 代码库索引
struct CodebaseIndex {
    // 接口名 -> 文件路径
    interface_locations: HashMap<String, Vec<String>>,
    // 函数名 -> 文件路径
    function_locations: HashMap<String, Vec<String>>,
    // 组件名 -> 文件路径
    component_locations: HashMap<String, Vec<String>>,
}

impl CodebaseIndex {
    /// 构建索引（可以后台运行）
    fn build_index(codebase_path: &Path) -> Result<Self> {
        // 使用 AST 解析器扫描代码库
        // 建立索引
        // ...
    }

    /// 使用索引快速查找
    fn find_calls(&self, interface: &str) -> Vec<String> {
        self.interface_locations
            .get(interface)
            .cloned()
            .unwrap_or_default()
    }
}
```

## 📊 性能对比

### 场景：10,000 个文件的代码库

| 方案 | 首次搜索时间 | 后续搜索时间 | 内存占用 | 磁盘占用 | 适用场景 |
|------|------------|------------|---------|---------|---------|
| **全量读取** | 30-60秒 | 30-60秒 | 500MB+ | 0 | 不推荐 |
| **Git grep** | 2-5秒 | 2-5秒 | 10MB | 0 | 本地 Git 仓库 |
| **ripgrep** | 1-3秒 | 1-3秒 | 5MB | 0 | 本地文件系统 |
| **GitHub MCP** | 3-8秒 | 3-8秒 | 5MB | 0 | GitHub 仓库，无需本地仓库 |
| **智能过滤** | 5-10秒 | 5-10秒 | 50MB | 0 | 本地文件系统 |
| **索引机制** | 60-120秒（构建） | 0.1-0.5秒 | 100MB | 50MB | 超大代码库 |

### 推荐方案

**对于 GitHub 仓库**：
1. **首选**：GitHub MCP（无需本地仓库，直接访问远程）
2. **备选**：Git grep（如果本地有 Git 仓库）
3. **备选**：ripgrep（如果系统已安装）

**对于本地 Git 仓库（非 GitHub）**：
1. **首选**：Git grep（可靠、性能好、无需额外依赖）
2. **备选**：ripgrep（如果系统已安装，性能最好）
3. **Fallback**：智能过滤 + 文件系统搜索

**对于超大代码库（>100,000 文件）**：
1. **考虑**：索引机制（需要定期更新）
2. **结合**：增量搜索（只搜索相关部分）

## 🛠️ 实现建议

### 阶段一：基础实现（MVP）

```rust
/// 代码库搜索器
pub struct CodebaseSearcher {
    repo_path: PathBuf,
    strategy: SearchStrategy,
}

impl CodebaseSearcher {
    /// 创建搜索器
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        let strategy = SearchStrategy::detect();
        Ok(Self { repo_path, strategy })
    }

    /// 搜索接口调用点
    pub fn search_interface_calls(
        &self,
        interface_path: &str,
    ) -> Result<Vec<SearchResult>> {
        // 提取接口路径（如 "/api/users"）
        let patterns = vec![
            format!("{}", interface_path),
            format!("\"{}\"", interface_path),
            format!("'{}'", interface_path),
        ];

        // 搜索所有模式
        let mut all_results = Vec::new();
        for pattern in patterns {
            let results = self.strategy.search(&pattern)?;
            all_results.extend(results);
        }

        Ok(all_results)
    }

    /// 搜索组件使用
    pub fn search_component_usage(
        &self,
        component_name: &str,
    ) -> Result<Vec<SearchResult>> {
        let patterns = vec![
            format!("<{}", component_name),
            format!("{}", component_name),
            format!("import.*{}", component_name),
        ];

        // 类似实现...
    }
}
```

### 阶段二：性能优化

1. **添加缓存**
2. **并行处理**
3. **智能过滤**

### 阶段三：高级功能

1. **索引机制**
2. **增量更新**
3. **后台索引构建**

## 📝 配置选项

```toml
# workflow.toml
[test_analysis]
# 搜索策略：git_grep, ripgrep, filesystem, auto
strategy = "auto"

# 包含的目录（空表示全部）
include_dirs = ["src/", "lib/", "app/"]

# 排除的目录
exclude_dirs = ["node_modules/", "target/", ".git/"]

# 包含的文件类型
include_types = ["rust", "typescript", "javascript"]

# 是否启用缓存
enable_cache = true

# 缓存过期时间（秒）
cache_ttl = 3600

# 是否启用并行搜索
enable_parallel = true

# 最大并发数
max_workers = 4
```

## ✅ 总结

### 推荐方案

1. **获取代码库**：
   - **GitHub 仓库**：优先使用 GitHub MCP（无需本地仓库）
   - **本地 Git 仓库**：使用 Git 命令（`git grep`、`git ls-tree`），不需要 checkout
   - **其他场景**：使用 ripgrep 或文件系统搜索

2. **性能优化**：
   - 使用 `git grep`、`ripgrep` 或 GitHub MCP 进行搜索（不读取文件内容）
   - 智能过滤相关目录和文件类型
   - 并行处理多个搜索任务
   - 缓存搜索结果

3. **大代码库处理**：
   - 增量搜索（只搜索相关部分）
   - 延迟加载（按需读取文件）
   - 考虑索引机制（如果代码库非常大）

### 关键优势

- ✅ **不需要 checkout**：使用 Git 命令或 GitHub MCP 直接访问
- ✅ **性能优秀**：Git grep、ripgrep 和 GitHub MCP 都经过高度优化
- ✅ **内存友好**：不加载整个代码库到内存
- ✅ **灵活可配置**：支持多种策略和过滤选项
- ✅ **平台适配**：GitHub MCP 适用于 GitHub 仓库，无需本地仓库

