//! 表单示例
//!
//! 演示 FormBuilder 组合多个字段的使用方式。
//!
//! 运行方式：
//! ```bash
//! cargo run -p prompt --example form_demo
//! ```
//!
//! 注意：此示例需要交互式终端，不能在非 TTY 环境下运行。

use std::sync::Arc;

use prompt::{
    form, is_user_cancelled, validators, ConfirmFormField, InputFormField, Message,
    MultiSelectFormField, PasswordFormField, PromptError, SelectFormField,
};

fn main() {
    println!("Form demonstration");
    println!("============");
    println!();
    println!("This example demonstrates how to use FormBuilder to combine multiple fields.");
    println!("Press Ctrl+C or Esc to cancel the form.");
    println!();

    let msg = Message::global();

    // 演示 1：简单表单
    if let Err(e) = demo_simple_form(&msg) {
        if is_cancelled(&e) {
            let _ = msg.warning("The form was cancelled by the user");
            return;
        }
        let _ = msg.error(format!("Error: {}", e));
        return;
    }

    // 演示 2：条件字段表单
    if let Err(e) = demo_conditional_form(&msg) {
        if is_cancelled(&e) {
            let _ = msg.warning("The form was cancelled by the user");
            return;
        }
        let _ = msg.error(format!("Error: {}", e));
        return;
    }

    let _ = msg.break_line();
    let _ = msg.success("All demonstrations completed!");
}

/// 检查是否是用户取消操作
fn is_cancelled(e: &PromptError) -> bool {
    is_user_cancelled(&e.to_string())
}

/// 演示 1：简单表单
fn demo_simple_form(msg: &prompt::MessageRef) -> prompt::Result<()> {
    let _ = msg.separator_with_text('-', 50, "Demo 1: Simple Form");

    let result = form()
        .with_title("User Registration")
        .add_input(InputFormField {
            key: "username".to_string(),
            prompt: "Please enter your username".to_string(),
            default_value: String::new(),
            validator: Some(Arc::new(validators::min_length(3))),
            condition: None,
            result_title: Some("Username".to_string()),
        })
        .add_input(InputFormField {
            key: "email".to_string(),
            prompt: "Please enter your email".to_string(),
            default_value: String::new(),
            validator: Some(Arc::new(
                validators::regex(
                    r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
                    Some("Please enter a valid email address"),
                )
                .expect("Invalid regex"),
            )),
            condition: None,
            result_title: Some("Email".to_string()),
        })
        .add_password(PasswordFormField {
            key: "password".to_string(),
            prompt: "Please enter your password".to_string(),
            default_value: String::new(),
            validator: Some(Arc::new(validators::min_length(8))),
            condition: None,
            result_title: Some("Password".to_string()),
        })
        .add_select(SelectFormField {
            key: "role".to_string(),
            prompt: "Please select a role".to_string(),
            options: vec!["User".to_string(), "Admin".to_string(), "Guest".to_string()],
            default_index: 0,
            condition: None,
            result_title: Some("Role".to_string()),
        })
        .add_confirm(ConfirmFormField {
            key: "newsletter".to_string(),
            prompt: "Do you want to subscribe to the newsletter?".to_string(),
            default_value: true,
            condition: None,
            result_title: Some("Newsletter".to_string()),
        })
        .run()?;

    // 显示结果
    let _ = msg.break_line();
    let _ = msg.success("Registration information:");

    let username = result.get_string("username");
    if !username.is_empty() {
        let _ = msg.print(format!("  Username: {}", username));
    }
    let email = result.get_string("email");
    if !email.is_empty() {
        let _ = msg.print(format!("  Email: {}", email));
    }
    let _ = msg.print("  Password: ****");

    let role = result.get_string("role");
    if !role.is_empty() {
        let _ = msg.print(format!("  Role: {}", role));
    }

    let newsletter = result.get_bool("newsletter");
    let _ = msg.print(format!(
        "  Subscribe to newsletter: {}",
        if newsletter { "Yes" } else { "No" }
    ));

    Ok(())
}

/// 演示 2：条件字段表单
fn demo_conditional_form(msg: &prompt::MessageRef) -> prompt::Result<()> {
    let _ = msg.break_line();
    let _ = msg.separator_with_text('-', 50, "Demo 2: Conditional Form");

    let result = form()
        .with_title("Project Setup")
        .add_input(InputFormField {
            key: "project_name".to_string(),
            prompt: "Please enter the project name".to_string(),
            default_value: "my-project".to_string(),
            validator: Some(Arc::new(validators::required())),
            condition: None,
            result_title: Some("Project".to_string()),
        })
        .add_select(SelectFormField {
            key: "project_type".to_string(),
            prompt: "Please select the project type".to_string(),
            options: vec![
                "Library".to_string(),
                "Binary".to_string(),
                "Workspace".to_string(),
            ],
            default_index: 0,
            condition: None,
            result_title: Some("Type".to_string()),
        })
        .add_confirm(ConfirmFormField {
            key: "use_git".to_string(),
            prompt: "Do you want to initialize a Git repository?".to_string(),
            default_value: true,
            condition: None,
            result_title: Some("Git".to_string()),
        })
        // 只有选择初始化 Git 时才显示此选项
        .add_input(InputFormField {
            key: "git_remote".to_string(),
            prompt: "Please enter the Git remote repository address (optional)".to_string(),
            default_value: String::new(),
            validator: None,
            condition: Some(Box::new(|result| result.get_bool("use_git"))),
            result_title: Some("Remote".to_string()),
        })
        .add_confirm(ConfirmFormField {
            key: "add_ci".to_string(),
            prompt: "Do you want to add CI configuration?".to_string(),
            default_value: false,
            condition: None,
            result_title: Some("CI".to_string()),
        })
        // 只有选择添加 CI 时才显示此选项
        .add_multiselect(MultiSelectFormField {
            key: "ci_platforms".to_string(),
            prompt: "Please select the CI platforms".to_string(),
            options: vec![
                "GitHub Actions".to_string(),
                "GitLab CI".to_string(),
                "CircleCI".to_string(),
                "Travis CI".to_string(),
            ],
            default_selected: vec![0],
            condition: Some(Box::new(|result| result.get_bool("add_ci"))),
            result_title: Some("CI Platforms".to_string()),
        })
        .run()?;

    // 显示结果
    let _ = msg.break_line();
    let _ = msg.success("Project setup:");

    let name = result.get_string("project_name");
    if !name.is_empty() {
        let _ = msg.print(format!("  Project name: {}", name));
    }

    let ptype = result.get_string("project_type");
    if !ptype.is_empty() {
        let _ = msg.print(format!("  Project type: {}", ptype));
    }

    let use_git = result.get_bool("use_git");
    let _ = msg.print(format!("  Git repository: {}", if use_git { "Yes" } else { "No" }));

    if use_git {
        let remote = result.get_string("git_remote");
        if !remote.is_empty() {
            let _ = msg.print(format!("  Remote address: {}", remote));
        }
    }

    let add_ci = result.get_bool("add_ci");
    let _ = msg.print(format!("  CI configuration: {}", if add_ci { "Yes" } else { "No" }));

    if add_ci {
        let platforms = result.get_int_slice("ci_platforms");
        if !platforms.is_empty() {
            let _ = msg.print(format!("  CI platforms: {:?}", platforms));
        }
    }

    Ok(())
}
