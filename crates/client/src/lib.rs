//! 客户端定义层（Client Definition Layer）
//!
//! 包含外部 API 客户端的 trait 定义、类型和错误定义。
//! 实现层（如 infra）提供具体实现，实现依赖倒置与可替换实现。
//!
//! ## 计划中的模块
//!
//! - HTTP: `HttpClient` trait
//! - LLM: `LLMClient` trait
//! - GitHub: `GitHubClient` trait
//! - Jira: `JiraClient` trait
//! - Codeup: `CodeupClient` trait

mod codeup;
mod github;
mod http;
mod jira;
mod llm;

pub use codeup::{
    CodeupClient, CodeupClientError, CodeupConfigContext, CodeupErrorResponse,
    CodeupPullRequestListResponse, CodeupRequest, CodeupResponse, CreateCodeupPullRequestRequest,
    CreateCodeupPullRequestResponse,
};
pub use github::{
    GitHubClient, GitHubClientError, GitHubConfigContext, GitHubErrorResource, GitHubErrorResponse,
    GitHubRequest, GitHubResponse,
};
pub use http::{
    Authorization, ErrorContext, HttpClient, HttpClientConfig, HttpClientExt, HttpClientHolder,
    HttpError, HttpMethod, HttpRequest, HttpResponse, MultipartPart, MultipartRequest,
    RequestBuilder,
};
pub use jira::{JiraClient, JiraClientError, JiraConfigContext, JiraRequest, JiraResponse};
pub use llm::{
    ChatCompletionChoice, ChatCompletionResponse, ChatMessage, IntoLLMConfig,
    IntoLLMRequestParameters, LLMClient, LLMConfigContext, LLMConversation, LLMError,
    LLMRequestParameters, LanguageManager, SupportedLanguage, Usage,
};
