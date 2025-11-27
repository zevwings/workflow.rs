# PR 测试需求分析功能需求文档

## 📋 需求概述

根据 PR 的代码变更，自动分析并总结需要进行的测试工作：
- **后端测试**：识别修改的接口（API endpoints），查找调用这些接口的地方
- **前端测试**：识别修改的前端代码，查找对应的调用点（可能是多个）

## 🎯 功能目标

### 1. 后端测试需求分析

**目标**：当 PR 修改了后端代码（service、controller、API 层等）时，自动识别：
- 修改了哪些接口（API endpoints）
- 这些接口的调用方（前端调用点、其他服务调用等）
- 需要测试的接口列表

**识别范围**：
- REST API endpoints（GET、POST、PUT、DELETE、PATCH）
- GraphQL queries/mutations
- RPC 接口
- Service 层方法
- 数据库操作（如果影响接口行为）

### 2. 前端测试需求分析

**目标**：当 PR 修改了前端代码时，自动识别：
- 修改了哪些组件/页面
- 这些组件/页面的调用点（路由、其他组件引用等）
- 相关的 API 调用点（如果修改了 API 调用逻辑）

**识别范围**：
- React/Vue/Angular 组件
- 页面路由
- API 调用函数（fetch、axios、API client 等）
- 状态管理（Redux、Vuex、Zustand 等）
- 事件处理函数

## 🔍 技术实现分析

### 可行性评估

#### ✅ 可行的部分

1. **代码类型识别**
   - 通过文件路径和扩展名识别前端/后端代码
   - 后端：`.rs`、`.go`、`.java`、`.py`、`.js`（Node.js）、`.ts`（Node.js）
   - 前端：`.jsx`、`.tsx`、`.vue`、`.js`（前端）、`.ts`（前端）、`.html`、`.css`

2. **接口定义识别**
   - 后端：通过代码模式识别
     - REST：`@GetMapping`、`@PostMapping`、`router.get()`、`app.post()` 等
     - 函数签名：`pub fn function_name()`、`async fn function_name()`
     - 路由定义：`router.route()`、`app.use()`
   - 使用 AST（抽象语法树）解析更准确

3. **接口调用点查找**
   - 前端：通过代码搜索
     - `fetch('/api/...')`、`axios.get('/api/...')`
     - API client 调用：`api.getUser()`、`apiService.fetchData()`
   - 需要在整个代码库中搜索（不仅仅是 PR 中的文件）

4. **组件调用点查找**
   - 前端：通过 import 语句和组件使用
     - `import Component from './Component'`
     - `<Component />`、`<component-name />`
   - 需要在整个代码库中搜索

#### ⚠️ 挑战和限制

1. **代码库范围**
   - 需要访问完整的代码库（不仅仅是 PR diff）
   - 需要能够搜索整个代码库来查找调用点
   - 当前 PR summarize 功能只分析 PR diff，不访问完整代码库

2. **语言和框架多样性**
   - 不同语言和框架有不同的模式
   - 需要支持多种框架（Spring Boot、Express、FastAPI、Rails 等）
   - 需要支持多种前端框架（React、Vue、Angular 等）

3. **代码分析复杂度**
   - 简单的文本搜索可能不够准确
   - 需要 AST 解析来准确识别接口定义和调用
   - 跨文件引用分析需要构建依赖图

4. **测试类型识别**
   - 需要区分不同类型的测试：
     - 单元测试（Unit Tests）
     - 集成测试（Integration Tests）
     - E2E 测试（End-to-End Tests）
     - API 测试
     - 组件测试

## 📊 实现方案分析

### 方案一：基于 LLM 的智能分析（推荐）

**优点**：
- 利用现有的 LLM 基础设施
- 可以理解代码语义，识别接口和调用关系
- 可以处理多种语言和框架
- 可以生成结构化的测试建议

**实现步骤**：
1. 解析 PR diff，提取修改的文件
2. 识别代码类型（前端/后端）
3. 使用 LLM 分析代码变更：
   - 识别修改的接口/组件
   - 分析接口的输入输出
   - 推断可能的调用点
4. 生成测试建议文档

**限制**：
- LLM 可能无法准确找到所有调用点（需要完整代码库上下文）
- 需要较大的 token 限制
- 可能产生误报

### 方案二：基于代码搜索的分析

**优点**：
- 可以准确找到调用点
- 不依赖 LLM，结果更可靠
- 可以处理大型代码库

