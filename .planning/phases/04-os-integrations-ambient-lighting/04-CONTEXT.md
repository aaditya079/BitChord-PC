# Phase 4 Context: OS Integrations & Ambient Lighting

## Focus
Seamlessly weave BitChord PC into the desktop operating system:
1. **Windows System Media Transport Controls (SMTC)**:
   - Handle hardware media keys (Play/Pause, Next, Prev, Stop).
   - Display track metadata (Title, Artist, Album, Thumbnail) in the Windows taskbar preview and volume overlay flyout.
   - React to OS transport events to toggle or advance playback in `AudioEngine`.
2. **Discord Rich Presence (RPC)**:
   - Connect to local Discord named pipe (`discord-ipc-0`).
   - Broadcast live presence: "Listening to <Title>", by "<Artist>", with high-res cover art and live timestamp progress.
3. **OpenRGB Ambient Palette Sync**:
   - Sample the active track's cover thumbnail to compute the dominant RGB color.
   - Connect via OpenRGB SDK TCP protocol (`127.0.0.1:6742`) to sync keyboards, RGB RAM, fans, and desk strips dynamically to the music's aesthetic.

## Architecture & Soundness
- Non-blocking background workers: Network or IPC glitches in Discord RPC or OpenRGB must NEVER block or stutter the real-time audio playback pump.
- Graceful degradation: If Discord is not running or OpenRGB server is offline, the integrations silently sleep and retry periodically without error popups or CPU waste.
