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

use prompt::{
    form, is_user_cancelled, validators, ConfirmFormField, InputFormField, Message,
    MultiSelectFormField, PasswordFormField, PromptError, SelectFormField,
};
use std::sync::Arc;

fn main() {
    println!("表单功能演示");
    println!("============");
    println!();
    println!("此示例演示如何使用 FormBuilder 组合多个字段。");
    println!("按 Ctrl+C 或 Esc 可以取消表单。");
    println!();

    let msg = Message::global();

    // 演示 1：简单表单
    if let Err(e) = demo_simple_form(&msg) {
        if is_cancelled(&e) {
            let _ = msg.warning("表单被用户取消");
            return;
        }
        let _ = msg.error(&format!("错误: {}", e));
        return;
    }

    // 演示 2：条件字段表单
    if let Err(e) = demo_conditional_form(&msg) {
        if is_cancelled(&e) {
            let _ = msg.warning("表单被用户取消");
            return;
        }
        let _ = msg.error(&format!("错误: {}", e));
        return;
    }

    let _ = msg.break_line();
    let _ = msg.success("所有演示完成！");
}

/// 检查是否是用户取消操作
fn is_cancelled(e: &PromptError) -> bool {
    is_user_cancelled(&e.to_string())
}

/// 演示 1：简单表单
fn demo_simple_form(msg: &prompt::MessageRef) -> prompt::Result<()> {
    let _ = msg.separator_with_text('-', 50, "Demo 1: Simple Form");

    let result = form()
        .with_title("用户注册")
        .add_input(InputFormField {
            key: "username".to_string(),
            prompt: "请输入用户名".to_string(),
            default_value: String::new(),
            validator: Some(Arc::new(validators::min_length(3))),
            condition: None,
            result_title: Some("Username".to_string()),
        })
        .add_input(InputFormField {
            key: "email".to_string(),
            prompt: "请输入邮箱".to_string(),
            default_value: String::new(),
            validator: Some(Arc::new(
                validators::regex(
                    r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
                    Some("请输入有效的邮箱地址"),
                )
                .expect("Invalid regex"),
            )),
            condition: None,
            result_title: Some("Email".to_string()),
        })
        .add_password(PasswordFormField {
            key: "password".to_string(),
            prompt: "请输入密码".to_string(),
            default_value: String::new(),
            validator: Some(Arc::new(validators::min_length(8))),
            condition: None,
            result_title: Some("Password".to_string()),
        })
        .add_select(SelectFormField {
            key: "role".to_string(),
            prompt: "请选择角色".to_string(),
            options: vec![
                "User".to_string(),
                "Admin".to_string(),
                "Guest".to_string(),
            ],
            default_index: 0,
            condition: None,
            result_title: Some("Role".to_string()),
        })
        .add_confirm(ConfirmFormField {
            key: "newsletter".to_string(),
            prompt: "是否订阅新闻通讯？".to_string(),
            default_value: true,
            condition: None,
            result_title: Some("Newsletter".to_string()),
        })
        .run()?;

    // 显示结果
    let _ = msg.break_line();
    let _ = msg.success("注册信息：");

    let username = result.get_string("username");
    if !username.is_empty() {
        let _ = msg.print(&format!("  用户名: {}", username));
    }
    let email = result.get_string("email");
    if !email.is_empty() {
        let _ = msg.print(&format!("  邮箱: {}", email));
    }
    let _ = msg.print("  密码: ****");

    let role = result.get_string("role");
    if !role.is_empty() {
        let _ = msg.print(&format!("  角色: {}", role));
    }

    let newsletter = result.get_bool("newsletter");
    let _ = msg.print(&format!(
        "  订阅新闻: {}",
        if newsletter { "是" } else { "否" }
    ));

    Ok(())
}

/// 演示 2：条件字段表单
fn demo_conditional_form(msg: &prompt::MessageRef) -> prompt::Result<()> {
    let _ = msg.break_line();
    let _ = msg.separator_with_text('-', 50, "Demo 2: Conditional Form");

    let result = form()
        .with_title("项目配置")
        .add_input(InputFormField {
            key: "project_name".to_string(),
            prompt: "请输入项目名称".to_string(),
            default_value: "my-project".to_string(),
            validator: Some(Arc::new(validators::required())),
            condition: None,
            result_title: Some("Project".to_string()),
        })
        .add_select(SelectFormField {
            key: "project_type".to_string(),
            prompt: "请选择项目类型".to_string(),
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
            prompt: "是否初始化 Git 仓库？".to_string(),
            default_value: true,
            condition: None,
            result_title: Some("Git".to_string()),
        })
        // 只有选择初始化 Git 时才显示此选项
        .add_input(InputFormField {
            key: "git_remote".to_string(),
            prompt: "请输入 Git 远程仓库地址（可选）".to_string(),
            default_value: String::new(),
            validator: None,
            condition: Some(Box::new(|result| result.get_bool("use_git"))),
            result_title: Some("Remote".to_string()),
        })
        .add_confirm(ConfirmFormField {
            key: "add_ci".to_string(),
            prompt: "是否添加 CI 配置？".to_string(),
            default_value: false,
            condition: None,
            result_title: Some("CI".to_string()),
        })
        // 只有选择添加 CI 时才显示此选项
        .add_multiselect(MultiSelectFormField {
            key: "ci_platforms".to_string(),
            prompt: "请选择 CI 平台".to_string(),
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
    let _ = msg.success("项目配置：");

    let name = result.get_string("project_name");
    if !name.is_empty() {
        let _ = msg.print(&format!("  项目名称: {}", name));
    }

    let ptype = result.get_string("project_type");
    if !ptype.is_empty() {
        let _ = msg.print(&format!("  项目类型: {}", ptype));
    }

    let use_git = result.get_bool("use_git");
    let _ = msg.print(&format!("  Git 仓库: {}", if use_git { "是" } else { "否" }));

    if use_git {
        let remote = result.get_string("git_remote");
        if !remote.is_empty() {
            let _ = msg.print(&format!("  远程地址: {}", remote));
        }
    }

    let add_ci = result.get_bool("add_ci");
    let _ = msg.print(&format!("  CI 配置: {}", if add_ci { "是" } else { "否" }));

    if add_ci {
        let platforms = result.get_int_slice("ci_platforms");
        if !platforms.is_empty() {
            let _ = msg.print(&format!("  CI 平台索引: {:?}", platforms));
        }
    }

    Ok(())
}
