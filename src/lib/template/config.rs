//! Template configuration
//!
//! Loads templates from configuration files (global and project-level).

use crate::base::settings::paths::Paths;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

/// Template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Template engine type (default: "handlebars")
    #[serde(default = "default_engine")]
    pub engine: String,
    /// Branch templates
    #[serde(default)]
    pub branch: BranchTemplates,
    /// Commit templates
    #[serde(default)]
    pub commit: CommitTemplates,
    /// PR templates
    #[serde(default)]
    pub pull_requests: PullRequestsTemplates,
}

fn default_engine() -> String {
    "handlebars".to_string()
}

/// Branch templates configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchTemplates {
    /// Default branch template
    #[serde(default = "default_branch_template")]
    pub default: String,
    /// Feature branch template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature: Option<String>,
    /// Bugfix branch template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bugfix: Option<String>,
    /// Hotfix branch template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hotfix: Option<String>,
    /// Refactoring branch template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refactoring: Option<String>,
    /// Chore branch template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chore: Option<String>,
}

fn default_branch_template() -> String {
    "{{jira_key}}-{{summary_slug}}".to_string()
}

impl Default for BranchTemplates {
    fn default() -> Self {
        Self {
            default: "{{jira_key}}-{{summary_slug}}".to_string(),
            feature: Some("feature/{{jira_key}}-{{summary_slug}}".to_string()),
            bugfix: Some("bugfix/{{jira_key}}-{{summary_slug}}".to_string()),
            hotfix: Some("hotfix/{{jira_key}}-{{summary_slug}}".to_string()),
            refactoring: Some("refactoring/{{jira_key}}-{{summary_slug}}".to_string()),
            chore: Some("chore/{{jira_key}}-{{summary_slug}}".to_string()),
        }
    }
}

/// Commit templates configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitTemplates {
    /// Default commit template
    #[serde(default = "CommitTemplates::default_commit_template")]
    pub default: String,
    /// Whether to use scope for commit messages (when no ticket id)
    ///
    /// When `true`, uses Conventional Commits format: `{commit_type}({scope}): {title}`
    /// When `false`, uses simple format: `# {title}`
    #[serde(default = "default_use_scope")]
    pub use_scope: bool,
}

fn default_use_scope() -> bool {
    false // Keep backward compatibility
}

