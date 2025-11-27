# Summarize 功能中新增测试步骤分析

## 📋 需求概述

在现有的 `workflow pr summarize` 功能中，新增一个**测试步骤**，目前只在生成的文档中体现，不需要实现实际的测试执行代码。

## 🎯 目标

1. **在 PR 总结文档中增加测试相关内容**
2. **提供测试建议和指导**
3. **为后续实现实际测试功能做准备**
4. **不改变现有 summarize 的工作流程**

## 🔍 现有结构分析

### 当前 PR 总结文档结构

根据 `summarize_pr.system.rs`，当前文档包含以下部分：

1. **PR Title**（一级标题）
2. **Overview**（概述）
3. **Requirements Analysis**（需求分析）
   - Business Requirements
   - Functional Requirements
   - User Scenarios
   - Impact Analysis
   - Change Categories
   - Dependencies
4. **Key Changes**（主要变更）
5. **Files Changed**（文件变更列表）
6. **Technical Details**（技术细节）
7. **Testing**（测试）- ⚠️ **已存在，但内容较简单**
8. **Usage Instructions**（使用说明）
9. **Code Changes**（代码变更详情）

### 现有 "Testing" 部分的内容

当前 prompt 要求：
- 分析代码变更以确定测试需求
- 单元测试添加/修改（如果有）
- 集成测试（如果有）
- 手动测试步骤或场景
- 测试覆盖率信息（如果可用）
- 如果没有测试，建议应该测试什么

**问题**：
- 内容比较通用，不够具体
- 没有针对接口测试的专门指导
- 没有提供可执行的测试命令或 CURL
- 没有测试优先级和分类

## 💡 设计方案

### 方案一：扩展现有的 "Testing" 部分（推荐）

**优点**：
- 不改变文档结构
- 保持向后兼容
- 实现简单

**实现方式**：
- 在 prompt 中增强 "Testing" 部分的要求
- 要求 LLM 生成更详细的测试内容，包括：
  - 接口测试建议（如果涉及接口）
  - 测试优先级
  - 可执行的测试命令（CURL 等）
  - 测试数据建议

**新的 "Testing" 部分结构**：

```markdown
## Testing

### Test Coverage Analysis
- Unit tests added/modified (if any)
- Integration tests (if any)
- Manual testing steps

### API Testing (if applicable)
If this PR modifies or adds API endpoints, provide:

#### Modified/Added Endpoints
- **Endpoint 1**: `POST /api/users`
  - **Purpose**: Create a new user
  - **Test Priority**: High
  - **Required Parameters**:
    - `name` (string, required)
    - `email` (string, required)
  - **Suggested Test Data**:
    ```json
    {
      "name": "test_user",
      "email": "test@example.com"
    }
    ```
  - **Expected Response**:
    - Status: 200 OK
    - Body: `{ "id": 123, "name": "test_user", "email": "test@example.com" }`
  - **CURL Command** (example):
    ```bash
    curl -X POST https://api.example.com/api/users \
      -H "Content-Type: application/json" \
      -H "Authorization: Bearer <token>" \
      -d '{"name":"test_user","email":"test@example.com"}'
    ```
  - **Test Scenarios**:
    - ✅ Normal case: Create user with valid data
    - ✅ Validation: Missing required fields
    - ✅ Validation: Invalid email format
    - ✅ Edge case: Duplicate email

- **Endpoint 2**: `GET /api/users/:id`
  - ...

#### Testing Recommendations
1. **High Priority**: Test endpoints that are newly added
2. **Medium Priority**: Test endpoints with significant modifications
3. **Low Priority**: Test endpoints with minor changes

### Component Testing (if applicable)
If this PR modifies frontend components, provide:
- Component test scenarios
- User interaction flows to test
- Edge cases to consider

### Integration Testing
- End-to-end test scenarios
- Cross-module interaction tests
- Database/API integration tests

### Manual Testing Checklist
- [ ] Test scenario 1
- [ ] Test scenario 2
- [ ] ...
```

### 方案二：新增独立的 "Test Plan" 部分

**优点**：
- 测试内容更突出
- 结构更清晰
- 便于后续扩展

**实现方式**：
- 在 "Testing" 部分之后，新增 "Test Plan" 部分
- 专门用于详细的测试计划和指导

