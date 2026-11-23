# BitChord PC v3 (Clean-Slate Architecture)

## What This Is

A ground-up, clean-slate desktop YouTube Music client engineered with a pure Rust streaming audio core and an Apple Music-inspired Tauri v2 frontend. Built to deliver 1:1 design and feature parity with Kushagra Singh's mobile BitChord client, with zero legacy code or ethical ambiguity from upstream forks.

## Core Value

Deliver an ultra-fluid, zero-lag, gapless desktop music listening experience with Apple Music aesthetics, offline sync, and 100% clean, original open-source architecture.

## Requirements

### Validated
(None yet — greenfield build)

### Active
- [ ] **ARCH-01**: Clean-slate Rust audio pipeline (Symphonia / CPAL / Rodio) supporting Opus, WebM, and AAC decoding.
- [ ] **ARCH-02**: Reverse-engineered Innertube API client in Rust with automatic cipher/n-sig challenge solver.
- [ ] **ARCH-03**: Tauri v2 application scaffold with custom frameless window, Mica/acrylic backdrop blur, and smooth animations.
- [ ] **UI-01**: 1:1 BitChord mobile aesthetic parity: Liquid glass now-playing bar, animated vinyl/album art carousel, and clean typography.
- [ ] **UI-02**: Synchronized timestamped karaoke lyrics with smooth auto-scroll.
- [ ] **OS-01**: Native Windows SMTC integration (taskbar thumbnail controls, global media hotkeys, OS volume flyout).
- [ ] **OS-02**: Discord Rich Presence with dynamic album artwork and live playback timeline.
- [ ] **FEAT-01**: Universal ambient lighting sync (iCUE, Razer Chroma, ASUS Aura / ROG TUF, OpenRGB).
- [ ] **FEAT-02**: Local music caching and playlist download manager with ID3 metadata embedding.

### Out of Scope
- Reusing or vendoring code from `grapedjuice/ytm-rs` or `jackychan4891/BitChord-PC-Official` — strictly clean-room implementation.
- Heavy Chromium / Electron runtime — strictly lightweight Tauri v2 (system WebView2 on Windows) to keep memory footprint under 90MB.

## Context

- **Ecosystem**: BitChord is an established open-source YouTube Music mobile client (7.2k+ GitHub stars) by Kushagra Singh.
- **Problem Solved**: Previous PC ports were controversial forks of `ytm-rs` that stripped credits. BitChord PC v3 is a fresh, original, clean-room community initiative built in close alignment with BitChord's design language.
- **Hardware Profile**: Engineered for Windows 10/11 64-bit systems, optimized for low VRAM and CPU efficiency.

## Constraints & Performance Targets

- **Memory Ceiling:** Total client idle RAM consumption must stay strictly under **70 MB** (vs 400MB+ in typical Electron apps).
- **Real-Time Audio Invariant:** Zero heap allocations, zero mutex locks, and zero file I/O inside the `cpal` audio callback thread.
- **Streaming Eviction:** Decode audio on-the-fly; never buffer uncompressed PCM for long tracks.
- **CPU Overhead:** Target < 0.5% CPU utilization during background playback on modern multi-core x86_64 CPUs.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Tauri v2 over egui / Iced | Matches mobile's rich blur/glassmorphism aesthetics via CSS while keeping audio engine in high-performance Rust | ✓ Good |
| Pure Rust Audio Backend | Zero playback stuttering, bit-perfect gapless audio, and full offline caching control | ✓ Good |
| Independent Clean-Room Repo | Eliminates all licensing baggage, fork drama, and GitHub fork-tree yellow banner clutter | ✓ Good |

---
*Last updated: 2026-11-23 after GSD setup*
