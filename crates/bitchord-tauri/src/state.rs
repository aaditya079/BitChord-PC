use bitchord_core::innertube::client::InnertubeClient;
use bitchord_core::innertube::models::Track;
use bitchord_core::player::engine::AudioEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackProgress {
    pub position_ms: u64,
    pub duration_ms: u64,
    pub is_playing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackDto {
    pub video_id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub duration_secs: u64,
    pub thumbnail_url: Option<String>,
}

impl From<Track> for TrackDto {
    fn from(t: Track) -> Self {
        Self {
            video_id: t.video_id,
            title: t.title,
            artists: t.artists,
            album: t.album,
            duration_secs: t.duration_secs,
            thumbnail_url: t.thumbnail_url,
        }
    }
}

impl From<TrackDto> for Track {
    fn from(dto: TrackDto) -> Self {
        Self {
            video_id: dto.video_id,
            title: dto.title,
            artists: dto.artists,
            album: dto.album,
            duration_secs: dto.duration_secs,
            thumbnail_url: dto.thumbnail_url,
        }
    }
}

pub struct AppState {
    pub engine: Arc<AudioEngine>,
    pub innertube: Arc<InnertubeClient>,
}

impl AppState {
    pub fn new() -> Result<Self, String> {
        let engine = AudioEngine::new().map_err(|e| e.to_string())?;
        let innertube = Arc::new(InnertubeClient::new());

        Ok(Self {
            engine: Arc::new(engine),
            innertube,
        })
    }
}
