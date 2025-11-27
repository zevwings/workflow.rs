# Service 层修改对应的接口发现方案分析

## 📋 问题描述

当 PR 修改了 Service 层文件（如 `CerebrasService.ts`、`cerebras_service.py`、`cerebras_service.rs` 等）时，需要找到调用该 Service 的接口，以便生成准确的测试计划。

### 案例：CerebrasService.ts 修改

**PR 信息**：
- 修改文件：`intent-os-backend/src/services/CerebrasService.ts`
- 修改内容：修复焦点上下文中的英文单词（修改 `createFocus` 方法中的 prompt 模板）
- 问题：无法找到对应的 HTTP 接口

**期望结果**：
- 从 PR diff 中识别修改的方法（如 `createFocus`）
- 搜索调用该方法的地方
- 找到调用该方法的 Controller 和 HTTP 接口
- 生成针对这些接口的测试计划

### 核心思路：从方法开始查找

**为什么从方法开始查找？**

1. **更精确**：
   - PR diff 会显示具体修改了哪些方法
   - 从方法名可以更精确地找到调用点
   - 避免找到不相关的 Service 使用

2. **更高效**：
   - 方法名通常比类名更具体
   - 搜索范围更小，结果更准确
   - 减少误报

3. **更符合实际需求**：
   - 测试计划需要针对具体修改的方法
   - 只有调用修改方法的接口才需要测试

### 多语言支持需求

**支持的语言和框架**：
- **TypeScript/JavaScript**: Express, NestJS, Koa
- **Python**: FastAPI, Flask, Django
- **Rust**: Actix-web, Axum, Rocket
- **Java**: Spring Boot, JAX-RS
- **其他**: Go, Ruby, PHP 等（后续扩展）

**不同语言的挑战**：
- 命名约定不同（PascalCase vs snake_case）
- Import/Use 语法不同
- 接口定义模式不同
- 需要语言特定的搜索策略

## 🔍 问题分析

### 为什么需要找到调用接口？

1. **Service 层不直接暴露接口**：
   - Service 层通常被 Controller 调用
   - Controller 才定义 HTTP 路由
   - 需要找到 Controller 才能知道接口路径

2. **测试计划需要接口信息**：
   - 需要接口路径（如 `/api/focuses`）
   - 需要 HTTP 方法（GET、POST 等）
   - 需要参数结构
   - 需要生成 CURL 命令

3. **准确性的要求**：
   - 不能只靠 LLM 推断
   - 需要从代码库中找到实际的接口定义
   - 需要确保测试计划的准确性

## 🎯 解决方案分析

### 方案一：基于修改的方法名搜索调用点（推荐）⭐

**核心思想**：从 PR diff 中提取修改的方法名，在代码库中搜索调用这些方法的地方。

#### 1.1 从 PR Diff 提取修改的方法名

**工作流程**：

```
1. 解析 PR diff，获取修改的代码内容
   ↓
2. 识别修改的方法/函数定义
   - TypeScript: `createFocus() { ... }`, `async createFocus() { ... }`
   - Python: `def create_focus():`, `async def create_focus():`
   - Rust: `fn create_focus()`, `pub fn create_focus()`
   - Java: `public Focus createFocus()`, `public void createFocus()`
   ↓
3. 提取方法名（支持多种命名约定）
   - PascalCase: `createFocus` → `createFocus`
   - snake_case: `create_focus` → `create_focus`
   - 转换为统一的搜索格式
```

**实现**：

```rust
/// 从 PR diff 中提取修改的方法名
fn extract_modified_methods(
    file_path: &str,
    diff_content: &str,
) -> Result<Vec<MethodInfo>> {
    let language = detect_language_from_path(file_path);
    let mut methods = Vec::new();

    match language {
        Some(Language::TypeScript) | Some(Language::JavaScript) => {
            // TypeScript/JavaScript 方法模式
            // 匹配: function methodName(), async methodName(), methodName() {, methodName = () => {
            let patterns = vec![
                Regex::new(r"(?:async\s+)?(?:function\s+)?(\w+)\s*\([^)]*\)\s*\{")?,
                Regex::new(r"(\w+)\s*=\s*(?:async\s*)?\([^)]*\)\s*=>")?,
                Regex::new(r"(\w+)\s*\([^)]*\)\s*:\s*")?,  // 方法签名
            ];

            for pattern in patterns {
                for cap in pattern.captures_iter(diff_content) {
                    if let Some(method_name) = cap.get(1) {
                        methods.push(MethodInfo {
                            name: method_name.as_str().to_string(),
                            language,
                            file_path: file_path.to_string(),
                        });
                    }
                }
            }
        }
        Some(Language::Python) => {
            // Python 方法模式
            // 匹配: def method_name():, async def method_name():, def method_name(self, ...):
            let pattern = Regex::new(
                r"(?:async\s+)?def\s+(\w+)\s*\([^)]*\)\s*:"
            )?;

            for cap in pattern.captures_iter(diff_content) {
                if let Some(method_name) = cap.get(1) {
                    methods.push(MethodInfo {
                        name: method_name.as_str().to_string(),
                        language,
                        file_path: file_path.to_string(),
                    });
                }
            }
        }
        Some(Language::Rust) => {
            // Rust 方法模式
            // 匹配: fn method_name(), pub fn method_name(), async fn method_name()
            let pattern = Regex::new(
                r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\([^)]*\)"
            )?;

            for cap in pattern.captures_iter(diff_content) {
                if let Some(method_name) = cap.get(1) {
                    methods.push(MethodInfo {
                        name: method_name.as_str().to_string(),
                        language,
                        file_path: file_path.to_string(),
                    });
                }
            }
        }
        Some(Language::Java) => {
            // Java 方法模式
            // 匹配: public ReturnType methodName(), private void methodName()
            let pattern = Regex::new(
                r"(?:public|private|protected)\s+(?:\w+\s+)*(\w+)\s*\([^)]*\)"
            )?;

            for cap in pattern.captures_iter(diff_content) {
                if let Some(method_name) = cap.get(1) {
                    methods.push(MethodInfo {
                        name: method_name.as_str().to_string(),
                        language,
                        file_path: file_path.to_string(),
                    });
                }
            }
        }
        _ => {
            // 通用模式：尝试识别函数定义
            // ...
        }
    }

    // 去重
    methods.dedup_by(|a, b| a.name == b.name);

    Ok(methods)
}

#[derive(Debug, Clone)]
pub struct MethodInfo {
    /// 方法名
    pub name: String,
    /// 语言类型
    pub language: Option<Language>,
    /// 文件路径
    pub file_path: String,
}
```

