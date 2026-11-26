# Changelog

All notable changes to **BitChord PC** are documented in this file.

## [2.2.0] - 2026-11-26

### Added
- **Damped Spring Physics & Liquid Glass UI** (`src/theme.rs`, `src/ui/`): Sub-stepped harmonic oscillator (`Spring`) driving Now Playing, Queue drawer, volume controls, and card hover bounce (`0 FPS` idle when settled, respects Reduce Motion), plus specular frosted borders (`glass_stroke`) and a cached `128x128` offscreen radial player glow.
- **16-Band Live Audio Spectrum Visualizer** (`src/audio/spectrum.rs`, `src/ui/nowplaying.rs`): Zero-allocation biquad bandpass filter bank (`60 Hz – 14 kHz`) rendered beneath the Now Playing transport.
- **Beat-Reactive Hardware RGB** (`src/lighting.rs`, `src/ui/pages.rs`): Optional audio-loudness brightness modulation for ASUS TUF/ROG, OpenRGB, and SteelSeries backlights while preserving the 250 ms hardware rate-limiter.
- **Sleep Timer** (`src/ui/shell.rs`, `src/ui/responses.rs`): `15m`, `30m`, `45m`, `60m`, and `End of current track` options in the Queue drawer header.
- **Listening Replay & Provider Health** (`src/listening.rs`, `src/providers.rs`, `src/ui/pages.rs`): Local Listening Replay summary card in **Library › History** and live latency/degradation status badges in **Settings › Music sources**.

### Fixed
- Top-left app logo now loads the full-resolution master icon (`512x512` / `256x256`) with hardware trilinear mipmapping (`src/ui/shell.rs`).
- Removed unsupported cross-target macOS check on Linux CI runners and aligned Clippy `-D warnings` checks.

## [2.1.1] - 2026-11-24

### Security
- Narrowed Windows `\\.\ATKACPI` hardware guard to verified ASUS TUF/ROG SMBIOS/Registry identifiers with RAII handle cleanup.
- Locked down the auto-updater with HTTPS-only URLs, SHA-256 checksum verification, and prerelease filtering.

### Added
- Automatic updates toggle and silent install on app exit (`Settings › Updates`).
- Power Saver blur toggle (`< 0.2%` GPU shader load) and nightly CI build workflow.

### Fixed
- Stream `403 Forbidden` auto-renewal with exponential backoff and WebMusic cipher fallback for restricted tracks.
- Increased provider search/probe/connect timeouts to prevent Monochrome timeouts.