**实现步骤**：
1. 解析 PR diff，提取修改的文件和接口/组件
2. 在整个代码库中搜索：
   - 接口路径（如 `/api/users`）
   - 函数名（如 `getUser`）
   - 组件名（如 `UserList`）
3. 分析搜索结果，生成调用点列表
4. 使用 LLM 生成测试建议

**限制**：
- 需要访问完整代码库（可能需要 Git 操作）
- 需要实现代码搜索功能（可能需要 AST 解析）
- 对于动态路由/调用，可能无法准确识别

### 方案三：混合方案（推荐）

**结合方案一和方案二**：
1. 使用代码搜索找到明确的调用点
2. 使用 LLM 分析代码语义，识别潜在的测试需求
3. 结合两者结果，生成完整的测试建议

## 🛠️ 需要的技术组件

### 1. 代码分析工具

- **AST 解析器**：
  - Rust：`syn`、`quote`（Rust 代码）
  - JavaScript/TypeScript：`@babel/parser`、`typescript`（需要 Node.js）
  - Python：`ast` 模块
  - Java：可能需要 `javaparser`（需要 Java 环境）

- **代码搜索**：
  - 使用 `ripgrep`（rg）进行文本搜索
  - 使用 `tree-sitter` 进行结构化搜索（可选）

### 2. Git 操作

- 需要能够：
  - 获取 PR diff
  - 获取完整代码库（用于搜索调用点）
  - 可能需要 checkout 到特定分支

### 3. LLM 集成

- 复用现有的 `PullRequestLLM` 基础设施
- 创建新的 prompt 用于测试需求分析
- 可能需要多次 LLM 调用（分析接口、生成测试建议等）

### 4. 代码库访问

- **选项 A**：假设在 Git 仓库中运行，可以直接访问代码库
- **选项 B**：通过 Git API 获取代码库内容
- **选项 C**：要求用户提供代码库路径

## 📝 输出格式

### 测试需求文档结构

```markdown
# PR #123 测试需求分析

## 代码变更概览
- 后端文件：3 个
- 前端文件：2 个
- 修改类型：新功能

## 后端测试需求

### 修改的接口

#### 1. POST /api/users
- **文件**：`src/api/users.rs`
- **修改内容**：添加了用户创建接口
- **输入参数**：`{ name: string, email: string }`
- **输出**：`{ id: number, name: string, email: string }`

#### 2. GET /api/users/:id
- **文件**：`src/api/users.rs`
- **修改内容**：修改了用户查询逻辑
- **输入参数**：`id: number`
- **输出**：`{ id: number, name: string, email: string }`

### 需要测试的接口

1. **POST /api/users**
   - 测试用例：
     - ✅ 正常创建用户
     - ✅ 重复邮箱验证
     - ✅ 参数验证（必填字段、格式验证）
     - ✅ 错误处理

2. **GET /api/users/:id**
   - 测试用例：
     - ✅ 正常查询用户
     - ✅ 用户不存在的情况
     - ✅ 参数验证

### 调用点分析

- **前端调用**：
  - `src/pages/UserCreate.tsx`：使用 `POST /api/users`
  - `src/pages/UserDetail.tsx`：使用 `GET /api/users/:id`

- **其他服务调用**：
  - 未发现其他服务调用

## 前端测试需求

### 修改的组件

#### 1. UserCreate 组件
- **文件**：`src/pages/UserCreate.tsx`
- **修改内容**：添加了用户创建表单
- **功能**：用户创建界面

#### 2. UserDetail 组件
- **文件**：`src/pages/UserDetail.tsx`
- **修改内容**：修改了用户详情显示逻辑
- **功能**：用户详情展示

### 需要测试的组件

1. **UserCreate 组件**
   - 测试用例：
     - ✅ 表单渲染
     - ✅ 表单提交
     - ✅ 表单验证
     - ✅ 成功/失败处理

2. **UserDetail 组件**
   - 测试用例：
     - ✅ 数据加载
     - ✅ 数据显示
     - ✅ 错误处理

### 调用点分析

- **路由**：
  - `/users/create` → `UserCreate` 组件
  - `/users/:id` → `UserDetail` 组件

- **其他组件引用**：
  - `src/components/UserList.tsx`：可能引用 `UserDetail`
  - 未发现其他直接引用

## 测试建议

### 优先级

1. **高优先级**：
   - POST /api/users（新功能，核心功能）
   - UserCreate 组件（新功能，用户可见）

2. **中优先级**：
   - GET /api/users/:id（修改现有功能）
   - UserDetail 组件（修改现有功能）

### 测试类型建议

- **单元测试**：Service 层方法、工具函数
- **集成测试**：API 接口（POST /api/users、GET /api/users/:id）
- **E2E 测试**：用户创建流程、用户详情查看流程
- **组件测试**：UserCreate、UserDetail 组件

### 测试覆盖建议

- 正常流程测试
- 边界条件测试
- 错误处理测试
- 性能测试（如果涉及大量数据）
```

