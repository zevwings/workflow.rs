# PR 测试功能实施计划

## 📋 文档概述

本文档是 PR 测试功能的**最佳实施计划**，整合了所有需求文档，提供：
- ✅ 清晰的实施路径和优先级
- ✅ 详细的实现步骤和代码结构
- ✅ 依赖关系和技术选型
- ✅ 测试策略和验收标准
- ✅ 风险评估和应对方案

## 🎯 实施目标

### 核心目标
为 PR 修改提供全面的测试支持，包括：
1. **测试需求分析**：自动识别需要测试的内容
2. **接口自动化测试**：自动执行接口测试并生成报告
3. **测试建议生成**：基于代码变更生成测试建议

### 成功标准
- ✅ 能够自动识别 PR 修改的接口和组件
- ✅ 能够搜索代码库找到调用点
- ✅ 能够自动执行接口测试
- ✅ 能够生成清晰的测试报告和建议

## 📊 实施优先级

### 优先级 P0（必须实现 - MVP）
1. **代码库搜索模块** - 基础设施，其他功能依赖
2. **接口识别模块** - 核心功能，识别修改的接口
3. **测试需求分析命令** - 基础分析功能

### 优先级 P1（重要功能）
4. **接口测试参数生成** - 基础测试数据生成
5. **接口测试执行** - 基础 HTTP 请求和验证
6. **测试报告生成** - 结果展示

### 优先级 P2（增强功能）
7. **调用点查找** - 查找前端调用、其他服务调用
8. **高级参数生成** - 支持复杂类型（Object、Array）
9. **响应验证增强** - 结构验证、业务逻辑验证

### 优先级 P3（高级功能）
10. **AST 解析支持** - 提高识别准确度
11. **索引机制** - 性能优化
12. **CI/CD 集成** - 自动化集成

## 🏗️ 架构设计

### 模块结构

```
src/
├── lib/
│   ├── codebase/              # 代码库访问模块（新增）
│   │   ├── mod.rs
│   │   ├── search.rs          # 代码库搜索（GitHub MCP、Git grep、ripgrep）
│   │   └── filter.rs          # 智能过滤
│   │
│   └── pr/
│       └── test/              # PR 测试模块（新增）
│           ├── mod.rs
│           ├── endpoint.rs    # 接口识别
│           ├── analysis.rs    # 测试需求分析
│           ├── api.rs         # 接口测试执行
│           ├── generator.rs   # 测试参数生成
│           └── report.rs      # 测试报告生成
│
└── commands/
    └── pr/
        ├── test_analysis.rs   # test-analysis 命令（新增）
        └── test_api.rs        # test-api 命令（新增）
```

### 数据流

```
PR Diff
  ↓
[接口识别模块] → 识别接口路径和方法
  ↓
[代码库搜索模块] → 搜索接口定义和调用点
  ↓
[测试需求分析模块] → 生成测试建议
  ↓
[接口测试模块] → 生成参数 → 执行测试 → 验证响应
  ↓
[测试报告模块] → 生成报告
```

## 🚀 实施步骤

### 阶段一：基础设施（P0 - 2-3 天）

#### 1.1 代码库搜索模块

**文件**：`src/lib/codebase/search.rs`

**功能**：
- 支持多种搜索策略（GitHub MCP、Git grep、ripgrep、文件系统）
- 自动检测最优策略
- 智能过滤和缓存

**实现步骤**：

