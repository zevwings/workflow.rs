# PR 测试功能最终方案

## 📋 文档概述

本文档整合了所有测试方案相关的需求分析，形成最终的、可执行的实施方案。

**文档版本**：v1.0
**最后更新**：2024
**维护者**：开发团队

## 🎯 核心目标

为 PR 修改提供全面的测试支持，包括：

1. **测试计划生成**：在 PR 总结中自动生成详细的测试计划
2. **接口自动化测试**：自动执行接口测试并生成报告
3. **测试建议生成**：基于代码变更生成测试建议

## 📊 整体架构

### 功能模块关系

```
PR 修改
  │
  ├─→ [Summarize 功能增强]
  │     └─→ 在 PR 总结中生成测试计划
  │           ├─→ 测试说明（Test Description）
  │           └─→ 测试计划（Test Plan）- 新增独立步骤
  │
  ├─→ [代码库访问策略]
  │     └─→ 提供高效的代码库搜索能力
  │           ├─→ GitHub MCP（GitHub 仓库优先）
  │           ├─→ Git grep（本地 Git 仓库）
  │           └─→ ripgrep/文件系统（其他场景）
  │
  ├─→ [PR 测试需求分析]
  │     ├─→ 识别修改的接口/组件
  │     ├─→ 查找调用点
  │     └─→ 生成测试建议
  │
  └─→ [PR 接口自动化测试]
        ├─→ 识别修改的接口
        ├─→ 搜索接口定义
        ├─→ 生成测试参数
        └─→ 执行测试并验证
```

## 🚀 实施方案

### 方案一：Summarize 功能增强（当前阶段）

**目标**：在 `workflow pr summarize` 中新增测试计划生成功能

#### 1.1 文档结构增强

**新的 Testing 部分结构**：

```markdown
## Testing

### Test Description
(现有的测试内容：测试覆盖率、测试类型、单元测试、集成测试等)

### Test Plan
(新增的详细测试计划：接口测试、组件测试、CURL 命令等)
```

#### 1.2 流程增强

**新的 summarize 流程**：

```
1. Fetching PR information
   ↓
2. Fetching PR diff
   ↓
3. Generating summary with LLM
   - 生成基础总结
   - Testing 部分只包含 "Test Description"
   - "Test Plan" 部分使用占位符
   ↓
4. Parsing PR diff to extract file changes
   ↓
5. Generating summary for each file
   ↓
6. Generating test plan (NEW STEP) ⭐
   - 分析 PR diff 和文件变更
   - 识别接口/组件
   - 获取额外代码上下文（可选）
   - 使用专门的 LLM prompt 生成详细测试计划
   ↓
7. Merging and saving
   - 将测试计划插入到 Testing 部分的 "Test Plan" 子节
   - 保存到文件
```

#### 1.3 代码上下文获取

**策略选择**（自动检测）：

```
1. 检查是否是 GitHub 仓库
   ├─ 是 → 检查 GitHub MCP 是否可用
   │   ├─ 可用 → 使用 GitHub MCP ⭐（推荐）
   │   └─ 不可用 → 使用 Git grep
   └─ 否 → 使用 Git grep 或其他方法
```

**获取内容**：
- 接口的完整定义（参数、响应结构）
- 调用点信息（前端调用、其他服务调用）
- 相关类型定义
- 现有测试文件（如果存在）

#### 1.4 实施步骤

**阶段一：MVP（当前阶段）**

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

**开发时间**：2-3 天

**阶段二：增强功能**

1. **接口识别**：
   - 从 PR diff 识别接口路径和方法
   - 支持多种框架模式

2. **代码上下文获取**：
   - 实现混合策略自动选择
   - GitHub 仓库优先使用 GitHub MCP
   - 其他场景使用 Git grep 或 ripgrep

**开发时间**：3-5 天

### 方案二：PR 接口自动化测试（后续阶段）

**目标**：实现完整的接口自动化测试功能

#### 2.1 核心流程

