# PR 接口自动化测试功能需求分析

## 📋 需求概述

针对当前 PR 修改的接口，实现自动化测试功能：
1. **从 PR diff 识别修改的接口**
2. **在代码库中搜索接口定义**（获取完整接口信息）
3. **生成测试参数**（根据接口定义生成测试数据）
4. **读取公共参数**（token、location 等配置）
5. **使用 HttpClient 发送请求并验证结果**

## 🎯 功能目标

### 核心流程

```
PR Diff
  ↓
识别修改的接口（路径、方法）
  ↓
在代码库中搜索接口定义（获取参数结构、响应结构）
  ↓
生成测试参数（基于参数类型生成测试数据）
  ↓
读取公共参数（token、location、base_url 等）
  ↓
构建完整请求（URL + Headers + Body/Query）
  ↓
使用 HttpClient 发送请求
  ↓
验证响应结果（状态码、响应体结构）
  ↓
生成测试报告
```

## 🔍 可行性分析

### ✅ 高度可行的部分

#### 1. PR Diff 解析和接口识别

**现有能力**：
- ✅ 已有 `parse_diff_to_file_changes()` 函数，可以解析 PR diff
- ✅ 可以提取修改的文件和代码内容
- ✅ 支持多种文件类型识别

**实现方式**：
- 使用正则表达式或 AST 解析识别接口定义
- 支持多种框架模式：
  - Rust: `#[get("/api/users")]`、`router.get("/api/users")`
  - Spring Boot: `@GetMapping("/api/users")`
  - Express: `router.get('/api/users')`
  - FastAPI: `@app.get("/api/users")`

**挑战**：
- 不同语言和框架的模式差异较大
- 需要支持多种路由定义方式（装饰器、函数调用、配置文件等）

#### 2. 代码库搜索接口定义

**现有能力**：
- ✅ 项目在 Git 仓库中运行，可以直接访问代码库
- ✅ 可以使用 `duct` crate 执行 shell 命令（如 `grep`、`ripgrep`）
- ✅ 已有文件搜索的经验（如日志搜索功能）

**实现方式**：
- 使用 `ripgrep`（rg）或 `grep` 搜索接口路径
- 搜索模式：
  - 接口路径：`/api/users`
  - 路由定义：`@GetMapping`、`router.get`、`#[get(`
  - 函数名：`getUser`、`createUser`

**优势**：
- 可以搜索整个代码库，不仅仅是 PR diff
- 可以找到接口的完整定义（包括参数、响应结构）

#### 3. 公共参数管理

**现有能力**：
- ✅ 已有 `Settings` 系统，支持 TOML 配置
- ✅ 已有 `ConfigManager<T>` 泛型配置管理器
- ✅ 支持多种配置项（Jira、GitHub、LLM 等）

**实现方式**：
- 在 `Settings` 中添加 `ApiTestSettings`：
  ```toml
  [api_test]
  base_url = "https://api.example.com"
  token = "your-api-token"
  location = "us-east-1"
  common_headers = { "X-Request-ID" = "test-123" }
  ```

**优势**：
- 复用现有配置系统
- 支持多环境配置（开发、测试、生产）
- 配置安全（文件权限 600）

#### 4. HTTP 请求发送

**现有能力**：
- ✅ 已有完整的 `HttpClient` 实现
- ✅ 支持 GET、POST、PUT、DELETE、PATCH
- ✅ 支持 Basic Auth、自定义 Headers
- ✅ 支持请求体（JSON）、查询参数
- ✅ 支持响应解析（JSON、Text）

**实现方式**：
- 直接使用 `HttpClient::global()`
- 使用 `RequestConfig` 构建请求
- 使用 `HttpResponse` 解析响应

**优势**：
- 无需额外实现 HTTP 客户端
- 已有完善的错误处理
- 支持超时设置

### ⚠️ 需要实现的部分

#### 1. 接口定义解析

**挑战**：
- 需要从代码中提取接口的完整定义：
  - 请求参数结构（Query、Path、Body）
  - 参数类型（String、Number、Boolean、Object）
  - 必填/可选参数
  - 响应结构

**实现方案**：

**方案 A：基于正则表达式（简单但不够准确）**
- 使用正则表达式匹配常见模式
- 快速实现，但可能遗漏复杂情况