## 🚀 实现计划

### 阶段一：基础功能（MVP）

1. **代码类型识别**
   - 通过文件路径和扩展名识别前端/后端
   - 支持常见语言和框架

2. **接口/组件识别**
   - 使用 LLM 分析 PR diff，识别修改的接口/组件
   - 生成接口/组件列表

3. **基础测试建议**
   - 使用 LLM 生成测试建议
   - 基于代码变更推断测试需求

### 阶段二：调用点查找

1. **代码搜索功能**
   - 实现代码库搜索功能
   - 支持搜索接口路径、函数名、组件名

2. **调用点分析**
   - 分析搜索结果，识别调用点
   - 区分前端调用、后端调用、其他服务调用

### 阶段三：高级功能

1. **AST 解析**
   - 集成 AST 解析器，提高识别准确度
   - 支持更多语言和框架

2. **依赖图分析**
   - 构建代码依赖图
   - 分析组件/模块之间的依赖关系

3. **测试用例生成**
   - 自动生成测试用例模板
   - 基于接口定义生成测试数据

## 📋 需要的配置和依赖

### 新增依赖

- **代码搜索**：
  - `ripgrep`（rg）- 已可能通过系统安装
  - 或使用 Rust 的 `grep` crate

- **AST 解析**（可选）：
  - `syn`、`quote` - Rust 代码解析
  - 其他语言的解析器（可能需要外部工具）

### 配置项

- **代码库路径**：如果不在当前 Git 仓库中运行
- **搜索范围**：指定搜索的目录（如 `src/`、`frontend/`、`backend/`）
- **排除目录**：指定排除的目录（如 `node_modules/`、`target/`）

## ❓ 待解决的问题

1. **代码库访问** ✅ 已解决
   - **解决方案**：使用 Git 命令（`git grep`、`git ls-tree`）直接访问，不需要 checkout
   - **性能优化**：使用 `git grep` 或 `ripgrep` 进行搜索，不读取文件内容
   - **详细方案**：参见 [代码库访问策略文档](./CODEBASE_ACCESS_STRATEGY.md)

2. **多语言支持**
   - 如何支持多种后端语言（Rust、Go、Java、Python 等）？
   - 如何支持多种前端框架（React、Vue、Angular 等）？
   - 是否需要为每种语言/框架实现特定的解析器？

3. **准确性**
   - 如何确保找到所有调用点？
   - 如何处理动态调用（如 `router[method](path, handler)`）？
   - 如何处理间接调用（如通过中间件、装饰器等）？

4. **性能**
   - 大型代码库的搜索性能
   - LLM 调用的 token 消耗
   - 是否需要缓存？

5. **输出格式**
   - 是否需要生成测试用例代码？
   - 是否需要集成到 CI/CD？
   - 是否需要生成测试报告？

## 💡 建议

### 最小可行方案（MVP）

1. **使用 LLM 进行智能分析**
   - 复用现有的 `PullRequestLLM` 基础设施
   - 创建新的 prompt 用于测试需求分析
   - 基于 PR diff 生成测试建议

2. **简单的代码搜索**
   - 使用 `ripgrep` 或 Rust 的 `grep` crate
   - 搜索接口路径、函数名、组件名
   - 在 PR diff 中查找，或要求用户提供代码库路径

3. **结构化输出**
   - 生成 Markdown 格式的测试需求文档
   - 包含接口列表、调用点、测试建议

### 后续优化

1. **集成 AST 解析**：提高识别准确度
2. **构建依赖图**：分析组件/模块依赖
3. **自动生成测试用例**：基于接口定义生成测试模板
4. **CI/CD 集成**：自动生成测试需求并添加到 PR 评论

## 📚 参考资源

- 现有 PR summarize 功能：`src/commands/pr/summarize.rs`
- LLM 模块：`src/lib/pr/llm.rs`
- Prompt 定义：`src/lib/base/prompt/summarize_pr.system.rs`
- Git 操作：`src/lib/git/`
- **代码库访问策略**：[CODEBASE_ACCESS_STRATEGY.md](./CODEBASE_ACCESS_STRATEGY.md) - 详细的代码库访问和性能优化方案

