import React from 'react';
import { Minus, Square, X, Music } from 'lucide-react';

export const Titlebar: React.FC = () => {
  const handleMinimize = async () => {
    try {
      const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      await getCurrentWebviewWindow().minimize();
    } catch (e) {
      console.warn('Window control unavailable in browser', e);
    }
  };

  const handleMaximize = async () => {
    try {
      const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      const win = getCurrentWebviewWindow();
      const isMax = await win.isMaximized();
      if (isMax) {
        await win.unmaximize();
      } else {
        await win.maximize();
      }
    } catch (e) {
      console.warn('Window control unavailable in browser', e);
    }
  };

  const handleClose = async () => {
    try {
      const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      await getCurrentWebviewWindow().close();
    } catch (e) {
      console.warn('Window control unavailable in browser', e);
    }
  };

  return (
    <header
      data-tauri-drag-region
      className="h-10 w-full flex items-center justify-between px-3 select-none z-50 fixed top-0 left-0 right-0 border-b border-white/[0.06] bg-black/40 backdrop-blur-xl"
    >
      <div data-tauri-drag-region className="flex items-center space-x-2 pointer-events-none">
        <div className="w-5 h-5 rounded-md bg-gradient-to-tr from-rose-600 to-rose-400 flex items-center justify-center shadow-sm">
          <Music className="w-3 h-3 text-white" />
        </div>
        <span className="text-xs font-semibold tracking-wider uppercase text-white/90">
          BitChord <span className="text-[10px] text-rose-400 font-mono font-normal">v3.0</span>
        </span>
      </div>

      <div className="flex items-center space-x-1 no-drag">
        <button
          onClick={handleMinimize}
          className="w-7 h-7 flex items-center justify-center rounded-md hover:bg-white/10 text-white/60 hover:text-white transition-colors"
          title="Minimize"
        >
          <Minus className="w-3.5 h-3.5" />
        </button>
        <button
          onClick={handleMaximize}
          className="w-7 h-7 flex items-center justify-center rounded-md hover:bg-white/10 text-white/60 hover:text-white transition-colors"
          title="Maximize"
        >
          <Square className="w-3 h-3" />
        </button>
        <button
          onClick={handleClose}
          className="w-7 h-7 flex items-center justify-center rounded-md hover:bg-rose-600/80 text-white/60 hover:text-white transition-colors"
          title="Close"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
    </header>
  );
};
