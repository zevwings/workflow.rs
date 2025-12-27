//! Homebrew Formula 更新实现

use color_eyre::{eyre::WrapErr, Result};
use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::git::GitCommand;
use crate::{log_error, log_info, log_success, log_warning};

/// Homebrew Formula 更新命令
pub struct HomebrewUpdateCommand {
    version: String,
    tag: String,
    formula_path: String,
    template_path: Option<String>,
    repo: String,
    commit: bool,
    push: bool,
}

impl HomebrewUpdateCommand {
    /// 创建新的 Homebrew Formula 更新命令
    pub fn new(
        version: String,
        tag: String,
        formula_path: Option<String>,
        template_path: Option<String>,
        repo: Option<String>,
        commit: bool,
        push: bool,
    ) -> Self {
        let formula_path = formula_path.unwrap_or_else(|| "Formula/workflow.rb".to_string());
        let repo = repo.unwrap_or_else(|| {
            std::env::var("GITHUB_REPOSITORY").unwrap_or_else(|_| "unknown/repo".to_string())
        });

        Self {
            version,
            tag,
            formula_path,
            template_path,
            repo,
            commit,
            push,
        }
    }

    /// 更新 Formula 文件
    pub fn update(&self) -> Result<()> {
        let formula_path = Path::new(&self.formula_path);

        // 备份原始文件
        if formula_path.exists() {
            let backup_path = format!("{}.bak", self.formula_path);
            fs::copy(&self.formula_path, &backup_path).wrap_err("Failed to backup Formula file")?;
            log_info!("📝 Backed up Formula file to {}", backup_path);
        }

        // 从模板生成或更新现有文件
        if let Some(ref template_path) = self.template_path {
            if Path::new(template_path).exists() {
                log_info!("📝 Generating Formula file from template...");
                self.generate_from_template(template_path, &self.formula_path)?;
                log_success!("Formula file generated from template");
            } else {
                log_warning!(
                    "Template file not found: {}, updating existing file",
                    template_path
                );
                self.update_existing_file(&self.formula_path)?;
            }
        } else {
            log_info!("📝 Updating version in Formula file...");
            self.update_existing_file(&self.formula_path)?;
        }

        // 验证文件结构
        self.validate_formula(&self.formula_path)?;

        // 显示生成的 Formula 文件
        log_info!("\n📄 Generated Formula file:");
        log_info!("--- {} ---", self.formula_path);
        let content = fs::read_to_string(&self.formula_path)?;
        log_info!("{}", content);

        // Git 操作
        if self.commit {
            self.git_operations()?;
        }

        Ok(())
    }

    /// 从模板生成 Formula 文件
    fn generate_from_template(&self, template_path: &str, output_path: &str) -> Result<()> {
        let template_content = fs::read_to_string(template_path)
            .wrap_err_with(|| format!("Failed to read template: {}", template_path))?;

        // 替换模板变量
        let content = template_content
            .replace("{{VERSION}}", &self.version)
            .replace("{{TAG}}", &self.tag);

        fs::write(output_path, content)
            .wrap_err_with(|| format!("Failed to write Formula file: {}", output_path))?;

        Ok(())
    }

    /// 更新现有 Formula 文件
    fn update_existing_file(&self, formula_path: &str) -> Result<()> {
        let content = fs::read_to_string(formula_path)
            .wrap_err_with(|| format!("Failed to read Formula file: {}", formula_path))?;

        // 更新版本号
        let version_regex = Regex::new(r#"version\s+"[^"]+""#)?;
        let updated = version_regex.replace(&content, &format!(r#"version "{}""#, self.version));

        // 更新下载 URL
        let url_pattern = format!(
            r#"url\s+"https://github\.com/{}/releases/download/[^"]+""#,
            regex::escape(&self.repo)
        );
        let url_regex = Regex::new(&url_pattern)?;
        let download_url = format!(
            "https://github.com/{}/releases/download/{}/workflow-{}-x86_64-apple-darwin.tar.gz",
            self.repo, self.tag, self.version
        );
        let updated = url_regex.replace(&updated, &format!(r#"url "{}""#, download_url));

        fs::write(formula_path, updated.as_ref())
            .wrap_err_with(|| format!("Failed to write Formula file: {}", formula_path))?;

        log_success!("Formula file updated");

        Ok(())
    }

    /// 验证 Formula 文件语法
    fn validate_formula(&self, formula_path: &str) -> Result<()> {
        log_info!("🔍 Validating Formula file structure...");

        // 尝试使用 ruby -c 验证语法
        let result = Command::new("ruby").arg("-c").arg(formula_path).output();

        match result {
            Ok(output) if output.status.success() => {
                log_success!("Formula file syntax is valid");
                Ok(())
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log_error!("Formula file has syntax errors");
                log_error!("{}", stderr);
                Err(color_eyre::eyre::eyre!(
                    "Formula file syntax validation failed"
                ))
            }
            Err(_) => {
                // Ruby 未安装，跳过验证
                log_warning!("Ruby not found, skipping syntax validation");
                Ok(())
            }
        }
    }

    /// Git 操作（配置、提交、推送）
    fn git_operations(&self) -> Result<()> {
        // 配置 Git
        GitCommand::new(["config", "user.name", "github-actions[bot]"]).run()?;
        GitCommand::new([
            "config",
            "user.email",
            "github-actions[bot]@users.noreply.github.com",
        ])
        .run()?;

        // 添加文件
        GitCommand::new(["add", &self.formula_path]).run()?;

        // 检查是否有更改
        let status_output = GitCommand::new(["diff", "--staged", "--quiet"]).quiet_success();
        if status_output {
            log_info!("No changes to commit. Formula file is already up to date.");
            return Ok(());
        }

        // 验证 Formula 文件格式（可选）
        if Command::new("brew").arg("--version").output().is_ok() {
            let audit_result =
                Command::new("brew").args(["audit", "--strict", &self.formula_path]).output();

            if let Ok(output) = audit_result {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    log_warning!("brew audit failed, but continuing...");
                    log_warning!("{}", stderr);
                }
            }
        }

        // 提交更改
        let commit_message = format!("Update workflow to {}", self.tag);
        GitCommand::new(["commit", "-m", &commit_message]).run()?;
        log_success!("Committed changes: {}", commit_message);

        // 推送到远程
        if self.push {
            let current_branch = GitCommand::new(["branch", "--show-current"]).read()?;
            log_info!("Pushing to branch: {}", current_branch);

            GitCommand::new(["push", "origin", &current_branch])
                .run()
                .wrap_err_with(|| format!("Failed to push to branch: {}", current_branch))?;

            log_success!("Successfully pushed to {} branch", current_branch);
        }

        Ok(())
    }
}