```rust
// 1. 定义搜索策略枚举
pub enum SearchStrategy {
    GitHubMCP,
    GitGrep,
    RipGrep,
    FileSystem,
}

// 2. 实现策略检测
impl SearchStrategy {
    pub fn detect() -> Self {
        // 检查是否是 GitHub 仓库且 MCP 可用
        if Self::is_github_repo() && Self::is_mcp_available() {
            return SearchStrategy::GitHubMCP;
        }
        // ... 其他检测逻辑
    }
}

// 3. 实现搜索接口
pub struct CodebaseSearcher {
    strategy: SearchStrategy,
    cache: SearchCache,
}

impl CodebaseSearcher {
    pub fn search(&self, pattern: &str) -> Result<Vec<SearchResult>> {
        // 检查缓存
        if let Some(cached) = self.cache.get(pattern) {
            return Ok(cached);
        }

        // 执行搜索
        let results = match self.strategy {
            SearchStrategy::GitHubMCP => self.search_via_github_mcp(pattern)?,
            SearchStrategy::GitGrep => self.search_via_git_grep(pattern)?,
            SearchStrategy::RipGrep => self.search_via_ripgrep(pattern)?,
            SearchStrategy::FileSystem => self.search_via_filesystem(pattern)?,
        };

        // 缓存结果
        self.cache.set(pattern, results.clone());
        Ok(results)
    }
}
```

**验收标准**：
- ✅ 能够检测并选择最优搜索策略
- ✅ 能够搜索代码库找到匹配项
- ✅ 支持缓存机制
- ✅ 支持智能过滤（目录、文件类型）

#### 1.2 接口识别模块

**文件**：`src/lib/pr/test/endpoint.rs`

**功能**：
- 从 PR diff 识别接口路径和方法
- 支持多种框架模式
- 使用 LLM 解析接口定义

**实现步骤**：

```rust
// 1. 定义接口信息结构
pub struct ApiEndpoint {
    pub path: String,
    pub method: HttpMethod,
    pub file_path: String,
    pub line_number: Option<u32>,
}

// 2. 实现接口识别
pub struct EndpointIdentifier;

impl EndpointIdentifier {
    pub fn identify_from_diff(diff: &str) -> Result<Vec<ApiEndpoint>> {
        let file_changes = parse_diff_to_file_changes(diff)?;
        let mut endpoints = Vec::new();

        for (file_path, content) in file_changes {
            // 使用正则表达式识别常见模式
            let found = Self::identify_by_patterns(&file_path, &content)?;
            endpoints.extend(found);
        }

        Ok(endpoints)
    }

    fn identify_by_patterns(file_path: &str, content: &str) -> Result<Vec<ApiEndpoint>> {
        // Rust: #[get("/api/users")]
        // Spring Boot: @GetMapping("/api/users")
        // Express: router.get('/api/users')
        // ... 其他模式
    }
}
```

**验收标准**：
- ✅ 能够识别常见框架的接口定义
- ✅ 能够提取接口路径和方法
- ✅ 支持 Rust、Spring Boot、Express 等框架

### 阶段二：测试需求分析（P0 - 2-3 天）

#### 2.1 测试需求分析模块

**文件**：`src/lib/pr/test/analysis.rs`

**功能**：
- 分析 PR 修改
- 识别测试需求
- 生成测试建议

**实现步骤**：

```rust
pub struct TestAnalysis;

impl TestAnalysis {
    pub fn analyze(pr_diff: &str) -> Result<TestAnalysisResult> {
        // 1. 识别修改的接口/组件
        let endpoints = EndpointIdentifier::identify_from_diff(pr_diff)?;
        let components = ComponentIdentifier::identify_from_diff(pr_diff)?;

        // 2. 查找调用点
        let searcher = CodebaseSearcher::new()?;
        let mut call_sites = Vec::new();

        for endpoint in &endpoints {
            let calls = searcher.search(&endpoint.path)?;
            call_sites.extend(calls);
        }

        // 3. 使用 LLM 生成测试建议
        let suggestions = Self::generate_suggestions_with_llm(
            &endpoints,
            &components,
            &call_sites,
        )?;

        Ok(TestAnalysisResult {
            endpoints,
            components,
            call_sites,
            suggestions,
        })
    }
}
```

**验收标准**：
- ✅ 能够识别修改的接口和组件
- ✅ 能够查找调用点
- ✅ 能够生成测试建议

