use crate::{log_error, log_success, Clipboard, Logs};
use crate::settings::Settings;
use anyhow::{Context, Result};
use std::path::Path;

/// 日志查找命令
pub struct LogsFindCommand;

impl LogsFindCommand {
    /// 在日志文件中查找请求 ID 并提取响应内容
    pub fn find(log_file: &Path, request_id: &str, jira_id: Option<&str>) -> Result<()> {
        log_success!("Searching for request ID: {}...", request_id);

        // 1. 提取响应内容
        let response_content = Logs::extract_response_content(log_file, request_id)
            .context("Failed to extract response content")?;

        // 2. 复制到剪贴板
        Clipboard::copy(&response_content).context("Failed to copy to clipboard")?;
        log_success!("Response content copied to clipboard");

        // 3. 获取日志条目的 URL 信息（用于生成 name）
        let entry = Logs::find_request_id(log_file, request_id)?;
        let name = if let Some(entry) = entry {
            if let Some(url) = entry.url {
                // 提取 URL 的最后两段路径
                let url_parts: Vec<&str> = url.split('/').collect();
                if url_parts.len() >= 2 {
                    format!(
                        "#{} {}",
                        request_id,
                        url_parts[url_parts.len() - 2..].join("/")
                    )
                } else {
                    format!("#{} {}", request_id, url_parts.last().unwrap_or(&"unknown"))
                }
            } else {
                format!("#{} unknown", request_id)
            }
        } else {
            format!("#{} unknown", request_id)
        };

        // 4. 生成 domain
        let domain = if let Some(jira_id) = jira_id {
            let settings = Settings::get();
            let jira_service = &settings.jira_service_address;
            format!("{}/browse/{}", jira_service, jira_id)
        } else {
            log_file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        };

        // 5. 生成时间戳（格式：MM/DD/YYYY, HH:MM:SS AM/PM）
        use std::time::SystemTime;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 简单的格式化（实际项目中可以使用 chrono）
        let formatted_timestamp = format!("Unix timestamp: {}", now);

        // 6. 创建 JSON payload
        let payload = serde_json::json!({
            "encodedKey": "",
            "data": response_content,
            "combineLine": "",
            "separator": "",
            "domain": domain,
            "name": name,
            "timestamp": formatted_timestamp
        });

        // 7. 发送到 Streamock 服务
        let streamock_url = "http://localhost:3001/api/submit";
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(streamock_url)
            .header("Content-Type", "application/json")
            .header("Connection", "keep-alive")
            .json(&payload)
            .send()
            .context("Failed to send request to Streamock")?;

        if response.status().is_success() {
            log_success!("Response sent to Streamock successfully");
        } else {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            log_error!("Failed to send to Streamock: {} - {}", status, body);
            anyhow::bail!("Streamock request failed: {}", status);
        }

        Ok(())
    }
}