#### 1.2 搜索方法调用点

**搜索策略**：

1. **直接搜索方法调用**（主要策略）：
   ```typescript
   // TypeScript/JavaScript
   service.createFocus()
   this.cerebrasService.createFocus()
   await cerebrasService.createFocus()
   ```

   ```python
   # Python
   service.create_focus()
   self.cerebras_service.create_focus()
   await cerebras_service.create_focus()
   ```

   ```rust
   // Rust
   service.create_focus()
   self.cerebras_service.create_focus()
   CerebrasService::create_focus()
   ```

2. **搜索方法名（考虑命名约定转换）**：
   - PascalCase → snake_case（Python/Rust）
   - snake_case → PascalCase（TypeScript/Java）
   - 搜索两种格式

3. **搜索 Service 类 + 方法名组合**（备选策略）：
   - 如果直接搜索方法名结果太多，可以结合 Service 类名
   - 例如：`CerebrasService.*createFocus` 或 `cerebras_service.*create_focus`

**实现方式**：

```rust
/// 搜索方法调用点
fn find_method_call_sites(
    method_info: &MethodInfo,
) -> Result<Vec<CallSite>> {
    let mut call_sites = Vec::new();
    let language = method_info.language;

    // 构建搜索查询（考虑命名约定）
    let queries = build_method_search_queries(&method_info.name, language);

    // 策略 1: 使用 GitHub MCP 搜索（如果可用）
    if is_github_repo() && is_mcp_available() {
        for query in &queries {
            let results = search_via_github_mcp(query)?;
            call_sites.extend(parse_search_results(results)?);
        }
    }
    // 策略 2: 使用 Git grep（本地仓库）
    else if is_git_repo() {
        for query in &queries {
            let output = Command::new("git")
                .args(&["grep", "-n", "-E", query])
                .output()?;

            call_sites.extend(parse_git_grep_output(&output.stdout)?);
        }
    }
    // 策略 3: 使用 ripgrep（如果可用）
    else if is_ripgrep_available() {
        for query in &queries {
            let output = Command::new("rg")
                .args(&["-n", query])
                .output()?;

            call_sites.extend(parse_ripgrep_output(&output.stdout)?);
        }
    }

    // 过滤结果：只保留实际的方法调用（排除定义）
    call_sites.retain(|site| is_method_call(&site.content, &method_info.name, language));

    Ok(call_sites)
}

/// 构建方法搜索查询（考虑命名约定）
fn build_method_search_queries(
    method_name: &str,
    language: Option<Language>,
) -> Vec<String> {
    let mut queries = Vec::new();

    match language {
        Some(Language::TypeScript) | Some(Language::JavaScript) => {
            // TypeScript/JavaScript: 方法名通常是 camelCase
            // 搜索: .createFocus(, .createFocus(), this.createFocus
            queries.push(format!(r"\.{}\s*\(", method_name));
            queries.push(format!(r"this\.{}", method_name));
            queries.push(format!(r"service\.{}", method_name));
        }
        Some(Language::Python) => {
            // Python: 方法名通常是 snake_case
            // 搜索: .create_focus(, .create_focus(), self.create_focus
            queries.push(format!(r"\.{}\s*\(", method_name));
            queries.push(format!(r"self\.{}", method_name));
            queries.push(format!(r"service\.{}", method_name));

            // 如果方法名是 PascalCase，也搜索 snake_case 版本
            if method_name.chars().any(|c| c.is_uppercase()) {
                let snake_case = pascal_to_snake_case(method_name);
                queries.push(format!(r"\.{}\s*\(", snake_case));
                queries.push(format!(r"self\.{}", snake_case));
            }
        }
        Some(Language::Rust) => {
            // Rust: 方法名通常是 snake_case
            // 搜索: .create_focus(, ::create_focus(, self.create_focus
            queries.push(format!(r"\.{}\s*\(", method_name));
            queries.push(format!(r"::{}\s*\(", method_name));
            queries.push(format!(r"self\.{}", method_name));
        }
        Some(Language::Java) => {
            // Java: 方法名通常是 camelCase
            // 搜索: .createFocus(, this.createFocus
            queries.push(format!(r"\.{}\s*\(", method_name));
            queries.push(format!(r"this\.{}", method_name));
        }
        _ => {
            // 通用搜索
            queries.push(format!(r"\.{}\s*\(", method_name));
            queries.push(format!("{}", method_name));
        }
    }

    queries
}

/// 判断是否是方法调用（排除方法定义）
fn is_method_call(
    content: &str,
    method_name: &str,
    language: Option<Language>,
) -> bool {
    // 排除方法定义的关键词
    let definition_keywords = match language {
        Some(Language::TypeScript) | Some(Language::JavaScript) => {
            vec!["function", "async function", "const", "let", "="]
        }
        Some(Language::Python) => {
            vec!["def", "async def"]
        }
        Some(Language::Rust) => {
            vec!["fn", "pub fn", "async fn"]
        }
        Some(Language::Java) => {
            vec!["public", "private", "protected", "static"]
        }
        _ => vec![],
    };

    // 如果包含定义关键词，可能是方法定义，不是调用
    for keyword in definition_keywords {
        if content.contains(keyword) && content.contains(method_name) {
            // 进一步检查：如果是 "function methodName" 或 "def method_name"，则是定义
            let pattern = format!(r"(?:function|def|fn|pub fn)\s+{}", method_name);
            if Regex::new(&pattern).unwrap().is_match(content) {
                return false;
            }
        }
    }

    // 包含方法调用模式
    let call_patterns = vec![
        format!(r"\.{}\s*\(", method_name),
        format!(r"::{}\s*\(", method_name),
        format!(r"this\.{}", method_name),
        format!(r"self\.{}", method_name),
    ];

    call_patterns.iter().any(|pattern| {
        Regex::new(pattern).unwrap().is_match(content)
    })
}
```