#### 2.2 测试需求分析命令

**文件**：`src/commands/pr/test_analysis.rs`

**功能**：
- 实现 `workflow pr test-analysis` 命令
- 生成测试需求文档

**实现步骤**：

```rust
pub struct TestAnalysisCommand;

impl TestAnalysisCommand {
    pub fn run(pr_id: Option<String>) -> Result<String> {
        // 1. 获取 PR diff
        let provider = create_provider()?;
        let pr_id = resolve_pr_id(pr_id)?;
        let pr_diff = provider.get_pull_request_diff(&pr_id)?;

        // 2. 执行分析
        let analysis = TestAnalysis::analyze(&pr_diff)?;

        // 3. 生成报告
        let report = TestReport::generate_analysis_report(&analysis)?;

        // 4. 保存报告
        let output_path = Self::save_report(&pr_id, &report)?;

        Ok(output_path)
    }
}
```

**验收标准**：
- ✅ 命令能够正常执行
- ✅ 能够生成 Markdown 格式的测试需求文档
- ✅ 文档包含接口列表、调用点、测试建议

### 阶段三：接口自动化测试（P1 - 3-4 天）

#### 3.1 测试参数生成模块

**文件**：`src/lib/pr/test/generator.rs`

**功能**：
- 根据参数类型生成测试数据
- 支持基础类型（String、Number、Boolean）
- 从配置文件读取公共参数

**实现步骤**：

```rust
pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn generate_for_endpoint(
        endpoint: &ApiEndpoint,
        definition: &EndpointDefinition,
    ) -> Result<TestRequest> {
        // 1. 生成路径参数
        let path_params = Self::generate_path_params(&definition.path_params)?;

        // 2. 生成查询参数
        let query_params = Self::generate_query_params(&definition.query_params)?;

        // 3. 生成请求体
        let body = if let Some(body_def) = &definition.body {
            Some(Self::generate_body(&body_def)?)
        } else {
            None
        };

        // 4. 读取公共参数
        let settings = Settings::get();
        let common_headers = settings.pr_test.api.common_headers.clone();

        Ok(TestRequest {
            path_params,
            query_params,
            body,
            headers: common_headers,
        })
    }

    fn generate_path_params(params: &[Param]) -> Result<HashMap<String, String>> {
        // 根据参数类型生成测试值
    }
}
```

**验收标准**：
- ✅ 能够根据类型生成测试数据
- ✅ 支持 String、Number、Boolean
- ✅ 能够读取公共参数配置

#### 3.2 接口测试执行模块

**文件**：`src/lib/pr/test/api.rs`

**功能**：
- 执行接口测试
- 验证响应结果
- 生成测试结果

**实现步骤**：

```rust
pub struct ApiTester;

impl ApiTester {
    pub async fn test_endpoint(
        endpoint: &ApiEndpoint,
        request: &TestRequest,
    ) -> Result<TestResult> {
        // 1. 构建完整 URL
        let url = Self::build_url(&endpoint.path, &request.path_params)?;

        // 2. 构建请求配置
        let client = HttpClient::global()?;
        let mut config = RequestConfig::new()
            .headers(&request.headers);

        // 3. 发送请求
        let start = Instant::now();
        let response = match endpoint.method {
            HttpMethod::Get => client.get(&url, config)?,
            HttpMethod::Post => {
                config = config.body(&request.body);
                client.post(&url, config)?
            },
            // ... 其他方法
        };
        let duration = start.elapsed();

        // 4. 验证响应
        let validation = Self::validate_response(&response, &endpoint)?;

        Ok(TestResult {
            endpoint: endpoint.clone(),
            request: request.clone(),
            response: TestResponse {
                status_code: response.status(),
                body: response.as_json().ok(),
                duration,
            },
            validation,
        })
    }

    fn validate_response(
        response: &HttpResponse,
        endpoint: &ApiEndpoint,
    ) -> Result<ValidationResult> {
        // 基础验证：状态码
        // TODO: 增强验证（响应体结构、字段类型等）
        Ok(ValidationResult {
            passed: response.status().is_success(),
            checks: vec![],
        })
    }
}
```

