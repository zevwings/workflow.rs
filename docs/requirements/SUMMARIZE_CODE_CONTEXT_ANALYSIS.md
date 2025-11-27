# Summarize 功能中代码上下文获取分析

## 📋 问题分析

### 当前情况

**现有的 summarize 功能**：
- 只使用 PR diff（最多 15000 字符）
- LLM 基于 diff 内容生成总结和测试计划
- 对于大型 PR 或复杂修改，可能缺少足够的上下文

### 问题

1. **接口定义不完整**：
   - PR diff 可能只显示接口的部分修改
   - 缺少完整的参数定义、响应结构
   - 缺少相关的类型定义、验证规则

2. **调用点信息缺失**：
   - 不知道接口在哪里被调用
   - 不知道前端如何调用这个接口
   - 不知道其他服务如何依赖这个接口

3. **测试文件信息缺失**：
   - 不知道是否已有测试文件
   - 不知道测试的覆盖情况
   - 不知道测试的最佳实践

4. **相关代码缺失**：
   - 缺少相关的 Service 层代码
   - 缺少相关的 Model/Entity 定义
   - 缺少相关的配置或依赖

## 🎯 解决方案

### 方案一：智能代码搜索 + 上下文增强（推荐）

**核心思想**：从 PR diff 中识别关键信息，然后在代码库中搜索相关代码，将额外的上下文添加到 LLM prompt 中。

#### 工作流程

```
1. 解析 PR diff
   ↓
2. 识别关键信息（接口路径、函数名、组件名等）
   ↓
3. 在代码库中搜索相关代码
   - 搜索接口定义
   - 搜索调用点
   - 搜索测试文件
   - 搜索相关类型定义
   ↓
4. 提取相关代码片段
   ↓
5. 将额外上下文添加到 LLM prompt
```

#### 实现步骤

**步骤 1：从 PR diff 识别关键信息**

```rust
// 识别接口路径和方法
fn identify_endpoints(diff: &str) -> Vec<EndpointInfo> {
    // 使用正则表达式或 LLM 识别
    // 例如：POST /api/users, GET /api/users/:id
}

// 识别函数名
fn identify_functions(diff: &str) -> Vec<String> {
    // 例如：createUser, getUserById
}

// 识别组件名
fn identify_components(diff: &str) -> Vec<String> {
    // 例如：UserCreate, UserDetail
}
```

**步骤 2：在代码库中搜索相关代码**

```rust
use std::process::Command;

// 搜索接口定义
fn search_endpoint_definition(endpoint: &EndpointInfo) -> Result<String> {
    // 使用 git grep 搜索接口路径
    let output = Command::new("git")
        .args(&["grep", "-n", "-A", "20", &endpoint.path])
        .output()?;

    // 解析输出，提取接口定义代码
    parse_git_grep_output(&output.stdout)
}

// 搜索调用点
fn search_call_sites(function_name: &str) -> Result<Vec<String>> {
    // 使用 git grep 搜索函数调用
    let output = Command::new("git")
        .args(&["grep", "-n", function_name])
        .output()?;

    parse_git_grep_output(&output.stdout)
}

// 搜索测试文件
fn search_test_files(file_path: &str) -> Result<Vec<String>> {
    // 查找对应的测试文件
    // 例如：src/api/users.rs -> tests/api/users_test.rs
    let test_file = infer_test_file_path(file_path);

    if test_file_exists(&test_file) {
        read_file_content(&test_file)
    } else {
        Ok(Vec::new())
    }
}
```

**步骤 3：提取相关代码片段**

```rust
// 提取接口的完整定义
fn extract_endpoint_definition(code: &str, endpoint: &EndpointInfo) -> String {
    // 提取函数签名、参数定义、响应类型等
    // 限制长度，避免 token 过多
}

// 提取相关的类型定义
fn extract_type_definitions(code: &str, types: &[String]) -> String {
    // 提取 struct、enum、interface 等类型定义
}
```

**步骤 4：构建增强的 LLM prompt**

