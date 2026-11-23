use crate::cipher::solver::CipherSolver;
use crate::innertube::lyrics::LyricsClient;
use crate::innertube::models::{AudioStream, LyricLine, SearchResults, Track};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use serde_json::{json, Value};
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Error)]
pub enum InnertubeError {
    #[error("HTTP request error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Failed to parse JSON response: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Video stream unavailable or unplayable: {0}")]
    Unplayable(String),
    #[error("No compatible audio formats found")]
    NoFormatFound,
    #[error("Cipher deciphering failed: {0}")]
    Cipher(#[from] crate::cipher::solver::CipherError),
}

pub struct InnertubeClient {
    client: reqwest::Client,
    cipher: CipherSolver,
    lyrics: LyricsClient,
}

impl InnertubeClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_default();

        Self {
            lyrics: LyricsClient::new(client.clone()),
            client,
            cipher: CipherSolver::new(),
        }
    }

    /// Searches YouTube Music for songs matching the query.
    pub async fn search(&self, query: &str) -> Result<SearchResults, InnertubeError> {
        let url = "https://music.youtube.com/youtubei/v1/search";
        let body = json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20241118.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "query": query
        });

        let resp: Value = self
            .client
            .post(url)
            .header("x-youtube-client-name", "67")
            .header("x-youtube-client-version", "1.20241118.01.00")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        let mut tracks = Vec::new();

        let sections = resp
            .pointer("/contents/tabbedSearchResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents")
            .or_else(|| resp.pointer("/contents/sectionListRenderer/contents"))
            .and_then(|v| v.as_array());

        if let Some(contents) = sections {
            for section in contents {
                if let Some(items) = section
                    .pointer("/musicShelfRenderer/contents")
                    .or_else(|| section.pointer("/musicCardShelfRenderer/contents"))
                    .and_then(|v| v.as_array())
                {
                    for item in items {
                        if let Some(track) = Self::parse_music_shelf_item(item) {
                            tracks.push(track);
                        }
                    }
                }
            }
        }

        if tracks.is_empty() {
            if let Some(contents) = resp.pointer("/contents/itemSectionRenderer/contents").and_then(|v| v.as_array()) {
                for item in contents {
                    if let Some(track) = Self::parse_music_shelf_item(item) {
                        tracks.push(track);
                    }
                }
            }
        }

        Ok(SearchResults { tracks })
    }

    /// Resolves an optimal playable audio stream for a given video ID.
    /// Uses the VISIONOS client for plain unthrottled streaming URLs,
    /// with WEB_REMIX cipher solver fallback for Made-For-Kids / COPPA tracks.
    pub async fn get_stream(&self, video_id: &str) -> Result<AudioStream, InnertubeError> {
        let vision_ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_7_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Safari/605.1.15";
        let url = "https://www.youtube.com/youtubei/v1/player?prettyPrint=false";
        let body = json!({
            "context": {
                "client": {
                    "clientName": "VISIONOS",
                    "clientVersion": "1.02",
                    "deviceMake": "Apple",
                    "deviceModel": "RealityDevice17,1",
                    "userAgent": vision_ua,
                    "osName": "visionOS",
                    "osVersion": "26.5.23O471",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "videoId": video_id,
            "contentCheckOk": true,
            "racyCheckOk": true
        });

        let resp: Value = self
            .client
            .post(url)
            .header("User-Agent", vision_ua)
            .header("X-YouTube-Client-Name", "101")
            .header("X-YouTube-Client-Version", "1.02")
            .header("Origin", "https://www.youtube.com")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        // If unplayable on visionOS (e.g. Made for Kids), fall back to webmusic solver
        if let Some(status) = resp.pointer("/playabilityStatus/status").and_then(|s| s.as_str()) {
            if status != "OK" {
                warn!("Track {} unplayable on VISIONOS ({}), falling back to WEB_REMIX solver", video_id, status);
                return self.get_stream_web_remix(video_id).await;
            }
        }

        let loudness = resp
            .pointer("/playerConfig/audioConfig/loudnessDb")
            .and_then(|v| v.as_f64())
            .map(|f| f as f32);

        // Parse adaptive formats
        if let Some(formats) = resp.pointer("/streamingData/adaptiveFormats").and_then(|f| f.as_array()) {
            let mut best_stream: Option<AudioStream> = None;

            for fmt in formats {
                let mime = fmt.pointer("/mimeType").and_then(|m| m.as_str()).unwrap_or("");
                if !mime.starts_with("audio/") {
                    continue;
                }

                if let Some(stream_url) = fmt.pointer("/url").and_then(|u| u.as_str()) {
                    let bitrate = fmt.pointer("/bitrate").and_then(|b| b.as_u64()).unwrap_or(0) as u32;
                    let sample_rate = fmt
                        .pointer("/audioSampleRate")
                        .and_then(|r| r.as_str())
                        .and_then(|r| r.parse().ok())
                        .unwrap_or(44100);
                    let content_length = fmt
                        .pointer("/contentLength")
                        .and_then(|c| c.as_str())
                        .and_then(|c| c.parse().ok());

                    let is_opus = mime.contains("opus");
                    let codec = if is_opus { "opus" } else { "aac" };

                    let stream = AudioStream {
                        url: stream_url.to_string(),
                        mime_type: mime.to_string(),
                        codec: codec.to_string(),
                        bitrate,
                        sample_rate,
                        content_length,
                        loudness_db: loudness,
                        is_direct: true,
                    };

                    if let Some(ref current) = best_stream {
                        if (is_opus && current.codec != "opus") || (is_opus == (current.codec == "opus") && bitrate > current.bitrate) {
                            best_stream = Some(stream);
                        }
                    } else {
                        best_stream = Some(stream);
                    }
                }
            }

            if let Some(stream) = best_stream {
                return Ok(stream);
            }
        }

        self.get_stream_web_remix(video_id).await
    }

    /// Fallback for Made-for-Kids or restricted streams via WEB_REMIX + cipher solver
    async fn get_stream_web_remix(&self, video_id: &str) -> Result<AudioStream, InnertubeError> {
        let url = "https://music.youtube.com/youtubei/v1/player";
        let body = json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20241118.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "videoId": video_id,
            "playbackContext": {
                "contentPlaybackContext": {
                    "signatureTimestamp": 20042
                }
            }
        });

        let resp: Value = self
            .client
            .post(url)
            .header("x-youtube-client-name", "67")
            .header("x-youtube-client-version", "1.20241118.01.00")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        let loudness = resp
            .pointer("/playerConfig/audioConfig/loudnessDb")
            .and_then(|v| v.as_f64())
            .map(|f| f as f32);

        if let Some(formats) = resp.pointer("/streamingData/adaptiveFormats").and_then(|f| f.as_array()) {
            for fmt in formats {
                let mime = fmt.pointer("/mimeType").and_then(|m| m.as_str()).unwrap_or("");
                if !mime.starts_with("audio/") {
                    continue;
                }

                let stream_url = if let Some(direct) = fmt.pointer("/url").and_then(|u| u.as_str()) {
                    direct.to_string()
                } else if let Some(cipher) = fmt.pointer("/signatureCipher").and_then(|c| c.as_str()) {
                    self.cipher.solve_signature_cipher(cipher, None).await?
                } else {
                    continue;
                };

                let bitrate = fmt.pointer("/bitrate").and_then(|b| b.as_u64()).unwrap_or(0) as u32;
                let sample_rate = fmt
                    .pointer("/audioSampleRate")
                    .and_then(|r| r.as_str())
                    .and_then(|r| r.parse().ok())
                    .unwrap_or(44100);

                return Ok(AudioStream {
                    url: stream_url,
                    mime_type: mime.to_string(),
                    codec: if mime.contains("opus") { "opus".to_string() } else { "aac".to_string() },
                    bitrate,
                    sample_rate,
                    content_length: fmt.pointer("/contentLength").and_then(|c| c.as_str()).and_then(|c| c.parse().ok()),
                    loudness_db: loudness,
                    is_direct: false,
                });
            }
        }

        Err(InnertubeError::NoFormatFound)
    }

    /// Fetches synchronized lyrics for a track.
    pub async fn get_lyrics(&self, artist: &str, title: &str) -> Result<Vec<LyricLine>, crate::innertube::lyrics::LyricsError> {
        self.lyrics.get_lyrics(artist, title).await
    }

    fn parse_music_shelf_item(item: &Value) -> Option<Track> {
        let renderer = item.get("musicResponsiveListItemRenderer")?;
        let video_id = renderer
            .pointer("/playlistItemData/videoId")
            .and_then(|id| id.as_str())?;

        let title = renderer
            .pointer("/flexColumns/0/musicResponsiveListItemFlexColumnRenderer/text/runs/0/text")
            .and_then(|t| t.as_str())?
            .to_string();

        let mut artists = Vec::new();
        let mut album = None;

        if let Some(runs) = renderer
            .pointer("/flexColumns/1/musicResponsiveListItemFlexColumnRenderer/text/runs")
            .and_then(|r| r.as_array())
        {
            for (idx, run) in runs.iter().enumerate() {
                if let Some(text) = run.get("text").and_then(|t| t.as_str()) {
                    if text == " • " {
                        continue;
                    }
                    if idx == 0 {
                        artists.push(text.to_string());
                    } else if album.is_none() && !text.chars().all(|c| c.is_numeric() || c == ':') {
                        album = Some(text.to_string());
                    }
                }
            }
        }

        let thumbnail_url = renderer
            .pointer("/thumbnail/musicThumbnailRenderer/thumbnail/thumbnails/0/url")
            .and_then(|u| u.as_str())
            .map(|u| u.to_string());

        Some(Track {
            video_id: video_id.to_string(),
            title,
            artists,
            album,
            duration_secs: 0,
            thumbnail_url,
        })
    }
}