**方案 B：基于 AST 解析（准确但复杂）**
- 使用语言特定的 AST 解析器
- Rust: `syn`、`quote`
- JavaScript/TypeScript: 需要 Node.js 环境
- Python: `ast` 模块
- Java: 需要 Java 环境

**方案 C：基于 LLM 智能解析（推荐）**
- 使用现有的 LLM 基础设施
- 将接口定义代码发送给 LLM
- LLM 提取结构化信息（参数、响应等）
- 可以处理多种语言和框架

#### 2. 测试参数生成

**挑战**：
- 需要根据参数类型生成测试数据：
  - String: 生成随机字符串或使用模板
  - Number: 生成随机数字或使用边界值
  - Boolean: true/false
  - Object: 递归生成嵌套对象
  - Array: 生成数组元素

**实现方案**：

**方案 A：基于类型模板生成**
```rust
fn generate_test_value(param_type: &ParamType) -> Value {
    match param_type {
        ParamType::String => json!("test_string"),
        ParamType::Number => json!(123),
        ParamType::Boolean => json!(true),
        ParamType::Object(schema) => generate_object(schema),
        ParamType::Array(item_type) => json!([generate_test_value(item_type)]),
    }
}
```

**方案 B：基于 LLM 生成（更智能）**
- 使用 LLM 根据参数描述和类型生成更合理的测试数据
- 可以生成符合业务逻辑的测试数据

**方案 C：从配置文件读取（最灵活）**
- 允许用户配置测试数据模板
- 支持环境变量替换
- 支持数据文件（JSON、CSV）

#### 3. 响应验证

**挑战**：
- 需要验证响应是否符合预期：
  - 状态码（200、400、500 等）
  - 响应体结构（字段存在、类型匹配）
  - 业务逻辑验证（数据一致性）

**实现方案**：

**方案 A：基础验证（状态码 + 结构）**
```rust
fn validate_response(response: &HttpResponse, expected: &ExpectedResponse) -> Result<()> {
    // 验证状态码
    if response.status() != expected.status_code {
        return Err(anyhow!("Expected status {}, got {}", expected.status_code, response.status()));
    }

    // 验证响应体结构
    let body: Value = response.as_json()?;
    validate_schema(&body, &expected.schema)?;

    Ok(())
}
```

**方案 B：基于 LLM 验证（更智能）**
- 使用 LLM 分析响应是否符合预期
- 可以理解业务逻辑，验证数据一致性

#### 4. 接口定义存储和索引

**挑战**：
- 需要高效查找接口定义
- 支持增量更新（只更新修改的接口）

**实现方案**：

**方案 A：本地缓存（推荐）**
- 解析接口定义后缓存到本地（TOML/JSON）
- 建立索引（路径 → 接口定义）
- 支持增量更新

**方案 B：实时解析（每次重新解析）**
- 每次测试时重新解析接口定义
- 无需缓存，但性能较差

## 📊 技术实现方案

### 方案一：基于代码搜索 + LLM 解析（推荐）

**工作流程**：
1. 从 PR diff 识别接口路径和方法
2. 使用 `ripgrep` 在代码库中搜索接口定义
3. 使用 LLM 解析接口定义（提取参数、响应结构）
4. 生成测试参数（基于类型模板或 LLM）
5. 读取公共参数配置
6. 使用 HttpClient 发送请求
7. 验证响应并生成报告

**优点**：
- 利用现有基础设施（HttpClient、LLM、Settings）
- 可以处理多种语言和框架
- 智能生成测试数据

**缺点**：
- 依赖 LLM，可能有 token 消耗
- 解析可能不够准确（需要多次迭代）

### 方案二：基于 AST 解析（更准确但复杂）

**工作流程**：
1. 从 PR diff 识别接口路径和方法
2. 使用 AST 解析器解析接口定义文件
3. 提取参数和响应结构（类型安全）
4. 基于类型生成测试参数
5. 读取公共参数配置
6. 使用 HttpClient 发送请求
7. 验证响应并生成报告

**优点**：
- 解析准确，类型安全
- 不依赖 LLM

**缺点**：
- 需要为每种语言实现解析器
- 实现复杂度高
- 需要外部依赖（Node.js、Java 等）

### 方案三：混合方案（最灵活）

**结合方案一和方案二**：
1. 优先使用 AST 解析（如果支持该语言）
2. 不支持的语言使用 LLM 解析
3. 其他步骤相同

