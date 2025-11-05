use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::translator::{generate_branch_name_with_llm, should_translate, translate_with_llm};
use crate::Jira;

/// LLM 工具模块 - 用于生成 PR 标题和分支名
pub struct LLM;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueDesc {
    pub issue_desc: String,
    pub need_translate: bool,
    pub translated_desc: Option<String>,
}

impl LLM {
    /// 从 Jira ticket 获取描述并生成 PR 标题
    ///
    /// 返回格式与 aiwflow 兼容：
    /// ```
    /// {
    ///   "issue_desc": "...",
    ///   "need_translate": true/false,
    ///   "translated_desc": "..." // 如果需要翻译
    /// }
    /// ```
    pub fn get_issue_desc(jira_ticket: &str) -> Result<IssueDesc> {
        // 1. 从 Jira 获取 issue summary（使用 REST API）
        let summary = Jira::get_summary(jira_ticket)?;

        // 2. 检查是否需要翻译
        let need_translate = should_translate(&summary);

        // 3. 如果需要翻译，使用 LLM 翻译
        let translated_desc = if need_translate {
            Some(translate_with_llm(&summary)?)
        } else {
            None
        };

        Ok(IssueDesc {
            issue_desc: summary,
            need_translate,
            translated_desc,
        })
    }

    /// 根据 commit_title 使用 LLM 生成分支名
    ///
    /// 生成的分支名应该：
    /// - 全小写
    /// - 使用连字符分隔单词
    /// - 符合 Git 分支命名规范
    /// - 简洁明了
    pub fn generate_branch_name(commit_title: &str) -> Result<String> {
        generate_branch_name_with_llm(commit_title)
    }
}
