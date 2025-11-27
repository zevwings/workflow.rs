# 接口识别问题分析

## 📋 问题描述

在测试计划生成过程中，某些 PR 修改没有被识别为接口修改，导致测试计划中没有生成相应的接口测试内容。

### 案例：CerebrasService.ts 修改

**PR 信息**：
- 修改文件：`intent-os-backend/src/services/CerebrasService.ts`
- 修改内容：修复焦点上下文中的英文单词
- 变更类型：Service 层 prompt 修改

**问题**：
- 测试计划生成功能没有识别到这个 PR 中的接口
- 生成的测试计划可能显示 "No API or component changes requiring specific test plans"

## 🔍 问题分析

### 当前识别方式

**阶段一（MVP）的实现方式**：
- 完全依赖 LLM 从 PR diff 中识别接口
- LLM 基于 diff 内容分析，判断是否有接口修改
- 没有代码上下文获取（只基于 PR diff）

### 为什么没有识别到接口？

#### 1. **间接修改问题**

**问题**：
- PR 修改的是 Service 层的 prompt 内容
- 没有直接修改 HTTP 路由定义（如 `@PostMapping`、`router.post()` 等）
- LLM 从 diff 中看不到明显的接口定义模式

**示例**：
```typescript
// PR diff 中只看到这样的修改：
- "title": "Short actionable title",
+ "title": "Short actionable title in ${languageName}",

// 没有看到：
// - @PostMapping("/api/focuses")
// - router.post('/api/focuses')
// - app.post("/api/focuses")
```

#### 2. **文件路径和命名问题**

**问题**：
- 文件路径是 `services/CerebrasService.ts`
- 文件名包含 "Service"，但没有明显的 "Controller"、"Route"、"API" 等关键词
- LLM 可能认为这只是服务层逻辑修改，不是接口修改

#### 3. **缺少代码上下文**

**问题**：
- 当前阶段不获取额外代码上下文
- 无法知道这个 Service 是否被 Controller 调用
- 无法知道这个 Service 是否暴露了 HTTP 接口

#### 4. **LLM 理解限制**

**问题**：
- LLM 可能无法从 prompt 修改推断出接口影响
- 需要理解代码架构和调用关系
- 需要知道 Service 层修改会影响哪些接口

## 💡 解决方案分析

### 方案一：增强 Prompt 指导（当前阶段可实施）

**核心思想**：在 prompt 中更明确地指导 LLM 识别间接的接口修改

#### 1.1 增强测试计划 prompt

**当前 prompt 的问题**：
- 只要求识别"修改或添加的 API endpoint"
- 没有说明 Service 层修改也可能影响接口

**改进方案**：
```rust
// 在 summarize_test_plan.system.rs 中增强

### API Testing (if applicable)

For each modified or added API endpoint, OR service/controller that affects API behavior, provide:

**Important**: Even if the PR doesn't directly modify route definitions, consider:
- Service layer changes that affect API responses
- Controller changes that affect API behavior
- Middleware changes that affect API processing
- Model/Entity changes that affect API data structure

Look for:
1. **Direct endpoint definitions**:
   - HTTP route decorators (@GetMapping, @PostMapping, router.get, etc.)
   - Route handlers (app.post, router.post, etc.)

2. **Indirect endpoint impacts**:
   - Service layer files (Service.ts, Service.js) that are called by controllers
   - Controller files (Controller.ts, Controller.js) that define endpoints
   - Files in routes/, api/, controllers/ directories
   - Files that modify request/response handling logic
```

#### 1.2 增强 user prompt

**改进方案**：
```rust
// 在 test_plan_user_prompt 中添加文件路径分析

fn test_plan_user_prompt(...) -> String {
    // ... 现有代码

    // 添加文件路径分析提示
    if !file_changes.is_empty() {
        let service_files: Vec<&str> = file_changes
            .iter()
            .filter(|(path, _)| {
                path.contains("Service") ||
                path.contains("service") ||
                path.contains("Controller") ||
                path.contains("controller") ||
                path.contains("/api/") ||
                path.contains("/routes/")
            })
            .map(|(path, _)| path.as_str())
            .collect();

        if !service_files.is_empty() {
            parts.push(format!(
                "Note: The following files may be related to API endpoints:\n{}",
                service_files.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n")
            ));
        }
    }

    parts.join("\n\n")
}
```

