import React, { useState } from 'react';
import { Titlebar } from './components/Titlebar';
import { SearchBar } from './components/SearchBar';
import { TrackCard } from './components/TrackCard';
import { NowPlayingBar } from './components/NowPlayingBar';
import { useAudioEngine, Track } from './hooks/useAudioEngine';
import { Sparkles, Music2, Radio } from 'lucide-react';

export const App: React.FC = () => {
  const {
    currentTrack,
    isPlaying,
    positionMs,
    durationMs,
    volume,
    isSearching,
    searchResults,
    playTrack,
    togglePlayback,
    setVolume,
    search,
  } = useAudioEngine();

  const [activeTab, setActiveTab] = useState<'search' | 'trending'>('search');

  return (
    <div className="relative w-screen h-screen overflow-hidden flex flex-col bg-zinc-950/70 select-none text-zinc-100">
      {/* Frameless OS Titlebar */}
      <Titlebar />

      {/* Ambient background glow */}
      <div className="absolute top-1/4 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[300px] bg-rose-600/10 rounded-full blur-[120px] pointer-events-none" />

      {/* Main Content Area */}
      <main className="flex-1 overflow-y-auto pt-14 pb-28 px-6 max-w-5xl mx-auto w-full">
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
                  onPlay={playTrack}
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
      </main>

      {/* Floating Liquid Glass Now-Playing Pill */}
      <NowPlayingBar
        track={currentTrack}
        isPlaying={isPlaying}
        positionMs={positionMs}
        durationMs={durationMs}
        volume={volume}
        onTogglePlay={togglePlayback}
        onVolumeChange={setVolume}
      />
    </div>
  );
};

export default App;
