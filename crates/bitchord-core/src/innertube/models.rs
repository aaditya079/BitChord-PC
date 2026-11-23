use serde::{Deserialize, Serialize};

/// Represents an audio track in BitChord.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Track {
    pub video_id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub duration_secs: u64,
    pub thumbnail_url: Option<String>,
}

/// An audio stream format with playback metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStream {
    pub url: String,
    pub mime_type: String,
    pub codec: String,
    pub bitrate: u32,
    pub sample_rate: u32,
    pub content_length: Option<u64>,
    pub loudness_db: Option<f32>,
    pub is_direct: bool,
}

/// Search result grouping.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchResults {
    pub tracks: Vec<Track>,
}

/// Synchronized lyric line with millisecond timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LyricLine {
    pub timestamp_ms: u64,
    pub text: String,
}
