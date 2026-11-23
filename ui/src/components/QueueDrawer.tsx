import React from 'react';
import { Track } from '../hooks/useAudioEngine';
import { ListMusic, Trash2, X, Music } from 'lucide-react';

interface QueueDrawerProps {
  queue: Track[];
  onPlay: (track: Track) => void;
  onRemove: (index: number) => void;
  onClear: () => void;
}

export const QueueDrawer: React.FC<QueueDrawerProps> = ({
  queue,
  onPlay,
  onRemove,
  onClear,
}) => {
  return (
    <div className="h-full flex flex-col select-none">
      <div className="flex items-center justify-between px-6 py-4 border-b border-white/[0.08]">
        <div className="flex items-center space-x-2">
          <ListMusic className="w-5 h-5 text-rose-400" />
          <h3 className="text-base font-semibold text-white">Up Next</h3>
          <span className="text-xs bg-white/10 text-white/70 px-2 py-0.5 rounded-full font-mono">
            {queue.length}
          </span>
        </div>
        {queue.length > 0 && (
          <button
            onClick={onClear}
            className="flex items-center space-x-1.5 text-xs text-white/50 hover:text-rose-400 transition-colors px-2 py-1 rounded-md hover:bg-white/[0.05]"
          >
            <Trash2 className="w-3.5 h-3.5" />
            <span>Clear</span>
          </button>
        )}
      </div>

      <div className="flex-1 overflow-y-auto px-4 py-3 space-y-2">
        {queue.length === 0 ? (
          <div className="h-full flex flex-col items-center justify-center space-y-2 text-white/40">
            <ListMusic className="w-8 h-8 text-white/20" />
            <p className="text-sm">No songs in queue</p>
            <p className="text-xs text-white/30 text-center max-w-xs">
              Search for songs and add them to your queue for uninterrupted listening.
            </p>
          </div>
        ) : (
          queue.map((track, idx) => (
            <div
              key={`${track.video_id}-${idx}`}
              className="group flex items-center justify-between p-2 rounded-xl glass-card hover:bg-white/[0.08] transition-all"
            >
              <div
                onClick={() => onPlay(track)}
                className="flex items-center space-x-3 min-w-0 flex-1 cursor-pointer"
              >
                <div className="w-10 h-10 rounded-lg overflow-hidden bg-white/5 flex-shrink-0">
                  {track.thumbnail_url ? (
                    <img
                      src={track.thumbnail_url}
                      alt={track.title}
                      className="w-full h-full object-cover"
                    />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center text-white/30">
                      <Music className="w-5 h-5" />
                    </div>
                  )}
                </div>
                <div className="min-w-0 flex-1">
                  <h4 className="text-sm font-medium text-white/90 truncate group-hover:text-rose-400 transition-colors">
                    {track.title}
                  </h4>
                  <p className="text-xs text-white/50 truncate">
                    {track.artists.join(', ')}
                  </p>
                </div>
              </div>

              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onRemove(idx);
                }}
                className="w-8 h-8 flex items-center justify-center text-white/30 hover:text-rose-400 rounded-lg hover:bg-white/10 transition-colors opacity-0 group-hover:opacity-100"
                title="Remove from queue"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