#### 1.3 从调用点找到 Controller 和接口

**工作流程**：

```
1. 找到使用 Service 的文件（如 Controller.ts）
   ↓
2. 在该文件中搜索 HTTP 路由定义
   - @PostMapping("/api/focuses")
   - router.post('/api/focuses')
   - app.post("/api/focuses")
   ↓
3. 提取接口信息（方法、路径、参数等）
```

**实现方式**：

```rust
fn find_endpoints_in_file(file_path: &str) -> Result<Vec<EndpointInfo>> {
    // 获取文件内容
    let content = get_file_content(file_path)?;

    // 搜索 HTTP 路由定义模式
    let mut endpoints = Vec::new();

    // TypeScript/Express: router.post('/api/focuses', ...)
    let express_pattern = Regex::new(
        r#"router\.(get|post|put|delete|patch)\(['"]([^'"]+)['"]"#
    )?;

    // Spring Boot: @PostMapping("/api/focuses")
    let spring_pattern = Regex::new(
        r#"@(Get|Post|Put|Delete|Patch)Mapping\(['"]([^'"]+)['"]"#
    )?;

    // NestJS: @Post('/api/focuses')
    let nestjs_pattern = Regex::new(
        r#"@(Get|Post|Put|Delete|Patch)\(['"]([^'"]+)['"]"#
    )?;

    // 在文件内容中搜索这些模式
    for cap in express_pattern.captures_iter(&content) {
        endpoints.push(EndpointInfo {
            method: cap[1].to_uppercase(),
            path: cap[2].to_string(),
            file_path: file_path.to_string(),
            line_number: find_line_number(&content, &cap[0]),
        });
    }

    // 类似处理其他模式...

    Ok(endpoints)
}
```

### 方案二：基于文件路径推断（启发式）

**核心思想**：根据 Service 文件路径，推断可能的 Controller 路径。

#### 2.1 路径映射规则（多语言支持）

**不同语言的路径映射模式**：

```rust
fn infer_controller_path(service_path: &str) -> Vec<String> {
    let language = detect_language_from_path(service_path);
    let mut possible_paths = Vec::new();

    match language {
        Some(Language::TypeScript) | Some(Language::JavaScript) => {
            // TypeScript/JavaScript
            // 规则 1: services/CerebrasService.ts -> controllers/CerebrasController.ts
            if let Some(service_name) = extract_service_name(service_path) {
                let controller_name = service_name.replace("Service", "Controller");
                possible_paths.push(
                    service_path.replace("services", "controllers")
                        .replace(&service_name, &controller_name)
                );
            }

            // 规则 2: services/CerebrasService.ts -> api/cerebras.ts
            possible_paths.push(
                service_path.replace("services", "api")
                    .replace("Service.ts", ".ts")
            );

            // 规则 3: services/CerebrasService.ts -> routes/cerebras.ts
            possible_paths.push(
                service_path.replace("services", "routes")
                    .replace("Service.ts", ".ts")
            );
        }
        Some(Language::Python) => {
            // Python
            // 规则 1: services/cerebras_service.py -> controllers/cerebras_controller.py
            if let Some(service_name) = extract_service_name(service_path) {
                let snake_case = pascal_to_snake_case(&service_name);
                let controller_snake = snake_case.replace("_service", "_controller");
                possible_paths.push(
                    service_path.replace("services", "controllers")
                        .replace(&snake_case, &controller_snake)
                );
            }

            // 规则 2: services/cerebras_service.py -> api/cerebras.py
            possible_paths.push(
                service_path.replace("services", "api")
                    .replace("_service.py", ".py")
            );

            // 规则 3: services/cerebras_service.py -> routes/cerebras.py
            possible_paths.push(
                service_path.replace("services", "routes")
                    .replace("_service.py", ".py")
            );

            // 规则 4: services/cerebras_service.py -> views/cerebras.py (Django)
            possible_paths.push(
                service_path.replace("services", "views")
                    .replace("_service.py", ".py")
            );
        }
        Some(Language::Rust) => {
            // Rust
            // 规则 1: services/cerebras_service.rs -> controllers/cerebras_controller.rs
            if let Some(service_name) = extract_service_name(service_path) {
                let snake_case = pascal_to_snake_case(&service_name);
                let controller_snake = snake_case.replace("_service", "_controller");
                possible_paths.push(
                    service_path.replace("services", "controllers")
                        .replace(&snake_case, &controller_snake)
                );
            }

            // 规则 2: services/cerebras_service.rs -> handlers/cerebras.rs
            possible_paths.push(
                service_path.replace("services", "handlers")
                    .replace("_service.rs", ".rs")
            );

            // 规则 3: services/cerebras_service.rs -> routes/cerebras.rs
            possible_paths.push(
                service_path.replace("services", "routes")
                    .replace("_service.rs", ".rs")
            );
        }
        Some(Language::Java) => {
            // Java
            // 规则 1: services/CerebrasService.java -> controllers/CerebrasController.java
            if let Some(service_name) = extract_service_name(service_path) {
                let controller_name = service_name.replace("Service", "Controller");
                possible_paths.push(
                    service_path.replace("services", "controllers")
                        .replace(&service_name, &controller_name)
                );
            }

            // 规则 2: services/CerebrasService.java -> api/CerebrasApi.java
            if let Some(service_name) = extract_service_name(service_path) {
                let api_name = service_name.replace("Service", "Api");
                possible_paths.push(
                    service_path.replace("services", "api")
                        .replace(&service_name, &api_name)
                );
            }
        }
        _ => {
            // 通用规则：尝试常见的路径替换
            possible_paths.push(
                service_path.replace("services", "controllers")
            );
            possible_paths.push(
                service_path.replace("services", "api")
            );
            possible_paths.push(
                service_path.replace("services", "routes")
            );
        }
    }

    possible_paths
}
```

