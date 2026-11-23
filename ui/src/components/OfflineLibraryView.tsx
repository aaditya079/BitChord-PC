import React, { useState } from 'react';
import { Download, Play, Shuffle, Search, FolderDown, Trash2 } from 'lucide-react';
import { Track } from '../hooks/useAudioEngine';
import { TrackCard } from './TrackCard';

interface OfflineLibraryViewProps {
  tracks: Track[];
  currentTrack: Track | null;
  isPlaying: boolean;
  onPlay: (track: Track) => void;
  onEnqueue: (track: Track) => void;
  onDeleteTrack: (videoId: string) => void;
}

export const OfflineLibraryView: React.FC<OfflineLibraryViewProps> = ({
  tracks,
  currentTrack,
  isPlaying,
  onPlay,
  onEnqueue,
  onDeleteTrack,
}) => {
  const [filterQuery, setFilterQuery] = useState('');

  const filteredTracks = tracks.filter((t) => {
    if (!filterQuery.trim()) return true;
    const query = filterQuery.toLowerCase();
    return (
      t.title.toLowerCase().includes(query) ||
      t.artists.some((a) => a.toLowerCase().includes(query)) ||
      (t.album && t.album.toLowerCase().includes(query))
    );
  });

  const totalDurationSecs = tracks.reduce((acc, t) => acc + (t.duration_secs || 0), 0);
  const totalMinutes = Math.floor(totalDurationSecs / 60);

  const handlePlayAll = () => {
    if (filteredTracks.length === 0) return;
    onPlay(filteredTracks[0]);
    for (let i = 1; i < filteredTracks.length; i++) {
      onEnqueue(filteredTracks[i]);
    }
  };

  const handleShufflePlay = () => {
    if (filteredTracks.length === 0) return;
    const shuffled = [...filteredTracks].sort(() => Math.random() - 0.5);
    onPlay(shuffled[0]);
    for (let i = 1; i < shuffled.length; i++) {
      onEnqueue(shuffled[i]);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header and stats */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-5 rounded-2xl glass-card border border-white/[0.08]">
        <div className="flex items-center gap-3.5">
          <div className="w-12 h-12 rounded-xl bg-rose-500/10 border border-rose-500/20 flex items-center justify-center text-rose-400">
            <Download className="w-6 h-6" />
          </div>
          <div>
            <h2 className="text-lg font-bold text-white tracking-tight">Offline Library</h2>
            <p className="text-xs text-white/50">
              {tracks.length} {tracks.length === 1 ? 'song' : 'songs'} saved
              {totalMinutes > 0 ? ` • ${totalMinutes} mins of offline audio` : ''}
            </p>
          </div>
        </div>

        {tracks.length > 0 && (
          <div className="flex items-center gap-2">
            <button
              onClick={handlePlayAll}
              className="px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 active:scale-95 text-xs font-semibold text-white flex items-center gap-2 transition-all shadow-sm"
            >
              <Play className="w-3.5 h-3.5 fill-current" />
              <span>Play All</span>
            </button>
            <button
              onClick={handleShufflePlay}
              className="px-4 py-2 rounded-xl bg-white/5 hover:bg-white/15 active:scale-95 text-xs font-semibold text-white/80 hover:text-white flex items-center gap-2 transition-all border border-white/5"
            >
              <Shuffle className="w-3.5 h-3.5" />
              <span>Shuffle</span>
            </button>
          </div>
        )}
      </div>

      {/* Local Filter Bar */}
      {tracks.length > 3 && (
        <div className="relative">
          <Search className="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-white/30 pointer-events-none" />
          <input
            type="text"
            value={filterQuery}
            onChange={(e) => setFilterQuery(e.target.value)}
            placeholder="Filter saved tracks by title, artist, or album..."
            className="w-full pl-10 pr-4 py-2.5 rounded-xl bg-white/[0.04] border border-white/[0.08] text-xs text-white placeholder-white/30 focus:outline-none focus:border-rose-400/50 transition-colors"
          />
        </div>
      )}

      {/* Track list or Empty state */}
      {filteredTracks.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-2.5">
          {filteredTracks.map((track) => (
            <TrackCard
              key={track.video_id}
              track={track}
              isPlaying={currentTrack?.video_id === track.video_id && isPlaying}
              isDownloaded={true}
              onPlay={onPlay}
              onEnqueue={onEnqueue}
              onDeleteDownload={onDeleteTrack}
            />
          ))}
        </div>
      ) : tracks.length > 0 ? (
        <div className="py-12 text-center text-white/40 text-xs">
          No saved tracks match "{filterQuery}".
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center py-20 text-center space-y-3.5">
          <div className="w-14 h-14 rounded-2xl bg-white/[0.03] border border-white/[0.06] flex items-center justify-center text-white/30">
            <FolderDown className="w-7 h-7" />
          </div>
          <div className="space-y-1 max-w-sm">
            <h3 className="text-sm font-medium text-white/80">No offline music yet</h3>
            <p className="text-xs text-white/40 leading-relaxed">
              Click the download icon on any search result to save full-fidelity Opus audio to your PC for zero-latency, offline playback.
            </p>
          </div>
        </div>
      )}
    </div>
  );
};
