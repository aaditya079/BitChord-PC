import { useState, useEffect, useCallback, useRef } from 'react';
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
      await invoke('play_track', { videoId: track.video_id });
    } catch (err) {
      console.error('Failed to play track:', err);
    }
  }, []);

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

  return {
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
  };
}