#### 2.2 验证和搜索

```rust
fn find_endpoints_by_inference(service_path: &str) -> Result<Vec<EndpointInfo>> {
    let possible_paths = infer_controller_path(service_path);
    let mut endpoints = Vec::new();

    for path in possible_paths {
        // 检查文件是否存在
        if file_exists(&path) {
            // 读取文件内容，搜索接口定义
            let found = find_endpoints_in_file(&path)?;
            endpoints.extend(found);
        }
    }

    Ok(endpoints)
}
```

### 方案三：混合策略（最佳实践）

**核心思想**：结合方案一和方案二，先搜索调用点，再推断路径。

#### 3.1 完整流程

```
1. 从 Service 文件提取 Service 名称
   ↓
2. 在代码库中搜索使用该 Service 的文件
   - 搜索 import 语句
   - 搜索类实例化
   - 搜索方法调用
   ↓
3. 在找到的文件中搜索 HTTP 路由定义
   - 如果找到 Controller，提取接口定义
   ↓
4. 如果没找到，使用路径推断
   - 推断可能的 Controller 路径
   - 验证文件是否存在
   - 搜索接口定义
   ↓
5. 返回找到的接口列表
```

#### 3.2 实现结构

```rust
pub struct ServiceEndpointFinder {
    strategy: SearchStrategy,
    owner: Option<String>,
    repo: Option<String>,
}

impl ServiceEndpointFinder {
    /// 找到修改的方法对应的接口
    pub fn find_endpoints_for_modified_methods(
        &self,
        file_changes: &[(String, String)],  // (file_path, diff_content)
    ) -> Result<Vec<EndpointInfo>> {
        let mut all_endpoints = Vec::new();

        // 1. 从每个文件的 diff 中提取修改的方法
        for (file_path, diff_content) in file_changes {
            // 只处理 Service 层文件
            if !is_service_file(file_path) {
                continue;
            }

            let methods = extract_modified_methods(file_path, diff_content)?;

            // 2. 对每个修改的方法，搜索调用点
            for method in &methods {
                let call_sites = self.find_method_call_sites(method)?;

                // 3. 从调用点找到接口
                for call_site in &call_sites {
                    let found = self.find_endpoints_in_file(&call_site.file_path)?;
                    all_endpoints.extend(found);
                }
            }
        }

        // 4. 如果没找到，使用路径推断（备选策略）
        if all_endpoints.is_empty() {
            for (file_path, _) in file_changes {
                if is_service_file(file_path) {
                    let inferred = self.find_endpoints_by_inference(file_path)?;
                    all_endpoints.extend(inferred);
                }
            }
        }

        // 去重
        all_endpoints.dedup_by(|a, b| a.path == b.path && a.method == b.method);

        Ok(all_endpoints)
    }

    /// 搜索方法的调用点
    fn find_method_call_sites(
        &self,
        method_info: &MethodInfo,
    ) -> Result<Vec<CallSite>> {
        match &self.strategy {
            SearchStrategy::GitHubMCP => {
                self.find_method_calls_via_github_mcp(method_info)
            }
            SearchStrategy::GitGrep => {
                self.find_method_calls_via_git_grep(method_info)
            }
            SearchStrategy::RipGrep => {
                self.find_method_calls_via_ripgrep(method_info)
            }
            SearchStrategy::FileSystem => {
                self.find_method_calls_via_filesystem(method_info)
            }
        }
    }

    /// 判断是否是 Service 文件
    fn is_service_file(file_path: &str) -> bool {
        let path_lower = file_path.to_lowercase();
        path_lower.contains("service") ||
        path_lower.contains("/services/") ||
        path_lower.contains("\\services\\")
    }
}
```

## 📊 技术实现细节

### 1. 语言检测

**根据文件扩展名识别语言**：