```rust
fn build_enhanced_prompt(
    pr_title: &str,
    pr_diff: &str,
    additional_context: &AdditionalContext,
) -> String {
    let mut parts = vec![
        format!("PR Title: {}", pr_title),
        format!("PR Diff:\n{}", pr_diff),
    ];

    // 添加接口定义
    if !additional_context.endpoint_definitions.is_empty() {
        parts.push("## Endpoint Definitions".to_string());
        for (endpoint, definition) in &additional_context.endpoint_definitions {
            parts.push(format!("### {}\n```\n{}\n```", endpoint, definition));
        }
    }

    // 添加调用点信息
    if !additional_context.call_sites.is_empty() {
        parts.push("## Call Sites".to_string());
        for call_site in &additional_context.call_sites {
            parts.push(format!("- {}", call_site));
        }
    }

    // 添加测试文件信息
    if !additional_context.test_files.is_empty() {
        parts.push("## Existing Test Files".to_string());
        for test_file in &additional_context.test_files {
            parts.push(format!("- {}", test_file));
        }
    }

    parts.join("\n\n")
}
```

### 方案二：分阶段 LLM 调用

**核心思想**：先使用 LLM 识别需要什么上下文，然后再获取相关代码。

#### 工作流程

```
1. 第一次 LLM 调用：分析 PR diff，识别需要的信息
   - 需要哪些接口的完整定义？
   - 需要哪些函数的调用点？
   - 需要哪些相关的类型定义？
   ↓
2. 根据 LLM 的建议，在代码库中搜索相关代码
   ↓
3. 第二次 LLM 调用：使用完整的上下文生成测试计划
```

**优点**：
- LLM 可以智能地决定需要什么上下文
- 避免获取不必要的代码
- 更灵活

**缺点**：
- 需要两次 LLM 调用，成本更高
- 实现更复杂

### 方案三：配置化的上下文获取

**核心思想**：允许用户配置需要获取哪些类型的上下文。

#### 配置示例

```toml
[summarize.context]
# 是否获取接口定义
fetch_endpoint_definitions = true

# 是否获取调用点
fetch_call_sites = true

# 是否获取测试文件
fetch_test_files = true

# 是否获取相关类型定义
fetch_type_definitions = true

# 最大上下文长度（字符）
max_context_length = 10000

# 每个接口的最大代码行数
max_lines_per_endpoint = 50
```

## 📊 推荐的实现方案

### MVP 版本（最小可行产品）

**采用方案一，但简化实现**：

1. **只获取接口定义**：
   - 从 PR diff 识别接口路径
   - 使用 `git grep` 搜索接口定义
   - 提取接口的完整代码（限制长度）

2. **不获取调用点**（后续版本支持）

3. **不获取测试文件**（后续版本支持）

**实现复杂度**：低
**开发时间**：1-2 天

### 完整版本

**采用方案一，完整实现**：

1. **获取接口定义**
2. **获取调用点**（前端调用、其他服务调用）
3. **获取测试文件**（如果存在）
4. **获取相关类型定义**

**实现复杂度**：中
**开发时间**：3-5 天

## 🔧 技术实现细节

### 1. 接口识别

**从 PR diff 中识别接口**：

```rust
// 使用正则表达式识别常见模式
fn identify_endpoints_from_diff(diff: &str) -> Vec<EndpointInfo> {
    let mut endpoints = Vec::new();

    // Rust: #[post("/api/users")]
    let rust_pattern = Regex::new(r#"#\[(get|post|put|delete|patch)\(["']([^"']+)["']\)\]"#)?;

    // Spring Boot: @PostMapping("/api/users")
    let spring_pattern = Regex::new(r#"@(Get|Post|Put|Delete|Patch)Mapping\(["']([^"']+)["']\)"#)?;

    // Express: router.post('/api/users')
    let express_pattern = Regex::new(r#"router\.(get|post|put|delete|patch)\(["']([^"']+)["']"#)?;

    // 在 diff 中搜索这些模式
    // ...

    endpoints
}
```

### 2. 代码搜索

**使用 Git 命令搜索**：

```rust
use std::process::Command;

fn search_codebase(pattern: &str, context_lines: usize) -> Result<String> {
    let output = Command::new("git")
        .args(&[
            "grep",
            "-n",
            "-A", &context_lines.to_string(),
            "-B", "5",  // 前 5 行上下文
            pattern,
        ])
        .output()?;

    Ok(String::from_utf8(output.stdout)?)
}
```

### 3. 代码提取和限制

**提取相关代码片段，限制长度**：

```rust
fn extract_relevant_code(
    code: &str,
    target_line: usize,
    max_lines: usize,
) -> String {
    let lines: Vec<&str> = code.lines().collect();
    let start = target_line.saturating_sub(10);  // 前 10 行
    let end = (target_line + max_lines).min(lines.len());

    lines[start..end].join("\n")
}
```

