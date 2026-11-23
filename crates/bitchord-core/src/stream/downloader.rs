use crate::innertube::client::InnertubeClient;
use bytes::Bytes;
use reqwest::header::RANGE;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Stream expired and renewal failed: {0}")]
    RenewalFailed(String),
    #[error("HTTP error status: {0}")]
    HttpStatus(reqwest::StatusCode),
}

pub struct ChunkDownloader {
    client: reqwest::Client,
    video_id: String,
    stream_url: Arc<RwLock<String>>,
    innertube: Arc<InnertubeClient>,
    pub content_length: Option<u64>,
    pub chunk_size: usize,
}

impl ChunkDownloader {
    pub fn new(
        client: reqwest::Client,
        video_id: String,
        initial_url: String,
        content_length: Option<u64>,
        innertube: Arc<InnertubeClient>,
    ) -> Self {
        Self {
            client,
            video_id,
            stream_url: Arc::new(RwLock::new(initial_url)),
            innertube,
            content_length,
            chunk_size: 512 * 1024, // 512 KB optimal streaming chunk
        }
    }

    /// Fetches a specific byte range with automatic 403 retry and stream renewal.
    pub async fn fetch_range(&self, start: u64, end: u64) -> Result<Bytes, StreamError> {
        let max_retries = 6;
        let mut retry = 0;

        loop {
            let current_url = self.stream_url.read().await.clone();
            let range_header = format!("bytes={}-{}", start, end);

            let resp_result = self
                .client
                .get(&current_url)
                .header(RANGE, &range_header)
                .timeout(Duration::from_secs(12))
                .send()
                .await;

            match resp_result {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() || status == reqwest::StatusCode::PARTIAL_CONTENT {
                        let bytes = resp.bytes().await?;
                        return Ok(bytes);
                    } else if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::GONE {
                        warn!(
                            "Stream chunk {}-{} encountered {} (attempt {}/{}); renewing stream signature",
                            start, end, status, retry + 1, max_retries
                        );

                        if retry >= max_retries {
                            return Err(StreamError::HttpStatus(status));
                        }

                        // Renew stream URL from Innertube
                        match self.innertube.get_stream(&self.video_id).await {
                            Ok(new_stream) => {
                                info!("Successfully renewed stream URL for track {}", self.video_id);
                                let mut lock = self.stream_url.write().await;
                                *lock = new_stream.url;
                            }
                            Err(e) => {
                                error!("Failed to renew stream URL for track {}: {:?}", self.video_id, e);
                            }
                        }

                        // Exponential jitter backoff
                        let backoff = Duration::from_millis(150 * (1 << retry) + (rand_jitter() % 100));
                        tokio::time::sleep(backoff).await;
                        retry += 1;
                    } else {
                        warn!("Unexpected status {} for chunk {}-{}", status, start, end);
                        if retry >= max_retries {
                            return Err(StreamError::HttpStatus(status));
                        }
                        tokio::time::sleep(Duration::from_millis(200)).await;
                        retry += 1;
                    }
                }
                Err(e) => {
                    warn!("Network failure fetching chunk {}-{}: {:?}", start, end, e);
                    if retry >= max_retries {
                        return Err(StreamError::Network(e));
                    }
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    retry += 1;
                }
            }
        }
    }
}

fn rand_jitter() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(42)
}
