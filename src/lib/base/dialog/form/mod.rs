//! 表单构建器模块
//!
//! 提供 FormBuilder，支持 Group 概念，可以将整个 setup 的所有内容封装为一个 form。
//!
//! ## 特性
//!
//! - **Group 支持**：可以将表单组织成多个组（Group）
//! - **步骤化构建**：每个组内可以包含多个步骤（Step）
//! - **字段构建**：每个步骤可以包含多个字段（Field）
//! - **条件逻辑**：支持多种条件类型（step_if, step_if_all, step_if_any, step_if_dynamic）
//! - **可选组**：支持可选组，可以询问用户是否配置
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use workflow::base::dialog::{FormBuilder, GroupConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let form_result = FormBuilder::new()
//!     // Group 1: Jira 配置（必填）
//!     .add_group("jira", |g| {
//!         g.step(|f| {
//!             f.add_text("jira_email", "Jira email address").required()
//!         })
//!         .step(|f| {
//!             f.add_text("jira_service_address", "Jira service address").required()
//!         })
//!     }, GroupConfig::required())
//!     // Group 2: LLM 配置（可选）
//!     .add_group("llm", |g| {
//!         g.step(|f| {
//!             f.add_selection("llm_provider", "Select LLM provider", vec!["openai".into(), "deepseek".into()])
//!         })
//!         .step_if("llm_provider", "openai", |f| {
//!             f.add_text("openai_key", "OpenAI API key")
//!         })
//!     }, GroupConfig::optional()
//!         .with_title("LLM/AI Configuration")
//!         .with_default_enabled(true))
//!     .run()?;
//! # Ok(())
//! # }
//! ```

mod builder;
mod condition_evaluator;
mod field_builder;
mod group_builder;
mod types;

pub use builder::FormBuilder;
pub use types::{
    Condition, ConditionOperator, ConditionValue, FieldDefaultValue, FormField, FormFieldType,
    FormGroup, FormResult, FormStep, GroupConfig, StepType,
};
