use crate::exception::Result;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCapabilities {
    pub content_length: Option<u64>,
    pub accepts_ranges: bool,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content_type: Option<String>,
    pub suggested_filename: Option<String>,
    pub final_url: String,
    pub is_resumable: bool,
}

impl RemoteCapabilities {
    pub fn is_segmented_download_supported(&self) -> bool {
        self.accepts_ranges && self.content_length.map(|l| l > 1024 * 512).unwrap_or(false)
    }
}

/// Service trait defining remote endpoint probing contract
pub trait NetworkService: Send + Sync {
    fn probe<'a>(&'a self, url: &'a str) -> Pin<Box<dyn Future<Output = Result<RemoteCapabilities>> + Send + 'a>>;
}

pub use crate::service::r#impl::network::NetworkClient;
