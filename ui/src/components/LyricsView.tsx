import React, { useEffect, useRef } from 'react';
import { LyricLine } from '../hooks/useAudioEngine';
import { Mic2, Loader2 } from 'lucide-react';

interface LyricsViewProps {
  lyrics: LyricLine[];
  positionMs: number;
  isLoading: boolean;
}

export const LyricsView: React.FC<LyricsViewProps> = ({ lyrics, positionMs, isLoading }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const activeLineRef = useRef<HTMLDivElement>(null);

  // Find active line index using binary or linear scan
  let activeIndex = -1;
  for (let i = 0; i < lyrics.length; i++) {
    if (lyrics[i].timestamp_ms <= positionMs + 150) {
      activeIndex = i;
    } else {
      break;
    }
  }

  // Smooth scroll active line to center of container
  useEffect(() => {
    if (activeLineRef.current && containerRef.current) {
      activeLineRef.current.scrollIntoView({
        behavior: 'smooth',
        block: 'center',
      });
    }
  }, [activeIndex]);

  if (isLoading) {
    return (
      <div className="h-full flex flex-col items-center justify-center space-y-3 text-white/50 select-none">
        <Loader2 className="w-8 h-8 animate-spin text-rose-500" />
        <p className="text-sm font-medium">Fetching synchronized lyrics...</p>
      </div>
    );
  }

  if (lyrics.length === 0) {
    return (
      <div className="h-full flex flex-col items-center justify-center space-y-3 text-white/40 select-none">
        <div className="w-12 h-12 rounded-full bg-white/[0.04] flex items-center justify-center">
          <Mic2 className="w-6 h-6 text-white/30" />
        </div>
        <p className="text-sm font-medium">No synchronized lyrics available for this track</p>
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className="h-full overflow-y-auto px-6 py-20 space-y-7 scroll-smooth no-scrollbar select-none"
    >
      {lyrics.map((line, idx) => {
        const isActive = idx === activeIndex;
        const isPast = idx < activeIndex;

        return (
          <div
            key={idx}
            ref={isActive ? activeLineRef : null}
            className={`transition-all duration-300 transform origin-left cursor-pointer ${
              isActive
                ? 'text-white text-2xl font-bold scale-105 opacity-100 tracking-tight drop-shadow-[0_4px_12px_rgba(244,63,94,0.35)]'
                : isPast
                ? 'text-white/40 text-lg font-medium opacity-50 blur-[0.4px]'
                : 'text-white/30 text-lg font-medium opacity-40 hover:opacity-70'
            }`}
          >
            {line.text || '...'}
          </div>
        );
      })}
    </div>
  );
};