```
步骤一：通过 PR 修改内容，定位到需要测试的内容
  ├─ 从 PR diff 识别接口（路径、方法）
  └─ 输出：需要测试的接口列表

[新增] 步骤 1.5：获取接口定义
  ├─ 在代码库中搜索接口定义代码
  ├─ 解析接口定义，提取参数信息
  └─ 识别需要的请求头、认证方式等

[新增] 步骤 1.6：生成测试参数
  ├─ 根据参数类型生成测试数据
  ├─ 读取公共参数配置（token、location 等）
  └─ 合并生成完整请求参数

步骤二：如果是接口，需要拿到返回数据
  ├─ 构建完整请求（URL + Headers + Body/Query）
  ├─ 发送 HTTP 请求
  ├─ 获取响应数据
  └─ [新增] 验证响应（状态码、结构、业务逻辑）

步骤三：拼接一个 CURL 和结果
  ├─ 生成 CURL 命令（包含所有参数）
  ├─ 格式化响应结果
  ├─ [新增] 保存测试结果
  └─ [新增] 生成测试报告
```

#### 2.2 关键补全点

**必须实现（P0）**：

1. ✅ **接口定义获取**（步骤 1.5）
   - 重要性：高（没有接口定义，无法生成正确的测试参数）
   - 实现难度：中（需要代码搜索和解析）

2. ✅ **测试参数生成**（步骤 1.6）
   - 重要性：高（没有测试参数，无法执行测试）
   - 实现难度：低（基础类型简单）

3. ✅ **公共参数管理**（步骤 1.6）
   - 重要性：高（token、location 等是必需的）
   - 实现难度：低（复用现有 Settings 系统）

4. ✅ **基础响应验证**（步骤二之后）
   - 重要性：中（基础验证即可）
   - 实现难度：低（状态码检查很简单）

#### 2.3 实施步骤

**阶段一：MVP**

1. **接口识别**
   - 从 PR diff 识别接口路径和方法（基于正则表达式）
   - 支持常见框架模式（Rust、Spring Boot、Express）

2. **代码搜索**
   - 使用 `git grep` 或 GitHub MCP 搜索接口定义
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

7. **CURL 生成**
   - 生成 CURL 命令和结果

**开发时间**：5-7 天

**阶段二：增强功能**

1. **高级参数生成**：支持 Object、Array 类型
2. **响应验证增强**：结构验证、业务逻辑验证
3. **测试环境管理**：多环境支持
4. **结果存储**：保存和查询历史结果

**开发时间**：3-5 天

## 🔧 技术实现细节

### 代码库访问策略

**混合策略（自动选择）**：

```rust
enum SearchStrategy {
    /// 使用 GitHub MCP（GitHub 仓库，无需本地仓库）
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
}
```

**性能对比**（10,000 个文件的代码库）：

| 方案 | 首次搜索时间 | 内存占用 | 适用场景 |
|------|------------|---------|---------|
| **GitHub MCP** | 3-8秒 | 5MB | GitHub 仓库，无需本地仓库 |
| **Git grep** | 2-5秒 | 10MB | 本地 Git 仓库 |
| **ripgrep** | 1-3秒 | 5MB | 本地文件系统 |
| **文件系统** | 5-10秒 | 50MB | Fallback |

### 接口识别

**支持的模式**：

- Rust: `#[post("/api/users")]`、`router.get("/api/users")`
- Spring Boot: `@GetMapping("/api/users")`
- Express: `router.get('/api/users')`
- FastAPI: `@app.get("/api/users")`

**实现方式**：

1. **阶段一**：正则表达式匹配常见模式
2. **阶段二**：使用 LLM 解析接口定义
3. **阶段三**：AST 解析（可选，提高准确度）

### 测试参数生成

**基础类型支持**：

- String: 生成随机字符串或使用模板
- Number: 生成随机数字或使用边界值
- Boolean: true/false

**高级类型支持**（后续版本）：

- Object: 递归生成嵌套对象
- Array: 生成数组元素

**数据来源**：

1. 基于类型模板生成
2. 从配置文件读取公共参数
3. 使用 LLM 生成更合理的测试数据（可选）

## 📝 配置设计

### 阶段一：MVP（当前阶段）

**第一步只需要实现测试计划生成，无需额外配置**：

- 测试计划生成功能默认启用
- 使用现有的 LLM 配置（从 Settings 读取）
- 不获取额外代码上下文（只基于 PR diff）
- 使用默认的 token 限制

**说明**：
- 当前阶段专注于实现核心功能（测试计划生成）
- 配置优化和代码上下文获取是后续阶段的内容

