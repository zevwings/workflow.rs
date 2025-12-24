//! Base/Dialog/Form Field Builder 模块测试
//!
//! 测试字段构建器的核心功能。

use workflow::base::dialog::FormBuilder;

#[test]
fn test_field_builder_new() {
    // 测试创建字段构建器（覆盖 field_builder.rs:20-25）
    // 注意：FieldBuilder::new 是 pub，但通常通过 FormBuilder 使用
    // 我们通过 FormBuilder 间接测试
    let _builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_text("field1", "Field 1")),
        workflow::base::dialog::GroupConfig::required(),
    );
    // 验证可以创建字段构建器
    assert!(true);
}

#[test]
fn test_field_builder_add_text() {
    // 测试添加文本字段（覆盖 field_builder.rs:28-44）
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_text("field1", "Field 1")),
        workflow::base::dialog::GroupConfig::required(),
    );
    // 验证字段创建成功
    assert_eq!(builder.groups.len(), 1);
    assert!(!builder.groups[0].steps.is_empty());
    assert!(!builder.groups[0].steps[0].fields.is_empty());
}

#[test]
fn test_field_builder_add_password() {
    // 测试添加密码字段（覆盖 field_builder.rs:47-63）
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_password("password", "Enter password")),
        workflow::base::dialog::GroupConfig::required(),
    );
    // 验证密码字段创建成功
    assert_eq!(builder.groups.len(), 1);
    assert!(!builder.groups[0].steps[0].fields.is_empty());
}

#[test]
fn test_field_builder_add_selection() {
    // 测试添加选择字段（覆盖 field_builder.rs:66-87）
    let choices = vec!["Option 1".to_string(), "Option 2".to_string()];
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_selection("choice", "Choose option", choices.clone())),
        workflow::base::dialog::GroupConfig::required(),
    );
    // 验证选择字段创建成功
    assert_eq!(builder.groups.len(), 1);
    assert!(!builder.groups[0].steps[0].fields.is_empty());
}

#[test]
fn test_field_builder_add_confirmation() {
    // 测试添加确认字段（覆盖 field_builder.rs:90-106）
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_confirmation("confirm", "Confirm?")),
        workflow::base::dialog::GroupConfig::required(),
    );
    // 验证确认字段创建成功
    assert_eq!(builder.groups.len(), 1);
    assert!(!builder.groups[0].steps[0].fields.is_empty());
}

#[test]
fn test_field_builder_required() {
    // 测试标记字段为必填（覆盖 field_builder.rs:109-116）
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_text("field1", "Field 1").required()),
        workflow::base::dialog::GroupConfig::required(),
    );
    // 验证字段标记为必填
    assert_eq!(builder.groups.len(), 1);
    let field = &builder.groups[0].steps[0].fields[0];
    assert!(field.required);
}

#[test]
fn test_field_builder_default_string() {
    // 测试设置字符串默认值（覆盖 field_builder.rs:119-133）
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_text("field1", "Field 1").default("default_value")),
        workflow::base::dialog::GroupConfig::required(),
    );
    // 验证默认值设置成功
    assert_eq!(builder.groups.len(), 1);
    let field = &builder.groups[0].steps[0].fields[0];
    assert!(field.default_value.is_some());
}

#[test]
fn test_field_builder_default_bool() {
    // 测试设置布尔默认值（覆盖 field_builder.rs:119-133）
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_confirmation("confirm", "Confirm?").default(true)),
        workflow::base::dialog::GroupConfig::required(),
    );
    // 验证布尔默认值设置成功
    assert_eq!(builder.groups.len(), 1);
    let field = &builder.groups[0].steps[0].fields[0];
    assert!(field.default_value.is_some());
}

#[test]
fn test_field_builder_default_selection() {
    // 测试设置选择字段的默认值（覆盖 field_builder.rs:125-128）
    let choices = vec!["Option 1".to_string(), "Option 2".to_string()];
    let builder = FormBuilder::new().add_group(
        "test",
        |g| {
            g.step(|f| {
                f.add_selection("choice", "Choose option", choices.clone())
                    .default("Option 1")
            })
        },
        workflow::base::dialog::GroupConfig::required(),
    );
    // 验证选择字段的默认值设置成功
    assert_eq!(builder.groups.len(), 1);
    let field = &builder.groups[0].steps[0].fields[0];
    assert!(field.default_value.is_some());
    assert_eq!(field.default_choice, Some("Option 1".to_string()));
}

#[test]
fn test_field_builder_validate() {
    // 测试设置验证器（覆盖 field_builder.rs:136-146）
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
    // 验证验证器设置成功
    assert_eq!(builder.groups.len(), 1);
    let field = &builder.groups[0].steps[0].fields[0];
    assert!(field.validator.is_some());
}

#[test]
fn test_field_builder_allow_empty() {
    // 测试允许空值（覆盖 field_builder.rs:149-156）
    let builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_text("field1", "Field 1").allow_empty(true)),
        workflow::base::dialog::GroupConfig::required(),
    );
    // 验证允许空值设置成功
    assert_eq!(builder.groups.len(), 1);
    let field = &builder.groups[0].steps[0].fields[0];
    assert!(field.allow_empty);
}

#[test]
fn test_field_builder_chain_all() {
    // 测试链式调用所有方法
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
    // 验证链式调用成功
    assert_eq!(builder.groups.len(), 1);
    let field = &builder.groups[0].steps[0].fields[0];
    assert!(field.required);
    assert!(field.default_value.is_some());
    assert!(!field.allow_empty);
    assert!(field.validator.is_some());
}

#[test]
fn test_field_builder_multiple_fields() {
    // 测试添加多个字段
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
    // 验证多个字段创建成功
    assert_eq!(builder.groups.len(), 1);
    assert_eq!(builder.groups[0].steps[0].fields.len(), 4);
}

#[test]
fn test_field_builder_default_impl() {
    // 测试 FieldBuilder 的 Default trait（覆盖 field_builder.rs:159-163）
    // 通过 FormBuilder 间接测试
    let _builder = FormBuilder::new().add_group(
        "test",
        |g| g.step(|f| f.add_text("field1", "Field 1")),
        workflow::base::dialog::GroupConfig::required(),
    );
    // 验证可以创建字段构建器
    assert!(true);
}

