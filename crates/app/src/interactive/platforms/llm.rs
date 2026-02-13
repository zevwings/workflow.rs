//! LLM 工作流阶段 (v2)

use std::error::Error;

use domain::{GlobalConfig, LLMSettings, VerificationService};
use prompt::{
    br, confirm, info, separator, FormBuilder, FormResult, InputFormField, PasswordFormField,
    PromptError, SelectBuilder, SelectFormField,
};
use toolkit::Sensitive;

use crate::bootstrap::get_language_manager;
use crate::interactive::{
    core::{
        context::{WorkflowContext, WorkflowMode},
        stage::WorkflowStage,
    },
    display::VerificationResultFormatter,
};

/// LLM 工作流阶段
pub struct LlmStage;

impl LlmStage {
    fn run_form(settings: &mut GlobalConfig) -> Result<(), Box<dyn Error>> {
        info!("配置 LLM 提供商和 API 密钥。留空字段以保留默认值或跳过。");
        br!();

        let llm = &mut settings.llm;
        let has_llm = !llm.is_empty();

        let language_manager = get_language_manager();
        let language_codes = language_manager.get_supported_codes();
        let language_options = language_manager.get_supported_display_names();
        let (language_prompt, default_language_index) =
            build_language_prompt(&llm.language, language_codes.as_slice());

        let provider_options: Vec<String> =
            LlmProvider::options().iter().map(|option| option.to_string()).collect();
        let default_provider = LlmProvider::from_str(&llm.provider).unwrap_or_default();
        let provider_prompt = if has_llm {
            format!("请选择您的 LLM 提供商 [当前: {}]", llm.provider)
        } else {
            "请选择您的 LLM 提供商（必填）".to_string()
        };

        let provider_value = SelectBuilder::new(provider_prompt, provider_options.clone())
            .default(default_provider.index())
            .result_title("您的 LLM 提供商")
            .prompt()
            .map_err(|e: PromptError| Box::new(e) as Box<dyn Error>)?;

        let provider = LlmProvider::from_str(provider_value.as_str()).unwrap_or(default_provider);

        let provider_input = match provider {
            LlmProvider::OpenAi => configure_openai(
                llm.openai.key.clone().unwrap_or_default(),
                llm.openai.model.clone().unwrap_or_default(),
                &language_prompt,
                &language_options,
                default_language_index,
                language_codes.as_slice(),
            )?,
            LlmProvider::DeepSeek => configure_deepseek(
                llm.deepseek.key.clone().unwrap_or_default(),
                llm.deepseek.model.clone().unwrap_or_default(),
                &language_prompt,
                &language_options,
                default_language_index,
                language_codes.as_slice(),
            )?,
            LlmProvider::Proxy => configure_proxy(
                llm.proxy.url.clone().unwrap_or_default(),
                llm.proxy.key.clone().unwrap_or_default(),
                llm.proxy.model.clone().unwrap_or_default(),
                &language_prompt,
                &language_options,
                default_language_index,
                language_codes.as_slice(),
            )?,
        };

        llm.provider = provider.as_str().to_string();
        if !provider_input.language.trim().is_empty() {
            llm.language = provider_input.language.trim().to_string();
        }

        if !provider_input.model.trim().is_empty()
            || !provider_input.api_key.trim().is_empty()
            || !provider_input.proxy_url.trim().is_empty()
        {
            let provider_settings = llm.current_provider_mut();
            if !provider_input.model.trim().is_empty() {
                provider_settings.model = Some(provider_input.model.trim().to_string());
            }
            if !provider_input.api_key.trim().is_empty() {
                provider_settings.key = Some(provider_input.api_key);
            }
            if !provider_input.proxy_url.trim().is_empty() {
                provider_settings.url = Some(provider_input.proxy_url.trim().to_string());
            }
        }

        Ok(())
    }
}

