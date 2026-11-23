import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface Track {
  video_id: string;
  title: string;
  artists: string[];
  album?: string;
  duration_secs: number;
  thumbnail_url?: string;
}

export interface LyricLine {
  text: string;
  timestamp_ms: number;
}

export interface PlaybackProgress {
  position_ms: number;
  duration_ms: number;
  is_playing: boolean;
}

export function useAudioEngine() {
  const [currentTrack, setCurrentTrack] = useState<Track | null>(null);
  const [isPlaying, setIsPlaying] = useState<boolean>(false);
  const [positionMs, setPositionMs] = useState<number>(0);
  const [durationMs, setDurationMs] = useState<number>(0);
  const [volume, setVolumeState] = useState<number>(1.0);
  const [isSearching, setIsSearching] = useState<boolean>(false);
  const [searchResults, setSearchResults] = useState<Track[]>([]);
  const [queue, setQueue] = useState<Track[]>([]);
  const [lyrics, setLyrics] = useState<LyricLine[]>([]);
  const [isLoadingLyrics, setIsLoadingLyrics] = useState<boolean>(false);
  const [repeatMode, setRepeatModeState] = useState<number>(0);
  const [isShuffle, setIsShuffleState] = useState<boolean>(false);

  // Sync queue from backend
  const refreshQueue = useCallback(async () => {
    try {
      const q = await invoke<Track[]>('get_queue');
      setQueue(q);
    } catch (err) {
      console.error('Failed to get queue:', err);
    }
  }, []);

  // Fetch synchronized lyrics
  const fetchLyrics = useCallback(async (artist: string, title: string) => {
    setIsLoadingLyrics(true);
    try {
      const result = await invoke<LyricLine[]>('get_lyrics', { artist, title });
      setLyrics(result);
    } catch (err) {
      console.warn('No lyrics found:', err);
      setLyrics([]);
    } finally {
      setIsLoadingLyrics(false);
    }
  }, []);

  // Track progress event listener
  useEffect(() => {
    let unlistenFn: (() => void) | null = null;

    listen<PlaybackProgress>('bitchord://progress', (event) => {
      setPositionMs(event.payload.position_ms);
      if (event.payload.duration_ms > 0) {
        setDurationMs(event.payload.duration_ms);
      }
      setIsPlaying(event.payload.is_playing);
    }).then((unlisten) => {
      unlistenFn = unlisten;
    }).catch(console.error);

    return () => {
      if (unlistenFn) unlistenFn();
    };
  }, []);

  const playTrack = useCallback(async (track: Track) => {
    try {
      setCurrentTrack(track);
      setIsPlaying(true);
      setPositionMs(0);
      setDurationMs(track.duration_secs * 1000);
      await invoke('play_track', { track });
      await refreshQueue();
      // Fetch lyrics concurrently
      if (track.artists.length > 0) {
        fetchLyrics(track.artists[0], track.title);
      }
    } catch (err) {
      console.error('Failed to play track:', err);
    }
  }, [refreshQueue, fetchLyrics]);

  const togglePlayback = useCallback(async () => {
    try {
      const playing = await invoke<boolean>('toggle_playback');
      setIsPlaying(playing);
    } catch (err) {
      console.error('Failed to toggle playback:', err);
    }
  }, []);

  const setVolume = useCallback(async (val: number) => {
    try {
      const clamped = Math.max(0, Math.min(1, val));
      setVolumeState(clamped);
      await invoke('set_volume', { volume: clamped });
    } catch (err) {
      console.error('Failed to set volume:', err);
    }
  }, []);

  const search = useCallback(async (query: string) => {
    if (!query.trim()) {
      setSearchResults([]);
      return;
    }
    setIsSearching(true);
    try {
      const tracks = await invoke<Track[]>('search_tracks', { query });
      setSearchResults(tracks);
    } catch (err) {
      console.error('Search failed:', err);
    } finally {
      setIsSearching(false);
    }
  }, []);

  const enqueueTrack = useCallback(async (track: Track) => {
    try {
      await invoke('enqueue_track', { track });
      await refreshQueue();
    } catch (err) {
      console.error('Failed to enqueue track:', err);
    }
  }, [refreshQueue]);

  const removeFromQueue = useCallback(async (index: number) => {
    try {
      await invoke('remove_from_queue', { index });
      await refreshQueue();
    } catch (err) {
      console.error('Failed to remove from queue:', err);
    }
  }, [refreshQueue]);

  const clearQueue = useCallback(async () => {
    try {
      await invoke('clear_queue');
      setQueue([]);
    } catch (err) {
      console.error('Failed to clear queue:', err);
    }
  }, []);

  const skipNext = useCallback(async () => {
    try {
      const next = await invoke<Track | null>('skip_next');
      if (next) {
        setCurrentTrack(next);
        setIsPlaying(true);
        setPositionMs(0);
        setDurationMs(next.duration_secs * 1000);
        await refreshQueue();
        if (next.artists.length > 0) {
          fetchLyrics(next.artists[0], next.title);
        }
      } else {
        setIsPlaying(false);
      }
    } catch (err) {
      console.error('Failed to skip next:', err);
    }
  }, [refreshQueue, fetchLyrics]);

  const skipPrevious = useCallback(async () => {
    try {
      const prev = await invoke<Track | null>('skip_previous');
      if (prev) {
        setCurrentTrack(prev);
        setIsPlaying(true);
        setPositionMs(0);
        setDurationMs(prev.duration_secs * 1000);
        await refreshQueue();
        if (prev.artists.length > 0) {
          fetchLyrics(prev.artists[0], prev.title);
        }
      }
    } catch (err) {
      console.error('Failed to skip previous:', err);
    }
  }, [refreshQueue, fetchLyrics]);

  const setRepeatMode = useCallback(async (mode: number) => {
    try {
      await invoke('set_repeat_mode', { mode });
      setRepeatModeState(mode);
    } catch (err) {
      console.error('Failed to set repeat mode:', err);
    }
  }, []);

  const toggleShuffle = useCallback(async () => {
    try {
      const shuffle = await invoke<boolean>('toggle_shuffle');
      setIsShuffleState(shuffle);
    } catch (err) {
      console.error('Failed to toggle shuffle:', err);
    }
  }, []);

  return {
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
  };
}
