//! HTTP Headers 工具

use reqwest::header::HeaderMap;

/// Trait for types that can be converted to HeaderMap
pub trait IntoHeaderMap {
    fn into_header_map(self) -> HeaderMap;
}

impl IntoHeaderMap for HeaderMap {
    fn into_header_map(self) -> HeaderMap {
        self
    }
}

impl IntoHeaderMap for &HeaderMap {
    fn into_header_map(self) -> HeaderMap {
        self.clone()
    }
}
