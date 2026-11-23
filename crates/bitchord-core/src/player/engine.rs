use crate::cache::disk::DiskCache;
use crate::innertube::client::InnertubeClient;
use crate::player::decoder::AudioDecoder;
use crate::sink::cpal_sink::CpalAudioSink;
use crate::stream::buffer::StreamBuffer;
use crate::stream::downloader::ChunkDownloader;
use rtrb::Producer;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::{error, info};

#[derive(Debug, Error)]
pub enum PlayerError {
    #[error("Audio sink error: {0}")]
    Sink(#[from] crate::sink::cpal_sink::SinkError),
    #[error("Innertube error: {0}")]
    Innertube(#[from] crate::innertube::client::InnertubeError),
    #[error("Decoder error: {0}")]
    Decoder(#[from] crate::player::decoder::DecoderError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Master BitChord audio playback engine.
/// Coordinates stream downloading, on-the-fly decoding, loudness leveling, and real-time output.
pub struct AudioEngine {
    innertube: Arc<InnertubeClient>,
    disk_cache: Arc<DiskCache>,
    sink: Arc<CpalAudioSink>,
    producer: Arc<Mutex<Option<Producer<f32>>>>,
    is_playing: Arc<AtomicBool>,
    position_ms: Arc<AtomicU64>,
    duration_ms: Arc<AtomicU64>,
    user_volume: Arc<Mutex<f32>>,
    normalizer_gain: Arc<Mutex<f32>>,
    cancel_signal: Arc<AtomicBool>,
    runtime_handle: tokio::runtime::Handle,
}

impl AudioEngine {
    pub fn new() -> Result<Self, PlayerError> {
        let (sink, producer) = CpalAudioSink::new()?;
        let disk_cache = DiskCache::new().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        Ok(Self {
            innertube: Arc::new(InnertubeClient::new()),
            disk_cache: Arc::new(disk_cache),
            sink: Arc::new(sink),
            producer: Arc::new(Mutex::new(Some(producer))),
            is_playing: Arc::new(AtomicBool::new(false)),
            position_ms: Arc::new(AtomicU64::new(0)),
            duration_ms: Arc::new(AtomicU64::new(0)),
            user_volume: Arc::new(Mutex::new(1.0)),
            normalizer_gain: Arc::new(Mutex::new(1.0)),
            cancel_signal: Arc::new(AtomicBool::new(false)),
            runtime_handle: tokio::runtime::Handle::current(),
        })
    }

    /// Plays a track by its YouTube video ID.
    pub async fn play(&self, video_id: &str) -> Result<(), PlayerError> {
        // Stop any currently running playback thread
        self.stop();

        info!("Starting playback for track: {}", video_id);

        let stream_info = self.innertube.get_stream(video_id).await?;

        // Calculate loudness gain normalization (target -14 LUFS)
        let loudness_gain = if let Some(loudness_db) = stream_info.loudness_db {
            let gain = 10.0f32.powf(-loudness_db / 20.0);
            gain.clamp(0.2, 1.8)
        } else {
            1.0
        };

        {
            let mut gain_lock = self.normalizer_gain.lock().await;
            *gain_lock = loudness_gain;
            self.apply_master_volume().await;
        }

        let dl = Arc::new(ChunkDownloader::new(
            reqwest::Client::new(),
            video_id.to_string(),
            stream_info.url,
            stream_info.content_length,
            self.innertube.clone(),
        ));

        let stream_buf = StreamBuffer::new(
            dl,
            &self.disk_cache,
            video_id,
            self.runtime_handle.clone(),
        )?;

        let mut decoder = AudioDecoder::new(stream_buf, &stream_info.codec)?;

        if let Some(secs) = decoder.duration_secs {
            self.duration_ms.store((secs * 1000.0) as u64, Ordering::Release);
        }

        self.cancel_signal.store(false, Ordering::Release);
        self.is_playing.store(true, Ordering::Release);
        self.position_ms.store(0, Ordering::Release);

        let cancel = self.cancel_signal.clone();
        let playing_flag = self.is_playing.clone();
        let pos_ms = self.position_ms.clone();
        let sample_rate = decoder.sample_rate as f64;
        let channels = decoder.channels as f64;

        let mut producer_guard = self.producer.lock().await;
        let mut producer = producer_guard.take().unwrap_or_else(|| {
            let buffer_capacity = (decoder.sample_rate as usize) * decoder.channels * 2;
            let (p, _) = rtrb::RingBuffer::<f32>::new(buffer_capacity);
            p
        });

        // Dedicated audio pump worker thread
        std::thread::spawn(move || {
            let mut total_samples_rendered = 0u64;

            while !cancel.load(Ordering::Relaxed) {
                if !playing_flag.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(25));
                    continue;
                }

                match decoder.next_packet_samples() {
                    Ok(Some(samples)) => {
                        if samples.is_empty() {
                            std::thread::sleep(Duration::from_millis(15));
                            continue;
                        }

                        let mut offset = 0;
                        while offset < samples.len() && !cancel.load(Ordering::Relaxed) {
                            while offset < samples.len() {
                                if producer.push(samples[offset]).is_ok() {
                                    offset += 1;
                                    total_samples_rendered += 1;
                                } else {
                                    break; // Ring buffer full
                                }
                            }

                            let ms = (total_samples_rendered as f64 / (sample_rate * channels)) * 1000.0;
                            pos_ms.store(ms as u64, Ordering::Release);

                            if offset < samples.len() {
                                // Buffer full, sleep briefly to let the real-time audio thread consume
                                std::thread::sleep(Duration::from_millis(15));
                            }
                        }
                    }
                    Ok(None) => {
                        info!("Playback stream ended");
                        playing_flag.store(false, Ordering::Release);
                        break;
                    }
                    Err(e) => {
                        error!("Decoder error during playback: {:?}", e);
                        playing_flag.store(false, Ordering::Release);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub fn pause(&self) {
        self.is_playing.store(false, Ordering::Release);
        self.sink.set_paused(true);
    }

    pub fn resume(&self) {
        self.is_playing.store(true, Ordering::Release);
        self.sink.set_paused(false);
    }

    pub fn stop(&self) {
        self.cancel_signal.store(true, Ordering::Release);
        self.is_playing.store(false, Ordering::Release);
        self.position_ms.store(0, Ordering::Release);
    }

    pub async fn set_volume(&self, volume: f32) {
        let mut vol_lock = self.user_volume.lock().await;
        *vol_lock = volume.clamp(0.0, 1.0);
        drop(vol_lock);
        self.apply_master_volume().await;
    }

    async fn apply_master_volume(&self) {
        let user_vol = *self.user_volume.lock().await;
        let norm_gain = *self.normalizer_gain.lock().await;
        let master = user_vol * norm_gain;
        self.sink.set_volume(master);
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::Acquire)
    }

    pub fn position(&self) -> Duration {
        Duration::from_millis(self.position_ms.load(Ordering::Acquire))
    }

    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.duration_ms.load(Ordering::Acquire))
    }
}
