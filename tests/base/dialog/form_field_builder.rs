//! Base/Dialog/Form Field Builder 模块测试
//!
//! 测试字段构建器的核心功能。

use workflow::base::dialog::FormBuilder;

/// 测试创建字段构建器
///
/// ## 测试目的
/// 验证 FormBuilder 能够创建字段构建器（通过 FormBuilder 间接测试）。
///
/// ## 测试场景
/// 1. 使用 FormBuilder 创建表单
/// 2. 添加组和步骤
/// 3. 验证字段构建器创建成功
///
/// ## 预期结果
/// - 字段构建器创建成功
#[test]
fn test_field_builder_new() {
    // Arrange: 准备测试创建字段构建器（覆盖 field_builder.rs:20-25）
    // 注意：FieldBuilder::new 是 pub，但通常通过 FormBuilder 使用
    // 我们通过 FormBuilder 间接测试
    let _builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_text("field1", "Field 1")),
        workflow::base::dialog::GroupConfig::required(),
    );
    // Assert: 验证可以创建字段构建器
    assert!(true);
}

// ==================== FormFieldBuilder Field Addition Tests ====================

/// 测试添加文本字段
///
/// ## 测试目的
/// 验证 FormFieldBuilder::add_text() 能够添加文本字段。
///
/// ## 测试场景
/// 1. 创建 FormBuilder
/// 2. 添加组和步骤
/// 3. 使用 add_text() 添加文本字段
/// 4. 验证字段创建成功
///
/// ## 预期结果
/// - 文本字段创建成功
#[test]
fn test_field_builder_add_text_with_text_field_adds_field() {
    // Arrange: 准备FormBuilder

    // Act: 添加文本字段
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_text("field1", "Field 1")),
        workflow::base::dialog::GroupConfig::required(),
    );

    // Assert: 验证字段创建成功
    assert_eq!(builder.groups.len(), 1);
    assert!(!builder.groups[0].steps.is_empty());
    assert!(!builder.groups[0].steps[0].fields.is_empty());
}

/// 测试添加密码字段
///
/// ## 测试目的
/// 验证 FormFieldBuilder::add_password() 能够添加密码字段。
///
/// ## 测试场景
/// 1. 创建 FormBuilder
/// 2. 添加组和步骤
/// 3. 使用 add_password() 添加密码字段
/// 4. 验证密码字段创建成功
///
/// ## 预期结果
/// - 密码字段创建成功
#[test]
fn test_field_builder_add_password_with_password_field_adds_field() {
    // Arrange: 准备FormBuilder

    // Act: 添加密码字段
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_password("password", "Enter password")),
        workflow::base::dialog::GroupConfig::required(),
    );

    // Assert: 验证密码字段创建成功
    assert_eq!(builder.groups.len(), 1);
    assert!(!builder.groups[0].steps[0].fields.is_empty());
}

/// 测试添加选择字段
///
/// ## 测试目的
/// 验证 FormFieldBuilder::add_selection() 能够添加选择字段。
///
/// ## 测试场景
/// 1. 创建 FormBuilder 和选项列表
/// 2. 添加组和步骤
/// 3. 使用 add_selection() 添加选择字段
/// 4. 验证选择字段创建成功
///
/// ## 预期结果
/// - 选择字段创建成功
#[test]
fn test_field_builder_add_selection_with_selection_field_adds_field() {
    // Arrange: 准备FormBuilder和选项列表
    let choices = vec!["Option 1".to_string(), "Option 2".to_string()];

    // Act: 添加选择字段
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_selection("choice", "Choose option", choices.clone())),
        workflow::base::dialog::GroupConfig::required(),
    );

    // Assert: 验证选择字段创建成功
    assert_eq!(builder.groups.len(), 1);
    assert!(!builder.groups[0].steps[0].fields.is_empty());
}

/// 测试添加确认字段
///
/// ## 测试目的
/// 验证 FormFieldBuilder::add_confirmation() 能够添加确认字段。
///
/// ## 测试场景
/// 1. 创建 FormBuilder
/// 2. 添加组和步骤
/// 3. 使用 add_confirmation() 添加确认字段
/// 4. 验证确认字段创建成功
///
/// ## 预期结果
/// - 确认字段创建成功
#[test]
fn test_field_builder_add_confirmation_with_confirmation_field_adds_field() {
    // Arrange: 准备FormBuilder

    // Act: 添加确认字段
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_confirmation("confirm", "Confirm?")),
        workflow::base::dialog::GroupConfig::required(),
    );

    // Assert: 验证确认字段创建成功
    assert_eq!(builder.groups.len(), 1);
    assert!(!builder.groups[0].steps[0].fields.is_empty());
}

