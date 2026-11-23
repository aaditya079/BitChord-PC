import React from 'react';
import { Play, Pause, Volume2, VolumeX, Music } from 'lucide-react';
import { Track } from '../hooks/useAudioEngine';

interface NowPlayingBarProps {
  track: Track | null;
  isPlaying: boolean;
  positionMs: number;
  durationMs: number;
  volume: number;
  onTogglePlay: () => void;
  onVolumeChange: (vol: number) => void;
}

function formatTime(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

export const NowPlayingBar: React.FC<NowPlayingBarProps> = ({
  track,
  isPlaying,
  positionMs,
  durationMs,
  volume,
  onTogglePlay,
  onVolumeChange,
}) => {
  if (!track) return null;

  const progressPercent = durationMs > 0 ? (positionMs / durationMs) * 100 : 0;

  return (
    <div className="fixed bottom-5 left-1/2 -translate-x-1/2 w-[92%] max-w-3xl h-18 py-2.5 px-4 rounded-2xl glass-pill flex items-center justify-between z-40 select-none">
      {/* Track Info */}
      <div className="flex items-center space-x-3 w-1/3 min-w-0">
        <div className="relative w-12 h-12 rounded-xl overflow-hidden bg-white/10 flex-shrink-0 shadow-lg border border-white/10">
          {track.thumbnail_url ? (
            <img
              src={track.thumbnail_url}
              alt={track.title}
              className="w-full h-full object-cover"
            />
          ) : (
            <div className="w-full h-full flex items-center justify-center text-white/40">
              <Music className="w-5 h-5" />
            </div>
          )}
        </div>
        <div className="min-w-0 pr-2">
          <h4 className="text-sm font-semibold text-white truncate">{track.title}</h4>
          <p className="text-xs text-white/50 truncate mt-0.5">{track.artists.join(', ')}</p>
        </div>
      </div>

      {/* Playback Controls & Scrubber */}
      <div className="flex flex-col items-center justify-center w-1/3 space-y-1.5">
        <div className="flex items-center space-x-3">
          <button
            onClick={onTogglePlay}
            className="w-9 h-9 rounded-full bg-gradient-to-tr from-rose-600 to-rose-400 hover:from-rose-500 hover:to-rose-300 flex items-center justify-center text-white shadow-md hover:scale-105 active:scale-95 transition-all"
          >
            {isPlaying ? (
              <Pause className="w-4 h-4 fill-current" />
            ) : (
              <Play className="w-4 h-4 fill-current translate-x-0.5" />
            )}
          </button>
        </div>

        {/* Progress Bar */}
        <div className="w-full flex items-center space-x-2 text-[10px] font-mono text-white/40">
          <span>{formatTime(positionMs)}</span>
          <div className="relative flex-1 h-1 bg-white/10 rounded-full overflow-hidden">
            <div
              className="h-full bg-rose-500 rounded-full transition-all duration-100 ease-out"
              style={{ width: `${Math.min(100, Math.max(0, progressPercent))}%` }}
            />
          </div>
          <span>{formatTime(durationMs)}</span>
        </div>
      </div>

      {/* Volume Control */}
      <div className="flex items-center justify-end space-x-2 w-1/3">
        <button
          onClick={() => onVolumeChange(volume > 0 ? 0 : 0.8)}
          className="text-white/50 hover:text-white transition-colors p-1"
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
          className="w-20 h-1 bg-white/10 accent-rose-500 rounded-full appearance-none cursor-pointer"
        />
      </div>
    </div>
  );
};
