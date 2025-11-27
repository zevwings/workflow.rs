# PR 测试功能总览文档

## 📋 概述

本文档整合了三个相关的需求文档，它们共同构成了完整的 **PR 测试功能体系**：

1. **[代码库访问策略](./CODEBASE_ACCESS_STRATEGY.md)** - 解决如何高效访问和搜索完整代码库的问题
2. **[PR 测试需求分析](./PR_TEST_ANALYSIS_REQUIREMENTS.md)** - 分析 PR 修改需要进行的测试工作
3. **[PR 接口自动化测试](./PR_API_TEST_REQUIREMENTS.md)** - 自动测试 PR 修改的接口

## 🎯 整体目标

**核心目标**：为 PR 修改提供全面的测试支持，包括：
- **测试需求分析**：自动识别需要测试的内容
- **接口自动化测试**：自动执行接口测试并生成报告
- **测试建议生成**：基于代码变更生成测试建议

## 📊 功能关系图

```
PR 修改
  │
  ├─→ [代码库访问策略]
  │     └─→ 提供高效的代码库搜索能力
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

## 🔗 文档关系

### 1. 代码库访问策略（基础层）

**作用**：为其他两个功能提供基础设施

**核心能力**：
- 使用 Git 命令（`git grep`）搜索代码库
- 使用 `ripgrep` 进行快速搜索
- **使用 GitHub MCP 直接访问 GitHub 仓库**（新增）
- 智能过滤和缓存机制
- 性能优化策略

**被依赖关系**：
- ✅ PR 测试需求分析需要搜索调用点
- ✅ PR 接口自动化测试需要搜索接口定义

**访问方式选择**：
- **GitHub 仓库**：优先使用 GitHub MCP（无需本地仓库）
- **本地 Git 仓库**：使用 Git 命令
- **其他场景**：使用 ripgrep 或文件系统搜索

### 2. PR 测试需求分析（分析层）

**作用**：分析 PR 修改，识别测试需求

**核心能力**：
- 识别修改的接口（后端）
- 识别修改的组件（前端）
- 查找调用点（前端调用、其他服务调用）
- 生成测试建议（测试类型、优先级）

**依赖关系**：
- ✅ 依赖代码库访问策略（搜索调用点）
- ✅ 使用现有的 PR diff 解析能力
- ✅ 使用现有的 LLM 基础设施

### 3. PR 接口自动化测试（执行层）

**作用**：自动执行接口测试

**核心能力**：
- 从 PR diff 识别修改的接口
- 在代码库中搜索接口定义
- 生成测试参数
- 发送 HTTP 请求并验证响应
- 生成测试报告

**依赖关系**：
- ✅ 依赖代码库访问策略（搜索接口定义）
- ✅ 使用现有的 HttpClient
- ✅ 使用现有的 Settings 系统
- ✅ 使用现有的 LLM 基础设施

## 🛠️ 技术实现方案

### 共享基础设施

所有功能都复用现有的基础设施：

1. **PR Diff 解析**
   - 已有 `parse_diff_to_file_changes()` 函数
   - 支持标准 unified diff 格式
   - 自动跳过二进制文件

2. **LLM 集成**
   - 已有 `PullRequestLLM` 模块
   - 支持多种 LLM 提供商
   - 已有 prompt 系统

3. **HTTP 客户端**
   - 已有完整的 `HttpClient` 实现
   - 支持多种 HTTP 方法
   - 支持认证和自定义 headers

4. **配置管理**
   - 已有 `Settings` 系统
   - 支持 TOML 配置
   - 支持多环境配置

5. **Git 操作**
   - 已有 `GitRepo` 模块
   - 支持 Git 仓库检测
   - 支持分支操作

### 新增功能模块

需要实现以下新模块：

1. **代码库搜索模块**（`src/lib/codebase/search.rs`）
   - 实现代码库搜索功能
   - 支持多种搜索策略（GitHub MCP、Git grep、ripgrep、文件系统）
   - 智能过滤和缓存
   - 自动检测最优策略（GitHub 仓库优先使用 MCP）

2. **接口识别模块**（`src/lib/pr/test/endpoint.rs`）
   - 从 PR diff 识别接口
   - 支持多种框架模式
   - 提取接口定义信息

3. **测试需求分析模块**（`src/lib/pr/test/analysis.rs`）
   - 分析 PR 修改
   - 识别测试需求
   - 生成测试建议

4. **接口测试模块**（`src/lib/pr/test/api.rs`）
   - 生成测试参数
   - 执行接口测试
   - 验证响应结果

5. **测试报告模块**（`src/lib/pr/test/report.rs`）
   - 生成测试报告
   - 格式化输出

## 🚀 实现计划

### 阶段一：基础设施（MVP）

**目标**：实现基础的代码库搜索和接口识别能力

1. **代码库搜索模块**
   - [ ] 实现 Git grep 搜索
   - [ ] 实现 ripgrep 搜索（可选）
   - [ ] 实现智能过滤
   - [ ] 实现基础缓存

2. **接口识别模块**
   - [ ] 从 PR diff 识别接口路径和方法
   - [ ] 支持常见框架模式（Rust、Spring Boot、Express）
   - [ ] 使用 LLM 解析接口定义

3. **测试需求分析命令**
   - [ ] 实现 `workflow pr test-analysis` 命令
   - [ ] 识别修改的接口/组件
   - [ ] 生成基础测试建议

### 阶段二：接口自动化测试

**目标**：实现接口自动化测试功能

1. **测试参数生成**
   - [ ] 基于类型模板生成测试数据
   - [ ] 支持 String、Number、Boolean
   - [ ] 从配置文件读取公共参数

2. **接口测试执行**
   - [ ] 使用 HttpClient 发送请求
   - [ ] 基础响应验证（状态码）
   - [ ] 生成测试报告

3. **接口测试命令**
   - [ ] 实现 `workflow pr test-api` 命令
   - [ ] 自动识别并测试修改的接口

### 阶段三：增强功能

**目标**：提高准确性和功能完整性

1. **高级参数生成**
   - [ ] 支持 Object、Array 类型
   - [ ] 使用 LLM 生成更合理的测试数据
   - [ ] 支持测试数据模板

2. **响应验证增强**
   - [ ] 验证响应体结构
   - [ ] 验证字段类型
   - [ ] 业务逻辑验证

3. **测试需求分析增强**
   - [ ] 查找调用点（前端调用、其他服务调用）
   - [ ] 生成详细的测试建议
   - [ ] 支持测试用例生成

### 阶段四：高级功能

**目标**：提供更强大的功能和更好的性能

1. **AST 解析支持**
   - [ ] 支持 Rust 代码 AST 解析
   - [ ] 提高接口识别准确度

2. **索引机制**
   - [ ] 为代码库建立索引
   - [ ] 加速搜索和查找

3. **CI/CD 集成**
   - [ ] 自动在 PR 中运行测试
   - [ ] 生成测试报告并添加到 PR 评论

## 📝 命令设计

### 1. `workflow pr test-analysis`

**功能**：分析 PR 修改，生成测试需求文档

**用法**：
```bash
workflow pr test-analysis [PR_ID]
```

**输出**：
- Markdown 格式的测试需求文档
- 包含修改的接口/组件列表
- 包含调用点分析
- 包含测试建议

### 2. `workflow pr test-api`

**功能**：自动测试 PR 修改的接口

**用法**：
```bash
workflow pr test-api [PR_ID] [--env ENV]
```

**输出**：
- 测试执行结果
- 测试报告（Markdown 格式）
- 包含请求、响应、验证结果

### 3. `workflow pr test`（可选）

**功能**：统一的测试入口，整合所有测试功能

**用法**：
```bash
workflow pr test [PR_ID] [--analysis] [--api] [--all]
```

## ⚙️ 配置项

需要在 `Settings` 中添加以下配置：

```toml
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