impl WorkflowStage for LlmStage {
    fn stage_name(&self) -> &'static str {
        "LLM"
    }

    fn configure(&self, context: &mut WorkflowContext) -> Result<(), Box<dyn Error>> {
        let mode = context.mode();
        let settings = context.settings_mut();

        separator!('─', 80, "LLM 配置");
        br!();

        let llm = &settings.llm;
        let has_llm = !llm.is_empty();

        if has_llm {
            info!("检测到 LLM 配置！");
            info!("  - 提供商: {}", llm.provider);
            if let Some(model) = llm.current_provider().model.as_ref() {
                info!("  - 模型: {}", model);
            }
            if let Some(key) = llm.current_provider().key.as_ref() {
                info!("  - 密钥: {}", key.mask());
            }
            info!("  - 语言: {}", llm.language);
            br!();
        }

        // 处理模式特定的交互
        if mode == WorkflowMode::Setup {
            if has_llm {
                let keep = confirm!(
                    "检测到现有 LLM 配置（提供商: {}）。是否保留当前值？",
                    llm.provider
                )
                .default(true)
                .result_title("保留 LLM 配置")
                .prompt()
                .map_err(|e: PromptError| Box::new(e) as Box<dyn Error>)?;

                if keep {
                    return Ok(());
                }
            } else {
                let configure = confirm!("是否配置 LLM？")
                    .default(false)
                    .result_title("配置 LLM")
                    .prompt()
                    .map_err(|e: PromptError| Box::new(e) as Box<dyn Error>)?;

                if !configure {
                    return Ok(());
                }
            }
        }

        Self::run_form(settings)?;

        // 显示最终配置摘要
        br!();
        separator!('─', 80, "LLM 配置摘要");
        br!();
        let llm = &settings.llm;
        info!("提供商: {}", llm.provider);
        if let Some(model) = llm.current_provider().model.as_ref() {
            info!("模型: {}", model);
        }
        if let Some(key) = llm.current_provider().key.as_ref() {
            info!("API 密钥: {}", key.mask());
        }
        if let Some(url) = llm.current_provider().url.as_ref() {
            info!("代理 URL: {}", url);
        }
        info!("语言: {}", llm.language);
        br!();

        Ok(())
    }

    fn is_configured(&self, settings: &GlobalConfig) -> bool {
        !settings.llm.is_empty()
    }

    fn verify(
        &self,
        service: &dyn VerificationService,
    ) -> Result<Box<dyn VerificationResultFormatter>, Box<dyn Error>> {
        service
            .verify_llm_config()
            .map(|r| Box::new(r) as Box<dyn VerificationResultFormatter>)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

/// 获取 LLM 阶段实例
pub fn llm_stage() -> &'static dyn WorkflowStage {
    &LlmStage
}

#[derive(Debug, Clone)]
struct ProviderFormResult {
    api_key: String,
    model: String,
    proxy_url: String,
    language: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
enum LlmProvider {
    #[default]
    OpenAi,
    DeepSeek,
    Proxy,
}

impl LlmProvider {
    const VARIANTS: [(Self, &'static str); 3] = [
        (Self::OpenAi, "openai"),
        (Self::DeepSeek, "deepseek"),
        (Self::Proxy, "proxy"),
    ];

    fn options() -> &'static [&'static str] {
        const OPTIONS: [&str; 3] = ["openai", "deepseek", "proxy"];
        &OPTIONS
    }

    fn as_str(self) -> &'static str {
        Self::VARIANTS[self.index()].1
    }

