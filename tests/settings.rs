//! Settings 测试
//!
//! 测试配置加载和初始化功能。

use workflow::base::settings::settings::Settings;

#[test]
fn test_settings_initialization() {
    // 测试初始化（使用默认值）
    let settings = Settings::load();
    // 注意：这些测试会加载实际的配置文件，所以只测试结构是否正确加载
    assert_eq!(settings.log.output_folder_name, "logs");
    // LLM provider 可能是 openai 或用户配置的其他值
    assert!(!settings.llm.provider.is_empty());
}

#[test]
fn test_llm_provider() {
    // 测试 LLM provider 是否被正确加载
    let settings = Settings::load();
    // 可能是 openai (默认) 或用户配置的其他值
    assert!(!settings.llm.provider.is_empty());
}
