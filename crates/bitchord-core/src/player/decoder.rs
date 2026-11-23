use crate::stream::buffer::StreamBuffer;
use std::time::Duration;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("Failed to probe audio format: {0}")]
    ProbeError(#[from] SymphoniaError),
    #[error("No compatible audio track found in media container")]
    NoTrackFound,
    #[error("Audio format not supported")]
    UnsupportedFormat,
}

/// On-the-fly audio decoder wrapping Symphonia.
/// Never stores full uncompressed PCM in memory; decodes chunk packets on-demand.
pub struct AudioDecoder {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    pub sample_rate: u32,
    pub channels: usize,
    pub duration_secs: Option<f64>,
}

impl AudioDecoder {
    pub fn new(stream_buffer: StreamBuffer, codec_hint: &str) -> Result<Self, DecoderError> {
        let mss = MediaSourceStream::new(Box::new(stream_buffer), Default::default());

        let mut hint = Hint::new();
        if codec_hint == "opus" {
            hint.with_extension("webm");
        } else if codec_hint == "aac" {
            hint.with_extension("m4a");
        }

        let format_opts = FormatOptions {
            enable_gapless: true,
            ..Default::default()
        };
        let meta_opts: MetadataOptions = Default::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &meta_opts)
            .map_err(DecoderError::ProbeError)?;

        let format_reader = probed.format;

        // Find the first valid audio track
        let track = format_reader
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or(DecoderError::NoTrackFound)?;

        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(48000);
        let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);

        let duration_secs = track.codec_params.n_frames.map(|f| (f as f64) / (sample_rate as f64));

        let dec_opts: DecoderOptions = Default::default();
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .map_err(DecoderError::ProbeError)?;

        Ok(Self {
            format_reader,
            decoder,
            track_id,
            sample_rate,
            channels,
            duration_secs,
        })
    }

    /// Decodes the next packet into an interleaved f32 sample vector.
    /// Returns None when reaching the end of the stream.
    pub fn next_packet_samples(&mut self) -> Result<Option<Vec<f32>>, DecoderError> {
        loop {
            let packet = match self.format_reader.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Ok(None);
                }
                Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::TimedOut => {
                    return Ok(Some(Vec::new())); // Wait for buffer
                }
                Err(e) => return Err(DecoderError::ProbeError(e)),
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    let mut samples = Vec::new();
                    Self::convert_audio_buffer(&decoded, &mut samples);
                    return Ok(Some(samples));
                }
                Err(SymphoniaError::DecodeError(e)) => {
                    warn!("Recoverable packet decode error: {:?}", e);
                    continue;
                }
                Err(e) => return Err(DecoderError::ProbeError(e)),
            }
        }
    }

    /// Seeks playback to a specific timestamp.
    pub fn seek(&mut self, time: Duration) -> Result<(), DecoderError> {
        let seek_to = SeekTo::Time {
            time: symphonia::core::units::Time::from(time.as_secs_f64()),
            track_id: Some(self.track_id),
        };

        self.format_reader
            .seek(SeekMode::Accurate, seek_to)
            .map_err(DecoderError::ProbeError)?;

        self.decoder.reset();
        Ok(())
    }

    fn convert_audio_buffer(buf: &AudioBufferRef, out: &mut Vec<f32>) {
        match buf {
            AudioBufferRef::F32(buf) => {
                let channels = buf.spec().channels.count();
                let frames = buf.frames();
                out.reserve(frames * channels);
                for frame in 0..frames {
                    for ch in 0..channels {
                        out.push(buf.chan(ch)[frame]);
                    }
                }
            }
            AudioBufferRef::S16(buf) => {
                let channels = buf.spec().channels.count();
                let frames = buf.frames();
                out.reserve(frames * channels);
                for frame in 0..frames {
                    for ch in 0..channels {
                        let sample = buf.chan(ch)[frame] as f32 / 32768.0;
                        out.push(sample);
                    }
                }
            }
            AudioBufferRef::U8(buf) => {
                let channels = buf.spec().channels.count();
                let frames = buf.frames();
                out.reserve(frames * channels);
                for frame in 0..frames {
                    for ch in 0..channels {
                        let sample = (buf.chan(ch)[frame] as f32 - 128.0) / 128.0;
                        out.push(sample);
                    }
                }
            }
            _ => {
                // Unsupported sample format fallback
            }
        }
    }
}
