//! Multipart 转换：将 client::MultipartRequest 转为 reqwest Form

use reqwest::blocking::multipart::{Form, Part};

use client::{HttpError, MultipartPart, MultipartRequest};
use toolkit::log_error;

/// 将 client 的 MultipartRequest 转换为 reqwest Form
pub fn to_reqwest_form(multipart: MultipartRequest) -> Result<Form, HttpError> {
    let mut form = Form::new();

    for part in multipart.parts {
        match part {
            MultipartPart::Text { name, value } => {
                form = form.text(name, value);
            }
            MultipartPart::File { name, path } => {
                let part = Part::file(&path).map_err(|e| {
                    log_error!(error = %e, path = %path.display(), "Failed to read file");
                    HttpError::FileReadFailed(path.display().to_string())
                })?;
                form = form.part(name, part);
            }
            MultipartPart::Bytes {
                name,
                data,
                filename,
                mime_type,
            } => {
                let mut part = Part::bytes(data);
                if let Some(f) = filename {
                    part = part.file_name(f);
                }
                if let Some(mt) = mime_type {
                    part = part
                        .mime_str(&mt)
                        .map_err(|e| HttpError::InvalidHeaderValue(e.to_string()))?;
                }
                form = form.part(name, part);
            }
        }
    }

    Ok(form)
}
