import { useState, useCallback } from "react";
import { ArrowLeft, Play, Heart, Share2, Search, Loader2, Music } from "lucide-react";
import { api } from "../lib/api";
import { usePolledApi } from "../lib/useApi";

export default function SongShare({ onBack }) {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTrack, setSelectedTrack] = useState(null);

  // Mock data for SongShare (would come from /songshare/tracks)
  const mockTracks = [
    { id: "1", title: "Blinding Lights", artist: "The Weeknd", album: "After Hours", duration: "3:20", genre: "Pop", color: "from-red-500 to-orange-400" },
    { id: "2", title: "Levitating", artist: "Dua Lipa", album: "Future Nostalgia", duration: "3:23", genre: "Pop", color: "from-pink-500 to-rose-400" },
    { id: "3", title: "Save Your Tears", artist: "The Weeknd", album: "After Hours", duration: "3:35", genre: "Pop", color: "from-purple-500 to-violet-400" },
    { id: "4", title: "Stay", artist: "Kid LAROI & Justin Bieber", album: "F*CK LOVE 3", duration: "2:21", genre: "Pop", color: "from-yellow-500 to-amber-400" },
    { id: "5", title: "Good 4 U", artist: "Olivia Rodrigo", album: "SOUR", duration: "2:58", genre: "Pop Rock", color: "from-green-500 to-emerald-400" },
    { id: "6", title: "Montero", artist: "Lil Nas X", album: "MONTERO", duration: "2:17", genre: "Pop Rap", color: "from-blue-500 to-indigo-400" },
    { id: "7", title: "Peaches", artist: "Justin Bieber", album: "Justice", duration: "3:18", genre: "R&B", color: "from-orange-500 to-yellow-400" },
    { id: "8", title: "Kiss Me More", artist: "Doja Cat ft. SZA", album: "Planet Her", duration: "3:28", genre: "Pop", color: "from-teal-500 to-cyan-400" },
  ];

  const filteredTracks = mockTracks.filter(
    (t) =>
      t.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      t.artist.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="flex h-full flex-col bg-bg-0">
      {/* Header */}
      <div
        className="flex items-center gap-3"
        style={{
          height: "56px",
          padding: "0 16px",
          background: "linear-gradient(180deg, var(--bg-1) 0%, var(--bg-0) 100%)",
          borderBottom: "1px solid var(--stroke-soft)",
        }}
      >
        <button
          onClick={onBack}
          className="flex h-9 w-9 items-center justify-center rounded-xl"
          style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", cursor: "pointer" }}
        >
          <ArrowLeft size={16} style={{ color: "var(--text-secondary)" }} />
        </button>
        <div>
          <h1 className="font-display text-base font-bold text-text-primary">SongShare</h1>
          <p className="text-[10px] text-text-muted">Descobre música nova</p>
        </div>
      </div>

      {/* Search */}
      <div className="px-4 py-3">
        <div
          className="flex items-center gap-2"
          style={{
            height: "40px",
            borderRadius: "12px",
            background: "var(--panel-soft)",
            border: "1px solid var(--stroke-soft)",
            padding: "0 12px",
          }}
        >
          <Search size={14} style={{ color: "var(--text-dim)" }} />
          <input
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Procurar músicas, artistas…"
            className="flex-1 bg-transparent text-sm text-text-primary outline-none placeholder:text-text-dim"
          />
        </div>
      </div>

      {/* Track list */}
      <div className="flex-1 overflow-y-auto px-4 pb-4">
        <div className="space-y-2">
          {filteredTracks.map((track, idx) => (
            <button
              key={track.id}
              onClick={() => setSelectedTrack(track.id === selectedTrack?.id ? null : track)}
              className="flex w-full items-center gap-3 rounded-xl p-3 transition-all duration-150"
              style={{
                background: selectedTrack?.id === track.id ? "var(--accent-tint)" : "var(--panel-soft)",
                border: `1px solid ${selectedTrack?.id === track.id ? "var(--accent)/40" : "var(--stroke-soft)"}`,
              }}
            >
              {/* Album art placeholder */}
              <div
                className={`flex h-12 w-12 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br ${track.color}`}
              >
                <Music size={18} style={{ color: "var(--bg-0)" }} />
              </div>

              {/* Track info */}
              <div className="flex-1 min-w-0 text-left">
                <p className="text-sm font-semibold text-text-primary truncate">{track.title}</p>
                <p className="text-[11px] text-text-muted truncate">{track.artist} · {track.album}</p>
              </div>

              {/* Duration & actions */}
              <div className="flex items-center gap-2 shrink-0">
                <span className="text-[10px] text-text-dim">{track.duration}</span>
                {selectedTrack?.id === track.id && (
                  <div className="flex gap-1 animate-fade-in">
                    <button
                      className="flex h-7 w-7 items-center justify-center rounded-full"
                      style={{ background: "var(--accent)", cursor: "pointer" }}
                    >
                      <Play size={12} style={{ color: "var(--bg-0)", marginLeft: "1px" }} />
                    </button>
                    <button
                      className="flex h-7 w-7 items-center justify-center rounded-full"
                      style={{ background: "var(--panel-elevated)", cursor: "pointer" }}
                    >
                      <Heart size={12} style={{ color: "var(--accent-pink)" }} />
                    </button>
                  </div>
                )}
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Now playing bar */}
      {selectedTrack && (
        <div
          className="flex items-center gap-3 px-4 py-3 animate-slide-up"
          style={{
            background: "linear-gradient(90deg, var(--accent-tint), var(--panel-soft))",
            borderTop: "1px solid var(--accent)/30",
          }}
        >
          <div className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br ${selectedTrack.color}`}>
            <Music size={14} style={{ color: "var(--bg-0)" }} />
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-xs font-semibold text-text-primary truncate">{selectedTrack.title}</p>
            <p className="text-[10px] text-text-muted">{selectedTrack.artist}</p>
          </div>
          <button
            className="flex h-9 w-9 items-center justify-center rounded-full"
            style={{ background: "var(--accent)", cursor: "pointer" }}
          >
            <Play size={14} style={{ color: "var(--bg-0)", marginLeft: "1px" }} />
          </button>
        </div>
      )}
    </div>
  );
}