## 🛠️ 需要的技术组件

### 1. 代码搜索模块

**功能**：
- 在代码库中搜索接口定义
- 支持多种搜索模式（路径、方法、函数名）

**实现**：
- 使用 `duct` 执行 `ripgrep` 命令
- 或使用 Rust 的 `grep` crate（如果可用）

### 2. 接口定义解析模块

**功能**：
- 解析接口定义代码
- 提取参数结构、响应结构

**实现**：
- LLM 解析（使用现有 `PullRequestLLM`）
- 或 AST 解析（需要语言特定解析器）

### 3. 测试参数生成模块

**功能**：
- 根据参数类型生成测试数据
- 支持模板和配置文件

**实现**：
- 基于类型的模板生成
- 或使用 LLM 生成

### 4. 公共参数管理模块

**功能**：
- 读取和管理公共参数（token、location 等）
- 支持多环境配置

**实现**：
- 扩展 `Settings` 结构
- 使用 `ConfigManager` 管理配置

### 5. 接口测试执行模块

**功能**：
- 构建完整请求
- 发送请求并验证响应
- 生成测试报告

**实现**：
- 使用现有 `HttpClient`
- 实现响应验证逻辑

## 📝 数据结构设计

### 接口定义结构

```rust
/// 接口定义
pub struct ApiEndpoint {
    /// 接口路径（如 /api/users）
    pub path: String,
    /// HTTP 方法
    pub method: HttpMethod,
    /// 请求参数
    pub request: RequestDefinition,
    /// 响应定义
    pub response: ResponseDefinition,
    /// 接口描述
    pub description: Option<String>,
    /// 定义文件路径
    pub file_path: String,
}

/// 请求定义
pub struct RequestDefinition {
    /// 路径参数（如 /api/users/:id）
    pub path_params: Vec<Param>,
    /// 查询参数
    pub query_params: Vec<Param>,
    /// 请求体参数
    pub body: Option<BodyDefinition>,
    /// 请求头（自定义）
    pub headers: Vec<Header>,
}

/// 参数定义
pub struct Param {
    /// 参数名
    pub name: String,
    /// 参数类型
    pub param_type: ParamType,
    /// 是否必填
    pub required: bool,
    /// 默认值
    pub default: Option<Value>,
    /// 参数描述
    pub description: Option<String>,
}

/// 参数类型
pub enum ParamType {
    String,
    Number,
    Boolean,
    Object(Schema),
    Array(Box<ParamType>),
}

/// 响应定义
pub struct ResponseDefinition {
    /// 状态码
    pub status_code: u16,
    /// 响应体结构
    pub body: Option<Schema>,
    /// 响应描述
    pub description: Option<String>,
}
```

### 测试配置结构

```rust
/// API 测试配置
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

### 测试结果结构

```rust
/// 测试结果
pub struct TestResult {
    /// 接口信息
    pub endpoint: ApiEndpoint,
    /// 请求信息
    pub request: TestRequest,
    /// 响应信息
    pub response: TestResponse,
    /// 验证结果
    pub validation: ValidationResult,
    /// 测试时间
    pub duration: Duration,
}

/// 测试请求
pub struct TestRequest {
    /// 完整 URL
    pub url: String,
    /// 请求方法
    pub method: HttpMethod,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// 请求体
    pub body: Option<Value>,
    /// 查询参数
    pub query: Option<HashMap<String, String>>,
}

/// 测试响应
pub struct TestResponse {
    /// 状态码
    pub status_code: u16,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 响应体
    pub body: Option<Value>,
    /// 响应时间
    pub duration: Duration,
}

/// 验证结果
pub struct ValidationResult {
    /// 是否通过
    pub passed: bool,
    /// 验证项列表
    pub checks: Vec<ValidationCheck>,
}