```rust
fn detect_language_from_path(file_path: &str) -> Option<Language> {
    let ext = Path::new(file_path)
        .extension()
        .and_then(|s| s.to_str())?;

    match ext.to_lowercase().as_str() {
        "ts" | "tsx" => Some(Language::TypeScript),
        "js" | "jsx" => Some(Language::JavaScript),
        "py" => Some(Language::Python),
        "rs" => Some(Language::Rust),
        "go" => Some(Language::Go),
        "java" => Some(Language::Java),
        "rb" => Some(Language::Ruby),
        "php" => Some(Language::PHP),
        _ => None,
    }
}
```

### 2. Service 名称提取（多语言支持）

**不同语言的命名模式**：

#### TypeScript/JavaScript

```typescript
// 文件路径 -> Service 名称
services/CerebrasService.ts -> CerebrasService
services/user.service.ts -> UserService (需要转换)
src/services/CerebrasService.ts -> CerebrasService
```

#### Python

```python
# 文件路径 -> Service 名称
services/cerebras_service.py -> CerebrasService (需要转换)
services/user_service.py -> UserService (需要转换)
src/services/cerebras.py -> CerebrasService (需要推断)
```

#### Rust

```rust
// 文件路径 -> Service 名称
services/cerebras_service.rs -> CerebrasService (需要转换)
src/services/cerebras.rs -> CerebrasService (需要推断)
```

#### Java

```java
// 文件路径 -> Service 名称
services/CerebrasService.java -> CerebrasService
com/example/services/UserService.java -> UserService
```

**实现**：

```rust
fn extract_service_name(file_path: &str) -> Result<String> {
    let language = detect_language_from_path(file_path);
    let file_name = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .context("Invalid file path")?;

    match language {
        Some(Language::TypeScript) | Some(Language::JavaScript) => {
            // TypeScript: user.service.ts -> UserService
            if file_name.contains('.') {
                let parts: Vec<&str> = file_name.split('.').collect();
                let name_part = parts[0];
                let pascal_case = to_pascal_case(name_part);
                Ok(format!("{}Service", pascal_case))
            } else {
                Ok(file_name.to_string())
            }
        }
        Some(Language::Python) => {
            // Python: cerebras_service.py -> CerebrasService
            let pascal_case = snake_to_pascal_case(file_name);
            if pascal_case.ends_with("Service") {
                Ok(pascal_case)
            } else {
                Ok(format!("{}Service", pascal_case))
            }
        }
        Some(Language::Rust) => {
            // Rust: cerebras_service.rs -> CerebrasService
            let pascal_case = snake_to_pascal_case(file_name);
            if pascal_case.ends_with("Service") {
                Ok(pascal_case)
            } else {
                Ok(format!("{}Service", pascal_case))
            }
        }
        Some(Language::Java) => {
            // Java: CerebrasService.java -> CerebrasService
            Ok(file_name.to_string())
        }
        _ => {
            // 默认：尝试提取类名
            Ok(file_name.to_string())
        }
    }
}

// 辅助函数：snake_case -> PascalCase
fn snake_to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
```

### 3. 调用点搜索模式（多语言支持）

#### TypeScript/JavaScript

```typescript
// Import 语句
import { CerebrasService } from '...'
import CerebrasService from '...'
import * as CerebrasService from '...'
const { CerebrasService } = require('...')

// 类使用
new CerebrasService()
this.cerebrasService
private cerebrasService: CerebrasService
const cerebrasService = new CerebrasService()

// 方法调用
cerebrasService.createFocus()
this.cerebrasService.generate()
await cerebrasService.process()
```

#### Python

```python
# Import 语句
from services.cerebras_service import CerebrasService
from services import cerebras_service
import services.cerebras_service as cerebras_service

# 类使用
service = CerebrasService()
self.cerebras_service = CerebrasService()
cerebras_service = CerebrasService()

# 方法调用
service.create_focus()
self.cerebras_service.generate()
await cerebras_service.process()
```

#### Rust

```rust
// Import 语句
use services::cerebras_service::CerebrasService;
use services::cerebras_service;

// 结构体使用
let service = CerebrasService::new();
let mut service = CerebrasService::default();

// 方法调用
service.create_focus();
CerebrasService::static_method();
```

#### Java

```java
// Import 语句
import com.example.services.CerebrasService;
import com.example.services.*;

// 类使用
CerebrasService service = new CerebrasService();
this.cerebrasService = new CerebrasService();

// 方法调用
service.createFocus();
this.cerebrasService.generate();
```

**搜索查询构建（多语言）**：

```rust
fn build_search_queries(service_name: &str, language: Option<Language>) -> Vec<String> {
    let mut queries = Vec::new();

    match language {
        Some(Language::TypeScript) | Some(Language::JavaScript) => {
            queries.push(format!("import.*{}", service_name));
            queries.push(format!("from.*{}", service_name));
            queries.push(format!("new {}", service_name));
            queries.push(format!("{}", service_name));
        }
        Some(Language::Python) => {
            // Python: CerebrasService -> cerebras_service
            let snake_case = pascal_to_snake_case(service_name);
            queries.push(format!("from.*{}", snake_case));
            queries.push(format!("import.*{}", snake_case));
            queries.push(format!("{}", service_name));
            queries.push(format!("{}", snake_case));
        }
        Some(Language::Rust) => {
            // Rust: CerebrasService -> cerebras_service
            let snake_case = pascal_to_snake_case(service_name);
            queries.push(format!("use.*{}", snake_case));
            queries.push(format!("{}::", service_name));
            queries.push(format!("{}", service_name));
        }
        Some(Language::Java) => {
            queries.push(format!("import.*{}", service_name));
            queries.push(format!("new {}", service_name));
            queries.push(format!("{}", service_name));
        }
        _ => {
            // 通用搜索
            queries.push(format!("{}", service_name));
        }
    }

    queries
}
```

