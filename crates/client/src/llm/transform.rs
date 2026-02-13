use crate::LLMError;

pub trait LLMResponseTransform {
    fn transform(&self, response: &str) -> Result<String, LLMError>;
}