impl CommitTemplates {
    /// Get default commit template
    pub fn default_commit_template() -> String {
        r#"{{#if jira_key}}{{jira_key}}: {{subject}}{{else}}{{#if use_scope}}{{commit_type}}{{#if scope}}({{scope}}){{/if}}: {{subject}}{{else}}# {{subject}}{{/if}}{{/if}}

{{#if body}}{{body}}{{/if}}

{{#if jira_key}}Closes {{jira_key}}{{/if}}"#
            .to_string()
    }
}

impl Default for CommitTemplates {
    fn default() -> Self {
        Self {
            default: CommitTemplates::default_commit_template(),
            use_scope: default_use_scope(),
        }
    }
}

/// PR templates configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestsTemplates {
    /// Default PR template
    #[serde(default = "PullRequestsTemplates::default_pull_request_template")]
    pub default: String,
}

impl PullRequestsTemplates {
    /// Get default PR template
    pub fn default_pull_request_template() -> String {
        r#"
# PR Ready

## Types of changes

{{#each change_types}}
- [{{#if this.selected}}x{{else}} {{/if}}] {{this.name}}
{{/each}}

{{#if short_description}}
#### Short description:

{{short_description}}
{{/if}}

{{#if jira_key}}
{{#if jira_service_address}}
#### Jira Link:

{{jira_service_address}}/browse/{{jira_key}}
{{/if}}
{{/if}}

{{#if dependency}}
#### Dependency

{{dependency}}
{{/if}}
"#
        .trim_start()
        .to_string()
    }
}

impl Default for PullRequestsTemplates {
    fn default() -> Self {
        Self {
            default: PullRequestsTemplates::default_pull_request_template(),
        }
    }
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            engine: default_engine(),
            branch: BranchTemplates::default(),
            commit: CommitTemplates::default(),
            pull_requests: PullRequestsTemplates::default(),
        }
    }
}

impl TemplateConfig {
    /// Load template configuration
    ///
    /// Loads templates from configuration file, with project-level config overriding global config.
    pub fn load() -> Result<Self> {
        // Try to load from project-level config first
        if let Ok(project_config) = Self::load_project() {
            return Ok(project_config);
        }

        // Fall back to global config
        if let Ok(global_config) = Self::load_global() {
            return Ok(global_config);
        }

        // Return default config if no config file exists
        Ok(Self::default())
    }

    /// Load project-level template config
    fn load_project() -> Result<Self> {
        // Check for .workflow/config.toml in project root
        let project_config_path = std::env::current_dir()
            .context("Failed to get current directory")?
            .join(".workflow")
            .join("config.toml");

        if !project_config_path.exists() {
            anyhow::bail!("Project config not found");
        }

        let content = fs::read_to_string(&project_config_path)
            .context("Failed to read project template config")?;

        // Parse TOML and extract template section
        let value: toml::Value =
            toml::from_str(&content).context("Failed to parse project template config")?;

        // Extract template section if exists
        if let Some(template_section) = value.get("template") {
            let config: TemplateConfig = toml::from_str(
                &toml::to_string(template_section)
                    .context("Failed to serialize template section")?,
            )
            .context("Failed to parse template config")?;
            Ok(config)
        } else {
            anyhow::bail!("No template section in project config")
        }
    }

    /// Load global template config
    fn load_global() -> Result<Self> {
        // Try to load from workflow.toml
        let config_path = Paths::workflow_config().context("Failed to get workflow config path")?;

        if !config_path.exists() {
            anyhow::bail!("Global config not found");
        }

        let content =
            fs::read_to_string(&config_path).context("Failed to read global template config")?;

        let value: toml::Value =
            toml::from_str(&content).context("Failed to parse global template config")?;

        // Extract template section if exists
        if let Some(template_section) = value.get("template") {
            let config: TemplateConfig = toml::from_str(
                &toml::to_string(template_section)
                    .context("Failed to serialize template section")?,
            )
            .context("Failed to parse template config")?;
            Ok(config)
        } else {
            anyhow::bail!("No template section in global config")
        }
    }

    /// Load branch template
    ///
    /// Loads branch template based on JIRA ticket type (feature/bugfix/hotfix) or uses default.
    /// This is the legacy function for backward compatibility.
    pub fn load_branch_template(jira_type: Option<&str>) -> Result<String> {
        let config = Self::load()?;

        // Select template based on JIRA type
        let template = match jira_type {
            Some("Feature") | Some("Story") | Some("Epic") => {
                config.branch.feature.as_ref().unwrap_or(&config.branch.default)
            }
            Some("Bug") => config.branch.bugfix.as_ref().unwrap_or(&config.branch.default),
            Some("Hotfix") => config.branch.hotfix.as_ref().unwrap_or(&config.branch.default),
            _ => &config.branch.default,
        };

        Ok(template.clone())
    }

    /// Load branch template by branch type
    ///
    /// Loads branch template based on branch type string (feature/bugfix/refactoring/hotfix/chore).
    pub fn load_branch_template_by_type(branch_type: Option<&str>) -> Result<String> {
        let config = Self::load()?;

        // Select template based on branch type
        let template = match branch_type {
            Some("feature") => config.branch.feature.as_ref().unwrap_or(&config.branch.default),
            Some("bugfix") => config.branch.bugfix.as_ref().unwrap_or(&config.branch.default),
            Some("hotfix") => config.branch.hotfix.as_ref().unwrap_or(&config.branch.default),
            Some("refactoring") => {
                config.branch.refactoring.as_ref().unwrap_or(&config.branch.default)
            }
            Some("chore") => config.branch.chore.as_ref().unwrap_or(&config.branch.default),
            _ => &config.branch.default,
        };

        Ok(template.clone())
    }

    /// Load commit template
    pub fn load_commit_template() -> Result<String> {
        let config = Self::load()?;
        Ok(config.commit.default.clone())
    }

    /// Load PR template
    pub fn load_pull_request_template() -> Result<String> {
        let config = Self::load()?;
        Ok(config.pull_requests.default.clone())
    }
}