### 4. 上下文管理

**管理上下文，避免 token 过多**：

```rust
struct AdditionalContext {
    endpoint_definitions: Vec<(String, String)>,  // (endpoint, code)
    call_sites: Vec<String>,
    test_files: Vec<String>,
    max_total_length: usize,
}

impl AdditionalContext {
    fn add_endpoint_definition(&mut self, endpoint: String, code: String) {
        // 检查总长度
        let current_length: usize = self.endpoint_definitions
            .iter()
            .map(|(_, code)| code.len())
            .sum();

        if current_length + code.len() <= self.max_total_length {
            self.endpoint_definitions.push((endpoint, code));
        }
    }
}
```

## 📝 Prompt 增强

### 在 `summarize_pr.system.rs` 中说明如何使用额外上下文

```rust
// 在 prompt 中添加说明
r#"
## Additional Context

If additional code context is provided (endpoint definitions, call sites, etc.),
use this information to generate more detailed and accurate test plans:

- **Endpoint Definitions**: Use the complete endpoint definitions to understand:
  - All parameters (path, query, body)
  - Parameter types and validation rules
  - Response structures
  - Authentication requirements

- **Call Sites**: Use call site information to understand:
  - How the endpoint is used in the codebase
  - What data is typically passed
  - What errors might occur

- **Test Files**: If existing test files are provided, use them to understand:
  - Current test coverage
  - Testing patterns used in the project
  - Test data structures

Generate test plans based on both the PR diff and the additional context provided.
"#
```

### 在 `summarize_user_prompt` 中添加额外上下文

```rust
fn summarize_user_prompt(
    pr_title: &str,
    pr_diff: &str,
    additional_context: Option<&AdditionalContext>,
) -> String {
    let mut parts = vec![
        format!("PR Title: {}", pr_title),
        format!("PR Diff:\n{}", pr_diff),
    ];

    if let Some(ctx) = additional_context {
        if !ctx.endpoint_definitions.is_empty() {
            parts.push("## Additional Context: Endpoint Definitions".to_string());
            for (endpoint, code) in &ctx.endpoint_definitions {
                parts.push(format!("### {}\n```rust\n{}\n```", endpoint, code));
            }
        }

        // 添加其他上下文...
    }

    parts.join("\n\n")
}
```

## ⚠️ 注意事项

### 1. Token 限制

- LLM 有 token 限制（通常 4K-128K）
- 需要限制额外上下文的长度
- 建议：额外上下文不超过 5000-10000 字符

### 2. 性能考虑

- Git 命令搜索可能较慢（大型代码库）
- 建议：并行搜索多个接口
- 建议：缓存搜索结果

### 3. 准确性

- 代码搜索可能返回不相关的结果
- 需要过滤和验证搜索结果
- 建议：使用更精确的搜索模式

### 4. 可选性

- 额外上下文应该是可选的
- 如果搜索失败，应该回退到只使用 PR diff
- 建议：添加配置项控制是否启用

## ✅ 实施建议

### 阶段一：MVP（当前阶段）

1. **只增强 Prompt**：
   - 修改 `summarize_pr.system.rs`，说明如何使用额外上下文
   - 但不实际获取额外上下文
   - 让 LLM 基于现有 PR diff 生成测试计划

2. **测试验证**：
   - 使用几个真实的 PR 测试
   - 验证输出质量

### 阶段二：基础上下文获取

1. **实现接口识别**：
   - 从 PR diff 识别接口路径和方法
   - 使用正则表达式匹配常见模式

2. **实现代码搜索**：
   - 使用 `git grep` 搜索接口定义
   - 提取接口的完整代码（限制长度）

3. **集成到 summarize**：
   - 在 `summarize` 函数中调用代码搜索
   - 将额外上下文添加到 LLM prompt

### 阶段三：完整上下文获取

1. **获取调用点**
2. **获取测试文件**
3. **获取相关类型定义**
4. **优化性能和准确性**

## 📚 参考

- 代码库访问策略：`docs/requirements/CODEBASE_ACCESS_STRATEGY.md`
- PR 测试方案分析：`docs/requirements/PR_TEST_SCHEME_ANALYSIS.md`
- Summarize 测试步骤分析：`docs/requirements/SUMMARIZE_TEST_STEP_ANALYSIS.md`

