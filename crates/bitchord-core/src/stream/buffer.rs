use crate::cache::disk::DiskCache;
use crate::stream::downloader::ChunkDownloader;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use symphonia::core::io::MediaSource;
use tracing::warn;

/// High-performance stream buffer supporting on-demand chunk streaming,
/// background pre-fetching, and seekable disk-backed caching.
pub struct StreamBuffer {
    pub downloader: Arc<ChunkDownloader>,
    cache_file: Arc<Mutex<File>>,
    cursor: u64,
    total_len: Option<u64>,
    bytes_downloaded: Arc<AtomicU64>,
    cancel_flag: Arc<AtomicBool>,
    _handle: tokio::runtime::Handle,
}

impl StreamBuffer {
    pub fn new(
        downloader: Arc<ChunkDownloader>,
        disk_cache: &DiskCache,
        video_id: &str,
        handle: tokio::runtime::Handle,
    ) -> std::io::Result<Self> {
        let cache_file = disk_cache
            .open_writer(video_id)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        let total_len = downloader.content_length;
        let bytes_downloaded = Arc::new(AtomicU64::new(0));
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let cache_file_arc = Arc::new(Mutex::new(cache_file));

        let buffer = Self {
            downloader: downloader.clone(),
            cache_file: cache_file_arc.clone(),
            cursor: 0,
            total_len,
            bytes_downloaded: bytes_downloaded.clone(),
            cancel_flag: cancel_flag.clone(),
            _handle: handle.clone(),
        };

        // Spawn background progressive chunk downloader
        let dl = downloader.clone();
        let c_file = cache_file_arc.clone();
        let b_down = bytes_downloaded.clone();
        let c_flag = cancel_flag.clone();

        handle.spawn(async move {
            let mut offset = 0u64;
            let chunk_size = dl.chunk_size as u64;

            while !c_flag.load(Ordering::Relaxed) {
                if let Some(total) = dl.content_length {
                    if offset >= total {
                        break;
                    }
                }

                let end = if let Some(total) = dl.content_length {
                    (offset + chunk_size - 1).min(total - 1)
                } else {
                    offset + chunk_size - 1
                };

                match dl.fetch_range(offset, end).await {
                    Ok(bytes) => {
                        let len = bytes.len() as u64;
                        if len == 0 {
                            break;
                        }

                        // Write to disk cache file at exact offset
                        if let Ok(mut f) = c_file.lock() {
                            if f.seek(SeekFrom::Start(offset)).is_ok() {
                                let _ = f.write_all(&bytes);
                                let _ = f.flush();
                            }
                        }

                        offset += len;
                        b_down.store(offset, Ordering::Release);
                    }
                    Err(e) => {
                        warn!("Background prefetch paused on error: {:?}", e);
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
            }
        });

        Ok(buffer)
    }

    /// Blocks synchronously until at least `needed_offset` bytes are available.
    fn ensure_buffered(&mut self, needed_offset: u64) -> std::io::Result<()> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(15);

        while self.bytes_downloaded.load(Ordering::Acquire) < needed_offset {
            if let Some(total) = self.total_len {
                if needed_offset > total {
                    return Ok(());
                }
            }

            if start.elapsed() > timeout {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Timed out waiting for audio chunk stream",
                ));
            }

            std::thread::sleep(Duration::from_millis(15));
        }

        Ok(())
    }
}

impl Drop for StreamBuffer {
    fn drop(&mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }
}

impl Read for StreamBuffer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        if let Some(total) = self.total_len {
            if self.cursor >= total {
                return Ok(0);
            }
        }

        let needed_end = self.cursor + (buf.len() as u64);
        let _ = self.ensure_buffered((self.cursor + 1).min(self.total_len.unwrap_or(needed_end)));

        let mut file = self.cache_file.lock().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "Cache file mutex poisoned")
        })?;

        file.seek(SeekFrom::Start(self.cursor))?;
        let bytes_read = file.read(buf)?;
        self.cursor += bytes_read as u64;

        Ok(bytes_read)
    }
}

impl Seek for StreamBuffer {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_cursor = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::Current(offset) => {
                if offset >= 0 {
                    self.cursor + (offset as u64)
                } else {
                    self.cursor.saturating_sub((-offset) as u64)
                }
            }
            SeekFrom::End(offset) => {
                let total = self.total_len.ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::Unsupported, "Cannot seek from end of unknown length stream")
                })?;
                if offset >= 0 {
                    total + (offset as u64)
                } else {
                    total.saturating_sub((-offset) as u64)
                }
            }
        };

        self.cursor = new_cursor;
        Ok(self.cursor)
    }
}

impl MediaSource for StreamBuffer {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        self.total_len
    }
}
