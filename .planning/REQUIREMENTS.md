# Requirements: BitChord PC v3

**Defined:** 2026-11-23  
**Core Value:** Deliver an ultra-fluid, zero-lag, gapless desktop music listening experience with Apple Music aesthetics, offline sync, and 100% clean, original open-source architecture.

## v1 Requirements

### 1. Audio Engine & Network Pipeline (Rust)
- [ ] **AUDIO-01**: Low-latency, gapless audio playback engine using `cpal` / `symphonia` supporting 48kHz Opus and 44.1kHz AAC streams.
- [ ] **AUDIO-02**: Adaptive chunk buffering with HTTP Range requests from `googlevideo.com` to prevent stutter or playback pause.
- [ ] **AUDIO-03**: Resilient 403 Forbidden auto-recovery with signature renewal and exponential backoff retry.
- [ ] **AUDIO-04**: Volume normalization / ReplayGain support based on YouTube track loudness metrics.

### 2. YouTube Music Innertube API (Rust)
- [ ] **YT-01**: Anonymous and authenticated client queries for Search, Browse, Track info, Next queue, and Explore charts.
- [ ] **YT-02**: In-memory JS cipher / n-parameter challenge solver supporting COPPA / Made-for-Kids tracks and age-restricted streams.
- [ ] **YT-03**: LRCLIB / Musixmatch synced lyrics retrieval with milliseconds-accurate timestamp parsing.

### 3. Tauri v2 Desktop Scaffold & UI (Rust + React/TypeScript)
- [ ] **UI-01**: Custom frameless Windows shell with native drag regions, minimize, maximize/restore, and close buttons.
- [ ] **UI-02**: Floating liquid glass now-playing pill capsule with track artwork, title, artist, progress scrubber, and transport controls.
- [ ] **UI-03**: Full-screen / expanded player view with dynamic ambient background gradient generated from album art palette.
- [ ] **UI-04**: Synced lyric view with smooth line auto-scrolling and active syllable/line highlight animations.
- [ ] **UI-05**: Responsive queue drawer with drag-and-drop reordering and track removal.

### 4. OS Integrations (Rust)
- [ ] **OS-01**: Windows System Media Transport Controls (SMTC) integration with taskbar thumbnail controls and media key hooks.
- [ ] **OS-02**: Discord Rich Presence showing track title, artist, elapsed/total duration, and live playing state.
- [ ] **OS-03**: Universal Ambient RGB sync supporting OpenRGB, Razer Chroma, and Corsair iCUE.

### 5. Offline & Downloads
- [ ] **SYNC-01**: One-click download of individual songs and playlists with embedded ID3v2/Vorbis tags and high-res cover art.
- [ ] **SYNC-02**: Local library browser allowing instant offline playback of cached tracks without internet connection.

---

## Out of Scope
| Feature | Reason |
|---------|--------|
| ytm-rs or egui code reuse | Clean-room development from scratch to avoid any copyright or attribution conflict. |
| Heavy Chromium / Electron bundler | Tauri v2 utilizes native Windows WebView2, reducing memory usage from ~400MB down to ~75MB. |

---

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| AUDIO-01, AUDIO-02, AUDIO-03, AUDIO-04 | Phase 1 | Pending |
| YT-01, YT-02, YT-03 | Phase 1 | Pending |
| UI-01, UI-02, UI-03 | Phase 2 | Pending |
| UI-04, UI-05 | Phase 3 | Pending |
| OS-01, OS-02, OS-03 | Phase 4 | Pending |
| SYNC-01, SYNC-02 | Phase 5 | Pending |
