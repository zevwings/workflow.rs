use std::sync::Arc;

use di::{bind, Container, InjectionError, Scope};

use client::{
    CodeupClient, CodeupConfigContext, GitHubClient, GitHubConfigContext, HttpClient, JiraClient,
    JiraConfigContext, LLMClient, LLMConfigContext, LanguageManager,
};

use crate::codeup::CodeupClientImpl;
use crate::github::GitHubClientImpl;
use crate::http::ReqwestHttpClient;
use crate::jira::JiraClientImpl;
use crate::llm::{LLMClientImpl, LanguageManagerImpl};

pub fn register_client() -> Result<(), InjectionError> {
    bind!(dyn HttpClient, |_c: &Container| {
        let client = ReqwestHttpClient::new()
            .map_err(|e| InjectionError::CreateInstanceFailed(e.to_string()))?;
        Ok(Arc::new(client))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn LanguageManager, |_c: &Container| {
        Ok(Arc::new(LanguageManagerImpl::new()))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn LLMClient, |c: &Container| {
        let context = c.get::<dyn LLMConfigContext>()?;
        let client = c.get::<dyn HttpClient>()?;
        Ok(Arc::new(LLMClientImpl::new(client, context)))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn GitHubClient, |c: &Container| {
        let context = c.get::<dyn GitHubConfigContext>()?;
        let client = c.get::<dyn HttpClient>()?;
        Ok(Arc::new(GitHubClientImpl::new(client, context)))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn JiraClient, |c: &Container| {
        let context = c.get::<dyn JiraConfigContext>()?;
        let client = c.get::<dyn HttpClient>()?;
        Ok(Arc::new(JiraClientImpl::new(client, context)))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn CodeupClient, |c: &Container| {
        let context = c.get::<dyn CodeupConfigContext>()?;
        let client = c.get::<dyn HttpClient>()?;
        Ok(Arc::new(CodeupClientImpl::new(client, context)))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
