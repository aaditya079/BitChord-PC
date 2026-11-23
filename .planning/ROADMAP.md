# Roadmap: BitChord PC v3

## Overview

A 5-phase greenfield journey building a clean-slate, high-performance BitChord desktop client on Tauri v2 and pure Rust. From core audio streaming architecture up to animated liquid glass UI, OS-level integrations, and offline library sync.

## Phases

- [ ] **Phase 1: Rust Audio Engine & Innertube Streaming Pipeline** - Build standalone Rust streaming core, chunk buffer, and YouTube deciphering pipeline.
- [ ] **Phase 2: Tauri v2 Scaffold & Liquid Glass UI** - Initialize Tauri v2 shell, custom window decorations, now-playing bar, and home feed.
- [ ] **Phase 3: Player Controls, Synced Lyrics & Queue Management** - Implement expanded now-playing view, lyrics sync engine, and queue manager.
- [ ] **Phase 4: OS Integrations & Ambient Lighting** - Wire Windows SMTC, Discord Rich Presence, global media hotkeys, and RGB hardware sync.
- [ ] **Phase 5: Offline Library Sync & Production Release** - Add batch downloader, local tagger, offline playback, and CI/CD release workflow.

---

## Phase Details

### Phase 1: Rust Audio Engine & Innertube Streaming Pipeline
**Goal**: Establish a bulletproof, gapless audio player and YouTube stream resolver in Rust with zero external UI dependencies.
**Depends on**: Nothing (Phase 1)
**Requirements**: AUDIO-01, AUDIO-02, AUDIO-03, AUDIO-04, YT-01, YT-02, YT-03
**Success Criteria**:
  1. Rust unit tests verify playback of 48kHz Opus and 44.1kHz AAC streams through `cpal`/`symphonia`.
  2. Range-based chunk downloader streams uninterrupted without 403 throttling or stalls.
  3. YouTube Innertube client successfully resolves search queries, playlists, and COPPA/kids tracks.
**Plans**: 3 plans

Plans:
- [ ] 01-01: Workspace Scaffold & Innertube Client with Challenge Solver
- [ ] 01-02: Zero-Copy Chunk Downloader & Adaptive Range Buffering
- [ ] 01-03: Real-Time Non-Blocking Audio Pipeline (symphonia + cpal + rtrb)

---

### Phase 2: Tauri v2 Scaffold & Liquid Glass UI
**Goal**: Bootstrap Tauri v2 desktop application with Apple Music-inspired dark aesthetic, fluid layout, and now-playing strip.
**Depends on**: Phase 1
**Requirements**: UI-01, UI-02, UI-03
**Success Criteria**:
  1. Frameless window renders smoothly with custom minimize/maximize/close and OS snap layouts.
  2. Floating now-playing bar displays live track info with playback controls bound to the Rust audio engine.
  3. Home feed browses charts, search results, and album pages smoothly with lazy-loaded art.
**Plans**: 2 plans

---

### Phase 3: Player Controls, Synced Lyrics & Queue Management
**Goal**: Build the full-screen immersive listening view with synchronized karaoke lyrics and queue manipulation.
**Depends on**: Phase 2
**Requirements**: UI-04, UI-05
**Success Criteria**:
  1. Expanding now-playing bar opens full-screen player with dynamic album art ambient background.
  2. Synced lyrics scroll smoothly with active line emphasis in real-time.
  3. Up-next queue supports reordering, removing, and quick adding tracks.
**Plans**: 2 plans

---

### Phase 4: OS Integrations & Ambient Lighting
**Goal**: Connect the desktop client into the Windows ecosystem and RGB peripherals.
**Depends on**: Phase 3
**Requirements**: OS-01, OS-02, OS-03
**Success Criteria**:
  1. Windows taskbar thumbnail preview shows playback buttons and track metadata via SMTC.
  2. Discord profile reflects current listening status with album artwork and time elapsed.
  3. Keyboard and case RGB dynamically shifts to match primary palette colors of current album art.
**Plans**: 2 plans

---

### Phase 5: Offline Library Sync & Production Release
**Goal**: Enable complete offline listening, playlist download management, and generate release installers.
**Depends on**: Phase 4
**Requirements**: SYNC-01, SYNC-02
**Success Criteria**:
  1. User can download songs or playlists with a single click, saving high-bitrate audio with ID3/Vorbis tags.
  2. App functions seamlessly without internet connection, playing all locally cached files.
  3. Automated build scripts produce signed/portable Windows `.exe` and `.msi` installers.
**Plans**: 2 plans
