//! Mock 场景预设库
//!
//! 提供常用 Mock 场景的预设配置，支持场景组合和复用。

use crate::common::mock::server::MockServer;
use crate::common::mock::validators::RequestValidator;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Mock 场景定义
///
/// 包含一组相关的 Mock 端点配置，用于模拟完整的工作流程。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockScenario {
    /// 场景名称
    pub name: String,
    /// 场景描述
    pub description: String,
    /// Mock 端点列表
    pub mocks: Vec<MockDefinition>,
}

/// Mock 端点定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockDefinition {
    /// HTTP 方法
    pub method: String,
    /// 请求路径（支持路径参数，如 `/repos/{owner}/{repo}/pulls/{pr_number}`）
    pub path: String,
    /// 响应配置
    pub response: MockResponse,
    /// 请求验证配置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationConfig>,
}

/// Mock 响应配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MockResponse {
    /// 静态响应体
    Static { body: String, status: u16 },
    /// 模板响应（支持变量替换）
    Template { template: String, status: u16 },
    /// 从文件加载响应
    File { file: String, status: u16 },
}

/// 验证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// 必需的请求头列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_headers: Option<Vec<String>>,
    /// 必需的请求体字段列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_body_fields: Option<Vec<String>>,
    /// 必需的查询参数列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_query_params: Option<Vec<String>>,
}

/// Mock 场景管理器
pub struct MockScenarioManager {
    scenarios: HashMap<String, MockScenario>,
    base_dir: PathBuf,
}

impl MockScenarioManager {
    /// 创建新的场景管理器
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            scenarios: HashMap::new(),
            base_dir,
        }
    }

    /// 加载场景文件
    pub fn load_scenario(&mut self, scenario_path: &Path) -> Result<()> {
        use color_eyre::eyre::Context;

        let content = fs::read_to_string(scenario_path).wrap_err_with(|| {
            format!("Failed to read scenario file: {}", scenario_path.display())
        })?;

        let scenario: MockScenario = serde_json::from_str(&content).wrap_err_with(|| {
            format!("Failed to parse scenario JSON: {}", scenario_path.display())
        })?;

        self.scenarios.insert(scenario.name.clone(), scenario);
        Ok(())
    }

    /// 从字符串加载场景
    pub fn load_scenario_from_str(&mut self, scenario_json: &str) -> Result<()> {
        use color_eyre::eyre::Context;

        let scenario: MockScenario =
            serde_json::from_str(scenario_json).context("Failed to parse scenario JSON")?;

        self.scenarios.insert(scenario.name.clone(), scenario);
        Ok(())
    }

    /// 获取场景
    pub fn get_scenario(&self, name: &str) -> Option<&MockScenario> {
        self.scenarios.get(name)
    }

    /// 应用场景到 MockServer
    pub fn apply_scenario(
        &self,
        mock_server: &mut MockServer,
        scenario_name: &str,
        variables: Option<&HashMap<String, String>>,
    ) -> Result<()> {
        use color_eyre::eyre::Context;

        let scenario = self
            .scenarios
            .get(scenario_name)
            .ok_or_else(|| color_eyre::eyre::eyre!("Scenario not found: {}", scenario_name))?;

        let vars = variables.cloned().unwrap_or_default();

        for mock_def in &scenario.mocks {
            // 替换路径中的变量
            let mut path = mock_def.path.clone();
            for (key, value) in &vars {
                let placeholder = format!("{{{}}}", key);
                path = path.replace(&placeholder, value);
            }

            // 处理响应
            let (response_body, status) = match &mock_def.response {
                MockResponse::Static { body, status } => (body.clone(), *status),
                MockResponse::Template { template, status } => {
                    let mut body = template.clone();
                    for (key, value) in &vars {
                        let placeholder = format!("{{{{{}}}}}", key);
                        body = body.replace(&placeholder, value);
                    }
                    (body, *status)
                }
                MockResponse::File { file, status } => {
                    let file_path = self.base_dir.join(file);
                    let body = fs::read_to_string(&file_path).wrap_err_with(|| {
                        format!("Failed to read response file: {}", file_path.display())
                    })?;
                    (body, *status)
                }
            };

            // 创建 Mock 端点
            if let Some(ref validation_config) = mock_def.validation {
                // 创建验证器
                let mut validator = RequestValidator::new();

                if let Some(ref headers) = validation_config.required_headers {
                    for header in headers {
                        validator = validator.require_header(header, ".+");
                    }
                }

                if let Some(ref body_fields) = validation_config.required_body_fields {
                    // 构建 JSON 模式
                    let pattern = format!(r#"{{"{}": ".+"}}"#, body_fields.join(r#"": ".+", ""#));
                    validator = validator.require_body_json(&pattern);
                }

                if let Some(ref query_params) = validation_config.required_query_params {
                    for param in query_params {
                        validator = validator.require_query_param(param, ".+");
                    }
                }

                // 注意：mockito 不支持请求验证，这里只是记录验证配置
                // 实际验证需要在测试代码中手动进行
            }

            // 创建 Mock 端点
            mock_server.mock_with_template(
                &mock_def.method,
                &path,
                &response_body,
                vars.clone(),
                status,
            );
        }

        Ok(())
    }
}

