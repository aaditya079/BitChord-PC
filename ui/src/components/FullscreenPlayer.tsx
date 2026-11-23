import React, { useState } from 'react';
import { Track, LyricLine } from '../hooks/useAudioEngine';
import { LyricsView } from './LyricsView';
import { QueueDrawer } from './QueueDrawer';
import {
  Play,
  Pause,
  SkipBack,
  SkipForward,
  Shuffle,
  Repeat,
  Repeat1,
  Volume2,
  VolumeX,
  ChevronDown,
  Music,
  ListMusic,
  Mic2,
} from 'lucide-react';

interface FullscreenPlayerProps {
  isOpen: boolean;
  onClose: () => void;
  track: Track | null;
  isPlaying: boolean;
  positionMs: number;
  durationMs: number;
  volume: number;
  lyrics: LyricLine[];
  isLoadingLyrics: boolean;
  queue: Track[];
  repeatMode: number;
  isShuffle: boolean;
  onTogglePlay: () => void;
  onSkipNext: () => void;
  onSkipPrevious: () => void;
  onVolumeChange: (vol: number) => void;
  onRepeatToggle: () => void;
  onShuffleToggle: () => void;
  onPlayTrack: (track: Track) => void;
  onRemoveFromQueue: (index: number) => void;
  onClearQueue: () => void;
}