### 4. 接口定义识别（多语言支持）

#### TypeScript/JavaScript (Express)

```typescript
// Express
router.post('/api/focuses', async (req, res) => {
  const service = new CerebrasService();
  // ...
})

app.post('/api/focuses', handler)
```

#### TypeScript/JavaScript (NestJS)

```typescript
// NestJS
@Post('/api/focuses')
async createFocus(@Body() dto: CreateFocusDto) {
  return this.cerebrasService.create(dto);
}

@Controller('/api/focuses')
export class FocusController {
  constructor(private cerebrasService: CerebrasService) {}
}
```

#### Python (FastAPI)

```python
# FastAPI
@app.post("/api/focuses")
async def create_focus(dto: CreateFocusDto):
    service = CerebrasService()
    return service.create(dto)

@router.post("/api/focuses")
async def create_focus(dto: CreateFocusDto, service: CerebrasService = Depends()):
    return service.create(dto)
```

#### Python (Flask)

```python
# Flask
@app.route('/api/focuses', methods=['POST'])
def create_focus():
    service = CerebrasService()
    return service.create()

@bp.route('/api/focuses', methods=['POST'])
def create_focus():
    service = CerebrasService()
    return service.create()
```

#### Python (Django)

```python
# Django
from django.urls import path
from .views import create_focus

urlpatterns = [
    path('api/focuses', create_focus, name='create_focus'),
]

# views.py
def create_focus(request):
    service = CerebrasService()
    return service.create()
```

#### Rust (Actix-web)

```rust
// Actix-web
#[post("/api/focuses")]
async fn create_focus(
    req: web::Json<CreateFocusDto>,
    service: web::Data<CerebrasService>,
) -> impl Responder {
    service.create_focus(req.into_inner())
}

// 或使用宏
route("/api/focuses", web::post().to(create_focus))
```

#### Rust (Axum)

```rust
// Axum
async fn create_focus(
    Json(dto): Json<CreateFocusDto>,
    State(service): State<CerebrasService>,
) -> impl IntoResponse {
    service.create_focus(dto)
}

let app = Router::new()
    .route("/api/focuses", post(create_focus));
```

#### Java (Spring Boot)

```java
// Spring Boot
@PostMapping("/api/focuses")
public ResponseEntity<Focus> createFocus(@RequestBody CreateFocusDto dto) {
    return cerebrasService.create(dto);
}

@RestController
@RequestMapping("/api")
public class FocusController {
    @Autowired
    private CerebrasService cerebrasService;
}
```

**接口定义识别实现**：

```rust
fn find_endpoints_in_file(file_path: &str) -> Result<Vec<EndpointInfo>> {
    let language = detect_language_from_path(file_path);
    let content = get_file_content(file_path)?;
    let mut endpoints = Vec::new();

    match language {
        Some(Language::TypeScript) | Some(Language::JavaScript) => {
            // Express: router.post('/api/focuses', ...)
            let express_pattern = Regex::new(
                r#"(?:router|app)\.(get|post|put|delete|patch)\(['"]([^'"]+)['"]"#
            )?;

            // NestJS: @Post('/api/focuses')
            let nestjs_pattern = Regex::new(
                r#"@(Get|Post|Put|Delete|Patch)\(['"]([^'"]+)['"]"#
            )?;

            // 搜索这些模式...
        }
        Some(Language::Python) => {
            // FastAPI: @app.post("/api/focuses")
            let fastapi_pattern = Regex::new(
                r#"@(?:app|router)\.(get|post|put|delete|patch)\(['"]([^'"]+)['"]"#
            )?;

            // Flask: @app.route('/api/focuses', methods=['POST'])
            let flask_pattern = Regex::new(
                r#"@(?:app|bp)\.route\(['"]([^'"]+)['"].*methods=\[['"](GET|POST|PUT|DELETE|PATCH)['"]"#
            )?;

            // Django: path('api/focuses', view)
            let django_pattern = Regex::new(
                r#"path\(['"]([^'"]+)['"]"#
            )?;

            // 搜索这些模式...
        }
        Some(Language::Rust) => {
            // Actix-web: #[post("/api/focuses")]
            let actix_pattern = Regex::new(
                r#"#\[(get|post|put|delete|patch)\(['"]([^'"]+)['"]"#
            )?;

            // Axum: .route("/api/focuses", post(...))
            let axum_pattern = Regex::new(
                r#"\.route\(['"]([^'"]+)['"].*(get|post|put|delete|patch)"#
            )?;

            // 搜索这些模式...
        }
        Some(Language::Java) => {
            // Spring Boot: @PostMapping("/api/focuses")
            let spring_pattern = Regex::new(
                r#"@(Get|Post|Put|Delete|Patch)Mapping\(['"]([^'"]+)['"]"#
            )?;

            // 搜索这些模式...
        }
        _ => {
            // 通用搜索：查找 HTTP 方法和路径
            // ...
        }
    }

    Ok(endpoints)
}
```

### 4. 代码上下文获取

**使用 GitHub MCP**：

