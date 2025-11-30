# Summarize 功能中新增测试计划步骤分析

## 📋 当前流程分析

### 现有的多步骤流程

根据代码和日志，当前的 `workflow pr summarize` 流程是：

```
1. Fetching PR information
   ↓
2. Fetching PR diff
   ↓
3. Generating summary with LLM
   - 这一步生成整个总结文档
   - 包括：Overview, Requirements, Key Changes, Testing, Usage 等
   - 但 Testing 部分可能不够详细
   ↓
4. Parsing PR diff to extract file changes
   - 解析 diff，提取每个文件的修改
   ↓
5. Generating summary for each file
   - 为每个文件单独调用 LLM
   - 生成每个文件的修改总结
   - 添加到 "Code Changes" 部分
   ↓
6. Merging and saving
   - 合并总结和代码变更部分
   - 保存到文件
```

### 问题分析

**当前 Testing 部分的问题**：
- 在步骤 3 中一次性生成，可能不够详细
- 没有获取额外的代码上下文（接口定义、调用点等）
- 没有针对测试计划的专门优化

## 🎯 解决方案：新增测试计划步骤

### 方案设计

**新增步骤 6：专门生成测试计划**

```
1. Fetching PR information
   ↓
2. Fetching PR diff
   ↓
3. Generating summary with LLM
   - 生成基础总结（不包括详细的测试计划）
   - Testing 部分只包含简单的测试说明
   ↓
4. Parsing PR diff to extract file changes
   ↓
5. Generating summary for each file
   ↓
6. Generating test plan (NEW STEP)
   - 分析 PR diff 和文件变更
   - 识别接口/组件
   - 获取额外代码上下文（可选）
   - 使用专门的 LLM prompt 生成详细测试计划
   ↓
7. Merging and saving
   - 合并所有部分
   - 将测试计划插入到 Testing 部分的 "Test Plan" 子节
   - 保存到文件
```

## 📊 详细设计

### 步骤 6：生成测试计划

#### 6.1 输入数据

```rust
struct TestPlanInput {
    pr_title: String,
    pr_diff: String,
    file_changes: Vec<(String, String)>,  // (file_path, diff_content)
    summary: String,  // 已生成的基础总结
}
```

#### 6.2 处理流程

```rust
fn generate_test_plan(input: &TestPlanInput) -> Result<String> {
    // 1. 从 PR diff 和文件变更中识别接口/组件
    let endpoints = identify_endpoints(&input.pr_diff, &input.file_changes)?;
    let components = identify_components(&input.pr_diff, &input.file_changes)?;

    // 2. 获取额外代码上下文（可选，后续版本支持）
    let additional_context = if should_fetch_context() {
        fetch_additional_context(&endpoints, &components)?
    } else {
        None
    };

    // 3. 构建测试计划 prompt
    let prompt = build_test_plan_prompt(
        &input.pr_title,
        &input.pr_diff,
        &input.file_changes,
        &endpoints,
        &components,
        additional_context.as_ref(),
    )?;

    // 4. 调用 LLM 生成测试计划
    let test_plan = PullRequestLLM::generate_test_plan(&prompt)?;

    Ok(test_plan)
}
```

#### 6.3 接口/组件识别

```rust
// 从 PR diff 和文件变更中识别接口
fn identify_endpoints(
    pr_diff: &str,
    file_changes: &[(String, String)],
) -> Result<Vec<EndpointInfo>> {
    let mut endpoints = Vec::new();

    // 在 PR diff 中搜索接口定义模式
    // Rust: #[post("/api/users")]
    // Spring Boot: @PostMapping("/api/users")
    // Express: router.post('/api/users')
    // ...

    // 在文件变更中搜索
    for (file_path, content) in file_changes {
        let found = search_endpoints_in_content(file_path, content)?;
        endpoints.extend(found);
    }

    Ok(endpoints)
}
```

#### 6.4 额外代码上下文获取（可选）

**策略选择**：根据仓库类型和可用性，自动选择最优方案

```rust
enum ContextFetchStrategy {
    /// 使用 GitHub MCP（GitHub 仓库，无需本地仓库）
    GitHubMCP,
    /// 使用 Git 命令（本地 Git 仓库）
    GitGrep,
    /// 使用 ripgrep（如果系统已安装）
    RipGrep,
    /// 使用文件系统搜索（fallback）
    FileSystem,
}

impl ContextFetchStrategy {
    fn detect() -> Self {
        // 检查是否是 GitHub 仓库且 MCP 可用
        if Self::is_github_repo() && Self::is_mcp_available() {
            return ContextFetchStrategy::GitHubMCP;
        }

        // 检查 ripgrep 是否可用
        if Command::new("rg").output().is_ok() {
            return ContextFetchStrategy::RipGrep;
        }

        // 检查是否在 Git 仓库中
        if Path::new(".git").exists() {
            return ContextFetchStrategy::GitGrep;
        }

        // Fallback 到文件系统
        ContextFetchStrategy::FileSystem
    }
}
```

