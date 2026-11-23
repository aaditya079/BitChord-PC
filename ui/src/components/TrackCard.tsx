import React from 'react';
import { Play, Plus, Music, Download, Check, Loader2, Trash2 } from 'lucide-react';
import { Track } from '../hooks/useAudioEngine';

interface TrackCardProps {
  track: Track;
  isPlaying: boolean;
  isDownloaded?: boolean;
  isDownloading?: boolean;
  onPlay: (track: Track) => void;
  onEnqueue?: (track: Track) => void;
  onDownload?: (track: Track) => void;
  onDeleteDownload?: (videoId: string) => void;
}

export const TrackCard: React.FC<TrackCardProps> = ({
  track,
  isPlaying,
  isDownloaded = false,
  isDownloading = false,
  onPlay,
  onEnqueue,
  onDownload,
  onDeleteDownload,
}) => {
  return (
    <div
      onClick={() => onPlay(track)}
      className="group relative flex items-center p-2.5 rounded-xl glass-card cursor-pointer gap-3.5 transition-all select-none hover:bg-white/[0.08]"
    >
      <div className="relative w-12 h-12 rounded-lg overflow-hidden bg-white/5 flex-shrink-0 shadow-md">
        {track.thumbnail_url ? (
          <img
            src={track.thumbnail_url}
            alt={track.title}
            className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
            loading="lazy"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center text-white/30">
            <Music className="w-6 h-6" />
          </div>
        )}
        <div
          className={`absolute inset-0 bg-black/40 flex items-center justify-center transition-opacity ${
            isPlaying ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'
          }`}
        >
          <Play className="w-5 h-5 text-white fill-current translate-x-0.5" />
        </div>
      </div>

      <div className="flex-1 min-w-0 pr-2">
        <div className="flex items-center gap-1.5">
          <h4
            className={`text-sm font-medium truncate ${
              isPlaying ? 'text-rose-400 font-semibold' : 'text-white/90 group-hover:text-white'
            }`}
          >
            {track.title}
          </h4>
          {isDownloaded && (
            <span title="Downloaded offline" className="flex-shrink-0">
              <Check className="w-3.5 h-3.5 text-emerald-400" />
            </span>
          )}
        </div>
        <p className="text-xs text-white/50 truncate mt-0.5">
          {track.artists.join(', ') || 'Unknown Artist'}
        </p>
      </div>

      <div className="flex items-center space-x-1.5">
        {/* Download / Saved status action */}
        {isDownloading ? (
          <div className="w-7 h-7 flex items-center justify-center text-rose-400" title="Downloading offline...">
            <Loader2 className="w-4 h-4 animate-spin" />
          </div>
        ) : isDownloaded && onDeleteDownload ? (
          <button
            onClick={(e) => {
              e.stopPropagation();
              onDeleteDownload(track.video_id);
            }}
            className="w-7 h-7 rounded-lg bg-white/5 hover:bg-red-500/20 text-white/40 hover:text-red-400 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-all"
            title="Delete offline download"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        ) : onDownload && !isDownloaded ? (
          <button
            onClick={(e) => {
              e.stopPropagation();
              onDownload(track);
            }}
            className="w-7 h-7 rounded-lg bg-white/5 hover:bg-white/15 text-white/40 hover:text-white flex items-center justify-center opacity-0 group-hover:opacity-100 transition-all"
            title="Save for offline playback"
          >
            <Download className="w-3.5 h-3.5" />
          </button>
        ) : null}

        {onEnqueue && (
          <button
            onClick={(e) => {
              e.stopPropagation();
              onEnqueue(track);
            }}
            className="w-7 h-7 rounded-lg bg-white/5 hover:bg-white/15 flex items-center justify-center text-white/40 hover:text-white opacity-0 group-hover:opacity-100 transition-all"
            title="Add to queue"
          >
            <Plus className="w-4 h-4" />
          </button>
        )}

        {track.duration_secs > 0 && (
          <span className="text-xs font-mono text-white/35 group-hover:text-white/60 tabular-nums pl-1">
            {Math.floor(track.duration_secs / 60)}:
            {(track.duration_secs % 60).toString().padStart(2, '0')}
          </span>
        )}
      </div>
    </div>
  );
};
