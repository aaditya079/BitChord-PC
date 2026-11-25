//! Zero-allocation 16-band real-time audio spectrum analyzer using biquad bandpass filters.

use std::f32::consts::PI;
use std::sync::atomic::{AtomicU32, Ordering};

pub const BANDS: usize = 16;

/// Logarithmically spaced center frequencies (Hz) from sub-bass to air.
const CENTERS: [f32; BANDS] = [
    60.0, 90.0, 135.0, 200.0, 300.0, 450.0, 675.0, 1000.0,
    1500.0, 2250.0, 3350.0, 5000.0, 7500.0, 10000.0, 12000.0, 14000.0,
];

#[derive(Clone, Copy)]
struct BandFilter {
    b0: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    acc: f32,
    smooth: f32,
}

impl BandFilter {
    fn new(freq: f32, sample_rate: f32) -> Self {
        let f = freq.min(sample_rate * 0.45);
        let w0 = 2.0 * PI * f / sample_rate;
        let q = 3.0;
        let alpha = w0.sin() / (2.0 * q);
        let a0 = 1.0 + alpha;
        Self {
            b0: alpha / a0,
            b2: -alpha / a0,
            a1: (-2.0 * w0.cos()) / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            acc: 0.0,
            smooth: 0.0,
        }
    }

    #[inline]
    fn process(&mut self, x: f32) {
        let y = self.b0 * x + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        self.acc += y * y;
    }

    fn finish_window(&mut self, n: f32) -> f32 {
        let rms = (self.acc / n.max(1.0)).sqrt();
        self.acc = 0.0;
        // Map -54 dB..0 dB perceptually into 0.0..1.0.
        let db = 20.0 * (rms + 1e-6).log10();
        let norm = ((db + 54.0) / 54.0).clamp(0.0, 1.0);
        let k = if norm > self.smooth { 0.65 } else { 0.14 };
        self.smooth += (norm - self.smooth) * k;
        self.smooth
    }
}

pub struct SpectrumBank {
    bands: [BandFilter; BANDS],
    channels: u16,
    ch_idx: u16,
    mono_acc: f32,
    frames: u32,
}

impl SpectrumBank {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        let sr = sample_rate.max(8000) as f32;
        let bands = std::array::from_fn(|i| BandFilter::new(CENTERS[i], sr));
        Self { bands, channels: channels.max(1), ch_idx: 0, mono_acc: 0.0, frames: 0 }
    }

    /// Feeds one interleaved sample; updates `out` every ~1024 frames (~23 ms at 44.1 kHz).
    #[inline]
    pub fn push(&mut self, sample: f32, out: Option<&[AtomicU32; BANDS]>) {
        self.mono_acc += sample;
        self.ch_idx += 1;
        if self.ch_idx < self.channels {
            return;
        }
        let mono = self.mono_acc / self.channels as f32;
        self.mono_acc = 0.0;
        self.ch_idx = 0;

        for b in &mut self.bands {
            b.process(mono);
        }
        self.frames += 1;
        if self.frames >= 1024 {
            let n = self.frames as f32;
            for (i, b) in self.bands.iter_mut().enumerate() {
                let v = b.finish_window(n);
                if let Some(atomics) = out {
                    atomics[i].store(v.to_bits(), Ordering::Relaxed);
                }
            }
            self.frames = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_wave_peaks_in_matching_frequency_band() {
        let sr = 44100;
        let out: [AtomicU32; BANDS] = std::array::from_fn(|_| AtomicU32::new(0));

        let mut low_bank = SpectrumBank::new(sr, 1);
        for i in 0..4096 {
            let s = (2.0 * PI * 90.0 * i as f32 / sr as f32).sin() * 0.8;
            low_bank.push(s, Some(&out));
        }
        let low_vals: [f32; BANDS] = std::array::from_fn(|i| f32::from_bits(out[i].load(Ordering::Relaxed)));
        let low_peak = low_vals.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap().0;
        assert!(low_peak <= 2, "expected 90 Hz sine to peak in bass bands, got band {low_peak}");

        let mut high_bank = SpectrumBank::new(sr, 1);
        for i in 0..4096 {
            let s = (2.0 * PI * 3350.0 * i as f32 / sr as f32).sin() * 0.8;
            high_bank.push(s, Some(&out));
        }
        let high_vals: [f32; BANDS] = std::array::from_fn(|i| f32::from_bits(out[i].load(Ordering::Relaxed)));
        let high_peak = high_vals.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap().0;
        assert!((9..=12).contains(&high_peak), "expected 3350 Hz sine to peak in upper-mid bands, got band {high_peak}");
    }
}
