//! 工作流集成测试
//!
//! 测试整个工作流的端到端功能。

use workflow::base::settings::Settings;

/// 测试工作流的基本初始化
#[test]
fn test_workflow_initialization() {
    // 测试设置加载
    let settings = Settings::load();
    assert!(!settings.llm.provider.is_empty());
}

/// 测试工作流配置完整性
#[test]
fn test_workflow_config_completeness() {
    let settings = Settings::load();

    // 验证基本配置项存在
    assert_eq!(settings.log.output_folder_name, "logs");
    assert!(!settings.llm.provider.is_empty());
}
