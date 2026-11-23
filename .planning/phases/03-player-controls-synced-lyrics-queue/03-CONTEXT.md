# Phase 3 Context: Player Controls, Synced Lyrics & Queue Management

## Focus
Deliver a world-class listening experience:
1. Native queue management in the Rust core with auto-advance on track completion.
2. Synchronized karaoke lyrics with active timestamp calculation, auto-scrolling, and smooth spring animations.
3. Apple Music-inspired full-screen immersion mode with dynamic ambient colored glow extracted from the album artwork.

## Architectural Requirements
- **Auto-Advance**: When Symphonia reaches EOF, the audio engine automatically pops the next track from the queue and begins streaming without UI intervention.
- **Repeat & Shuffle**: Off, Repeat All, Repeat One, and non-destructive Shuffle.
- **Synced Lyrics Synchronization**: Compute active lyric index via binary search on `bitchord://progress` position in milliseconds (< 1ms CPU overhead).
- **Smooth Fullscreen Transition**: Modal overlay with fluid entrance, displaying large album art, interactive progress bar, lyrics stream, and queue drawer.