### 阶段二及以后：配置扩展（后续优化）

**以下配置在后续阶段实现**：

```toml
# Summarize 测试计划配置（后续阶段）
[summarize.test_plan]
# 是否启用测试计划生成
enabled = true

# 代码上下文获取策略（后续阶段）
# 可选值：auto, github_mcp, git_grep, ripgrep, filesystem
context_strategy = "auto"

# 是否获取额外代码上下文（后续阶段）
fetch_additional_context = false

# 测试计划生成的最大 token 数
max_tokens = 2000

# 代码上下文的最大长度（字符）（后续阶段）
max_context_length = 10000

# PR 接口测试配置（后续阶段 - PR 接口自动化测试功能）
[pr_test]
# 代码库搜索配置
search_strategy = "auto"  # git_grep, ripgrep, filesystem, auto
include_dirs = ["src/", "lib/", "app/"]
exclude_dirs = ["node_modules/", "target/", ".git/"]
enable_cache = true
cache_ttl = 3600

# 接口测试配置
[pr_test.api]
base_url = "https://api.example.com"
token = "your-api-token"
location = "us-east-1"
common_headers = { "X-Request-ID" = "test-123" }
test_data_dir = "test_data/"
environment = "dev"
```

## 🗓️ 实施计划

### 阶段一：Summarize 测试计划生成（当前阶段 - MVP）

**目标**：在 PR 总结中生成详细的测试计划

**核心功能**：
- 在 PR 总结的 Testing 部分新增 "Test Plan" 子节
- 基于 PR diff 生成测试计划（不获取额外代码上下文）
- 生成包含 CURL 命令的详细测试计划

**任务**：
1. ✅ 修改 `summarize_pr.system.rs` prompt
   - 在 Testing 部分只生成 "Test Description"
   - 添加 "Test Plan" 占位符说明
2. ✅ 实现测试计划生成函数
   - 创建 `generate_test_plan()` 函数
   - 创建 `PullRequestLLM::generate_test_plan()` 方法
   - 创建 `generate_test_plan_system_prompt()` prompt
3. ✅ 集成到 summarize 流程
   - 在步骤 5（文件总结）之后调用测试计划生成
   - 在步骤 7（合并保存）中插入测试计划到文档
4. ✅ 测试验证
   - 使用几个真实的 PR 测试
   - 验证测试计划质量

**配置需求**：
- ❌ 无需额外配置
- ✅ 使用现有的 LLM 配置
- ✅ 使用默认的 token 限制

**时间**：2-3 天

**优先级**：P0（必须实现）

### 阶段二：代码上下文获取（增强 - 后续优化）

**目标**：获取额外代码上下文，提高测试计划质量

**任务**：
1. 实现接口识别功能
   - 从 PR diff 识别接口路径和方法
   - 支持多种框架模式
2. 实现混合策略代码搜索
   - GitHub MCP（GitHub 仓库优先）
   - Git grep（本地 Git 仓库）
   - ripgrep/文件系统（其他场景）
3. 集成代码上下文到测试计划生成
   - 获取接口的完整定义
   - 获取调用点信息
4. 添加配置选项
   - `context_strategy` 配置
   - `fetch_additional_context` 配置
   - `max_context_length` 配置

**时间**：3-5 天

**优先级**：P1（重要功能 - 后续优化）

### 阶段三：PR 接口自动化测试（后续）

**目标**：实现完整的接口自动化测试功能

**任务**：
1. 实现接口识别模块
2. 实现测试参数生成模块
3. 实现接口测试执行模块
4. 实现测试报告生成模块

**时间**：5-7 天（MVP）+ 3-5 天（增强）

**优先级**：P1（重要功能）

### 阶段四：高级功能（可选）

**目标**：提供更强大的功能和更好的性能

**任务**：
1. AST 解析支持
2. 索引机制
3. CI/CD 集成

**时间**：根据需求决定

**优先级**：P2/P3（增强/高级功能）

## 📚 文档索引

### 需求分析文档

1. **[Summarize 测试步骤分析](../SUMMARIZE_TEST_STEP_ANALYSIS.md)**
   - 在 summarize 中新增测试步骤的分析
   - 文档结构设计

2. **[Summarize 测试计划步骤分析](../SUMMARIZE_TEST_PLAN_STEP_ANALYSIS.md)**
   - 新增测试计划步骤的详细分析
   - 代码上下文获取策略