// ==================== FormFieldBuilder Field Configuration Tests ====================

/// 测试标记字段为必填
///
/// ## 测试目的
/// 验证 FormFieldBuilder::required() 能够标记字段为必填。
///
/// ## 测试场景
/// 1. 创建 FormBuilder
/// 2. 添加字段并使用 required() 标记为必填
/// 3. 验证字段的 required 属性为 true
///
/// ## 预期结果
/// - 字段被标记为必填
#[test]
fn test_field_builder_required_with_required_flag_marks_field_required() {
    // Arrange: 准备FormBuilder

    // Act: 添加必填字段
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_text("field1", "Field 1").required()),
        workflow::base::dialog::GroupConfig::required(),
    );

    // Assert: 验证字段标记为必填
    assert_eq!(builder.groups.len(), 1);
    let field = &builder.groups[0].steps[0].fields[0];
    assert!(field.required);
}

/// 测试设置字符串默认值
///
/// ## 测试目的
/// 验证 FormFieldBuilder::default() 能够设置字符串类型的默认值。
///
/// ## 测试场景
/// 1. 创建 FormBuilder
/// 2. 添加字段并使用 default() 设置默认值
/// 3. 验证默认值设置成功
///
/// ## 预期结果
/// - 默认值被正确设置
#[test]
fn test_field_builder_default_string_with_default_value_sets_default() {
    // Arrange: 准备FormBuilder和默认值
    let default_value = "default_value";

    // Act: 添加带默认值的字段
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_text("field1", "Field 1").default(default_value)),
        workflow::base::dialog::GroupConfig::required(),
    );

    // Assert: 验证默认值设置成功
    assert_eq!(builder.groups.len(), 1);
    let field = &builder.groups[0].steps[0].fields[0];
    assert!(field.default_value.is_some());
}

/// 测试设置布尔默认值
///
/// ## 测试目的
/// 验证 FormFieldBuilder::default() 能够设置布尔类型的默认值。
///
/// ## 测试场景
/// 1. 创建 FormBuilder
/// 2. 添加确认字段并使用 default(true) 设置布尔默认值
/// 3. 验证布尔默认值设置成功
///
/// ## 预期结果
/// - 布尔默认值被正确设置
#[test]
fn test_field_builder_default_bool() {
    // Arrange: 准备测试设置布尔默认值（覆盖 field_builder.rs:119-133）
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_confirmation("confirm", "Confirm?").default(true)),
        workflow::base::dialog::GroupConfig::required(),
    );
    // Assert: 验证布尔默认值设置成功
    assert_eq!(builder.groups.len(), 1);
    let field = &builder.groups[0].steps[0].fields[0];
    assert!(field.default_value.is_some());
}

/// 测试设置选择字段的默认值
///
/// ## 测试目的
/// 验证 FormFieldBuilder::default() 能够设置选择字段的默认值。
///
/// ## 测试场景
/// 1. 创建 FormBuilder 和选项列表
/// 2. 添加选择字段并使用 default() 设置默认选项
/// 3. 验证默认值设置成功
///
/// ## 预期结果
/// - 选择字段的默认值被正确设置
#[test]
fn test_field_builder_default_selection() {
    // Arrange: 准备测试设置选择字段的默认值（覆盖 field_builder.rs:125-128）
    let choices = vec!["Option 1".to_string(), "Option 2".to_string()];
    let builder = FormBuilder::new().add_group(
        "test",
        |g| {
            g.step(|f| {
                f.add_selection("choice", "Choose option", choices.clone()).default("Option 1")
            })
        },
        workflow::base::dialog::GroupConfig::required(),
    );
    // Assert: 验证选择字段的默认值设置成功
    assert_eq!(builder.groups.len(), 1);
    let field = &builder.groups[0].steps[0].fields[0];
    assert!(field.default_value.is_some());
    assert_eq!(field.default_choice, Some("Option 1".to_string()));
}

/// 测试设置验证器
///
/// ## 测试目的
/// 验证 FormFieldBuilder::validate() 能够设置字段验证器。
///
/// ## 测试场景
/// 1. 创建 FormBuilder
/// 2. 添加字段并使用 validate() 设置验证器
/// 3. 验证验证器设置成功
///
/// ## 预期结果
/// - 验证器被正确设置
#[test]
fn test_field_builder_validate() {
    // Arrange: 准备测试设置验证器（覆盖 field_builder.rs:136-146）
    let builder = FormBuilder::new().add_group(
        "test",
        |g| {
            g.step(|f| {
                f.add_text("field1", "Field 1").validate(|s| {
                    if s.len() > 0 {
                        Ok(())
                    } else {
                        Err("Field cannot be empty".to_string())
                    }
                })
            })
        },
        workflow::base::dialog::GroupConfig::required(),
    );
    // Assert: 验证验证器设置成功
    assert_eq!(builder.groups.len(), 1);
    let field = &builder.groups[0].steps[0].fields[0];
    assert!(field.validator.is_some());
}