```rust
fn find_call_sites_via_github_mcp(
    &self,
    service_name: &str,
) -> Result<Vec<CallSite>> {
    let owner = self.owner.as_ref().context("GitHub owner not available")?;
    let repo = self.repo.as_ref().context("GitHub repo not available")?;

    // 搜索 import 语句
    let import_query = format!("repo:{} {} import", owner, repo);
    let import_results = mcp_github_search_code(
        q: &format!("{} {}", import_query, service_name),
        per_page: Some(100),
    )?;

    // 搜索类使用
    let usage_query = format!("repo:{} {} {}", owner, repo, service_name);
    let usage_results = mcp_github_search_code(
        q: &usage_query,
        per_page: Some(100),
    )?;

    // 解析结果
    let mut call_sites = Vec::new();
    for result in import_results.items {
        call_sites.push(CallSite {
            file_path: result.path,
            line_number: Some(result.line_number),
            content: result.text,
        });
    }

    Ok(call_sites)
}
```

**使用 Git grep**：

```rust
fn find_call_sites_via_git_grep(
    &self,
    service_name: &str,
) -> Result<Vec<CallSite>> {
    // 搜索 import 语句
    let output = Command::new("git")
        .args(&["grep", "-n", "-E", &format!("import.*{}", service_name)])
        .output()?;

    parse_git_grep_output(&output.stdout)
}
```

## 🎯 推荐方案

### 阶段一：基础实现（当前阶段）

**方案**：基于 Service 名称搜索调用点

1. **提取 Service 名称**：
   - 从文件路径提取
   - 支持多种命名模式

2. **搜索调用点**：
   - 使用混合策略（GitHub MCP → Git grep → ripgrep）
   - 搜索 import 语句和类使用

3. **提取接口定义**：
   - 在找到的文件中搜索 HTTP 路由定义
   - 提取接口信息

**优点**：
- ✅ 相对准确
- ✅ 可以找到实际的调用关系
- ✅ 不依赖路径推断

**缺点**：
- ⚠️ 需要代码库搜索
- ⚠️ 可能找不到所有调用点

### 阶段二：增强功能（后续）

**方案**：添加路径推断作为 fallback

1. **路径推断**：
   - 根据 Service 路径推断 Controller 路径
   - 验证文件是否存在

2. **多策略组合**：
   - 先搜索调用点
   - 如果没找到，使用路径推断
   - 如果还没找到，使用 LLM 推断

## 📝 数据结构设计

### CallSite

```rust
pub struct CallSite {
    /// 文件路径
    pub file_path: String,
    /// 行号（如果可用）
    pub line_number: Option<u32>,
    /// 代码片段
    pub content: String,
    /// 调用类型（import, instantiation, method_call）
    pub call_type: CallType,
}
```

### EndpointInfo

```rust
pub struct EndpointInfo {
    /// HTTP 方法
    pub method: String,  // GET, POST, PUT, DELETE, PATCH
    /// 接口路径
    pub path: String,    // /api/focuses
    /// 定义文件路径
    pub file_path: String,
    /// 行号（如果可用）
    pub line_number: Option<u32>,
    /// 参数信息（如果可提取）
    pub parameters: Option<Vec<ParameterInfo>>,
}
```

## 🔄 集成到测试计划生成

### 修改 generate_test_plan 流程

```rust
pub fn generate_test_plan(
    pr_title: &str,
    pr_diff: &str,
    file_changes: &[(String, String)],  // (file_path, diff_content)
) -> Result<String> {
    // 1. 从 PR diff 中提取修改的方法
    // file_changes 已经包含了每个文件的 diff 内容

    // 2. 使用 ServiceEndpointFinder 找到修改的方法对应的接口
    let mut related_endpoints = Vec::new();
    if !file_changes.is_empty() {
        let finder = ServiceEndpointFinder::new()?;
        related_endpoints = finder.find_endpoints_for_modified_methods(file_changes)?;
    }

    // 3. 构建 user prompt，包含找到的接口信息
    let user_prompt = Self::test_plan_user_prompt(
        pr_title,
        pr_diff,
        file_changes,
        &related_endpoints,  // 新增参数
    )?;

    // 4. 调用 LLM 生成测试计划
    // ...
}
```

### 修改 user prompt

```rust
fn test_plan_user_prompt(
    pr_title: &str,
    pr_diff: &str,
    file_changes: &[(String, String)],
    related_endpoints: &[EndpointInfo],  // 新增参数
) -> String {
    // ... 现有代码

    // 添加找到的相关接口信息
    if !related_endpoints.is_empty() {
        parts.push("## Related Endpoints Found".to_string());
        parts.push("The following endpoints were found that may be affected by the service changes:".to_string());
        parts.push("".to_string());

        for endpoint in related_endpoints {
            let location = if let Some(line) = endpoint.line_number {
                format!("{}:{}", endpoint.file_path, line)
            } else {
                endpoint.file_path.clone()
            };
            parts.push(format!(
                "- **{} {}** (in `{}`)",
                endpoint.method, endpoint.path, location
            ));
        }

        parts.push("".to_string());
        parts.push("**Please include test plans for these endpoints in your response.**".to_string());
        parts.push("".to_string());
        parts.push("For each endpoint, provide:".to_string());
        parts.push("1. Test scenarios (normal case, validation cases, edge cases)".to_string());
        parts.push("2. CURL command with example parameters".to_string());
        parts.push("3. Expected response structure".to_string());
        parts.push("4. Test priority based on the service changes".to_string());
    }

    parts.join("\n\n")
}
```

## ⚠️ 注意事项

### 1. 性能考虑

- 代码库搜索可能较慢（大型代码库）
- 建议：并行搜索多个 Service
- 建议：缓存搜索结果
- 建议：按语言过滤文件，减少搜索范围

### 2. 准确性

- 搜索可能返回不相关的结果
- 建议：过滤和验证搜索结果
- 建议：使用更精确的搜索模式
- 建议：结合语言特定的模式匹配

### 3. 多语言支持

