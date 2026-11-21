use crate::exception::{IdmException, NetworkException, Result};
use crate::service::network::{NetworkService, RemoteCapabilities};
use bytes::Bytes;
use futures::Stream;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_RANGES, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE, ETAG, LAST_MODIFIED, RANGE, USER_AGENT};
use reqwest::Client;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tracing::{debug, info};
use url::Url;

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 1IDM/1.0";

#[derive(Clone)]
pub struct NetworkClient {
    client: Client,
}

impl NetworkClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(16)
            .tcp_nodelay(true)
            .build()
            .map_err(|e| IdmException::Network(NetworkException::ConnectionFailed(e.to_string())))?;

        Ok(Self { client })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn extract_filename_from_url(url_str: &str) -> String {
        if let Ok(parsed) = Url::parse(url_str) {
            if let Some(mut segments) = parsed.path_segments() {
                if let Some(last) = segments.next_back() {
                    let decoded = urlencoding::decode(last).unwrap_or_else(|_| last.into());
                    let trimmed = decoded.trim();
                    if !trimmed.is_empty() {
                        return trimmed.to_string();
                    }
                }
            }
        }
        "download.dat".to_string()
    }

    fn extract_filename_from_disposition(disposition: &str) -> Option<String> {
        for part in disposition.split(';') {
            let trimmed = part.trim();
            if trimmed.to_lowercase().starts_with("filename*=") {
                if let Some(val) = trimmed.split('=').nth(1) {
                    let clean = val.trim_matches('"').trim_matches('\'');
                    if let Some(utf_part) = clean.split("''").nth(1) {
                        return urlencoding::decode(utf_part).ok().map(|s| s.into_owned());
                    }
                    return Some(clean.to_string());
                }
            } else if trimmed.to_lowercase().starts_with("filename=") {
                if let Some(val) = trimmed.split('=').nth(1) {
                    let clean = val.trim().trim_matches('"').trim_matches('\'');
                    return Some(clean.to_string());
                }
            }
        }
        None
    }

    pub async fn probe(&self, url: &str) -> Result<RemoteCapabilities> {
        info!("Probing capabilities for URL: {}", url);

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENT));

        // 1. Try HEAD request
        let head_res = self
            .client
            .head(url)
            .headers(headers.clone())
            .send()
            .await;

        match head_res {
            Ok(resp) if resp.status().is_success() => {
                let final_url = resp.url().to_string();
                let headers_ref = resp.headers();

                let content_length = headers_ref
                    .get(CONTENT_LENGTH)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok());

                let accepts_ranges = headers_ref
                    .get(ACCEPT_RANGES)
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.to_lowercase() == "bytes")
                    .unwrap_or(false);

                let etag = headers_ref
                    .get(ETAG)
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.to_string());

                let last_modified = headers_ref
                    .get(LAST_MODIFIED)
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.to_string());

                let content_type = headers_ref
                    .get(CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.to_string());

                let suggested_filename = headers_ref
                    .get(CONTENT_DISPOSITION)
                    .and_then(|v| v.to_str().ok())
                    .and_then(Self::extract_filename_from_disposition)
                    .or_else(|| Some(Self::extract_filename_from_url(&final_url)));

                let range_verified = if !accepts_ranges && content_length.is_some() {
                    self.verify_range_support(url).await
                } else {
                    accepts_ranges
                };

                let is_resumable = range_verified && (etag.is_some() || last_modified.is_some() || content_length.is_some());

                debug!("Probe succeeded via HEAD: len={:?}, ranges={}", content_length, range_verified);

                return Ok(RemoteCapabilities {
                    content_length,
                    accepts_ranges: range_verified,
                    etag,
                    last_modified,
                    content_type,
                    suggested_filename,
                    final_url,
                    is_resumable,
                });
            }
            _ => {}
        }

        // 2. Fallback to GET range=0-0 probe
        debug!("HEAD probe failed/unsupported; executing GET byte-range probe");
        let range_probe = self
            .client
            .get(url)
            .headers(headers)
            .header(RANGE, "bytes=0-0")
            .send()
            .await
            .map_err(|e| IdmException::Network(NetworkException::ConnectionFailed(e.to_string())))?;

        let final_url = range_probe.url().to_string();
        let status = range_probe.status();
        let headers_ref = range_probe.headers();

        let accepts_ranges = status == reqwest::StatusCode::PARTIAL_CONTENT;

        let content_length = if accepts_ranges {
            headers_ref
                .get("content-range")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.split('/').nth(1))
                .and_then(|v| v.parse::<u64>().ok())
        } else {
            headers_ref
                .get(CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
        };

        let etag = headers_ref
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());

        let last_modified = headers_ref
            .get(LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());

        let content_type = headers_ref
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());

        let suggested_filename = headers_ref
            .get(CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .and_then(Self::extract_filename_from_disposition)
            .or_else(|| Some(Self::extract_filename_from_url(&final_url)));

        Ok(RemoteCapabilities {
            content_length,
            accepts_ranges,
            etag,
            last_modified,
            content_type,
            suggested_filename,
            final_url,
            is_resumable: accepts_ranges,
        })
    }

    async fn verify_range_support(&self, url: &str) -> bool {
        let res = self
            .client
            .get(url)
            .header(USER_AGENT, DEFAULT_USER_AGENT)
            .header(RANGE, "bytes=0-0")
            .send()
            .await;

        match res {
            Ok(resp) => resp.status() == reqwest::StatusCode::PARTIAL_CONTENT,
            Err(_) => false,
        }
    }

    pub async fn stream_range(
        &self,
        url: &str,
        start: u64,
        end: u64,
    ) -> Result<impl Stream<Item = Result<Bytes>>> {
        let range_header = format!("bytes={}-{}", start, end);
        let resp = self
            .client
            .get(url)
            .header(USER_AGENT, DEFAULT_USER_AGENT)
            .header(RANGE, range_header)
            .send()
            .await
            .map_err(|e| IdmException::Network(NetworkException::ConnectionFailed(e.to_string())))?;

        if resp.status() != reqwest::StatusCode::PARTIAL_CONTENT && resp.status() != reqwest::StatusCode::OK {
            return Err(IdmException::Network(NetworkException::RangeNotSupported(format!(
                "Unexpected status code for range request: {}",
                resp.status()
            ))));
        }

        let stream = resp.bytes_stream();
        Ok(futures::StreamExt::map(stream, |res| {
            res.map_err(|e| IdmException::Network(NetworkException::TransferInterrupted(e.to_string())))
        }))
    }

    pub async fn stream_full(
        &self,
        url: &str,
        resume_offset: u64,
    ) -> Result<(Option<u64>, impl Stream<Item = Result<Bytes>>)> {
        let mut req = self.client.get(url).header(USER_AGENT, DEFAULT_USER_AGENT);
        if resume_offset > 0 {
            req = req.header(RANGE, format!("bytes={}-", resume_offset));
        }

        let resp = req
            .send()
            .await
            .map_err(|e| IdmException::Network(NetworkException::ConnectionFailed(e.to_string())))?;
        let content_len = resp.content_length();
        let stream = resp.bytes_stream();
        Ok((
            content_len,
            futures::StreamExt::map(stream, |res| {
                res.map_err(|e| IdmException::Network(NetworkException::TransferInterrupted(e.to_string())))
            }),
        ))
    }
}

impl NetworkService for NetworkClient {
    fn probe<'a>(&'a self, url: &'a str) -> Pin<Box<dyn Future<Output = Result<RemoteCapabilities>> + Send + 'a>> {
        Box::pin(self.probe(url))
    }
}