**新的文档结构**：

```markdown
## Testing
(保持现有的简单测试说明)

## Test Plan
(新增的详细测试计划部分)
```

### 方案三：混合方案（推荐）✅ 已采用

**结合方案一和方案二**：
- 保留二级标题 `## Testing`
- 将现有的测试内容放到三级标题 `### Test Description` 下
- 新增的测试计划放到三级标题 `### Test Plan` 下

**文档结构**：

```markdown
## Testing

### Test Description
(现有的测试内容：测试覆盖率、测试类型、单元测试、集成测试等)

### Test Plan
(新增的详细测试计划：接口测试、组件测试、CURL 命令等)
```

**优势**：
- ✅ 保持二级标题结构不变，向后兼容
- ✅ 内容分类更清晰（说明 vs 计划）
- ✅ 结构层次更合理（二级标题下有两个三级标题）
- ✅ 便于后续扩展（可以在 Testing 下增加其他三级标题）

## 📝 推荐的 Prompt 增强方案

### 在 `summarize_pr.system.rs` 中增强 "Testing" 部分

**当前要求**（第 82-89 行）：
```rust
7. **Testing**: Testing approach or test coverage
   - Analyze the code changes to determine testing needs
   - Unit tests added/modified (if any, clearly state if none)
   - Integration tests (if any, clearly state if none)
   - Manual testing steps or scenarios
   - Test coverage information (if available)
   - If no tests are found in the changes, suggest what should be tested or state "No tests included in this PR"
   - DO NOT simply write "No specific testing details provided" - always provide testing guidance based on the changes
```

**增强后的要求**：
```rust
7. **Testing**: Testing approach or test coverage
   The Testing section MUST contain two subsections:

   a. **### Test Description**:
      - Analyze the code changes to determine testing needs
      - Unit tests added/modified (if any, clearly state if none)
      - Integration tests (if any, clearly state if none)
      - Manual testing steps or scenarios
      - Test coverage information (if available)
      - If no tests are found in the changes, suggest what should be tested or state "No tests included in this PR"
      - DO NOT simply write "No specific testing details provided" - always provide testing guidance based on the changes

   b. **### Test Plan**: Detailed test plan with executable test commands (NEW SUBSECTION)
      - **API Testing** (if this PR modifies or adds API endpoints):
     - For each modified/added endpoint, provide:
       - Endpoint path and HTTP method
       - Purpose and description
       - Test priority (High/Medium/Low)
       - Required parameters (path params, query params, request body)
       - Parameter types and whether they are required
       - Suggested test data (example values for each parameter)
       - Expected response (status code, response body structure)
       - CURL command example (with all parameters, headers, and authentication if needed)
       - Test scenarios (normal case, validation, edge cases, error handling)
     - Example format:
       ```markdown
       #### POST /api/users
       - **Purpose**: Create a new user
       - **Test Priority**: High
       - **Parameters**:
         - `name` (string, required): User name
         - `email` (string, required): User email
       - **Suggested Test Data**:
         ```json
         {
           "name": "test_user",
           "email": "test@example.com"
         }
         ```
       - **Expected Response**: 200 OK with user object
       - **CURL Command**:
         ```bash
         curl -X POST https://api.example.com/api/users \
           -H "Content-Type: application/json" \
           -H "Authorization: Bearer <token>" \
           -d '{"name":"test_user","email":"test@example.com"}'
         ```
       - **Test Scenarios**:
         - ✅ Normal case: Create user with valid data
         - ✅ Validation: Missing required fields
         - ✅ Validation: Invalid email format
       ```
   - **Component Testing** (if this PR modifies frontend components):
     - List modified components
     - Provide test scenarios for each component
     - User interaction flows to test
     - Edge cases to consider
   - **Integration Testing**:
     - End-to-end test scenarios
     - Cross-module interaction tests
   - **Testing Priority**:
     - High: New features, critical bug fixes
     - Medium: Significant modifications
     - Low: Minor changes, refactoring
   - If no API endpoints or components are modified, state "No API or component changes requiring specific test plans"

   **Important**: The Testing section structure should be:
   ```markdown
   ## Testing

   ### Test Description
   (Test description content here)

   ### Test Plan
   (Test plan content here)
   ```
```

