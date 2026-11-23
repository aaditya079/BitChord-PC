import React, { useState, useEffect } from 'react';
import { Titlebar } from './components/Titlebar';
import { SearchBar } from './components/SearchBar';
import { TrackCard } from './components/TrackCard';
import { NowPlayingBar } from './components/NowPlayingBar';
import { FullscreenPlayer } from './components/FullscreenPlayer';
import { OfflineLibraryView } from './components/OfflineLibraryView';
import { useAudioEngine } from './hooks/useAudioEngine';
import { Sparkles, Music2, Compass, HardDriveDownload } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

export const App: React.FC = () => {
  const {
    currentTrack,
    isPlaying,
    positionMs,
    durationMs,
    volume,
    isSearching,
    searchResults,
    queue,
    lyrics,
    isLoadingLyrics,
    repeatMode,
    isShuffle,
    downloadedTracks,
    downloadingIds,
    playTrack,
    togglePlayback,
    setVolume,
    search,
    enqueueTrack,
    removeFromQueue,
    clearQueue,
    skipNext,
    skipPrevious,
    setRepeatMode,
    toggleShuffle,
    downloadTrack,
    deleteDownloadedTrack,
  } = useAudioEngine();

  const [activeTab, setActiveTab] = useState<'explore' | 'offline'>('explore');
  const [isFullscreenOpen, setIsFullscreenOpen] = useState(false);
  const [accentColor, setAccentColor] = useState<{ r: number; g: number; b: number }>({
    r: 244,
    g: 63,
    b: 94,
  });

  const downloadedIdSet = React.useMemo(() => {
    return new Set(downloadedTracks.map((t) => t.video_id));
  }, [downloadedTracks]);

  // Extract dominant color from album artwork & sync OpenRGB peripherals
  useEffect(() => {
    if (!currentTrack?.thumbnail_url) return;

    const img = new Image();
    img.crossOrigin = 'Anonymous';
    img.onload = () => {
      try {
        const canvas = document.createElement('canvas');
        canvas.width = 32;
        canvas.height = 32;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        ctx.drawImage(img, 0, 0, 32, 32);
        const data = ctx.getImageData(0, 0, 32, 32).data;

        let rAcc = 0,
          gAcc = 0,
          bAcc = 0,
          count = 0;

        for (let i = 0; i < data.length; i += 4) {
          const r = data[i];
          const g = data[i + 1];
          const b = data[i + 2];

          const maxC = Math.max(r, g, b);
          const minC = Math.min(r, g, b);
          const brightness = (r + g + b) / 3;

          if (maxC - minC > 20 && brightness > 30 && brightness < 225) {
            rAcc += r;
            gAcc += g;
            bAcc += b;
            count += 1;
          }
        }

        if (count > 0) {
          const newColor = {
            r: Math.round(rAcc / count),
            g: Math.round(gAcc / count),
            b: Math.round(bAcc / count),
          };
          setAccentColor(newColor);
          invoke('sync_ambient_rgb', newColor).catch(() => {});
        }
      } catch (e) {
        console.warn('Could not extract palette:', e);
      }
    };
    img.src = currentTrack.thumbnail_url;
  }, [currentTrack?.video_id]);

  return (
    <div className="relative w-screen h-screen overflow-hidden flex flex-col bg-zinc-950/70 select-none text-zinc-100">
      {/* Frameless OS Titlebar */}
      <Titlebar />

      {/* Dynamic Ambient background glow */}
      <div
        className="absolute top-1/4 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[550px] h-[350px] rounded-full blur-[140px] pointer-events-none transition-all duration-700 ease-out"
        style={{
          backgroundColor: `rgba(${accentColor.r}, ${accentColor.g}, ${accentColor.b}, 0.16)`,
        }}
      />

      {/* Main Content Area */}
      <main className="flex-1 overflow-y-auto pt-14 pb-28 px-6 max-w-5xl mx-auto w-full">
        {/* Navigation Tabs */}
        <div className="flex items-center justify-center my-4">
          <div className="inline-flex p-1 rounded-xl bg-white/[0.04] border border-white/[0.08] backdrop-blur-md">
            <button
              onClick={() => setActiveTab('explore')}
              className={`px-4 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-2 transition-all ${
                activeTab === 'explore'
                  ? 'bg-rose-500/20 text-rose-300 shadow-sm border border-rose-500/30'
                  : 'text-zinc-400 hover:text-white'
              }`}
            >
              <Compass className="w-3.5 h-3.5" />
              <span>Explore</span>
            </button>
            <button
              onClick={() => setActiveTab('offline')}
              className={`px-4 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-2 transition-all ${
                activeTab === 'offline'
                  ? 'bg-rose-500/20 text-rose-300 shadow-sm border border-rose-500/30'
                  : 'text-zinc-400 hover:text-white'
              }`}
            >
              <HardDriveDownload className="w-3.5 h-3.5" />
              <span>Offline Library</span>
              {downloadedTracks.length > 0 && (
                <span className="px-1.5 py-0.2 rounded-full text-[10px] font-mono bg-white/10 text-white/80">
                  {downloadedTracks.length}
                </span>
              )}
            </button>
          </div>
        </div>

        {activeTab === 'offline' ? (
          <OfflineLibraryView
            tracks={downloadedTracks}
            currentTrack={currentTrack}
            isPlaying={isPlaying}
            onPlay={playTrack}
            onEnqueue={enqueueTrack}
            onDeleteTrack={deleteDownloadedTrack}
          />
        ) : (
          <>
            {/* Search Header */}
            <div className="my-6 space-y-4">
              <div className="text-center space-y-1">
                <h1 className="text-2xl font-bold tracking-tight bg-gradient-to-r from-white via-zinc-200 to-zinc-400 bg-clip-text text-transparent">
                  Listen without limits.
                </h1>
                <p className="text-xs text-zinc-400 font-medium">
                  High-fidelity YouTube Music streaming, pure Rust audio core, zero bloat.
                </p>
              </div>

              <SearchBar onSearch={search} isLoading={isSearching} />
            </div>

            {/* Content Section */}
            {searchResults.length > 0 ? (
              <div className="space-y-3">
                <div className="flex items-center justify-between px-1">
                  <h2 className="text-sm font-semibold tracking-wide uppercase text-zinc-400 flex items-center space-x-2">
                    <Sparkles className="w-3.5 h-3.5 text-rose-400" />
                    <span>Search Results ({searchResults.length})</span>
                  </h2>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-2.5">
                  {searchResults.map((track) => (
                    <TrackCard
                      key={track.video_id}
                      track={track}
                      isPlaying={currentTrack?.video_id === track.video_id && isPlaying}
                      isDownloaded={downloadedIdSet.has(track.video_id)}
                      isDownloading={downloadingIds.has(track.video_id)}
                      onPlay={playTrack}
                      onEnqueue={enqueueTrack}
                      onDownload={downloadTrack}
                      onDeleteDownload={deleteDownloadedTrack}
                    />
                  ))}
                </div>
              </div>
            ) : (
              <div className="flex flex-col items-center justify-center py-20 text-center space-y-3">
                <div className="w-14 h-14 rounded-2xl bg-white/[0.04] border border-white/[0.08] flex items-center justify-center text-zinc-500 shadow-inner">
                  <Music2 className="w-7 h-7" />
                </div>
                <div className="space-y-1 max-w-sm">
                  <h3 className="text-sm font-medium text-zinc-300">Start discovering music</h3>
                  <p className="text-xs text-zinc-500">
                    Search for any song, artist, or album above to stream directly with BitChord's native Opus engine.
                  </p>
                </div>
              </div>
            )}
          </>
        )}
      </main>

      {/* Floating Liquid Glass Now-Playing Pill */}
      <NowPlayingBar
        track={currentTrack}
        isPlaying={isPlaying}
        positionMs={positionMs}
        durationMs={durationMs}
        volume={volume}
        onTogglePlay={togglePlayback}
        onSkipNext={skipNext}
        onSkipPrevious={skipPrevious}
        onVolumeChange={setVolume}
        onExpandFullscreen={() => setIsFullscreenOpen(true)}
      />

      {/* Fullscreen Player Modal */}
      <FullscreenPlayer
        isOpen={isFullscreenOpen}
        onClose={() => setIsFullscreenOpen(false)}
        track={currentTrack}
        isPlaying={isPlaying}
        positionMs={positionMs}
        durationMs={durationMs}
        volume={volume}
        lyrics={lyrics}
        isLoadingLyrics={isLoadingLyrics}
        queue={queue}
        repeatMode={repeatMode}
        isShuffle={isShuffle}
        onTogglePlay={togglePlayback}
        onSkipNext={skipNext}
        onSkipPrevious={skipPrevious}
        onVolumeChange={setVolume}
        onRepeatToggle={() => setRepeatMode((repeatMode + 1) % 3)}
        onShuffleToggle={toggleShuffle}
        onPlayTrack={playTrack}
        onRemoveFromQueue={removeFromQueue}
        onClearQueue={clearQueue}
      />
    </div>
  );
};

export default App;
