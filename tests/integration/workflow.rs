//! 工作流集成测试
//!
//! 测试整个工作流的端到端功能。

use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use workflow::base::settings::Settings;

// ==================== Fixtures ====================

#[fixture]
fn settings() -> Settings {
    Settings::load()
}

// ==================== 工作流测试 ====================

/// 测试工作流的基本初始化
#[rstest]
fn test_workflow_initialization(settings: Settings) {
    // 测试设置加载
    assert!(!settings.llm.provider.is_empty());
}

/// 测试工作流配置完整性
#[rstest]
fn test_workflow_config_completeness(settings: Settings) {
    // 验证基本配置项存在
    assert_eq!(settings.log.output_folder_name, "logs");
    assert!(!settings.llm.provider.is_empty());
}