    fn index(self) -> usize {
        match self {
            Self::OpenAi => 0,
            Self::DeepSeek => 1,
            Self::Proxy => 2,
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        Self::VARIANTS.iter().find(|(_, s)| *s == value).map(|(variant, _)| *variant)
    }
}

fn configure_openai(
    default_key: String,
    default_model: String,
    language_prompt: &str,
    language_options: &[String],
    default_language_index: usize,
    language_codes: &[&'static str],
) -> Result<ProviderFormResult, String> {
    let builder = FormBuilder::new()
        .with_title("OpenAI 配置")
        .add_password(
            PasswordFormField::new("api_key", "Please enter your OpenAI API key")
                .default(default_key)
                .result_title("Your OpenAI API key")
                .required(),
        )
        .add_input(
            InputFormField::new("model", "Please enter your OpenAI model")
                .default(default_model)
                .result_title("Your OpenAI model")
                .required(),
        )
        .add_select(build_language_field(
            language_prompt,
            language_options,
            default_language_index,
        ));

    let result = builder.run().map_err(|e| e.to_string())?;
    Ok(ProviderFormResult {
        api_key: result.get_string("api_key"),
        model: result.get_string("model"),
        proxy_url: String::new(),
        language: extract_language(&result, language_codes, default_language_index),
    })
}

fn configure_deepseek(
    default_key: String,
    default_model: String,
    language_prompt: &str,
    language_options: &[String],
    default_language_index: usize,
    language_codes: &[&'static str],
) -> Result<ProviderFormResult, String> {
    let builder = FormBuilder::new()
        .with_title("DeepSeek 配置")
        .add_password(
            PasswordFormField::new("api_key", "Please enter your DeepSeek API key")
                .default(default_key)
                .result_title("Your DeepSeek API key"),
        )
        .add_input(
            InputFormField::new("model", "Please enter your DeepSeek model")
                .default(default_model)
                .result_title("Your DeepSeek model")
                .required(),
        )
        .add_select(build_language_field(
            language_prompt,
            language_options,
            default_language_index,
        ));

    let result = builder.run().map_err(|e| e.to_string())?;
    Ok(ProviderFormResult {
        api_key: result.get_string("api_key"),
        model: result.get_string("model"),
        proxy_url: String::new(),
        language: extract_language(&result, language_codes, default_language_index),
    })
}

fn configure_proxy(
    default_url: String,
    default_key: String,
    default_model: String,
    language_prompt: &str,
    language_options: &[String],
    default_language_index: usize,
    language_codes: &[&'static str],
) -> Result<ProviderFormResult, String> {
    let builder = FormBuilder::new()
        .with_title("自定义提供商（代理）配置")
        .add_input(
            InputFormField::new("url", "Please enter your LLM proxy URL")
                .default(default_url)
                .result_title("Your LLM proxy URL")
                .required(),
        )
        .add_password(
            PasswordFormField::new("api_key", "Please enter your LLM proxy key")
                .default(default_key)
                .result_title("Your LLM proxy key")
                .required(),
        )
        .add_input(
            InputFormField::new("model", "Please enter your LLM model")
                .default(default_model)
                .result_title("Your LLM model")
                .required(),
        )
        .add_select(build_language_field(
            language_prompt,
            language_options,
            default_language_index,
        ));

    let result = builder.run().map_err(|e| e.to_string())?;
    Ok(ProviderFormResult {
        api_key: result.get_string("api_key"),
        model: result.get_string("model"),
        proxy_url: result.get_string("url"),
        language: extract_language(&result, language_codes, default_language_index),
    })
}

fn build_language_field(prompt: &str, options: &[String], default_index: usize) -> SelectFormField {
    SelectFormField::new("language", prompt.to_string(), options.to_vec())
        .default(default_index)
        .result_title("Your output language")
}

fn build_language_prompt(
    current_language: &str,
    language_codes: &[&'static str],
) -> (String, usize) {
    let default_language_index =
        language_codes.iter().position(|code| code == &current_language).unwrap_or(0);

    let prompt = if current_language.is_empty() {
        "Please select your output language".to_string()
    } else {
        format!(
            "Please select your output language [current: {}]",
            current_language
        )
    };

    (prompt, default_language_index)
}

fn extract_language(
    result: &FormResult,
    language_codes: &[&'static str],
    default_index: usize,
) -> String {
    let selected = result.get_int("language");
    let index = selected;
    language_codes.get(index).map(|code| code.to_string()).unwrap_or_else(|| {
        language_codes
            .get(default_index)
            .map(|code| code.to_string())
            .unwrap_or_else(LLMSettings::default_language)
    })
}
