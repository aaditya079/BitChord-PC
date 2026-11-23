## BitChord PC v2.1.0 - Community Continued Edition

A high-performance, open-source desktop client for BitChord and YouTube Music with clean Apple Music-inspired aesthetics.

> **Open-Source Attribution & Credits:**
> * Forked and evolved from [**grapedjuice/ytm-rs**](https://github.com/grapedjuice/ytm-rs) (original desktop engine).
> * Based on design, aesthetics, and mobile parity with [**kushagrasinghx/BitChord**](https://github.com/kushagrasinghx/BitChord).
> * Licensed under **GNU General Public License v3.0 (GPL-3.0)**. Full credit and gratitude to grapedjuice, Kushagra Singh, and community contributors.

---

### What's New in v2.1.0:

#### Major Features
* **Universal Ambient RGB & Keyboard Sync**: Seamlessly syncs track artwork palette with Corsair iCUE, Razer Chroma, ASUS Aura / ROG TUF, Logitech LIGHTSYNC, and OpenRGB.
* **Offline Library Sync & Batch Downloader**: Download individual tracks or entire playlists with multi-threaded downloads, ID3/Vorbis tag embedding, and local disk synchronization.
* **Background Blur Power-Saver**: Optional toggle in Settings -> Appearance to disable heavy background Gaussian blur for power-saving on laptops and low-spec GPUs.

#### Fixes & Engine Reliability
* **Kids / COPPA Music Playback Fix** (Closes #23): Implemented WEB_REMIX client cipher solver fallback via embedded QuickJS jsc so nursery rhymes, soundtracks, and children-designated songs never get skipped.
* **403 Forbidden Stream Auto-Recovery** (Closes #38): Raised chunk retry budget with exponential jittered backoff and direct signature renewal for uninterrupted playback during long listening sessions.
* **Now-Playing Bar Visual Artifacts Backing** (Closes #22): Added solid base backing under floating pill capsules to eliminate text bleeding and blur shadow artifacts.
* **Adaptive Light Theme Thumbnail Controls** (Closes #36): Windows Taskbar preview buttons dynamically invert on light theme backgrounds.
* **Hi-DPI Vector Mipmaps** (Closes #21, #34): Crisp rendering for Apple Digital Master and Dolby Atmos badges across 100%, 125%, 150%, and 200% Windows display scaling.

---

### Installation
* **Installer**: Download and run **bitchord-setup-2.1.0.exe** (recommended, requires Windows 10/11 64-bit).
* **Portable**: Download **bitchord-v2.1.0-windows-x64.zip**, extract anywhere, and run **bitchord.exe**.