**验收标准**：
- ✅ 能够发送 HTTP 请求
- ✅ 能够验证响应状态码
- ✅ 能够记录请求和响应信息

#### 3.3 接口测试命令

**文件**：`src/commands/pr/test_api.rs`

**功能**：
- 实现 `workflow pr test-api` 命令
- 自动识别并测试修改的接口

**实现步骤**：

```rust
pub struct TestApiCommand;

impl TestApiCommand {
    pub async fn run(pr_id: Option<String>, env: Option<String>) -> Result<String> {
        // 1. 获取 PR diff
        let provider = create_provider()?;
        let pr_id = resolve_pr_id(pr_id)?;
        let pr_diff = provider.get_pull_request_diff(&pr_id)?;

        // 2. 识别接口
        let endpoints = EndpointIdentifier::identify_from_diff(&pr_diff)?;

        // 3. 搜索接口定义
        let searcher = CodebaseSearcher::new()?;
        let mut results = Vec::new();

        for endpoint in &endpoints {
            // 搜索接口定义
            let definition = Self::find_endpoint_definition(&searcher, endpoint)?;

            // 生成测试参数
            let request = TestDataGenerator::generate_for_endpoint(endpoint, &definition)?;

            // 执行测试
            let result = ApiTester::test_endpoint(endpoint, &request).await?;
            results.push(result);
        }

        // 4. 生成报告
        let report = TestReport::generate_api_report(&results)?;

        // 5. 保存报告
        let output_path = Self::save_report(&pr_id, &report)?;

        Ok(output_path)
    }
}
```

**验收标准**：
- ✅ 命令能够正常执行
- ✅ 能够自动识别并测试修改的接口
- ✅ 能够生成测试报告

### 阶段四：增强功能（P2 - 2-3 天）

#### 4.1 调用点查找增强

**功能**：
- 查找前端调用点
- 查找其他服务调用点
- 区分不同类型的调用

#### 4.2 高级参数生成

**功能**：
- 支持 Object、Array 类型
- 使用 LLM 生成更合理的测试数据
- 支持测试数据模板

#### 4.3 响应验证增强

**功能**：
- 验证响应体结构
- 验证字段类型
- 业务逻辑验证

## 🔧 技术选型

### 代码库搜索

**优先级**：
1. **GitHub MCP**（GitHub 仓库优先）
2. **Git grep**（本地 Git 仓库）
3. **ripgrep**（文件系统）
4. **文件系统搜索**（fallback）

### 接口识别

**方案**：
- **阶段一**：正则表达式匹配常见模式
- **阶段二**：使用 LLM 解析接口定义
- **阶段三**：AST 解析（可选，提高准确度）

### 测试参数生成

**方案**：
- **阶段一**：基于类型模板生成
- **阶段二**：使用 LLM 生成更合理的测试数据
- **阶段三**：支持测试数据模板和配置文件

## 📝 配置设计

### Settings 扩展

```rust
// src/lib/base/settings/settings.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // ... 现有字段

    #[serde(default)]
    pub pr_test: PrTestSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrTestSettings {
    /// 代码库搜索配置
    #[serde(default)]
    pub search: SearchSettings,

    /// 接口测试配置
    #[serde(default)]
    pub api: ApiTestSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSettings {
    /// 搜索策略：github_mcp, git_grep, ripgrep, filesystem, auto
    #[serde(default = "default_search_strategy")]
    pub strategy: String,

    /// 包含的目录
    #[serde(default)]
    pub include_dirs: Vec<String>,

    /// 排除的目录
    #[serde(default = "default_exclude_dirs")]
    pub exclude_dirs: Vec<String>,

    /// 是否启用缓存
    #[serde(default = "default_true")]
    pub enable_cache: bool,

    /// 缓存过期时间（秒）
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTestSettings {
    /// API 基础 URL
    pub base_url: String,

    /// API Token
    pub token: Option<String>,

    /// Location（地理位置参数）
    pub location: Option<String>,

    /// 公共请求头
    #[serde(default)]
    pub common_headers: HashMap<String, String>,

    /// 测试数据模板目录
    pub test_data_dir: Option<String>,

    /// 环境（dev、test、prod）
    pub environment: Option<String>,
}
```