**获取接口的完整定义**：

```rust
// 获取接口的完整定义
fn fetch_endpoint_definitions(
    endpoints: &[EndpointInfo],
    strategy: &ContextFetchStrategy,
) -> Result<Vec<(String, String)>> {
    let mut definitions = Vec::new();

    for endpoint in endpoints {
        let definition = match strategy {
            ContextFetchStrategy::GitHubMCP => {
                // 使用 GitHub MCP 获取文件内容
                fetch_endpoint_via_github_mcp(&endpoint)?
            }
            ContextFetchStrategy::GitGrep => {
                // 使用 git grep 搜索接口定义
                fetch_endpoint_via_git_grep(&endpoint)?
            }
            ContextFetchStrategy::RipGrep => {
                // 使用 ripgrep 搜索接口定义
                fetch_endpoint_via_ripgrep(&endpoint)?
            }
            ContextFetchStrategy::FileSystem => {
                // 使用文件系统搜索
                fetch_endpoint_via_filesystem(&endpoint)?
            }
        };

        definitions.push((endpoint.path.clone(), definition));
    }

    Ok(definitions)
}

// 使用 GitHub MCP 获取接口定义
fn fetch_endpoint_via_github_mcp(endpoint: &EndpointInfo) -> Result<String> {
    // 从 Git remote URL 提取 owner/repo
    let (owner, repo) = extract_github_repo_info()?;

    // 获取接口定义文件的内容
    let content = mcp_github_get_file_contents(
        &owner,
        &repo,
        &endpoint.file_path,
        Some("main"),  // 或从 PR 获取目标分支
    )?;

    // 提取接口定义部分（限制长度）
    extract_endpoint_definition_from_content(&content, endpoint)
}

// 使用 Git grep 获取接口定义
fn fetch_endpoint_via_git_grep(endpoint: &EndpointInfo) -> Result<String> {
    // 使用 git grep 搜索接口路径
    let output = Command::new("git")
        .args(&["grep", "-n", "-A", "20", &endpoint.path])
        .output()?;

    // 解析输出，提取接口定义代码
    parse_git_grep_output(&output.stdout)
}

// 使用 ripgrep 获取接口定义
fn fetch_endpoint_via_ripgrep(endpoint: &EndpointInfo) -> Result<String> {
    let output = Command::new("rg")
        .args(&["-n", "-A", "20", &endpoint.path])
        .output()?;

    parse_ripgrep_output(&output.stdout)
}
```

#### 6.5 LLM Prompt 构建

```rust
fn build_test_plan_prompt(
    pr_title: &str,
    pr_diff: &str,
    file_changes: &[(String, String)],
    endpoints: &[EndpointInfo],
    components: &[ComponentInfo],
    additional_context: Option<&AdditionalContext>,
) -> Result<String> {
    let mut parts = vec![
        format!("PR Title: {}", pr_title),
        format!("PR Diff:\n{}", pr_diff),
    ];

    // 添加识别的接口信息
    if !endpoints.is_empty() {
        parts.push("## Identified Endpoints".to_string());
        for endpoint in endpoints {
            parts.push(format!("- {} {} ({})",
                endpoint.method, endpoint.path, endpoint.file_path));
        }
    }

    // 添加额外代码上下文
    if let Some(ctx) = additional_context {
        if !ctx.endpoint_definitions.is_empty() {
            parts.push("## Endpoint Definitions".to_string());
            for (endpoint, code) in &ctx.endpoint_definitions {
                parts.push(format!("### {}\n```rust\n{}\n```", endpoint, code));
            }
        }
    }

    parts.join("\n\n")
}
```

#### 6.6 测试计划生成

```rust
impl PullRequestLLM {
    pub fn generate_test_plan(prompt: &str) -> Result<String> {
        let client = LLMClient::global();

        let system_prompt = generate_test_plan_system_prompt();
        let user_prompt = prompt.to_string();

        let params = LLMRequestParams {
            system_prompt,
            user_prompt,
            max_tokens: Some(2000),  // 测试计划可能需要更多 token
            temperature: 0.3,
            model: String::new(),
        };

        let response = client.call(&params)?;
        Ok(response)
    }
}
```

### 步骤 7：合并和保存

#### 7.1 插入测试计划到文档

