use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

pub struct PaletteExtractor;

impl PaletteExtractor {
    /// Extracts a vibrant dominant color from downsampled raw RGB/RGBA pixels.
    pub fn extract_dominant_color(pixels: &[u8], bytes_per_pixel: usize) -> RgbColor {
        if pixels.is_empty() || bytes_per_pixel < 3 {
            return RgbColor::new(244, 63, 94); // Default BitChord Rose
        }

        let mut r_acc = 0u64;
        let mut g_acc = 0u64;
        let mut b_acc = 0u64;
        let mut count = 0u64;

        let step = (bytes_per_pixel * 4).max(bytes_per_pixel); // Fast sampling
        let mut i = 0;
        while i + 2 < pixels.len() {
            let r = pixels[i] as u64;
            let g = pixels[i + 1] as u64;
            let b = pixels[i + 2] as u64;

            // Skip dull grays, harsh whites, and deep blacks
            let max_c = r.max(g).max(b);
            let min_c = r.min(g).min(b);
            let delta = max_c - min_c;
            let brightness = (r + g + b) / 3;

            if delta > 20 && brightness > 30 && brightness < 225 {
                r_acc += r;
                g_acc += g;
                b_acc += b;
                count += 1;
            }

            i += step;
        }

        if count > 0 {
            RgbColor::new(
                (r_acc / count) as u8,
                (g_acc / count) as u8,
                (b_acc / count) as u8,
            )
        } else {
            RgbColor::new(244, 63, 94)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_dominant_color() {
        // Sample buffer of bright teal pixels
        let mut buf = Vec::new();
        for _ in 0..100 {
            buf.extend_from_slice(&[20, 200, 180, 255]);
        }
        let color = PaletteExtractor::extract_dominant_color(&buf, 4);
        assert_eq!(color.r, 20);
        assert_eq!(color.g, 200);
        assert_eq!(color.b, 180);
    }
}