function formatTime(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

export const FullscreenPlayer: React.FC<FullscreenPlayerProps> = ({
  isOpen,
  onClose,
  track,
  isPlaying,
  positionMs,
  durationMs,
  volume,
  lyrics,
  isLoadingLyrics,
  queue,
  repeatMode,
  isShuffle,
  onTogglePlay,
  onSkipNext,
  onSkipPrevious,
  onVolumeChange,
  onRepeatToggle,
  onShuffleToggle,
  onPlayTrack,
  onRemoveFromQueue,
  onClearQueue,
}) => {
  const [activeTab, setActiveTab] = useState<'lyrics' | 'queue'>('lyrics');

  if (!isOpen || !track) return null;

  const progressPercent = durationMs > 0 ? (positionMs / durationMs) * 100 : 0;

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-zinc-950/90 backdrop-blur-3xl animate-in fade-in duration-300 select-none overflow-hidden">
      {/* Dynamic Ambient Background Glow */}
      <div className="absolute top-1/3 left-1/3 w-[600px] h-[600px] bg-rose-600/15 rounded-full blur-[160px] pointer-events-none" />
      <div className="absolute bottom-10 right-1/4 w-[500px] h-[500px] bg-violet-600/10 rounded-full blur-[140px] pointer-events-none" />

      {/* Top Bar with Dismiss and Tab Switcher */}
      <header className="h-16 px-8 flex items-center justify-between border-b border-white/[0.06] relative z-10">
        <button
          onClick={onClose}
          className="w-9 h-9 rounded-full bg-white/[0.06] hover:bg-white/[0.12] flex items-center justify-center text-white/70 hover:text-white transition-colors"
          title="Minimize player"
        >
          <ChevronDown className="w-5 h-5" />
        </button>

        <div className="flex items-center space-x-1 p-1 rounded-full bg-white/[0.06] border border-white/[0.08]">
          <button
            onClick={() => setActiveTab('lyrics')}
            className={`flex items-center space-x-1.5 px-4 py-1.5 rounded-full text-xs font-semibold transition-all ${
              activeTab === 'lyrics'
                ? 'bg-rose-500 text-white shadow-md'
                : 'text-white/60 hover:text-white'
            }`}
          >
            <Mic2 className="w-3.5 h-3.5" />
            <span>Lyrics</span>
          </button>
          <button
            onClick={() => setActiveTab('queue')}
            className={`flex items-center space-x-1.5 px-4 py-1.5 rounded-full text-xs font-semibold transition-all ${
              activeTab === 'queue'
                ? 'bg-rose-500 text-white shadow-md'
                : 'text-white/60 hover:text-white'
            }`}
          >
            <ListMusic className="w-3.5 h-3.5" />
            <span>Playing Next</span>
          </button>
        </div>

        <div className="w-9" />
      </header>

      {/* Two Column Layout */}
      <div className="flex-1 flex max-w-6xl mx-auto w-full p-8 gap-12 overflow-hidden relative z-10">
        {/* Left Column: Artwork and Transport Controls */}
        <div className="w-1/2 flex flex-col justify-center items-center max-w-md mx-auto space-y-6">
          <div className="relative w-72 h-72 md:w-80 md:h-80 rounded-3xl overflow-hidden shadow-2xl border border-white/10 group">
            {track.thumbnail_url ? (
              <img
                src={track.thumbnail_url}
                alt={track.title}
                className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
              />
            ) : (
              <div className="w-full h-full flex items-center justify-center bg-white/5 text-white/30">
                <Music className="w-16 h-16" />
              </div>
            )}
          </div>

          {/* Track Details */}
          <div className="w-full text-left space-y-1">
            <h2 className="text-2xl font-bold text-white tracking-tight truncate">
              {track.title}
            </h2>
            <p className="text-base text-white/60 truncate">
              {track.artists.join(', ')}
            </p>
          </div>

          {/* Progress Scrubber */}
          <div className="w-full space-y-1.5">
            <div className="relative h-1.5 bg-white/10 rounded-full overflow-hidden cursor-pointer group">
              <div
                className="h-full bg-rose-500 rounded-full group-hover:bg-rose-400 transition-all duration-100"
                style={{ width: `${Math.min(100, Math.max(0, progressPercent))}%` }}
              />
            </div>
            <div className="flex justify-between text-xs font-mono text-white/40">
              <span>{formatTime(positionMs)}</span>
              <span>{formatTime(durationMs)}</span>
            </div>
          </div>

          {/* Transport Controls */}
          <div className="flex items-center justify-between w-full px-4">
            <button
              onClick={onShuffleToggle}
              className={`p-2 transition-colors ${
                isShuffle ? 'text-rose-400' : 'text-white/40 hover:text-white'
              }`}
              title="Shuffle"
            >
              <Shuffle className="w-5 h-5" />
            </button>

            <button
              onClick={onSkipPrevious}
              className="p-2 text-white/80 hover:text-white active:scale-95 transition-all"
              title="Previous"
            >
              <SkipBack className="w-6 h-6 fill-current" />
            </button>

            <button
              onClick={onTogglePlay}
              className="w-14 h-14 rounded-full bg-gradient-to-tr from-rose-600 to-rose-400 hover:from-rose-500 hover:to-rose-300 flex items-center justify-center text-white shadow-xl hover:scale-105 active:scale-95 transition-all"
            >
              {isPlaying ? (
                <Pause className="w-6 h-6 fill-current" />
              ) : (
                <Play className="w-6 h-6 fill-current translate-x-0.5" />
              )}
            </button>

            <button
              onClick={onSkipNext}
              className="p-2 text-white/80 hover:text-white active:scale-95 transition-all"
              title="Next"
            >
              <SkipForward className="w-6 h-6 fill-current" />
            </button>

            <button
              onClick={onRepeatToggle}
              className={`p-2 transition-colors ${
                repeatMode > 0 ? 'text-rose-400' : 'text-white/40 hover:text-white'
              }`}
              title={`Repeat: ${repeatMode === 2 ? 'One' : repeatMode === 1 ? 'All' : 'Off'}`}
            >
              {repeatMode === 2 ? (
                <Repeat1 className="w-5 h-5" />
              ) : (
                <Repeat className="w-5 h-5" />
              )}
            </button>
          </div>

          {/* Volume Slider */}
          <div className="flex items-center space-x-3 w-full px-6 pt-2">
            <button
              onClick={() => onVolumeChange(volume > 0 ? 0 : 0.8)}
              className="text-white/40 hover:text-white transition-colors"
            >
              {volume === 0 ? <VolumeX className="w-4 h-4" /> : <Volume2 className="w-4 h-4" />}
            </button>
            <input
              type="range"
              min="0"
              max="1"
              step="0.02"
              value={volume}
              onChange={(e) => onVolumeChange(parseFloat(e.target.value))}
              className="w-full h-1 bg-white/10 accent-rose-500 rounded-full appearance-none cursor-pointer"
            />
          </div>
        </div>

        {/* Right Column: Lyrics or Queue */}
        <div className="w-1/2 h-full rounded-3xl glass-panel p-2 overflow-hidden flex flex-col">
          {activeTab === 'lyrics' ? (
            <LyricsView
              lyrics={lyrics}
              positionMs={positionMs}
              isLoading={isLoadingLyrics}
            />
          ) : (
            <QueueDrawer
              queue={queue}
              onPlay={onPlayTrack}
              onRemove={onRemoveFromQueue}
              onClear={onClearQueue}
            />
          )}
        </div>
      </div>
    </div>
  );
};