```rust
fn merge_test_plan_into_summary(
    summary: &str,
    test_plan: &str,
) -> String {
    // 查找 "## Testing" 部分
    // 在 "### Test Description" 之后插入 "### Test Plan"

    if summary.contains("## Testing") {
        // 查找 "### Test Description" 的位置
        if let Some(pos) = summary.find("### Test Description") {
            // 查找 "### Test Description" 部分的结束位置
            let test_desc_end = find_next_section_start(&summary[pos..]);

            // 插入测试计划
            format!(
                "{}\n\n### Test Plan\n\n{}\n\n{}",
                &summary[..pos + test_desc_end],
                test_plan,
                &summary[pos + test_desc_end..]
            )
        } else {
            // 如果没有 "### Test Description"，在 "## Testing" 后添加
            summary.replace(
                "## Testing",
                &format!("## Testing\n\n### Test Description\n\n(To be filled)\n\n### Test Plan\n\n{}", test_plan)
            )
        }
    } else {
        // 如果没有 "## Testing" 部分，添加整个部分
        format!("{}\n\n## Testing\n\n### Test Description\n\n(To be filled)\n\n### Test Plan\n\n{}", summary, test_plan)
    }
}
```

## 🔄 与现有流程的集成

### 修改步骤 3：基础总结生成

**当前**：在 prompt 中要求生成完整的 Testing 部分

**修改后**：在 prompt 中说明：
- Testing 部分只需要包含 "Test Description"
- "Test Plan" 部分会在后续步骤中单独生成

```rust
// 在 summarize_pr.system.rs 中修改
r#"
7. **Testing**: Testing approach or test coverage
   The Testing section MUST contain:

   a. **### Test Description**:
      - Analyze the code changes to determine testing needs
      - Unit tests added/modified (if any, clearly state if none)
      - Integration tests (if any, clearly state if none)
      - Manual testing steps or scenarios
      - Test coverage information (if available)
      - If no tests are found in the changes, suggest what should be tested or state "No tests included in this PR"
      - DO NOT simply write "No specific testing details provided" - always provide testing guidance based on the changes

   b. **### Test Plan**:
      - DO NOT generate this section in the initial summary
      - This section will be generated separately in a later step
      - Just include a placeholder: "### Test Plan\n\n(To be generated)"
"#
```

### 新增步骤 6：测试计划生成

**位置**：在步骤 5（逐个文件总结）之后，步骤 7（合并保存）之前

**实现**：
1. 创建新的函数 `generate_test_plan()`
2. 创建新的 LLM 方法 `PullRequestLLM::generate_test_plan()`
3. 创建新的 prompt `generate_test_plan_system_prompt()`

### 修改步骤 7：合并和保存

**修改**：在合并总结和代码变更时，同时插入测试计划

## 📝 新的 Prompt 设计

### Test Plan System Prompt

```rust
pub fn generate_test_plan_system_prompt() -> String {
    r#"You're a technical testing assistant that generates detailed test plans based on PR changes.

## Test Plan Generation Rules

Generate a comprehensive test plan in Markdown format that includes:

### API Testing (if applicable)

For each modified or added API endpoint, provide:

1. **Endpoint Information**:
   - HTTP method and path
   - Purpose and description
   - Test priority (High/Medium/Low)

2. **Parameters**:
   - Path parameters (if any)
   - Query parameters (if any)
   - Request body parameters (if any)
   - Parameter types and whether they are required

3. **Suggested Test Data**:
   - Example values for each parameter
   - Format as JSON if applicable

4. **Expected Response**:
   - Status code
   - Response body structure (if applicable)

5. **CURL Command**:
   - Complete CURL command with all parameters
   - Include headers (Content-Type, Authorization, etc.)
   - Use placeholder for authentication token: `<token>`
   - Use placeholder for base URL if not provided: `<base_url>`

6. **Test Scenarios**:
   - Normal case (happy path)
   - Validation cases (missing required fields, invalid formats)
   - Edge cases (boundary values, special characters)
   - Error handling (server errors, not found, etc.)

### Component Testing (if applicable)

For each modified or added frontend component, provide:
- Component name and purpose
- Test scenarios
- User interaction flows to test
- Edge cases to consider

### Integration Testing

- End-to-end test scenarios
- Cross-module interaction tests
- Database/API integration tests

### Testing Priority

- High: New features, critical bug fixes
- Medium: Significant modifications
- Low: Minor changes, refactoring

## Response Format

Return your response as a Markdown document starting with "### Test Plan" heading.

If no API endpoints or components are modified, state "No API or component changes requiring specific test plans."

**Important**:
- Generate executable CURL commands
- Use clear, descriptive test scenario names
- Include all necessary parameters and headers
- Provide realistic test data examples
"#
}
```