### 在文档结构顺序中增加

**当前顺序**（第 166-183 行）：
```rust
1. Level 1 heading with PR Title
2. ## Overview
3. ## Requirements Analysis
4. ## Key Changes
5. ## Files Changed
6. ## Technical Details
7. ## Testing
8. ## Usage Instructions
9. ## Code Changes
```

**新的顺序**（保持不变，因为是在 Testing 下增加三级标题）：
```rust
1. Level 1 heading with PR Title
2. ## Overview
3. ## Requirements Analysis
4. ## Key Changes
5. ## Files Changed
6. ## Technical Details
7. ## Testing
   - ### Test Description (existing content, enhanced)
   - ### Test Plan (NEW subsection)
8. ## Usage Instructions
9. ## Code Changes
```

**注意**：文档结构顺序不需要改变，因为是在现有的 `## Testing` 二级标题下增加三级标题，而不是新增独立的二级标题。

## 🎯 实现步骤（仅文档层面）

### 阶段一：Prompt 增强（不涉及代码实现）

1. **修改 `summarize_pr.system.rs`**：
   - 增强 "Testing" 部分的描述
   - 新增 "Test Plan" 部分的详细要求
   - 更新文档结构顺序

2. **测试 Prompt 效果**：
   - 使用几个真实的 PR 测试新的 prompt
   - 验证 LLM 是否能生成符合要求的测试内容
   - 根据结果调整 prompt

### 阶段二：文档格式优化（可选）

1. **优化 Markdown 格式**：
   - 确保 CURL 命令格式正确
   - 确保 JSON 示例格式正确
   - 确保测试场景列表清晰

2. **添加示例模板**：
   - 在 prompt 中提供更详细的示例
   - 帮助 LLM 理解期望的输出格式

## 📊 预期输出示例

### 新的 "Testing" 部分结构

```markdown
## Testing

### Test Description

#### Test Coverage Analysis
- **Unit Tests**: No unit tests added in this PR
- **Integration Tests**: No integration tests added in this PR
- **Manual Testing**: Required for the new user creation endpoint

#### Testing Recommendations
Based on the code changes, the following should be tested:
1. User creation endpoint with valid data
2. Input validation for required fields
3. Email format validation
4. Duplicate email handling

### Test Plan

#### API Testing

This PR adds a new user creation endpoint that requires comprehensive testing.

##### POST /api/users

- **Purpose**: Create a new user account
- **Test Priority**: High (new feature)
- **HTTP Method**: POST
- **Endpoint Path**: `/api/users`

**Parameters**:
- **Request Body** (JSON, required):
  - `name` (string, required): User's full name
  - `email` (string, required): User's email address (must be valid email format)

**Suggested Test Data**:
```json
{
  "name": "John Doe",
  "email": "john.doe@example.com"
}
```

**Expected Response**:
- **Status Code**: 200 OK
- **Response Body**:
  ```json
  {
    "id": 123,
    "name": "John Doe",
    "email": "john.doe@example.com",
    "created_at": "2024-01-01T12:00:00Z"
  }
  ```

**CURL Command**:
```bash
curl -X POST https://api.example.com/api/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-token>" \
  -d '{
    "name": "John Doe",
    "email": "john.doe@example.com"
  }'
```

**Test Scenarios**:
1. ✅ **Normal Case**: Create user with valid name and email
   - Expected: 200 OK with user object
2. ✅ **Validation**: Missing required field (name)
   - Expected: 400 Bad Request with error message
3. ✅ **Validation**: Missing required field (email)
   - Expected: 400 Bad Request with error message
4. ✅ **Validation**: Invalid email format
   - Expected: 400 Bad Request with validation error
5. ✅ **Edge Case**: Duplicate email
   - Expected: 409 Conflict or 400 Bad Request with error message
6. ✅ **Edge Case**: Very long name (boundary testing)
   - Expected: 400 Bad Request if exceeds limit, or 200 OK if within limit

**Testing Priority**:
- **High**: Scenarios 1, 2, 3 (core functionality and validation)
- **Medium**: Scenarios 4, 5 (edge cases)
- **Low**: Scenario 6 (boundary testing)

### Component Testing

No frontend components modified in this PR.

### Integration Testing

- Test user creation flow end-to-end
- Verify user data is correctly stored in database
- Verify email uniqueness constraint is enforced
```