/// MockServer 扩展方法
impl MockServer {
    /// 加载场景文件
    pub fn load_scenario(&mut self, scenario_path: &Path) -> Result<()> {
        let mut manager = MockScenarioManager::new(
            scenario_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("tests/fixtures")),
        );
        manager.load_scenario(scenario_path)?;

        // 应用第一个场景（简化版本）
        if let Some(scenario) = manager.scenarios.values().next() {
            let vars = HashMap::new();
            manager.apply_scenario(self, &scenario.name, Some(&vars))?;
        }

        Ok(())
    }

    /// 从字符串加载场景
    #[allow(dead_code)]
    pub fn load_scenario_from_str(&mut self, scenario_json: &str) -> Result<()> {
        let mut manager = MockScenarioManager::new(PathBuf::from("tests/fixtures"));
        manager.load_scenario_from_str(scenario_json)?;

        if let Some(scenario) = manager.scenarios.values().next() {
            let vars = HashMap::new();
            manager.apply_scenario(self, &scenario.name, Some(&vars))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_scenario_from_str() -> Result<()> {
        let scenario_json = r#"
        {
            "name": "test_scenario",
            "description": "Test scenario",
            "mocks": [
                {
                    "method": "GET",
                    "path": "/api/test",
                    "response": {
                        "body": "{\"result\": \"success\"}",
                        "status": 200
                    }
                }
            ]
        }
        "#;

        let mut manager = MockScenarioManager::new(PathBuf::from("tests/fixtures"));
        manager.load_scenario_from_str(scenario_json)?;

        let scenario = manager.get_scenario("test_scenario");
        assert!(scenario.is_some());
        assert_eq!(scenario.unwrap().mocks.len(), 1);

        Ok(())
    }

    #[test]
    fn test_template_response() -> Result<()> {
        let scenario_json = r#"
        {
            "name": "template_scenario",
            "description": "Template scenario",
            "mocks": [
                {
                    "method": "GET",
                    "path": "/api/test/{id}",
                    "response": {
                        "template": "{\"id\": {{id}}, \"status\": \"ok\"}",
                        "status": 200
                    }
                }
            ]
        }
        "#;

        let mut manager = MockScenarioManager::new(PathBuf::from("tests/fixtures"));
        manager.load_scenario_from_str(scenario_json)?;

        let mut vars = HashMap::new();
        vars.insert("id".to_string(), "123".to_string());

        let mut mock_server = MockServer::new();
        manager.apply_scenario(&mut mock_server, "template_scenario", Some(&vars))?;

        // 验证场景已应用（通过检查没有错误）
        // 注意：MockServer 没有公开的方法来获取 mocks 数量，这里只是验证没有错误
        Ok(())
    }
}
