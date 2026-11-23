import React, { useState } from 'react';
import { Search, X, Loader2 } from 'lucide-react';

interface SearchBarProps {
  onSearch: (query: string) => void;
  isLoading: boolean;
}

export const SearchBar: React.FC<SearchBarProps> = ({ onSearch, isLoading }) => {
  const [value, setValue] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (value.trim()) {
      onSearch(value.trim());
    }
  };

  const handleClear = () => {
    setValue('');
    onSearch('');
  };

  return (
    <form onSubmit={handleSubmit} className="relative w-full max-w-xl mx-auto">
      <div className="relative flex items-center">
        <Search className="absolute left-4 w-4 h-4 text-white/40 pointer-events-none" />
        <input
          type="text"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder="Search tracks, artists, albums on YouTube Music..."
          className="w-full h-11 pl-11 pr-11 bg-white/[0.05] hover:bg-white/[0.08] focus:bg-white/[0.1] border border-white/[0.08] focus:border-rose-500/50 rounded-full text-sm text-white placeholder-white/35 focus:outline-none transition-all duration-200 shadow-inner"
        />
        {isLoading ? (
          <Loader2 className="absolute right-4 w-4 h-4 text-rose-400 animate-spin" />
        ) : value ? (
          <button
            type="button"
            onClick={handleClear}
            className="absolute right-3.5 w-5 h-5 flex items-center justify-center text-white/40 hover:text-white rounded-full hover:bg-white/10 transition-colors"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        ) : null}
      </div>
    </form>
  );
};