- **不同语言的调用模式不同**：
  - TypeScript/JavaScript: `import`, `new`, `this.`
  - Python: `from ... import`, `import ...`, `self.`
  - Rust: `use`, `::`, `let ... =`
  - Java: `import`, `new`, `this.`

- **不同语言的命名约定不同**：
  - TypeScript/JavaScript: PascalCase (CerebrasService)
  - Python: snake_case (cerebras_service)
  - Rust: snake_case (cerebras_service)
  - Java: PascalCase (CerebrasService)

- **不同语言的接口定义模式不同**：
  - Express: `router.post('/api/...')`
  - FastAPI: `@app.post("/api/...")`
  - Actix-web: `#[post("/api/...")]`
  - Spring Boot: `@PostMapping("/api/...")`

- **建议**：
  - 为每种语言实现特定的搜索模式
  - 实现命名约定转换（PascalCase ↔ snake_case）
  - 使用语言检测来选择合适的搜索策略
  - 使用 LLM 辅助识别（作为 fallback）

### 4. 错误处理

- 搜索可能失败
- 建议：如果搜索失败，回退到 LLM 推断
- 建议：记录警告，但不中断流程
- 建议：如果语言检测失败，使用通用搜索模式

### 5. 语言检测的局限性

- 文件扩展名可能不准确（如 `.ts` 可能是 TypeScript 或 TSX）
- 某些文件可能没有扩展名
- 建议：结合文件内容和路径进行检测
- 建议：如果检测失败，尝试多种语言的模式

## ✅ 实施建议

### 阶段一：基础实现（当前阶段）

1. **实现语言检测**：
   - 根据文件扩展名识别语言
   - 支持 TypeScript、JavaScript、Python、Rust、Java 等

2. **实现方法提取（多语言）**：
   - 从 PR diff 中提取修改的方法名
   - 支持多种语言的方法定义模式
   - 识别方法定义（function, def, fn, method）
   - 提取方法名（考虑命名约定）

3. **实现方法调用点搜索（多语言）**：
   - 使用混合策略（GitHub MCP → Git grep）
   - 根据语言选择不同的搜索模式
   - 搜索方法调用（.methodName(, ::method_name(, this.methodName）
   - 考虑命名约定转换（PascalCase ↔ snake_case）
   - 过滤方法定义，只保留方法调用

4. **实现接口提取（多语言）**：
   - 在找到的调用点文件中搜索 HTTP 路由定义
   - 支持多种框架（Express、FastAPI、Actix-web、Spring Boot 等）
   - 提取接口信息（路径、方法、参数等）

5. **集成到测试计划生成**：
   - 修改 `generate_test_plan` 函数
   - 修改 `test_plan_user_prompt` 函数
   - 传入 `file_changes`（包含 diff 内容）而不是只传入文件路径

**开发时间**：3-4 天（增加了多语言支持和方法提取的工作量）

### 阶段二：增强功能（后续）

1. **路径推断**：
   - 实现路径映射规则
   - 验证和搜索

2. **优化和调优**：
   - 提高搜索准确性
   - 优化性能
   - 支持更多语言和框架

**开发时间**：1-2 天

## 🌍 多语言支持总结

### 支持的语言和框架

| 语言 | 框架 | Service 命名 | Import 模式 | 接口定义模式 |
|------|------|-------------|------------|-------------|
| **TypeScript** | Express | `CerebrasService` | `import { CerebrasService }` | `router.post('/api/...')` |
| **TypeScript** | NestJS | `CerebrasService` | `import { CerebrasService }` | `@Post('/api/...')` |
| **JavaScript** | Express | `CerebrasService` | `import CerebrasService` | `router.post('/api/...')` |
| **Python** | FastAPI | `CerebrasService` (类) | `from services import CerebrasService` | `@app.post("/api/...")` |
| **Python** | Flask | `CerebrasService` (类) | `from services import CerebrasService` | `@app.route('/api/...')` |
| **Python** | Django | `CerebrasService` (类) | `from services import CerebrasService` | `path('api/...', view)` |
| **Rust** | Actix-web | `CerebrasService` (struct) | `use services::CerebrasService` | `#[post("/api/...")]` |
| **Rust** | Axum | `CerebrasService` (struct) | `use services::CerebrasService` | `.route("/api/...", post(...))` |
| **Java** | Spring Boot | `CerebrasService` | `import com.example.CerebrasService` | `@PostMapping("/api/...")` |

### 关键实现点

1. **语言检测**：
   - 根据文件扩展名（`.ts`, `.py`, `.rs`, `.java`）
   - 支持多种扩展名变体（`.tsx`, `.jsx` 等）

2. **命名约定转换**：
   - PascalCase ↔ snake_case
   - 处理不同语言的 Service 后缀（`Service`, `_service`）

3. **搜索模式适配**：
   - 每种语言使用不同的 import/use 模式
   - 每种框架使用不同的接口定义模式

4. **路径推断规则**：
   - 不同语言的目录结构可能不同
   - 需要语言特定的路径映射规则

### 扩展性

- **新增语言**：实现 `Language` enum 和对应的处理逻辑
- **新增框架**：添加框架特定的接口定义识别模式
- **命名约定**：添加新的命名约定转换函数

## 📚 参考

- 代码库访问策略：`docs/requirements/CODEBASE_ACCESS_STRATEGY.md`
- 接口识别问题分析：`docs/requirements/testing/ENDPOINT_IDENTIFICATION_ANALYSIS.md`
- Summarize 代码上下文分析：`docs/requirements/SUMMARIZE_CODE_CONTEXT_ANALYSIS.md`