## 🔍 代码上下文获取策略详解

### 是否需要使用 GitHub MCP？

**答案**：取决于仓库类型和场景，推荐使用混合策略。

#### 策略选择逻辑

```
1. 检查是否是 GitHub 仓库
   ├─ 是 → 检查 GitHub MCP 是否可用
   │   ├─ 可用 → 使用 GitHub MCP ⭐（推荐）
   │   └─ 不可用 → 检查本地 Git 仓库
   │       ├─ 有 → 使用 Git grep
   │       └─ 无 → 使用 ripgrep 或文件系统
   └─ 否 → 检查本地 Git 仓库
       ├─ 有 → 使用 Git grep
       └─ 无 → 使用 ripgrep 或文件系统
```

#### GitHub MCP 的优势

**适用于 GitHub 仓库时，GitHub MCP 是首选**：

1. ✅ **无需本地仓库**：
   - 不需要 clone 或 checkout 代码
   - 可以直接访问远程仓库内容
   - 适合 CI/CD 环境或临时分析

2. ✅ **直接访问远程内容**：
   - 可以获取特定分支的文件
   - 可以获取 PR 的文件列表
   - 可以搜索整个代码库

3. ✅ **性能优秀**：
   - GitHub API 经过优化
   - 支持并行请求
   - 不占用本地磁盘空间

4. ✅ **与现有流程集成**：
   - summarize 功能已经使用 GitHub API 获取 PR 信息
   - 可以复用相同的认证和配置

#### 实现示例

```rust
/// 代码上下文获取器
pub struct CodeContextFetcher {
    strategy: ContextFetchStrategy,
    owner: Option<String>,
    repo: Option<String>,
}

impl CodeContextFetcher {
    /// 创建获取器，自动检测最优策略
    pub fn new() -> Result<Self> {
        let strategy = ContextFetchStrategy::detect();

        // 如果是 GitHub 仓库，提取 owner/repo
        let (owner, repo) = if matches!(strategy, ContextFetchStrategy::GitHubMCP) {
            let (o, r) = extract_github_repo_info()?;
            (Some(o), Some(r))
        } else {
            (None, None)
        };

        Ok(Self {
            strategy,
            owner,
            repo,
        })
    }

    /// 获取接口定义
    pub fn fetch_endpoint_definition(
        &self,
        endpoint: &EndpointInfo,
    ) -> Result<String> {
        match &self.strategy {
            ContextFetchStrategy::GitHubMCP => {
                self.fetch_via_github_mcp(endpoint)
            }
            ContextFetchStrategy::GitGrep => {
                self.fetch_via_git_grep(endpoint)
            }
            ContextFetchStrategy::RipGrep => {
                self.fetch_via_ripgrep(endpoint)
            }
            ContextFetchStrategy::FileSystem => {
                self.fetch_via_filesystem(endpoint)
            }
        }
    }

    /// 使用 GitHub MCP 获取
    fn fetch_via_github_mcp(&self, endpoint: &EndpointInfo) -> Result<String> {
        let owner = self.owner.as_ref()
            .context("GitHub owner not available")?;
        let repo = self.repo.as_ref()
            .context("GitHub repo not available")?;

        // 获取文件内容
        let content = mcp_github_get_file_contents(
            owner,
            repo,
            &endpoint.file_path,
            Some("main"),  // 可以从 PR 获取目标分支
        )?;

        // 提取接口定义部分
        extract_endpoint_definition_from_content(&content, endpoint)
    }

    /// 使用 Git grep 获取
    fn fetch_via_git_grep(&self, endpoint: &EndpointInfo) -> Result<String> {
        let output = Command::new("git")
            .args(&["grep", "-n", "-A", "20", &endpoint.path])
            .output()?;

        parse_git_grep_output(&output.stdout)
    }

    // ... 其他方法
}
```

#### 何时使用 GitHub MCP？

**推荐使用 GitHub MCP 的场景**：

1. ✅ **GitHub 仓库**：
   - 仓库托管在 GitHub
   - 需要访问远程内容（不需要本地仓库）

2. ✅ **CI/CD 环境**：
   - 在 CI/CD 中运行 summarize
   - 没有本地 Git 仓库

3. ✅ **临时分析**：
   - 临时分析某个 PR
   - 不想 clone 整个仓库

4. ✅ **多仓库场景**：
   - 需要分析多个仓库
   - 不想为每个仓库维护本地副本

**不推荐使用 GitHub MCP 的场景**：

1. ❌ **非 GitHub 仓库**：
   - Codeup、GitLab 等其他平台
   - 需要使用 Git grep 或其他方法