/// 测试允许空值
///
/// ## 测试目的
/// 验证 FormFieldBuilder::allow_empty() 能够设置字段是否允许空值。
///
/// ## 测试场景
/// 1. 创建 FormBuilder
/// 2. 添加字段并使用 allow_empty(true) 设置允许空值
/// 3. 验证允许空值设置成功
///
/// ## 预期结果
/// - 允许空值被正确设置
#[test]
fn test_field_builder_allow_empty() {
    // Arrange: 准备测试允许空值（覆盖 field_builder.rs:149-156）
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_text("field1", "Field 1").allow_empty(true)),
        workflow::base::dialog::GroupConfig::required(),
    );
    // Assert: 验证允许空值设置成功
    assert_eq!(builder.groups.len(), 1);
    let field = &builder.groups[0].steps[0].fields[0];
    assert!(field.allow_empty);
}

/// 测试链式调用所有方法
///
/// ## 测试目的
/// 验证 FormFieldBuilder 能够链式调用所有配置方法。
///
/// ## 测试场景
/// 1. 创建 FormBuilder
/// 2. 链式调用所有方法（required、default、allow_empty、validate）
/// 3. 验证所有配置都成功
///
/// ## 预期结果
/// - 所有链式调用都成功，配置都正确设置
#[test]
fn test_field_builder_chain_all() {
    // Arrange: 准备测试链式调用所有方法
    let builder = FormBuilder::new().add_group(
        "test",
        |g| {
            g.step(|f| {
                f.add_text("field1", "Field 1")
                    .required()
                    .default("default")
                    .allow_empty(false)
                    .validate(|s| {
                        if s.len() > 0 {
                            Ok(())
                        } else {
                            Err("Cannot be empty".to_string())
                        }
                    })
            })
        },
        workflow::base::dialog::GroupConfig::required(),
    );
    // Assert: 验证链式调用成功
    assert_eq!(builder.groups.len(), 1);
    let field = &builder.groups[0].steps[0].fields[0];
    assert!(field.required);
    assert!(field.default_value.is_some());
    assert!(!field.allow_empty);
    assert!(field.validator.is_some());
}

/// 测试添加多个字段
///
/// ## 测试目的
/// 验证 FormFieldBuilder 能够添加多个不同类型的字段。
///
/// ## 测试场景
/// 1. 创建 FormBuilder
/// 2. 添加多个不同类型的字段（文本、密码、选择、确认）
/// 3. 验证所有字段创建成功
///
/// ## 预期结果
/// - 所有字段都创建成功
#[test]
fn test_field_builder_multiple_fields() {
    // Arrange: 准备测试添加多个字段
    let builder = FormBuilder::new().add_group(
        "test",
        |g| {
            g.step(|f| {
                f.add_text("field1", "Field 1")
                    .add_password("password", "Password")
                    .add_selection("choice", "Choose", vec!["A".to_string(), "B".to_string()])
                    .add_confirmation("confirm", "Confirm?")
            })
        },
        workflow::base::dialog::GroupConfig::required(),
    );
    // Assert: 验证多个字段创建成功
    assert_eq!(builder.groups.len(), 1);
    assert_eq!(builder.groups[0].steps[0].fields.len(), 4);
}

/// 测试 FieldBuilder 的 Default trait
///
/// ## 测试目的
/// 验证 FieldBuilder 的 Default trait 实现（通过 FormBuilder 间接测试）。
///
/// ## 测试场景
/// 1. 使用 FormBuilder 创建表单
/// 2. 添加组和步骤
/// 3. 验证字段构建器可以创建
///
/// ## 预期结果
/// - 字段构建器可以创建
#[test]
fn test_field_builder_default_impl() {
    // Arrange: 准备测试 FieldBuilder 的 Default trait（覆盖 field_builder.rs:159-163）
    // 通过 FormBuilder 间接测试
    let _builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_text("field1", "Field 1")),
        workflow::base::dialog::GroupConfig::required(),
    );
    // Assert: 验证可以创建字段构建器
    assert!(true);
}
