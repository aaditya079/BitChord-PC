use crate::innertube::client::InnertubeClient;
use crate::innertube::models::Track;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadedTrackMeta {
    pub track: Track,
    pub codec: String,
    pub file_size: u64,
    pub downloaded_at: u64,
}

pub struct OfflineStorage {
    library_dir: PathBuf,
}

impl OfflineStorage {
    pub fn new() -> Result<Self, std::io::Error> {
        let base_dir = dirs_fallback().join("BitChord").join("library");
        fs::create_dir_all(&base_dir)?;
        Ok(Self {
            library_dir: base_dir,
        })
    }

    pub fn with_dir(dir: PathBuf) -> Result<Self, std::io::Error> {
        fs::create_dir_all(&dir)?;
        Ok(Self { library_dir: dir })
    }

    pub fn is_downloaded(&self, video_id: &str) -> bool {
        self.audio_path(video_id).exists() && self.meta_path(video_id).exists()
    }

    pub fn audio_path(&self, video_id: &str) -> PathBuf {
        self.library_dir.join(format!("{}.audio", video_id))
    }

    pub fn meta_path(&self, video_id: &str) -> PathBuf {
        self.library_dir.join(format!("{}.meta.json", video_id))
    }

    pub fn get_downloaded_track(&self, video_id: &str) -> Option<(Track, PathBuf)> {
        if !self.is_downloaded(video_id) {
            return None;
        }

        let meta_data = fs::read_to_string(self.meta_path(video_id)).ok()?;
        let meta: DownloadedTrackMeta = serde_json::from_str(&meta_data).ok()?;
        Some((meta.track, self.audio_path(video_id)))
    }

    pub fn list_downloaded_tracks(&self) -> Vec<Track> {
        let mut tracks = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.library_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(meta) = serde_json::from_str::<DownloadedTrackMeta>(&content) {
                            tracks.push(meta.track);
                        }
                    }
                }
            }
        }
        tracks
    }

    pub fn delete_track(&self, video_id: &str) -> Result<(), std::io::Error> {
        let a_path = self.audio_path(video_id);
        let m_path = self.meta_path(video_id);
        if a_path.exists() {
            fs::remove_file(a_path)?;
        }
        if m_path.exists() {
            fs::remove_file(m_path)?;
        }
        info!("Removed track from offline library: {}", video_id);
        Ok(())
    }

    /// Fetches and saves the track stream directly for offline access.
    pub async fn download_track(
        &self,
        client: &reqwest::Client,
        innertube: &InnertubeClient,
        track: &Track,
    ) -> Result<(), String> {
        let video_id = &track.video_id;
        info!("Starting offline download for track: {} ({})", track.title, video_id);

        let stream_info = innertube
            .get_stream(video_id)
            .await
            .map_err(|e| e.to_string())?;

        let audio_bytes = client
            .get(&stream_info.url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .bytes()
            .await
            .map_err(|e| e.to_string())?;

        let audio_path = self.audio_path(video_id);
        fs::write(&audio_path, &audio_bytes).map_err(|e| e.to_string())?;

        let meta = DownloadedTrackMeta {
            track: track.clone(),
            codec: stream_info.codec,
            file_size: audio_bytes.len() as u64,
            downloaded_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
        fs::write(self.meta_path(video_id), meta_json).map_err(|e| e.to_string())?;

        info!(
            "Successfully saved offline track: {} ({} KB)",
            track.title,
            audio_bytes.len() / 1024
        );
        Ok(())
    }
}

fn dirs_fallback() -> PathBuf {
    #[cfg(windows)]
    if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(appdata);
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_storage_crud() {
        let temp_dir = std::env::temp_dir().join("bitchord_test_offline");
        let _ = fs::remove_dir_all(&temp_dir);

        let storage = OfflineStorage::with_dir(temp_dir.clone()).unwrap();
        let track = Track {
            video_id: "test_vid_123".to_string(),
            title: "Test Track".to_string(),
            artists: vec!["Test Artist".to_string()],
            album: Some("Test Album".to_string()),
            duration_secs: 180,
            thumbnail_url: None,
        };

        assert!(!storage.is_downloaded(&track.video_id));

        // Save fake dummy audio and meta
        let meta = DownloadedTrackMeta {
            track: track.clone(),
            codec: "opus".to_string(),
            file_size: 4,
            downloaded_at: 1000,
        };
        fs::write(storage.audio_path(&track.video_id), b"RIFF").unwrap();
        fs::write(storage.meta_path(&track.video_id), serde_json::to_string(&meta).unwrap()).unwrap();

        assert!(storage.is_downloaded(&track.video_id));
        let list = storage.list_downloaded_tracks();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "Test Track");

        storage.delete_track(&track.video_id).unwrap();
        assert!(!storage.is_downloaded(&track.video_id));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