2. ❌ **网络不稳定**：
   - 网络连接不稳定
   - API 调用可能失败

3. ❌ **API 速率限制**：
   - 频繁调用可能触发速率限制
   - 需要大量上下文时

#### 混合策略实现

```rust
/// 智能选择获取策略
impl ContextFetchStrategy {
    fn detect() -> Self {
        // 1. 检查是否是 GitHub 仓库
        if let Ok(repo_type) = GitRepo::get_repo_type() {
            if repo_type == RepoType::GitHub {
                // 2. 检查 GitHub MCP 是否可用
                if Self::is_mcp_available() {
                    return ContextFetchStrategy::GitHubMCP;
                }
            }
        }

        // 3. 检查 ripgrep 是否可用
        if Command::new("rg").output().is_ok() {
            return ContextFetchStrategy::RipGrep;
        }

        // 4. 检查是否在 Git 仓库中
        if Path::new(".git").exists() {
            return ContextFetchStrategy::GitGrep;
        }

        // 5. Fallback 到文件系统
        ContextFetchStrategy::FileSystem
    }

    fn is_mcp_available() -> bool {
        // 检查 GitHub MCP 服务是否可用
        // 可以通过尝试调用 MCP 函数来判断
        // 或者检查配置中是否有 GitHub token
        Settings::get().github.accounts.first().is_some()
    }
}
```

## ⚙️ 配置选项

### 可选配置

```toml
[summarize.test_plan]
# 是否启用测试计划生成
enabled = true

# 是否获取额外代码上下文
fetch_additional_context = false  # 后续版本支持

# 代码上下文获取策略
# 可选值：auto, github_mcp, git_grep, ripgrep, filesystem
# auto: 自动检测最优策略（推荐）
context_strategy = "auto"

# 测试计划生成的最大 token 数
max_tokens = 2000

# 是否在基础总结中包含测试计划占位符
include_placeholder = true

# 代码上下文的最大长度（字符）
max_context_length = 10000

# 每个接口定义的最大代码行数
max_lines_per_endpoint = 50
```

## 📊 优势分析

### ✅ 优势

1. **专门优化**：
   - 测试计划有专门的 prompt，可以更详细
   - 可以针对测试计划优化 token 使用

2. **可扩展性**：
   - 可以逐步添加功能（代码上下文获取、调用点分析等）
   - 不影响基础总结的生成

3. **灵活性**：
   - 可以配置是否启用测试计划生成
   - 可以配置是否获取额外代码上下文

4. **性能**：
   - 测试计划生成是独立的步骤，可以并行或异步
   - 如果失败，不影响基础总结

### ⚠️ 注意事项

1. **Token 消耗**：
   - 新增一次 LLM 调用，会增加 token 消耗
   - 需要合理控制测试计划 prompt 的长度

2. **错误处理**：
   - 如果测试计划生成失败，应该回退到基础总结
   - 不应该影响整个 summarize 流程

3. **向后兼容**：
   - 如果配置禁用测试计划生成，应该保持原有行为
   - 基础总结中的 Testing 部分应该仍然可用

## 🚀 实施计划

### 阶段一：MVP（最小可行产品）

1. **修改基础总结 prompt**：
   - 在 Testing 部分只生成 "Test Description"
   - 添加 "Test Plan" 占位符

2. **实现测试计划生成**：
   - 创建 `generate_test_plan()` 函数
   - 创建 `PullRequestLLM::generate_test_plan()` 方法
   - 创建 `generate_test_plan_system_prompt()` prompt

3. **集成到 summarize 流程**：
   - 在步骤 5 之后调用测试计划生成
   - 在步骤 7 中合并测试计划到文档

4. **测试验证**：
   - 使用几个真实的 PR 测试
   - 验证测试计划质量

**开发时间**：2-3 天

### 阶段二：增强功能

1. **接口识别**：
   - 实现从 PR diff 识别接口的功能
   - 支持多种框架模式

2. **代码上下文获取**：
   - 实现从代码库搜索接口定义
   - 将额外上下文添加到测试计划 prompt

3. **优化和调优**：
   - 优化 prompt
   - 优化 token 使用
   - 提高测试计划质量

**开发时间**：3-5 天

## 📚 参考

- Summarize 测试步骤分析：`docs/requirements/SUMMARIZE_TEST_STEP_ANALYSIS.md`
- 代码上下文获取分析：`docs/requirements/SUMMARIZE_CODE_CONTEXT_ANALYSIS.md`
- PR 测试方案分析：`docs/requirements/PR_TEST_SCHEME_ANALYSIS.md`

