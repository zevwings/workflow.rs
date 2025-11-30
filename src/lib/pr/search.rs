//! 代码库搜索和接口发现功能
//!
//! 从 PR diff 中提取修改的方法名，搜索调用点，找到对应的 HTTP 接口定义。

use anyhow::{Context, Result};
use base64::Engine;
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::base::http::{HttpClient, RequestConfig};
use crate::base::settings::Settings;
use crate::git::{GitRepo, RepoType};
use crate::pr::helpers::extract_github_repo_from_url;
use reqwest::header::HeaderMap;

/// 编程语言类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    TypeScript,
    JavaScript,
    Python,
    Rust,
    Go,
    Java,
    Ruby,
    PHP,
    Unknown,
}

/// 方法信息
#[derive(Debug, Clone)]
pub struct MethodInfo {
    /// 方法名
    pub name: String,
    /// 语言类型
    pub language: Language,
    /// 文件路径
    pub file_path: String,
}

/// 调用点信息
#[derive(Debug, Clone)]
pub struct CallSite {
    /// 文件路径
    pub file_path: String,
    /// 行号（如果可用）
    pub line_number: Option<u32>,
    /// 代码片段
    pub content: String,
    /// 语言类型
    pub language: Language,
}

/// 接口信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointInfo {
    /// HTTP 方法
    pub method: String, // GET, POST, PUT, DELETE, PATCH
    /// 接口路径
    pub path: String, // /api/focuses
    /// 定义文件路径
    pub file_path: String,
    /// 行号（如果可用）
    pub line_number: Option<u32>,
    /// 语言类型
    pub language: Language,
}

/// 代码库搜索器
pub struct CodebaseSearcher {
    /// 搜索策略
    strategy: SearchStrategy,
    /// GitHub 仓库 owner（如果可用）
    github_owner: Option<String>,
    /// GitHub 仓库 repo（如果可用）
    github_repo: Option<String>,
}

/// 搜索策略
#[allow(dead_code)]
enum SearchStrategy {
    /// 使用 GitHub MCP（如果可用且是 GitHub 仓库）
    GitHubMCP,
    /// 使用 git grep（默认，最可靠）
    GitGrep,
    /// 使用 ripgrep（如果可用，性能最好）
    RipGrep,
    /// 使用文件系统搜索（fallback）
    FileSystem,
}

impl CodebaseSearcher {
    /// 创建新的代码库搜索器
    pub fn new() -> Result<Self> {
        // 尝试获取 GitHub 仓库信息
        let (github_owner, github_repo) = Self::extract_github_info().ok().unzip();

        // 检测可用的搜索策略
        let strategy = Self::detect_search_strategy(&github_owner, &github_repo)?;

        Ok(Self {
            strategy,
            github_owner,
            github_repo,
        })
    }

    /// 从 Git 配置中提取 GitHub 仓库信息
    fn extract_github_info() -> Result<(String, String)> {
        // 检查是否是 GitHub 仓库
        let repo_type = GitRepo::detect_repo_type()?;
        if repo_type != RepoType::GitHub {
            anyhow::bail!("Not a GitHub repository");
        }

        // 从 remote URL 提取 owner/repo
        let github_repo = extract_github_repo_from_url(&GitRepo::get_remote_url()?)?;

        // 解析 owner/repo
        let parts: Vec<&str> = github_repo.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid GitHub repo format: {}", github_repo);
        }

        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// 检测可用的搜索策略
    fn detect_search_strategy(
        github_owner: &Option<String>,
        github_repo: &Option<String>,
    ) -> Result<SearchStrategy> {
        // 优先使用 GitHub MCP（如果是 GitHub 仓库且 MCP 可用）
        if github_owner.is_some() && github_repo.is_some() && Self::is_github_mcp_available() {
            return Ok(SearchStrategy::GitHubMCP);
        }

        // 其次使用 git grep（最可靠）
        if Self::is_git_repo() {
            return Ok(SearchStrategy::GitGrep);
        }

        // 检查 ripgrep 是否可用
        if Self::is_ripgrep_available() {
            return Ok(SearchStrategy::RipGrep);
        }

        // Fallback 到文件系统搜索
        Ok(SearchStrategy::FileSystem)
    }

    /// 检查 GitHub MCP 是否可用
    fn is_github_mcp_available() -> bool {
        // 检查是否配置了 GitHub API token（可以直接使用 GitHub API）
        Settings::get().github.get_current_token().is_some()
    }