## 🔄 与现有功能的集成

### "Testing" 部分的结构

- **`## Testing`**（二级标题）：测试相关内容的容器
  - **`### Test Description`**（三级标题）：通用的测试说明，包括测试覆盖率、测试类型等
  - **`### Test Plan`**（三级标题）：详细的、可执行的测试计划，包含具体的测试命令和场景

### 与后续实际测试功能的关系

这个文档层面的测试步骤为后续实现实际测试功能做准备：

1. **测试内容识别**：
   - LLM 生成的测试计划可以帮助识别需要测试的接口
   - 为后续的接口识别功能提供参考

2. **测试参数生成**：
   - LLM 生成的测试数据示例可以作为测试参数生成的参考
   - 为后续的测试参数生成功能提供模板

3. **测试命令生成**：
   - LLM 生成的 CURL 命令可以作为测试执行的参考
   - 为后续的 CURL 生成功能提供格式参考

4. **测试场景定义**：
   - LLM 生成的测试场景可以作为测试用例的参考
   - 为后续的测试场景管理提供基础

## ⚠️ 注意事项

### 1. LLM 生成内容的准确性

- LLM 可能无法准确识别所有接口
- LLM 可能无法准确提取参数信息
- 需要在实际使用中验证和调整

### 2. 不同语言和框架的支持

- 不同框架的接口定义格式不同
- 需要在 prompt 中提供多种框架的示例
- 可能需要根据项目类型调整 prompt

### 3. 测试数据的合理性

- LLM 生成的测试数据可能不够合理
- 需要人工审查和调整
- 后续实现实际测试功能时，可以使用更智能的参数生成

### 4. CURL 命令的完整性

- LLM 生成的 CURL 命令可能缺少必要的参数
- 需要包含认证信息（token 等）
- 需要包含正确的 base URL

## 🔍 代码上下文获取（重要）

### 问题

当前 summarize 功能只使用 PR diff，可能缺少足够的上下文来生成详细的测试计划：
- 接口的完整定义（参数、响应结构）
- 调用点信息
- 相关类型定义
- 现有测试文件

### 解决方案

**阶段一（当前）**：只增强 Prompt，基于 PR diff 生成测试计划

**阶段二（后续）**：获取额外代码上下文
- 从 PR diff 识别接口
- 在代码库中搜索接口定义
- 将额外上下文添加到 LLM prompt

**详细方案**：参见 `docs/requirements/SUMMARIZE_CODE_CONTEXT_ANALYSIS.md`

## ✅ 实施建议

### MVP 版本（当前阶段）

1. **增强 Prompt**：
   - 修改 `summarize_pr.system.rs`
   - 增加 "Test Plan" 部分的详细要求
   - 更新文档结构顺序
   - 在 prompt 中说明：如果提供了额外代码上下文，应使用它来生成更详细的测试计划

2. **测试验证**：
   - 使用几个真实的 PR 测试
   - 验证输出质量
   - 根据结果调整 prompt

3. **文档说明**：
   - 在 README 或文档中说明新增的测试步骤
   - 说明这是文档层面的功能，不涉及实际测试执行
   - 说明后续会支持代码上下文获取

### 后续版本

1. **Prompt 优化**：
   - 根据使用反馈优化 prompt
   - 增加更多框架和语言的示例

2. **格式优化**：
   - 优化 Markdown 格式
   - 确保 CURL 命令可执行
   - 确保 JSON 示例格式正确

3. **与实际测试功能集成**：
   - 当实现实际测试功能时，可以从文档中提取测试计划
   - 使用文档中的测试数据作为参考

## 📚 参考

- 现有 Prompt：`src/lib/base/prompt/summarize_pr.system.rs`
- PR 测试需求分析：`docs/requirements/PR_TEST_ANALYSIS_REQUIREMENTS.md`
- PR 接口自动化测试：`docs/requirements/PR_API_TEST_REQUIREMENTS.md`
- PR 测试方案分析：`docs/requirements/PR_TEST_SCHEME_ANALYSIS.md`