3. **[Summarize 代码上下文分析](../SUMMARIZE_CODE_CONTEXT_ANALYSIS.md)**
   - 如何获取代码上下文
   - GitHub MCP 使用方案

4. **[PR 测试方案分析](../PR_TEST_SCHEME_ANALYSIS.md)**
   - 三步骤方案补全分析
   - 关键补全点总结

5. **[PR 测试功能总览](../PR_TEST_OVERVIEW.md)**
   - PR 测试功能体系总览
   - 功能关系图

6. **[PR 测试实施计划](../PR_TEST_IMPLEMENTATION_PLAN.md)**
   - 详细的实施步骤
   - 代码结构设计

7. **[PR 接口自动化测试需求](../PR_API_TEST_REQUIREMENTS.md)**
   - 接口自动化测试功能需求
   - 技术实现方案

8. **[PR 测试需求分析](../PR_TEST_ANALYSIS_REQUIREMENTS.md)**
   - 测试需求分析功能需求
   - 实现方案分析

9. **[代码库访问策略](../CODEBASE_ACCESS_STRATEGY.md)**
   - 代码库访问和搜索策略
   - 性能优化方案

### 实施文档

- **本文档**：最终方案整合
- **PR 测试功能实施计划**：详细的实施指南

## ✅ 总结

### 核心方案

1. **Summarize 功能增强**（当前阶段 - MVP）：
   - 在 PR 总结中新增独立的测试计划生成步骤
   - 基于 PR diff 生成测试计划（不获取额外代码上下文）
   - 生成详细的测试计划，包含 CURL 命令
   - **无需额外配置**，使用现有 LLM 配置

2. **代码上下文获取**（后续优化）：
   - 支持获取额外代码上下文（接口定义、调用点等）
   - 支持多种代码库访问策略（GitHub MCP、Git grep 等）
   - 添加相关配置选项

3. **PR 接口自动化测试**（后续阶段）：
   - 实现完整的三步骤流程（识别 → 测试 → 输出）
   - 补全关键步骤（接口定义获取、参数生成等）
   - 支持多种代码库访问策略

### 技术选型

1. **代码库访问**：
   - GitHub 仓库：优先使用 GitHub MCP
   - 本地 Git 仓库：使用 Git grep
   - 其他场景：使用 ripgrep 或文件系统

2. **接口识别**：
   - 阶段一：正则表达式
   - 阶段二：LLM 解析
   - 阶段三：AST 解析（可选）

3. **测试参数生成**：
   - 基础类型：模板生成
   - 复杂类型：LLM 生成（可选）

### 实施优先级

1. **P0（必须实现 - 当前阶段）**：
   - ✅ Summarize 测试计划生成（MVP）
     - 修改 prompt
     - 实现测试计划生成函数
     - 集成到 summarize 流程
     - **无需额外配置**

2. **P1（重要功能 - 后续优化）**：
   - 代码上下文获取
   - 接口识别功能
   - 配置选项扩展

3. **P1（重要功能 - 后续阶段）**：
   - PR 接口自动化测试（MVP）
   - 基础接口识别
   - 基础参数生成
   - 公共参数管理

4. **P2（增强功能）**：
   - 高级参数生成
   - 响应验证增强
   - 测试环境管理

5. **P3（高级功能）**：
   - AST 解析支持
   - 索引机制
   - CI/CD 集成

### 关键决策

1. ✅ **采用混合策略**：根据场景自动选择最优代码库访问方式
2. ✅ **分阶段实施**：先实现 MVP，再逐步增强
3. ✅ **向后兼容**：新功能不影响现有功能
4. ✅ **可配置**：支持启用/禁用和策略选择

---

**下一步行动**：

1. **立即开始**：阶段一 - Summarize 测试计划生成（MVP）
   - 只实现测试计划生成功能
   - 不获取额外代码上下文
   - 无需额外配置
   - 基于 PR diff 生成测试计划

2. **后续优化**：代码上下文获取功能
   - 实现接口识别
   - 实现代码搜索
   - 添加配置选项

3. **后续规划**：PR 接口自动化测试功能
   - 实现接口自动化测试
   - 实现测试参数生成
   - 实现测试报告生成

