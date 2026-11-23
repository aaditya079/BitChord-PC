use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("I/O error in disk cache: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cache directory unavailable")]
    Unavailable,
}

/// Manages temporary audio chunk caching on disk to avoid re-downloading.
pub struct DiskCache {
    cache_dir: PathBuf,
}

impl DiskCache {
    pub fn new() -> Result<Self, CacheError> {
        let temp_dir = std::env::temp_dir().join("bitchord_cache");
        fs::create_dir_all(&temp_dir)?;
        Ok(Self {
            cache_dir: temp_dir,
        })
    }

    pub fn file_path(&self, video_id: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.bcchunk", video_id))
    }

    /// Creates or opens an appendable cache file for a video ID.
    pub fn open_writer(&self, video_id: &str) -> Result<File, CacheError> {
        let path = self.file_path(video_id);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(path)?;
        Ok(file)
    }

    /// Opens a read-only stream to the cached file if it exists.
    pub fn open_reader(&self, video_id: &str) -> Option<File> {
        let path = self.file_path(video_id);
        File::open(path).ok()
    }

    /// Checks how many bytes have been written to disk for this track.
    pub fn cached_bytes(&self, video_id: &str) -> u64 {
        let path = self.file_path(video_id);
        fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    }

    /// Removes cache files older than max_age_days or when total cache exceeds max_mb.
    pub fn prune(&self, max_mb: u64) -> Result<(), CacheError> {
        if !self.cache_dir.exists() {
            return Ok(());
        }

        let mut entries = Vec::new();
        let mut total_size = 0u64;

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("bcchunk") {
                if let Ok(meta) = entry.metadata() {
                    let len = meta.len();
                    total_size += len;
                    if let Ok(modified) = meta.modified() {
                        entries.push((path, len, modified));
                    }
                }
            }
        }

        let max_bytes = max_mb * 1024 * 1024;
        if total_size > max_bytes {
            // Sort oldest modified first
            entries.sort_by_key(|(_, _, modified)| *modified);
            for (path, len, _) in entries {
                let _ = fs::remove_file(path);
                total_size = total_size.saturating_sub(len);
                if total_size <= max_bytes {
                    break;
                }
            }
        }

        Ok(())
    }
}
