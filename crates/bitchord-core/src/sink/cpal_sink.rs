use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info};

#[derive(Debug, Error)]
pub enum SinkError {
    #[error("Audio output device unavailable: {0}")]
    DeviceError(String),
    #[error("Failed to build output stream: {0}")]
    StreamError(String),
}

/// Real-time audio output sink powered by CPAL and a lock-free SPSC ring buffer.
/// Strictly non-blocking: ZERO heap allocations and ZERO mutexes in the audio callback.
pub struct CpalAudioSink {
    _stream: Stream,
    volume_bits: Arc<AtomicU32>,
    is_paused: Arc<AtomicBool>,
    pub sample_rate: u32,
    pub channels: u16,
}

// Stream is kept solely to maintain the active WASAPI output session lifecycle.
// All runtime audio control and feeding is mediated via atomics and lock-free SPSC queues.
unsafe impl Send for CpalAudioSink {}
unsafe impl Sync for CpalAudioSink {}

impl CpalAudioSink {
    /// Creates the sink and returns `(CpalAudioSink, Producer<f32>)`.
    /// The `Producer<f32>` is `Send` and passed to the audio decoder pump thread.
    pub fn new() -> Result<(Self, Producer<f32>), SinkError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| SinkError::DeviceError("No default output audio device found".to_string()))?;

        let supported_config = device
            .default_output_config()
            .map_err(|e| SinkError::DeviceError(e.to_string()))?;

        let sample_rate = supported_config.sample_rate().0;
        let channels = supported_config.channels();
        let sample_format = supported_config.sample_format();
        let config: StreamConfig = supported_config.into();

        // 2 seconds of stereo f32 samples capacity
        let buffer_capacity = (sample_rate as usize) * (channels as usize) * 2;
        let (producer, mut consumer) = RingBuffer::<f32>::new(buffer_capacity);

        let volume_bits = Arc::new(AtomicU32::new(1.0f32.to_bits()));
        let vol_clone = volume_bits.clone();

        let is_paused = Arc::new(AtomicBool::new(false));
        let pause_clone = is_paused.clone();

        let err_fn = |err| error!("CPAL audio stream error: {:?}", err);

        let stream = match sample_format {
            SampleFormat::F32 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        Self::render_audio_f32(data, &mut consumer, &vol_clone, &pause_clone);
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| SinkError::StreamError(e.to_string()))?,
            SampleFormat::I16 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        Self::render_audio_i16(data, &mut consumer, &vol_clone, &pause_clone);
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| SinkError::StreamError(e.to_string()))?,
            _ => return Err(SinkError::StreamError("Unsupported audio sample format".to_string())),
        };

        stream.play().map_err(|e| SinkError::StreamError(e.to_string()))?;

        info!("CPAL sink active: {} Hz, {} channels", sample_rate, channels);

        Ok((
            Self {
                _stream: stream,
                volume_bits,
                is_paused,
                sample_rate,
                channels,
            },
            producer,
        ))
    }

    /// Audio render callback for f32 stream — STRICTLY NON-BLOCKING & REAL-TIME SAFE.
    #[inline(always)]
    fn render_audio_f32(
        data: &mut [f32],
        consumer: &mut Consumer<f32>,
        volume_bits: &AtomicU32,
        is_paused: &AtomicBool,
    ) {
        if is_paused.load(Ordering::Relaxed) {
            for sample in data.iter_mut() {
                *sample = 0.0;
            }
            return;
        }

        let volume = f32::from_bits(volume_bits.load(Ordering::Relaxed));

        for sample in data.iter_mut() {
            match consumer.pop() {
                Ok(s) => {
                    let val = (s * volume).clamp(-1.0, 1.0);
                    *sample = val;
                }
                Err(_) => {
                    *sample = 0.0;
                }
            }
        }
    }

    /// Audio render callback for i16 stream — STRICTLY NON-BLOCKING & REAL-TIME SAFE.
    #[inline(always)]
    fn render_audio_i16(
        data: &mut [i16],
        consumer: &mut Consumer<f32>,
        volume_bits: &AtomicU32,
        is_paused: &AtomicBool,
    ) {
        if is_paused.load(Ordering::Relaxed) {
            for sample in data.iter_mut() {
                *sample = 0;
            }
            return;
        }

        let volume = f32::from_bits(volume_bits.load(Ordering::Relaxed));

        for sample in data.iter_mut() {
            match consumer.pop() {
                Ok(s) => {
                    let clamped = (s * volume).clamp(-1.0, 1.0);
                    *sample = (clamped * 32767.0) as i16;
                }
                Err(_) => {
                    *sample = 0;
                }
            }
        }
    }

    pub fn set_volume(&self, volume: f32) {
        let clamped = volume.clamp(0.0, 2.0);
        self.volume_bits.store(clamped.to_bits(), Ordering::Relaxed);
    }

    pub fn set_paused(&self, paused: bool) {
        self.is_paused.store(paused, Ordering::Relaxed);
    }
}