    /// 获取 GitHub API headers
    fn get_github_headers() -> Result<HeaderMap> {
        let settings = Settings::get();
        let token = settings.github.get_current_token().context(
            "GitHub API token is not configured. Please run 'workflow setup' to configure it",
        )?;

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token)
                .parse()
                .context("Failed to parse Authorization header")?,
        );
        headers.insert(
            "Accept",
            "application/vnd.github+json"
                .parse()
                .context("Failed to parse Accept header")?,
        );
        headers.insert(
            "X-GitHub-Api-Version",
            "2022-11-28"
                .parse()
                .context("Failed to parse X-GitHub-Api-Version header")?,
        );
        headers.insert(
            "User-Agent",
            "workflow-cli"
                .parse()
                .context("Failed to parse User-Agent header")?,
        );

        Ok(headers)
    }

    /// 检查是否是 Git 仓库
    fn is_git_repo() -> bool {
        Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .output()
            .is_ok()
    }

    /// 检查 ripgrep 是否可用
    fn is_ripgrep_available() -> bool {
        Command::new("rg").arg("--version").output().is_ok()
    }

    /// 从 PR diff 中提取修改的方法名
    pub fn extract_modified_methods(
        file_path: &str,
        diff_content: &str,
    ) -> Result<Vec<MethodInfo>> {
        let language = Self::detect_language_from_path(file_path);
        let mut methods = Vec::new();

        match language {
            Language::TypeScript | Language::JavaScript => {
                Self::extract_methods_typescript(diff_content, file_path, &mut methods)?;
            }
            Language::Python => {
                Self::extract_methods_python(diff_content, file_path, &mut methods)?;
            }
            Language::Rust => {
                Self::extract_methods_rust(diff_content, file_path, &mut methods)?;
            }
            Language::Java => {
                Self::extract_methods_java(diff_content, file_path, &mut methods)?;
            }
            _ => {
                // 通用模式：尝试识别函数定义
                // 暂时跳过
            }
        }

        // 去重
        methods.dedup_by(|a, b| a.name == b.name);

        Ok(methods)
    }

    /// 提取 TypeScript/JavaScript 方法
    fn extract_methods_typescript(
        diff_content: &str,
        file_path: &str,
        methods: &mut Vec<MethodInfo>,
    ) -> Result<()> {
        let patterns = vec![
            Regex::new(r"(?:async\s+)?(?:function\s+)?(\w+)\s*\([^)]*\)\s*\{")?,
            Regex::new(r"(\w+)\s*=\s*(?:async\s*)?\([^)]*\)\s*=>")?,
            Regex::new(r"(\w+)\s*\([^)]*\)\s*:\s*")?, // 方法签名
        ];

        for pattern in patterns {
            for cap in pattern.captures_iter(diff_content) {
                if let Some(method_name) = cap.get(1) {
                    methods.push(MethodInfo {
                        name: method_name.as_str().to_string(),
                        language: Language::TypeScript,
                        file_path: file_path.to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// 提取 Python 方法
    fn extract_methods_python(
        diff_content: &str,
        file_path: &str,
        methods: &mut Vec<MethodInfo>,
    ) -> Result<()> {
        let pattern = Regex::new(r"(?:async\s+)?def\s+(\w+)\s*\([^)]*\)\s*:")?;

        for cap in pattern.captures_iter(diff_content) {
            if let Some(method_name) = cap.get(1) {
                methods.push(MethodInfo {
                    name: method_name.as_str().to_string(),
                    language: Language::Python,
                    file_path: file_path.to_string(),
                });
            }
        }

        Ok(())
    }

    /// 提取 Rust 方法
    fn extract_methods_rust(
        diff_content: &str,
        file_path: &str,
        methods: &mut Vec<MethodInfo>,
    ) -> Result<()> {
        let pattern = Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\([^)]*\)")?;

        for cap in pattern.captures_iter(diff_content) {
            if let Some(method_name) = cap.get(1) {
                methods.push(MethodInfo {
                    name: method_name.as_str().to_string(),
                    language: Language::Rust,
                    file_path: file_path.to_string(),
                });
            }
        }

        Ok(())
    }

    /// 提取 Java 方法
    fn extract_methods_java(
        diff_content: &str,
        file_path: &str,
        methods: &mut Vec<MethodInfo>,
    ) -> Result<()> {
        let pattern = Regex::new(r"(?:public|private|protected)\s+(?:\w+\s+)*(\w+)\s*\([^)]*\)")?;

        for cap in pattern.captures_iter(diff_content) {
            if let Some(method_name) = cap.get(1) {
                methods.push(MethodInfo {
                    name: method_name.as_str().to_string(),
                    language: Language::Java,
                    file_path: file_path.to_string(),
                });
            }
        }

        Ok(())
    }

    /// 根据文件路径检测语言
    pub fn detect_language_from_path(file_path: &str) -> Language {
        if let Some(ext) = Path::new(file_path).extension().and_then(|s| s.to_str()) {
            match ext.to_lowercase().as_str() {
                "ts" | "tsx" => Language::TypeScript,
                "js" | "jsx" => Language::JavaScript,
                "py" => Language::Python,
                "rs" => Language::Rust,
                "go" => Language::Go,
                "java" => Language::Java,
                "rb" => Language::Ruby,
                "php" => Language::PHP,
                _ => Language::Unknown,
            }
        } else {
            Language::Unknown
        }
    }

    /// 找到修改的方法对应的接口
    pub fn find_endpoints_for_modified_methods(
        &self,
        file_changes: &[(String, String)], // (file_path, diff_content)
    ) -> Result<Vec<EndpointInfo>> {
        let mut all_endpoints = Vec::new();

        // 策略 1: 直接从路由文件中提取接口定义（优先级最高）
        for (file_path, _) in file_changes {
            if Self::is_route_file(file_path) {
                // 直接提取路由文件中的接口定义
                if let Ok(endpoints) = self.find_endpoints_in_file(file_path) {
                    all_endpoints.extend(endpoints);
                }
            }
        }

        // 策略 2: 从 Service 层文件提取方法，搜索调用点
        for (file_path, diff_content) in file_changes {
            // 只处理 Service 层文件
            if !Self::is_service_file(file_path) {
                continue;
            }

            let methods = Self::extract_modified_methods(file_path, diff_content)?;

            // 如果找到了方法，搜索调用点
            if !methods.is_empty() {
                // 2. 对每个修改的方法，搜索调用点
                for method in &methods {
                    let call_sites = self.find_method_call_sites(method)?;

                    // 3. 从调用点找到接口
                    for call_site in &call_sites {
                        if let Ok(found) = self.find_endpoints_in_file(&call_site.file_path) {
                            all_endpoints.extend(found);
                        }
                    }
                }
            }
        }

        // 去重
        all_endpoints.dedup_by(|a, b| a.path == b.path && a.method == b.method);

        Ok(all_endpoints)
    }

    /// 判断是否是路由文件（API 定义文件）
    fn is_route_file(file_path: &str) -> bool {
        let path_lower = file_path.to_lowercase();
        path_lower.contains("/rest/")
            || path_lower.contains("/api/")
            || path_lower.contains("/routes/")
            || path_lower.contains("\\rest\\")
            || path_lower.contains("\\api\\")
            || path_lower.contains("\\routes\\")
            || path_lower.contains("controller")
            || path_lower.contains("router")
    }

    /// 判断是否是 Service 文件
    fn is_service_file(file_path: &str) -> bool {
        let path_lower = file_path.to_lowercase();
        path_lower.contains("service")
            || path_lower.contains("/services/")
            || path_lower.contains("\\services\\")
    }

    /// 搜索方法的调用点
    fn find_method_call_sites(&self, method_info: &MethodInfo) -> Result<Vec<CallSite>> {
        match &self.strategy {
            SearchStrategy::GitHubMCP => self.find_method_calls_via_github_mcp(method_info),
            SearchStrategy::GitGrep => self.find_method_calls_via_git_grep(method_info),
            SearchStrategy::RipGrep => self.find_method_calls_via_ripgrep(method_info),
            SearchStrategy::FileSystem => {
                // Fallback 到 git grep（如果可用）
                if Self::is_git_repo() {
                    self.find_method_calls_via_git_grep(method_info)
                } else {
                    Ok(Vec::new())
                }
            }
        }
    }

    /// 使用 GitHub MCP 搜索方法调用
    fn find_method_calls_via_github_mcp(&self, method_info: &MethodInfo) -> Result<Vec<CallSite>> {
        let owner = self
            .github_owner
            .as_ref()
            .context("GitHub owner not available")?;
        let repo = self
            .github_repo
            .as_ref()
            .context("GitHub repo not available")?;

        let queries = Self::build_method_search_queries(&method_info.name, method_info.language);
        let mut call_sites = Vec::new();

        for query in &queries {
            // 构建 GitHub 搜索查询
            // 格式: repo:owner/repo query
            let full_query = format!("repo:{} {} {}", owner, repo, query);

            // 调用 GitHub MCP 搜索代码
            // 注意：这里需要实际调用 MCP 函数，但由于 MCP 函数是通过工具调用的，
            // 我们需要使用不同的方式。暂时先尝试使用 git grep 作为 fallback
            // 实际实现中，应该调用 mcp_github_search_code 工具

            // 如果 GitHub MCP 不可用，fallback 到 git grep
            match self.try_github_mcp_search(&full_query, method_info.language) {
                Ok(results) => {
                    // 过滤结果：只保留实际的方法调用（排除定义）
                    for result in results {
                        if Self::is_method_call(
                            &result.content,
                            &method_info.name,
                            method_info.language,
                        ) {
                            call_sites.push(result);
                        }
                    }
                }
                Err(_) => {
                    // Fallback 到 git grep
                    let git_results = self.find_method_calls_via_git_grep(method_info)?;
                    call_sites.extend(git_results);
                }
            }
        }

        Ok(call_sites)
    }

    /// 尝试使用 GitHub API 搜索（如果可用）
    fn try_github_mcp_search(&self, query: &str, language: Language) -> Result<Vec<CallSite>> {
        // 直接使用 GitHub API 搜索代码
        // GitHub API 文档: https://docs.github.com/en/rest/search/search?apiVersion=2022-11-28#search-code
        let api_url = "https://api.github.com/search/code";

        let headers = Self::get_github_headers()?;

        // 构建查询参数
        let query_params = [("q", query), ("per_page", "100")];

        // 发送 GET 请求
        let client = HttpClient::global()?;
        let config = RequestConfig::<Value, _>::new()
            .query(&query_params)
            .headers(&headers)
            .timeout(std::time::Duration::from_secs(30));

        let response = client
            .get(api_url, config)
            .context("Failed to call GitHub API")?;

        // 检查响应状态
        let http_response = response
            .ensure_success()
            .context("GitHub API returned error response")?;

        // 解析 JSON 响应
        let data: Value = http_response
            .as_json()
            .context("Failed to parse GitHub API response as JSON")?;

        // 解析 GitHub API 响应格式
        // GitHub API 响应格式: { "total_count": ..., "items": [...] }
        let call_sites = Self::parse_github_search_response(&data, language)
            .context("Failed to parse GitHub search response")?;

        Ok(call_sites)
    }

    /// 解析 GitHub API 搜索响应
    fn parse_github_search_response(data: &Value, language: Language) -> Result<Vec<CallSite>> {
        let mut call_sites = Vec::new();

        // GitHub API 响应格式: { "total_count": ..., "items": [...] }
        // 每个 item 包含: { "name", "path", "sha", "url", "git_url", "html_url", "repository", "score" }
        // 注意：GitHub API 的 code search 不直接返回代码内容，只返回匹配的文件信息
        // 我们需要根据返回的 path 和 repository 信息，后续再获取文件内容
        if let Some(items) = data.get("items").and_then(|v| v.as_array()) {
            for item in items {
                if let Some(path) = item.get("path").and_then(|v| v.as_str()) {
                    // GitHub API 返回的是文件路径，不是代码片段
                    // 我们暂时使用路径作为 content，后续可以通过 read_file_via_github_mcp 获取实际内容
                    let repository = item.get("repository");
                    let full_path = if let Some(repo) = repository {
                        let repo_name =
                            repo.get("full_name").and_then(|v| v.as_str()).unwrap_or("");
                        format!("{}:{}", repo_name, path)
                    } else {
                        path.to_string()
                    };

                    call_sites.push(CallSite {
                        file_path: full_path,
                        line_number: None, // GitHub API 不返回行号
                        content: format!("Found in {}", path), // 占位内容
                        language,
                    });
                }
            }
        }

        Ok(call_sites)
    }

    /// 使用 Git grep 搜索方法调用
    fn find_method_calls_via_git_grep(&self, method_info: &MethodInfo) -> Result<Vec<CallSite>> {
        let queries = Self::build_method_search_queries(&method_info.name, method_info.language);
        let mut call_sites = Vec::new();

        for query in &queries {
            let output = Command::new("git")
                .args(["grep", "-n", "-E", query])
                .output()
                .context("Failed to execute git grep")?;

            let results = Self::parse_git_grep_output(&output.stdout, method_info.language)?;

            // 过滤结果：只保留实际的方法调用（排除定义）
            for result in results {
                if Self::is_method_call(&result.content, &method_info.name, method_info.language) {
                    call_sites.push(result);
                }
            }
        }

        Ok(call_sites)
    }

    /// 使用 ripgrep 搜索方法调用
    fn find_method_calls_via_ripgrep(&self, method_info: &MethodInfo) -> Result<Vec<CallSite>> {
        let queries = Self::build_method_search_queries(&method_info.name, method_info.language);
        let mut call_sites = Vec::new();

        for query in &queries {
            let output = Command::new("rg")
                .args(["-n", query])
                .output()
                .context("Failed to execute ripgrep")?;

            let results = Self::parse_ripgrep_output(&output.stdout, method_info.language)?;

            // 过滤结果：只保留实际的方法调用（排除定义）
            for result in results {
                if Self::is_method_call(&result.content, &method_info.name, method_info.language) {
                    call_sites.push(result);
                }
            }
        }

        Ok(call_sites)
    }

    /// 构建方法搜索查询（考虑命名约定）
    fn build_method_search_queries(method_name: &str, language: Language) -> Vec<String> {
        let mut queries = Vec::new();

        match language {
            Language::TypeScript | Language::JavaScript => {
                // TypeScript/JavaScript: 方法名通常是 camelCase
                queries.push(format!(r"\.{}\s*\(", method_name));
                queries.push(format!(r"this\.{}", method_name));
                queries.push(format!(r"service\.{}", method_name));
            }
            Language::Python => {
                // Python: 方法名通常是 snake_case
                queries.push(format!(r"\.{}\s*\(", method_name));
                queries.push(format!(r"self\.{}", method_name));
                queries.push(format!(r"service\.{}", method_name));

                // 如果方法名是 PascalCase，也搜索 snake_case 版本
                if method_name.chars().any(|c| c.is_uppercase()) {
                    let snake_case = Self::pascal_to_snake_case(method_name);
                    queries.push(format!(r"\.{}\s*\(", snake_case));
                    queries.push(format!(r"self\.{}", snake_case));
                }
            }
            Language::Rust => {
                // Rust: 方法名通常是 snake_case
                queries.push(format!(r"\.{}\s*\(", method_name));
                queries.push(format!(r"::{}\s*\(", method_name));
                queries.push(format!(r"self\.{}", method_name));
            }
            Language::Java => {
                // Java: 方法名通常是 camelCase
                queries.push(format!(r"\.{}\s*\(", method_name));
                queries.push(format!(r"this\.{}", method_name));
            }
            _ => {
                // 通用搜索
                queries.push(format!(r"\.{}\s*\(", method_name));
                queries.push(method_name.to_string());
            }
        }

        queries
    }

    /// PascalCase 转 snake_case
    fn pascal_to_snake_case(s: &str) -> String {
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap_or(c));
        }
        result
    }

    /// 判断是否是方法调用（排除方法定义）
    fn is_method_call(content: &str, method_name: &str, language: Language) -> bool {
        // 排除方法定义的关键词
        let definition_keywords = match language {
            Language::TypeScript | Language::JavaScript => {
                vec!["function", "async function", "const", "let", "="]
            }
            Language::Python => {
                vec!["def", "async def"]
            }
            Language::Rust => {
                vec!["fn", "pub fn", "async fn"]
            }
            Language::Java => {
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
        let call_patterns = [
            format!(r"\.{}\s*\(", method_name),
            format!(r"::{}\s*\(", method_name),
            format!(r"this\.{}", method_name),
            format!(r"self\.{}", method_name),
        ];

        call_patterns
            .iter()
            .any(|pattern| Regex::new(pattern).unwrap().is_match(content))
    }

    /// 解析 git grep 输出
    fn parse_git_grep_output(output: &[u8], language: Language) -> Result<Vec<CallSite>> {
        let text = String::from_utf8_lossy(output);
        let mut call_sites = Vec::new();

        for line in text.lines() {
            if let Some((file_path, line_number, content)) = Self::parse_grep_line(line) {
                call_sites.push(CallSite {
                    file_path,
                    line_number,
                    content,
                    language,
                });
            }
        }

        Ok(call_sites)
    }

    /// 解析 ripgrep 输出
    fn parse_ripgrep_output(output: &[u8], language: Language) -> Result<Vec<CallSite>> {
        // ripgrep 输出格式与 git grep 类似
        Self::parse_git_grep_output(output, language)
    }

    /// 解析 grep 输出行：file:line:content
    fn parse_grep_line(line: &str) -> Option<(String, Option<u32>, String)> {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() < 2 {
            return None;
        }

        let file_path = parts[0].to_string();
        let rest = parts[1];

        // 尝试提取行号
        if let Some(colon_pos) = rest.find(':') {
            if let Ok(line_number) = rest[..colon_pos].parse::<u32>() {
                let content = rest[colon_pos + 1..].trim().to_string();
                return Some((file_path, Some(line_number), content));
            }
        }

        // 没有行号
        Some((file_path, None, rest.trim().to_string()))
    }

    /// 在文件中搜索 HTTP 路由定义
    fn find_endpoints_in_file(&self, file_path: &str) -> Result<Vec<EndpointInfo>> {
        let language = Self::detect_language_from_path(file_path);

        // 读取文件内容
        let content = self.read_file_content(file_path)?;

        let mut endpoints = Vec::new();

        match language {
            Language::TypeScript | Language::JavaScript => {
                // Express 和 NestJS 模式
                endpoints.extend(Self::extract_endpoints_typescript(&content, file_path)?);
            }
            Language::Python => {
                // FastAPI, Flask, Django 模式
                endpoints.extend(Self::extract_endpoints_python(&content, file_path)?);
            }
            Language::Rust => {
                // Actix-web, Axum 模式
                endpoints.extend(Self::extract_endpoints_rust(&content, file_path)?);
            }
            Language::Java => {
                // Spring Boot 模式
                endpoints.extend(Self::extract_endpoints_java(&content, file_path)?);
            }
            _ => {
                // 通用搜索：尝试识别常见的 HTTP 路由模式
                endpoints.extend(Self::extract_endpoints_generic(&content, file_path)?);
            }
        }

        Ok(endpoints)
    }

    /// 读取文件内容
    fn read_file_content(&self, file_path: &str) -> Result<String> {
        // 如果使用 GitHub MCP 策略，尝试从 GitHub 读取
        if matches!(self.strategy, SearchStrategy::GitHubMCP) {
            if let (Some(owner), Some(repo)) = (&self.github_owner, &self.github_repo) {
                if let Ok(content) = self.read_file_via_github_mcp(owner, repo, file_path) {
                    return Ok(content);
                }
            }
        }

        // 优先尝试使用 git show 读取（如果文件在 Git 仓库中）
        if Self::is_git_repo() {
            // 尝试从当前分支读取
            if let Ok(output) = Command::new("git")
                .args(["show", &format!("HEAD:{}", file_path)])
                .output()
            {
                if output.status.success() {
                    return Ok(String::from_utf8_lossy(&output.stdout).to_string());
                }
            }

            // 如果失败，尝试从工作目录读取
            if Path::new(file_path).exists() {
                return Ok(fs::read_to_string(file_path)?);
            }
        } else {
            // 直接读取文件
            if Path::new(file_path).exists() {
                return Ok(fs::read_to_string(file_path)?);
            }
        }

        anyhow::bail!("File not found: {}", file_path)
    }

    /// 使用 GitHub API 读取文件内容
    fn read_file_via_github_mcp(&self, owner: &str, repo: &str, file_path: &str) -> Result<String> {
        // 直接使用 GitHub API 获取文件内容
        // GitHub API 文档: https://docs.github.com/en/rest/repos/contents?apiVersion=2022-11-28#get-repository-content
        let api_url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, file_path
        );

        let headers = Self::get_github_headers()?;

        // 发送 GET 请求
        let client = HttpClient::global()?;
        let config = RequestConfig::<Value, Value>::new()
            .headers(&headers)
            .timeout(std::time::Duration::from_secs(30));

        let response = client
            .get(&api_url, config)
            .context("Failed to call GitHub API for file read")?;

        // 检查响应状态
        let http_response = response
            .ensure_success()
            .context("GitHub API returned error response for file read")?;

        // 解析 JSON 响应
        let data: Value = http_response
            .as_json()
            .context("Failed to parse GitHub API response as JSON")?;

        // GitHub API 响应格式: { "name", "path", "sha", "size", "url", "html_url", "git_url", "download_url", "type", "content", "encoding" }
        // content 是 base64 编码的
        if let Some(content_base64) = data.get("content").and_then(|v| v.as_str()) {
            // 移除 base64 字符串中的换行符（GitHub API 会在每 76 个字符后插入换行）
            let content_clean = content_base64.replace('\n', "");

            // 解码 base64
            let content_bytes = base64::engine::general_purpose::STANDARD
                .decode(&content_clean)
                .context("Failed to decode base64 content from GitHub API")?;

            let content = String::from_utf8(content_bytes)
                .context("Failed to convert file content to UTF-8 string")?;

            return Ok(content);
        }

        anyhow::bail!(
            "Unexpected GitHub API response format for file read: missing 'content' field"
        )
    }

    /// 提取 TypeScript/JavaScript 接口定义
    fn extract_endpoints_typescript(content: &str, file_path: &str) -> Result<Vec<EndpointInfo>> {
        let mut endpoints = Vec::new();

        // Express 模式: router.post('/api/focuses', ...) 或 app.post('/api/focuses', ...)
        let express_pattern = Regex::new(
            r#"(?:router|app)\.(get|post|put|delete|patch|options|head)\s*\(\s*['"]([^'"]+)['"]"#,
        )?;

        for cap in express_pattern.captures_iter(content) {
            if let (Some(method), Some(path)) = (cap.get(1), cap.get(2)) {
                let line_number = Self::find_line_number(content, cap.get(0).unwrap().start());
                endpoints.push(EndpointInfo {
                    method: method.as_str().to_uppercase(),
                    path: path.as_str().to_string(),
                    file_path: file_path.to_string(),
                    line_number,
                    language: Language::TypeScript,
                });
            }
        }

        // NestJS 模式: @Post('/api/focuses') 或 @Get('/api/focuses')
        let nestjs_pattern =
            Regex::new(r#"@(Get|Post|Put|Delete|Patch|Options|Head)\(['"]([^'"]+)['"]"#)?;

        for cap in nestjs_pattern.captures_iter(content) {
            if let (Some(method), Some(path)) = (cap.get(1), cap.get(2)) {
                let line_number = Self::find_line_number(content, cap.get(0).unwrap().start());
                endpoints.push(EndpointInfo {
                    method: method.as_str().to_uppercase(),
                    path: path.as_str().to_string(),
                    file_path: file_path.to_string(),
                    line_number,
                    language: Language::TypeScript,
                });
            }
        }

        Ok(endpoints)
    }

    /// 提取 Python 接口定义
    fn extract_endpoints_python(content: &str, file_path: &str) -> Result<Vec<EndpointInfo>> {
        let mut endpoints = Vec::new();

        // 首先提取 APIRouter 的 prefix（如果存在）
        // 模式: router = APIRouter(prefix="/tabs", ...) 或 router = APIRouter(prefix='/tabs', ...)
        let router_prefix_pattern = Regex::new(
            r#"(?:router|app|api_router)\s*=\s*APIRouter\s*\([^)]*prefix\s*=\s*['"]([^'"]+)['"]"#,
        )?;

        let mut prefix = String::new();
        if let Some(cap) = router_prefix_pattern.captures(content) {
            if let Some(prefix_match) = cap.get(1) {
                prefix = prefix_match.as_str().to_string();
            }
        }

        // FastAPI 模式: @router.post("/generate") 或 @app.post("/api/focuses")
        // 支持变量名：router, app, api_router, bp 等
        // 注意：这里使用 \w+ 匹配任意变量名，因为 FastAPI 中 router 变量名可能不同
        let fastapi_pattern =
            Regex::new(r#"@(\w+)\.(get|post|put|delete|patch|options|head)\(['"]([^'"]+)['"]"#)?;

        for cap in fastapi_pattern.captures_iter(content) {
            if let (Some(_router_var), Some(method), Some(path)) =
                (cap.get(1), cap.get(2), cap.get(3))
            {
                let line_number = Self::find_line_number(content, cap.get(0).unwrap().start());

                // 如果找到了 prefix，需要拼接路径
                let full_path = if !prefix.is_empty() {
                    // 确保路径格式正确
                    let path_str = path.as_str();
                    // 移除 prefix 和 path 中可能的前导斜杠，然后拼接
                    let prefix_clean = prefix.trim_start_matches('/');
                    let path_clean = path_str.trim_start_matches('/');
                    if prefix_clean.is_empty() {
                        format!("/{}", path_clean)
                    } else if path_clean.is_empty() {
                        format!("/{}", prefix_clean)
                    } else {
                        format!("/{}/{}", prefix_clean, path_clean)
                    }
                } else {
                    // 如果没有 prefix，确保路径以 / 开头
                    let path_str = path.as_str();
                    if path_str.starts_with('/') {
                        path_str.to_string()
                    } else {
                        format!("/{}", path_str)
                    }
                };

                endpoints.push(EndpointInfo {
                    method: method.as_str().to_uppercase(),
                    path: full_path,
                    file_path: file_path.to_string(),
                    line_number,
                    language: Language::Python,
                });
            }
        }

        // Flask 模式: @app.route('/api/focuses', methods=['POST'])
        // 支持多种格式：
        // - @app.route('/api/focuses', methods=['POST'])
        // - @app.route('/api/focuses', methods=["POST"])
        // - @app.route('/api/focuses', methods=['GET', 'POST'])
        // - @app.route('/api/focuses')  (默认 GET)
        let flask_pattern_with_method = Regex::new(
            r#"@(?:app|bp|router)\.route\(['"]([^'"]+)['"][^)]*methods\s*=\s*\[['"](GET|POST|PUT|DELETE|PATCH|OPTIONS|HEAD)['"]"#,
        )?;

        for cap in flask_pattern_with_method.captures_iter(content) {
            if let (Some(path), Some(method)) = (cap.get(1), cap.get(2)) {
                let line_number = Self::find_line_number(content, cap.get(0).unwrap().start());
                endpoints.push(EndpointInfo {
                    method: method.as_str().to_uppercase(),
                    path: path.as_str().to_string(),
                    file_path: file_path.to_string(),
                    line_number,
                    language: Language::Python,
                });
            }
        }

        // Flask 模式（无 methods 参数，默认 GET）: @app.route('/api/focuses')
        let flask_pattern_default =
            Regex::new(r#"@(?:app|bp|router)\.route\(['"]([^'"]+)['"](?:[^)]*)?\)"#)?;

        for cap in flask_pattern_default.captures_iter(content) {
            if let Some(path) = cap.get(1) {
                // 检查是否已经在上面匹配过（避免重复）
                let line_number = Self::find_line_number(content, cap.get(0).unwrap().start());
                let path_str = path.as_str();

                // 检查是否已经添加过这个路径（避免重复）
                if !endpoints
                    .iter()
                    .any(|e| e.path == path_str && e.line_number == line_number)
                {
                    endpoints.push(EndpointInfo {
                        method: "GET".to_string(), // 默认 GET
                        path: path_str.to_string(),
                        file_path: file_path.to_string(),
                        line_number,
                        language: Language::Python,
                    });
                }
            }
        }

        // Django 模式: path('api/focuses', view)
        // 注意：Django 的 path 通常不包含 HTTP 方法信息，需要从 view 函数推断
        // 这里先提取路径，方法默认为 GET
        let django_pattern = Regex::new(r#"path\(['"]([^'"]+)['"]"#)?;

        for cap in django_pattern.captures_iter(content) {
            if let Some(path) = cap.get(1) {
                let line_number = Self::find_line_number(content, cap.get(0).unwrap().start());
                endpoints.push(EndpointInfo {
                    method: "GET".to_string(), // Django path 默认是 GET，实际方法需要从 view 推断
                    path: path.as_str().to_string(),
                    file_path: file_path.to_string(),
                    line_number,
                    language: Language::Python,
                });
            }
        }

        Ok(endpoints)
    }

    /// 提取 Rust 接口定义
    fn extract_endpoints_rust(content: &str, file_path: &str) -> Result<Vec<EndpointInfo>> {
        let mut endpoints = Vec::new();

        // Actix-web 模式: #[post("/api/focuses")]
        let actix_pattern =
            Regex::new(r#"#\[(get|post|put|delete|patch|options|head)\(['"]([^'"]+)['"]"#)?;

        for cap in actix_pattern.captures_iter(content) {
            if let (Some(method), Some(path)) = (cap.get(1), cap.get(2)) {
                let line_number = Self::find_line_number(content, cap.get(0).unwrap().start());
                endpoints.push(EndpointInfo {
                    method: method.as_str().to_uppercase(),
                    path: path.as_str().to_string(),
                    file_path: file_path.to_string(),
                    line_number,
                    language: Language::Rust,
                });
            }
        }

        // Axum 模式: .route("/api/focuses", post(...)) 或 .route("/api/focuses", get(...))
        let axum_pattern = Regex::new(
            r#"\.route\(['"]([^'"]+)['"].*?(get|post|put|delete|patch|options|head)\s*\("#,
        )?;

        for cap in axum_pattern.captures_iter(content) {
            if let (Some(path), Some(method)) = (cap.get(1), cap.get(2)) {
                let line_number = Self::find_line_number(content, cap.get(0).unwrap().start());
                endpoints.push(EndpointInfo {
                    method: method.as_str().to_uppercase(),
                    path: path.as_str().to_string(),
                    file_path: file_path.to_string(),
                    line_number,
                    language: Language::Rust,
                });
            }
        }

        Ok(endpoints)
    }

    /// 提取 Java 接口定义
    fn extract_endpoints_java(content: &str, file_path: &str) -> Result<Vec<EndpointInfo>> {
        let mut endpoints = Vec::new();

        // Spring Boot 模式: @PostMapping("/api/focuses") 或 @GetMapping("/api/focuses")
        let spring_pattern =
            Regex::new(r#"@(Get|Post|Put|Delete|Patch|Options|Head)Mapping\(['"]([^'"]+)['"]"#)?;

        for cap in spring_pattern.captures_iter(content) {
            if let (Some(method), Some(path)) = (cap.get(1), cap.get(2)) {
                let line_number = Self::find_line_number(content, cap.get(0).unwrap().start());
                endpoints.push(EndpointInfo {
                    method: method.as_str().to_uppercase(),
                    path: path.as_str().to_string(),
                    file_path: file_path.to_string(),
                    line_number,
                    language: Language::Java,
                });
            }
        }

        // JAX-RS 模式: @POST @Path("/api/focuses")
        let jaxrs_pattern =
            Regex::new(r#"@(GET|POST|PUT|DELETE|PATCH|OPTIONS|HEAD)\s+@Path\(['"]([^'"]+)['"]"#)?;

        for cap in jaxrs_pattern.captures_iter(content) {
            if let (Some(method), Some(path)) = (cap.get(1), cap.get(2)) {
                let line_number = Self::find_line_number(content, cap.get(0).unwrap().start());
                endpoints.push(EndpointInfo {
                    method: method.as_str().to_uppercase(),
                    path: path.as_str().to_string(),
                    file_path: file_path.to_string(),
                    line_number,
                    language: Language::Java,
                });
            }
        }

        Ok(endpoints)
    }

    /// 提取通用接口定义（fallback）
    fn extract_endpoints_generic(content: &str, file_path: &str) -> Result<Vec<EndpointInfo>> {
        let mut endpoints = Vec::new();

        // 尝试匹配常见的 HTTP 路由模式
        let patterns = vec![
            // 通用模式: /api/... 路径
            Regex::new(r#"['"](/api/[^'"]+)['"]"#)?,
            // 通用模式: route('/api/...')
            Regex::new(r#"route\(['"]([^'"]+)['"]"#)?,
        ];

        for pattern in patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(path) = cap.get(1) {
                    let path_str = path.as_str();
                    // 只匹配看起来像 API 路径的
                    if path_str.starts_with("/api/") || path_str.starts_with("api/") {
                        let line_number =
                            Self::find_line_number(content, cap.get(0).unwrap().start());
                        endpoints.push(EndpointInfo {
                            method: "GET".to_string(), // 默认方法
                            path: path_str.to_string(),
                            file_path: file_path.to_string(),
                            line_number,
                            language: Language::Unknown,
                        });
                    }
                }
            }
        }

        Ok(endpoints)
    }

    /// 查找字符串在内容中的行号
    fn find_line_number(content: &str, byte_pos: usize) -> Option<u32> {
        let before = &content[..byte_pos.min(content.len())];
        let line_number = before.matches('\n').count() + 1;
        Some(line_number as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert_eq!(
            CodebaseSearcher::detect_language_from_path("test.ts"),
            Language::TypeScript
        );
        assert_eq!(
            CodebaseSearcher::detect_language_from_path("test.py"),
            Language::Python
        );
        assert_eq!(
            CodebaseSearcher::detect_language_from_path("test.rs"),
            Language::Rust
        );
    }

    #[test]
    fn test_pascal_to_snake_case() {
        assert_eq!(
            CodebaseSearcher::pascal_to_snake_case("CreateFocus"),
            "create_focus"
        );
        assert_eq!(
            CodebaseSearcher::pascal_to_snake_case("CerebrasService"),
            "cerebras_service"
        );
    }

    #[test]
    fn test_is_service_file() {
        assert!(CodebaseSearcher::is_service_file(
            "services/CerebrasService.ts"
        ));
        assert!(CodebaseSearcher::is_service_file(
            "src/services/user_service.py"
        ));
        assert!(!CodebaseSearcher::is_service_file(
            "controllers/UserController.ts"
        ));
    }
}