### 方案二：基于文件路径的启发式识别（后续阶段）

**核心思想**：根据文件路径和命名模式，推断可能的接口影响

#### 2.1 文件路径分析

```rust
fn analyze_file_for_endpoints(file_path: &str) -> Vec<String> {
    let mut hints = Vec::new();

    // 检查文件路径关键词
    if file_path.contains("Service") || file_path.contains("service") {
        hints.push("This file is a service layer file. It may be called by controllers that expose HTTP endpoints.");
    }

    if file_path.contains("Controller") || file_path.contains("controller") {
        hints.push("This file is a controller file. It likely contains HTTP endpoint definitions.");
    }

    if file_path.contains("/api/") || file_path.contains("/routes/") {
        hints.push("This file is in an API or routes directory. It likely contains endpoint definitions.");
    }

    hints
}
```

#### 2.2 代码模式识别

```rust
fn identify_potential_endpoints(content: &str) -> Vec<String> {
    let mut endpoints = Vec::new();

    // 查找常见的接口相关模式
    // 即使不是直接的路由定义，也可能是接口相关的代码

    // 查找 HTTP 方法调用
    // fetch, axios, http.get, etc.

    // 查找 API 调用
    // api.create, apiService.post, etc.

    endpoints
}
```

### 方案三：代码上下文获取（后续阶段）

**核心思想**：获取额外代码上下文，帮助识别接口

#### 3.1 搜索相关 Controller

```rust
// 如果修改了 Service 文件，搜索调用该 Service 的 Controller
fn find_related_controllers(service_file: &str) -> Result<Vec<String>> {
    // 提取 Service 类名
    let service_name = extract_service_name(service_file);

    // 搜索使用该 Service 的 Controller
    // 使用 git grep 或 GitHub MCP
    search_codebase(&format!("{}", service_name))
}
```

#### 3.2 搜索路由定义

```rust
// 搜索可能的路由定义
fn find_route_definitions(service_file: &str) -> Result<Vec<String>> {
    // 根据 Service 文件路径，推断可能的 Controller 路径
    // 例如：services/UserService.ts -> controllers/UserController.ts

    // 搜索 Controller 文件
    // 提取接口定义
}
```

## 📊 推荐方案

### 阶段一：立即改进（当前阶段）

**方案**：增强 Prompt 指导

1. **增强测试计划 prompt**：
   - 明确说明 Service 层修改也可能影响接口
   - 要求 LLM 分析文件路径和命名
   - 要求 LLM 推断可能的接口影响

2. **增强 user prompt**：
   - 添加文件路径分析提示
   - 标记可能的 Service/Controller 文件
   - 提示 LLM 关注这些文件

**优点**：
- ✅ 无需额外代码实现
- ✅ 可以立即应用
- ✅ 不增加复杂度

**缺点**：
- ⚠️ 依赖 LLM 的理解能力
- ⚠️ 可能仍然不够准确

### 阶段二：后续优化

**方案**：代码上下文获取 + 启发式识别

1. **文件路径分析**：
   - 识别 Service/Controller 文件
   - 提供提示给 LLM

2. **代码上下文获取**：
   - 搜索相关的 Controller
   - 搜索路由定义
   - 获取接口的完整定义

**优点**：
- ✅ 更准确的识别
- ✅ 可以找到间接的接口影响

**缺点**：
- ⚠️ 需要额外实现
- ⚠️ 增加复杂度

## 🔧 具体改进建议

### 改进 1：增强测试计划 prompt

在 `summarize_test_plan.system.rs` 中：

