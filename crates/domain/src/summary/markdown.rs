//! `CommitSummaryAnalysis` 的 Markdown 渲染实现
//!
//! 将结构化的提交分析结果格式化为可读的 Markdown 文本，
//! 适用于 PR 描述、终端输出等场景。

use super::entity::{
    AffectedModule, CommitSummaryAnalysis, DetailsByCategory, ImpactAnalysis,
};

impl CommitSummaryAnalysis {
    /// 将提交分析结果渲染为 Markdown 格式的字符串
    ///
    /// 输出包含以下章节：
    /// - Summary（主要目的 + 关键变更）
    /// - Changes（按类别划分的变更详情）
    /// - Impact Analysis（破坏性变更、受影响模块、风险评估、测试建议）
    /// - Statistics（文件变更统计）
    /// - Review Info（复杂度、优先级、预估时间、标签）
    pub fn to_markdown(&self) -> String {
        let mut body = String::new();

        // == Summary ==
        body.push_str("## Summary\n\n");
        if !self.structured_summary.main_purpose.is_empty() {
            body.push_str(&self.structured_summary.main_purpose);
            body.push_str("\n\n");
        }

        // Key changes
        if !self.structured_summary.key_changes.is_empty() {
            body.push_str("### Key Changes\n\n");
            for change in &self.structured_summary.key_changes {
                body.push_str(&format!("- {}\n", change));
            }
            body.push('\n');
        }

        // == Changes by Category ==
        render_details_by_category(&mut body, &self.structured_summary.details_by_category);

        // == Impact Analysis ==
        render_impact_analysis(&mut body, &self.impact_analysis);

        // == Statistics ==
        let stats = &self.statistics;
        body.push_str("## Statistics\n\n");
        body.push_str(&format!(
            "| Metric | Value |\n|--------|-------|\n| Total files | {} |\n| Additions | +{} |\n| Deletions | -{} |\n| Net change | {} |\n",
            stats.total_files,
            stats.additions,
            stats.deletions,
            stats.net_change,
        ));

        let fb = &stats.file_breakdown;
        if fb.added > 0 || fb.modified > 0 || fb.deleted > 0 || fb.renamed > 0 {
            body.push_str(&format!(
                "| Added files | {} |\n| Modified files | {} |\n| Deleted files | {} |\n| Renamed files | {} |\n",
                fb.added, fb.modified, fb.deleted, fb.renamed,
            ));
        }
        body.push('\n');

        // == Metadata ==
        let meta = &self.metadata;
        if !meta.complexity.is_empty() || !meta.review_priority.is_empty() {
            body.push_str("## Review Info\n\n");
            if !meta.complexity.is_empty() {
                body.push_str(&format!("- **Complexity**: {}\n", meta.complexity));
            }
            if !meta.review_priority.is_empty() {
                body.push_str(&format!(
                    "- **Review priority**: {}\n",
                    meta.review_priority
                ));
            }
            if !meta.estimated_review_time.is_empty() {
                body.push_str(&format!(
                    "- **Estimated review time**: {}\n",
                    meta.estimated_review_time
                ));
            }
            if !meta.tags.is_empty() {
                body.push_str(&format!("- **Tags**: {}\n", meta.tags.join(", ")));
            }
            body.push('\n');
        }

        body.trim_end().to_string()
    }
}

/// 渲染按类别划分的变更详情
fn render_details_by_category(body: &mut String, details: &DetailsByCategory) {
    let categories: Vec<(&str, &[String])> = vec![
        ("Features", &details.features),
        ("Bug Fixes", &details.fixes),
        ("Refactors", &details.refactors),
        ("Configuration", &details.config),
        ("Documentation", &details.docs),
        ("Tests", &details.tests),
        ("Others", &details.others),
    ];

    let has_any = categories.iter().any(|(_, items)| !items.is_empty());
    if !has_any {
        return;
    }

    body.push_str("## Changes\n\n");
    for (label, items) in &categories {
        if items.is_empty() {
            continue;
        }
        body.push_str(&format!("### {}\n\n", label));
        for item in *items {
            body.push_str(&format!("- {}\n", item));
        }
        body.push('\n');
    }
}

/// 渲染影响分析
fn render_impact_analysis(body: &mut String, impact: &ImpactAnalysis) {
    let has_breaking = impact.breaking_changes.has_breaking;
    let has_modules = !impact.affected_modules.is_empty();
    let has_risk = !impact.risk_assessment.overall_risk.is_empty();
    let has_testing = !impact.testing_suggestions.is_empty();

    if !has_breaking && !has_modules && !has_risk && !has_testing {
        return;
    }

    body.push_str("## Impact Analysis\n\n");

    // Breaking changes
    if has_breaking {
        body.push_str("### Breaking Changes\n\n");
        if !impact.breaking_changes.description.is_empty() {
            body.push_str(&format!("{}\n\n", impact.breaking_changes.description));
        }
        if !impact.breaking_changes.migration_guide.is_empty() {
            body.push_str(&format!(
                "**Migration guide**: {}\n\n",
                impact.breaking_changes.migration_guide
            ));
        }
    }

    // Affected modules
    if has_modules {
        render_affected_modules(body, &impact.affected_modules);
    }

    // Risk assessment
    if has_risk {
        body.push_str(&format!(
            "### Risk Assessment\n\n**Overall risk**: {}\n\n",
            impact.risk_assessment.overall_risk
        ));
        if !impact.risk_assessment.risk_factors.is_empty() {
            body.push_str("**Risk factors**:\n");
            for factor in &impact.risk_assessment.risk_factors {
                body.push_str(&format!("- {}\n", factor));
            }
            body.push('\n');
        }
        if !impact.risk_assessment.mitigation.is_empty() {
            body.push_str("**Mitigation**:\n");
            for m in &impact.risk_assessment.mitigation {
                body.push_str(&format!("- {}\n", m));
            }
            body.push('\n');
        }
    }

    // Testing suggestions
    if has_testing {
        body.push_str("### Testing Suggestions\n\n");
        for suggestion in &impact.testing_suggestions {
            body.push_str(&format!("- {}\n", suggestion));
        }
        body.push('\n');
    }
}

/// 渲染受影响模块表格
fn render_affected_modules(body: &mut String, modules: &[AffectedModule]) {
    body.push_str("### Affected Modules\n\n");
    body.push_str("| Module | Impact | Severity |\n|--------|--------|----------|\n");
    for m in modules {
        body.push_str(&format!(
            "| {} | {} | {} |\n",
            m.module, m.impact, m.severity
        ));
    }
    body.push('\n');
}
