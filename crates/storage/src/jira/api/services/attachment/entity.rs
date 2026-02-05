// //! Attachment 服务实体定义

// use crate::jira::types::JiraAttachment;
// use domain::jira::JiraConfigContext;
// use regex::Regex;
// use std::collections::HashMap;

// /// URL 解析器
// pub(crate) struct UrlResolver {
//     pub(crate) original_urls: HashMap<String, String>,
//     pub(crate) api_attachments_map: HashMap<String, String>,
// }

// impl UrlResolver {
//     pub(crate) fn get_download_urls(&self, context: &dyn JiraConfigContext, attachment: &JiraAttachment) -> Vec<String> {
//         let mut urls = vec![attachment.content_url.clone()];

//         if attachment.content_url.contains("cloudfront.net") {
//             if let Some(api_url) = self.api_attachments_map.get(&attachment.filename) {
//                 urls.push(api_url.clone());
//             }

//             if let Some(attachment_id) = Self::extract_attachment_id(&attachment.content_url) {
//                 let service_address = context.get_jira_service_address();
//                 if !service_address.is_empty() {
//                     urls.push(format!("{}/rest/api/2/attachment/content/{}", service_address, attachment_id));
//                 }
//             }

//             if let Some(original_url) = self.original_urls.get(&attachment.filename) {
//                 if original_url != &attachment.content_url {
//                     urls.push(original_url.clone());
//                 }
//             }
//         }

//         urls
//     }

//     fn extract_attachment_id(url: &str) -> Option<String> {
//         let id_pattern = Regex::new(r"/attachments/[^/]+/\d+/([^/]+/)*(\d+)/").unwrap();
//         id_pattern
//             .captures(url)?
//             .get(2)
//             .map(|m| m.as_str().to_string())
//     }
// }
