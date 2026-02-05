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

    /// 运行表单并获取结果（使用默认终端后端）
    fn run(&self) -> Result<Self> {
        let builder = self.build_form();
        let result = builder.run()?;
        self.build_result(result)
    }
}

/// 表单模型测试扩展 Trait（仅在测试时可用）
///
/// 提供 `run_with_backend` 方法，允许使用 mock 后端进行测试。
///
/// 此 trait 供外部 crate 测试使用，在本 crate 内仅用于 tests 模块。
#[cfg(any(test, feature = "testing"))]
#[allow(dead_code)]
pub trait FormModelTestExt: FormModel {
    /// 使用指定后端运行表单并获取结果
    fn run_with_backend<B: crate::backend::Backend>(&self, backend: &mut B) -> Result<Self>;
}

#[cfg(any(test, feature = "testing"))]
impl<T: FormModel> FormModelTestExt for T {
    fn run_with_backend<B: crate::backend::Backend>(&self, backend: &mut B) -> Result<Self> {
        let builder = self.build_form();
        let result = builder.run_with_backend(backend)?;
        self.build_result(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::MockBackend;
    use crate::form::InputFormField;

    #[derive(Default)]
    struct TestConfig {
        name: String,
        description: String,
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
                description: self.description.clone(),
            })
        }
    }

    #[test]
    fn test_form_model_build_form() {
        let config = TestConfig::new("initial");
        let builder = config.build_form();
        assert!(!builder.get_fields().is_empty());
    }

    #[test]
    fn test_form_model_build_result() {
        let config = TestConfig::new("initial");
        let mut result = FormResult::new();
        result.set("name".to_string(), "new_name".to_string());

        let new_config = config.build_result(result).unwrap();
        assert_eq!(new_config.name, "new_name");
        assert_eq!(new_config.description, "default desc");
    }

    #[test]
    fn test_form_model_with_mock_backend() {
        let events = [
            MockBackend::type_string("test_name"),
            vec![MockBackend::press_enter()],
        ]
        .concat();

        let mut backend = MockBackend::with_events(events);
        let config = TestConfig::new("initial");

        let result = config.run_with_backend(&mut backend).unwrap();
        assert_eq!(result.name, "test_name");
        assert_eq!(result.description, "default desc");
    }
}
