//! 表单模型 Trait
//!
//! 提供基于模型的表单构建和解析能力。

use crate::form::{FormBuilder, FormResult};
use crate::Result;

/// 表单模型 Trait
///
/// 实现此 Trait 的类型可以自动生成表单并从表单结果中解析出实例。
///
/// # 示例
///
/// ```no_run
/// use prompt::{FormBuilder, FormModel, FormResult, InputFormField};
/// use prompt::Result;
///
/// #[derive(Default)]
/// struct UserConfig {
///     username: String,
/// }
///
/// impl FormModel for UserConfig {
///     fn build_form(&self) -> FormBuilder {
///         FormBuilder::new()
///             .add_input(
///                 InputFormField::new("username", "请输入用户名")
///                     .default(self.username.clone())
///             )
///     }
///
///     fn build_result(&self, result: FormResult) -> Result<Self> {
///         Ok(Self {
///             username: result.get_string("username"),
///         })
///     }
/// }
///
/// // 使用
/// // let config = UserConfig::default().run()?;
/// ```
pub trait FormModel: Sized {
    /// 构建表单定义（基于当前实例）
    fn build_form(&self) -> FormBuilder;

    /// 从表单结果解析模型（基于当前实例）
    ///
    /// 可以使用 `self` 中的值来填充未在表单中出现的字段。
    fn build_result(&self, result: FormResult) -> Result<Self>;

    /// 运行表单并获取结果
    fn run(&self) -> Result<Self> {
        let builder = self.build_form();
        let result = builder.run()?;
        self.build_result(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::form::InputFormField;

    #[derive(Default)]
    struct TestConfig {
        name: String,
        description: String, // 这是一个不在表单中的字段
    }

    impl TestConfig {
        fn new(name: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                description: "default desc".to_string(),
            }
        }
    }

    impl FormModel for TestConfig {
        fn build_form(&self) -> FormBuilder {
            FormBuilder::new()
                .add_input(InputFormField::new("name", "Name").default(self.name.clone()))
        }

        fn build_result(&self, result: FormResult) -> Result<Self> {
            Ok(Self {
                name: result.get_string("name"),
                description: self.description.clone(), // 保留原值
            })
        }
    }

    #[test]
    fn test_form_model_compilation() {
        let config = TestConfig::new("initial");
        let _builder = config.build_form();
        let _new_config = config.run();
    }
}