```rust
### API Testing (if applicable)

**Important**: Identify API endpoints that are:
1. **Directly modified**: Route definitions, controllers, handlers
2. **Indirectly affected**: Service layers, models, middleware that affect API behavior

For each modified or added API endpoint, OR service/controller that affects API behavior, provide:

**File Analysis**:
- If the PR modifies files in `services/`, `Service.ts`, `Service.js`, consider that these services may be called by controllers that expose HTTP endpoints
- If the PR modifies files in `controllers/`, `Controller.ts`, `Controller.js`, these likely contain endpoint definitions
- If the PR modifies files in `api/`, `routes/`, these likely contain route definitions
- Analyze the file paths and names to infer potential API impacts

**Endpoint Detection**:
Look for:
1. **Direct patterns**:
   - `@GetMapping("/api/...")`, `@PostMapping("/api/...")`
   - `router.get('/api/...')`, `router.post('/api/...')`
   - `app.post("/api/...")`, `app.get("/api/...")`

2. **Indirect patterns**:
   - Service files that may be called by controllers
   - Files that modify request/response handling
   - Files that modify data models used by APIs
```

### 改进 2：增强 user prompt

在 `llm.rs` 的 `test_plan_user_prompt` 中：

```rust
fn test_plan_user_prompt(...) -> String {
    // ... 现有代码

    // 分析文件路径，提供提示
    if !file_changes.is_empty() {
        let mut api_related_files = Vec::new();
        let mut service_files = Vec::new();

        for (file_path, _) in file_changes {
            if file_path.contains("/api/") ||
               file_path.contains("/routes/") ||
               file_path.contains("Controller") ||
               file_path.contains("controller") {
                api_related_files.push(file_path.clone());
            } else if file_path.contains("Service") ||
                      file_path.contains("service") {
                service_files.push(file_path.clone());
            }
        }

        if !api_related_files.is_empty() {
            parts.push(format!(
                "## API-Related Files Detected\n\nThese files are likely related to API endpoints:\n{}",
                api_related_files.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n")
            ));
        }

        if !service_files.is_empty() {
            parts.push(format!(
                "## Service Layer Files Detected\n\nThese service files may be called by controllers that expose HTTP endpoints:\n{}\n\nPlease analyze if these service changes affect any API endpoints.",
                service_files.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n")
            ));
        }
    }

    parts.join("\n\n")
}
```

## 📝 针对当前案例的分析

### CerebrasService.ts 修改

**文件路径分析**：
- `intent-os-backend/src/services/CerebrasService.ts`
- ✅ 包含 "Service" 关键词
- ✅ 在 `services/` 目录下

**应该识别的线索**：
1. 文件路径包含 "Service"，可能是服务层
2. 服务层通常被 Controller 调用
3. Controller 通常暴露 HTTP 接口
4. 修改 Service 的 prompt 可能影响接口的响应内容

**应该生成的测试计划**：
- 识别可能调用 `CerebrasService` 的接口
- 分析 prompt 修改对接口响应的影响
- 生成测试场景：验证返回的 JSON 内容是否使用正确的语言

## ✅ 实施建议

### 立即实施（当前阶段）✅ 已完成

1. **增强测试计划 prompt**：✅
   - 添加 Service 层文件的分析指导
   - 添加文件路径分析的说明
   - 添加间接接口影响的识别指导

2. **增强 user prompt**：✅
   - 添加文件路径分析
   - 标记可能的 Service/Controller 文件
   - 提供文件类型提示给 LLM

### 后续优化（阶段二）

1. **实现文件路径分析**：
   - 识别 Service/Controller 文件
   - 提供更准确的提示

2. **实现代码上下文获取**：
   - 搜索相关的 Controller
   - 获取接口定义

## 🎯 预期效果

### 改进前

```
### Test Plan

No API or component changes requiring specific test plans.
```

### 改进后

```
### Test Plan

#### API Testing

**Note**: This PR modifies `CerebrasService.ts`, which is a service layer file.
This service may be called by controllers that expose HTTP endpoints.

**Potential Affected Endpoints**:
- Endpoints that use `CerebrasService` to generate Focus objects
- Endpoints that return JSON with `title` and `context` fields

**Test Scenarios**:
1. ✅ Verify that API responses use the correct language (${languageName})
2. ✅ Verify that `title` field is in the specified language
3. ✅ Verify that `context` field is in the specified language
4. ✅ Verify that all JSON field values use the specified language, not English

**CURL Command** (example):
```bash
curl -X POST <base_url>/api/focuses \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{"instruction": "Create a focus for learning Rust"}'
```

**Expected Response**:
- Status: 200 OK
- Body: JSON with `title` and `context` in the specified language
```