## 📚 参考资源

### 现有代码
- PR 总结功能：`src/commands/pr/summarize.rs`
- LLM 模块：`src/lib/pr/llm.rs`
- HTTP 客户端：`src/lib/base/http/client.rs`
- Settings 系统：`src/lib/base/settings/settings.rs`
- Git 操作：`src/lib/git/`

### 需求文档
- [代码库访问策略](./CODEBASE_ACCESS_STRATEGY.md) - 详细的代码库访问和性能优化方案
- [PR 测试需求分析](./PR_TEST_ANALYSIS_REQUIREMENTS.md) - 测试需求分析功能需求
- [PR 接口自动化测试](./PR_API_TEST_REQUIREMENTS.md) - 接口自动化测试功能需求

### 实施文档
- **[PR 测试功能实施计划](./PR_TEST_IMPLEMENTATION_PLAN.md)** ⭐ - **最佳实施指南**，包含详细的实施步骤、代码结构、时间估算和验收标准

## ✅ 总结

这三个文档共同构成了完整的 PR 测试功能体系：

1. **代码库访问策略**提供基础设施，确保能够高效访问和搜索代码库
2. **PR 测试需求分析**提供分析能力，识别测试需求并生成建议
3. **PR 接口自动化测试**提供执行能力，自动执行接口测试

通过整合这三个功能，可以为 PR 修改提供全面的测试支持，提高代码质量和开发效率。