## 🧪 测试策略

### 单元测试

- 代码库搜索模块：测试各种搜索策略
- 接口识别模块：测试各种框架模式识别
- 测试参数生成：测试各种类型的数据生成

### 集成测试

- 测试需求分析：端到端测试分析流程
- 接口测试：端到端测试执行流程

### 验收测试

- 使用真实 PR 进行测试
- 验证生成的报告质量

## ⚠️ 风险评估

### 风险 1：GitHub MCP 不可用

**影响**：高
**概率**：低
**应对**：
- 提供 fallback 到 Git grep
- 检测 MCP 可用性，自动切换策略

### 风险 2：接口识别准确度低

**影响**：高
**概率**：中
**应对**：
- 使用 LLM 提高识别准确度
- 支持用户手动指定接口
- 提供识别结果预览和确认

### 风险 3：测试参数生成不符合业务逻辑

**影响**：中
**概率**：中
**应对**：
- 支持测试数据模板
- 使用 LLM 生成更合理的测试数据
- 允许用户自定义测试数据

### 风险 4：性能问题（大代码库）

**影响**：中
**概率**：低
**应对**：
- 使用缓存机制
- 智能过滤相关目录
- 支持增量搜索

## 📅 时间估算

| 阶段 | 任务 | 时间 | 优先级 |
|------|------|------|--------|
| 阶段一 | 代码库搜索模块 | 2-3 天 | P0 |
| 阶段一 | 接口识别模块 | 2-3 天 | P0 |
| 阶段二 | 测试需求分析模块 | 2-3 天 | P0 |
| 阶段二 | 测试需求分析命令 | 1-2 天 | P0 |
| 阶段三 | 测试参数生成模块 | 2-3 天 | P1 |
| 阶段三 | 接口测试执行模块 | 2-3 天 | P1 |
| 阶段三 | 接口测试命令 | 1-2 天 | P1 |
| 阶段四 | 增强功能 | 2-3 天 | P2 |

**总计**：14-22 天（约 3-4 周）

## ✅ 验收标准

### MVP 验收标准

- ✅ 能够识别 PR 修改的接口
- ✅ 能够搜索代码库找到调用点
- ✅ 能够生成测试需求文档
- ✅ 能够执行基础接口测试
- ✅ 能够生成测试报告

### 完整功能验收标准

- ✅ 支持多种框架的接口识别
- ✅ 支持多种搜索策略
- ✅ 支持复杂类型的测试参数生成
- ✅ 支持响应验证增强
- ✅ 支持 CI/CD 集成

## 📚 参考文档

- [代码库访问策略](./CODEBASE_ACCESS_STRATEGY.md)
- [PR 测试需求分析](./PR_TEST_ANALYSIS_REQUIREMENTS.md)
- [PR 接口自动化测试](./PR_API_TEST_REQUIREMENTS.md)
- [PR 测试功能总览](./PR_TEST_OVERVIEW.md)

## 🎯 下一步行动

1. **立即开始**：阶段一 - 代码库搜索模块
2. **并行进行**：接口识别模块（可以并行开发）
3. **完成后**：阶段二 - 测试需求分析
4. **最后**：阶段三 - 接口自动化测试

---

**文档版本**：v1.0
**最后更新**：2024
**维护者**：开发团队