/// 验证项
pub struct ValidationCheck {
    /// 验证项名称
    pub name: String,
    /// 是否通过
    pub passed: bool,
    /// 错误信息
    pub error: Option<String>,
}
```

## 🚀 实现计划

### 阶段一：MVP（最小可行产品）

1. **接口识别**
   - 从 PR diff 识别接口路径和方法（基于正则表达式）
   - 支持常见框架模式（Rust、Spring Boot、Express）

2. **代码搜索**
   - 使用 `ripgrep` 搜索接口定义
   - 提取接口定义代码片段

3. **LLM 解析**
   - 使用 LLM 解析接口定义
   - 提取参数和响应结构

4. **基础参数生成**
   - 基于类型模板生成测试数据
   - 支持 String、Number、Boolean

5. **公共参数读取**
   - 扩展 `Settings` 添加 `ApiTestSettings`
   - 读取 token、location、base_url

6. **请求发送**
   - 使用 `HttpClient` 发送请求
   - 基础响应验证（状态码）

### 阶段二：增强功能

1. **高级参数生成**
   - 支持 Object、Array 类型
   - 支持从配置文件读取测试数据
   - 使用 LLM 生成更合理的测试数据

2. **响应验证增强**
   - 验证响应体结构
   - 验证字段类型
   - 业务逻辑验证

3. **接口定义缓存**
   - 本地缓存接口定义
   - 支持增量更新

4. **测试报告生成**
   - 生成 Markdown 格式测试报告
   - 包含请求、响应、验证结果

### 阶段三：高级功能

1. **AST 解析支持**
   - 支持 Rust 代码 AST 解析
   - 提高解析准确度

2. **多环境支持**
   - 支持多环境配置（dev、test、prod）
   - 环境变量替换

3. **测试用例管理**
   - 支持保存和复用测试用例
   - 支持测试套件

4. **CI/CD 集成**
   - 自动在 PR 中运行测试
   - 生成测试报告并添加到 PR 评论

## ⚠️ 潜在挑战与解决方案

### 挑战 1：接口定义格式多样性

**问题**：
- 不同语言和框架有不同的接口定义方式
- 可能使用装饰器、函数调用、配置文件等

**解决方案**：
- 优先支持常见框架（Rust、Spring Boot、Express、FastAPI）
- 使用 LLM 解析，可以处理多种格式
- 支持用户自定义解析规则

### 挑战 2：参数类型推断

**问题**：
- 从代码中推断参数类型可能不准确
- 动态类型语言（JavaScript、Python）类型信息不足

**解决方案**：
- 使用 LLM 分析代码语义
- 支持从注释、文档中提取类型信息
- 允许用户手动指定参数类型

### 挑战 3：测试数据生成

**问题**：
- 自动生成的测试数据可能不符合业务逻辑
- 某些参数需要特定的格式或约束

**解决方案**：
- 支持测试数据模板和配置文件
- 使用 LLM 生成更合理的测试数据
- 支持从数据库或外部数据源读取测试数据

### 挑战 4：响应验证

**问题**：
- 响应验证规则可能很复杂
- 需要理解业务逻辑

**解决方案**：
- 基础验证（状态码、结构）
- 使用 LLM 进行业务逻辑验证
- 支持自定义验证规则

### 挑战 5：性能问题

**问题**：
- 大型代码库搜索可能较慢
- LLM 调用可能有延迟

**解决方案**：
- 使用缓存（接口定义、测试结果）
- 并行处理多个接口测试
- 支持增量更新

## 💡 推荐实现路径

### 最小可行方案（MVP）

1. **使用现有基础设施**
   - 复用 `HttpClient`、`Settings`、`LLM` 模块
   - 使用 `duct` 执行 `ripgrep` 搜索

2. **简化实现**
   - 使用正则表达式识别接口（支持常见模式）
   - 使用 LLM 解析接口定义
   - 基于类型模板生成测试数据

3. **基础功能**
   - 从 PR diff 识别接口
   - 搜索接口定义
   - 生成测试参数
   - 发送请求并验证

### 后续优化

1. **提高准确性**
   - 集成 AST 解析器
   - 支持更多语言和框架

2. **增强功能**
   - 测试数据模板
   - 响应验证增强
   - 测试报告生成

3. **集成和自动化**
   - CI/CD 集成
   - 自动测试执行
   - 测试结果通知

## 📚 参考资源

- 现有 PR summarize 功能：`src/commands/pr/summarize.rs`
- LLM 模块：`src/lib/pr/llm.rs`
- HTTP 客户端：`src/lib/base/http/client.rs`
- Settings 系统：`src/lib/base/settings/settings.rs`
- PR 测试需求分析文档：`docs/requirements/PR_TEST_ANALYSIS_REQUIREMENTS.md`

