import { useState } from "react";
import { ArrowLeft, Play, Heart, Search, Loader2, Music, ExternalLink } from "lucide-react";
import { api } from "../lib/api";
import { usePolledApi } from "../lib/useApi";

const GRADIENT_COLORS = [
  "from-red-500 to-orange-400",
  "from-pink-500 to-rose-400",
  "from-purple-500 to-violet-400",
  "from-yellow-500 to-amber-400",
  "from-green-500 to-emerald-400",
  "from-blue-500 to-indigo-400",
  "from-orange-500 to-yellow-400",
  "from-teal-500 to-cyan-400",
];

function getGradient(id) {
  const idx = typeof id === "string" ? id.charCodeAt(0) : id;
  return GRADIENT_COLORS[idx % GRADIENT_COLORS.length];
}

export default function SongShare({ onBack }) {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTrack, setSelectedTrack] = useState(null);
  const { data: songshareData, loading } = usePolledApi(() => api.songshareTracks(true), { intervalMs: 60000 });

  const apiTracks = songshareData?.body?.items || [];
  const isOffline = songshareData?.ok === false;

  // Fallback demo tracks when API unavailable
  const demoTracks = [
    { id: "d1", name: "Blinding Lights", artists: [{ name: "The Weeknd" }], album: { name: "After Hours" } },
    { id: "d2", name: "Levitating", artists: [{ name: "Dua Lipa" }], album: { name: "Future Nostalgia" } },
    { id: "d3", name: "Save Your Tears", artists: [{ name: "The Weeknd" }], album: { name: "After Hours" } },
    { id: "d4", name: "Stay", artists: [{ name: "Kid LAROI" }, { name: "Justin Bieber" }], album: { name: "F*CK LOVE 3" } },
    { id: "d5", name: "Good 4 U", artists: [{ name: "Olivia Rodrigo" }], album: { name: "SOUR" } },
    { id: "d6", name: "Montero", artists: [{ name: "Lil Nas X" }], album: { name: "MONTERO" } },
    { id: "d7", name: "Peaches", artists: [{ name: "Justin Bieber" }], album: { name: "Justice" } },
    { id: "d8", name: "Kiss Me More", artists: [{ name: "Doja Cat" }, { name: "SZA" }], album: { name: "Planet Her" } },
    { id: "d9", name: "drivers license", artists: [{ name: "Olivia Rodrigo" }], album: { name: "SOUR" } },
    { id: "d10", name: "Butter", artists: [{ name: "BTS" }], album: { name: "Butter" } },
  ];

  const tracks = isOffline ? demoTracks : apiTracks;

  const filteredTracks = searchQuery
    ? tracks.filter(
        (t) =>
          t.name?.toLowerCase().includes(searchQuery.toLowerCase()) ||
          t.artists?.some((a) => a.name?.toLowerCase().includes(searchQuery.toLowerCase()))
      )
    : tracks;

  return (
    <div className="flex h-full flex-col bg-bg-0">
      {/* Header */}
      <div className="flex items-center gap-3" style={{ height: "56px", padding: "0 16px", background: "linear-gradient(180deg, var(--bg-1) 0%, var(--bg-0) 100%)", borderBottom: "1px solid var(--stroke-soft)" }}>
        <button onClick={onBack} className="flex h-9 w-9 items-center justify-center rounded-xl" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", cursor: "pointer" }}>
          <ArrowLeft size={16} style={{ color: "var(--text-secondary)" }} />
        </button>
        <div>
          <h1 className="font-display text-base font-bold text-text-primary">SongShare</h1>
          <p className="text-[10px] text-text-muted">Descobre música nova</p>
        </div>
        {isOffline && (
          <div className="ml-auto rounded-full px-2 py-0.5 text-[9px] font-medium" style={{ background: "var(--warning-tint)", color: "var(--warning)" }}>
            Demo
          </div>
        )}
      </div>

      {/* Search */}
      <div className="px-4 py-3">
        <div className="flex items-center gap-2" style={{ height: "40px", borderRadius: "12px", background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", padding: "0 12px" }}>
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
        {loading && !songshareData ? (
          <div className="flex items-center justify-center h-32">
            <Loader2 size={20} className="animate-spin" style={{ color: "var(--accent)" }} />
          </div>
        ) : filteredTracks.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-32 text-center">
            <Music size={24} style={{ color: "var(--text-dim)" }} />
            <p className="text-sm text-text-muted mt-2">Nenhuma faixa encontrada</p>
          </div>
        ) : (
          <div className="space-y-2">
            {filteredTracks.map((track, idx) => (
              <button
                key={track.id || idx}
                onClick={() => setSelectedTrack(selectedTrack?.id === track.id ? null : track)}
                className="flex w-full items-center gap-3 rounded-xl p-3 transition-all duration-150"
                style={{
                  background: selectedTrack?.id === track.id ? "var(--accent-tint)" : "var(--panel-soft)",
                  border: `1px solid ${selectedTrack?.id === track.id ? "var(--accent)/40" : "var(--stroke-soft)"}`,
                }}
              >
                {/* Album art placeholder */}
                <div className={`flex h-12 w-12 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br ${getGradient(track.id || idx)}`}>
                  <Music size={18} style={{ color: "var(--bg-0)" }} />
                </div>

                {/* Track info */}
                <div className="flex-1 min-w-0 text-left">
                  <p className="text-sm font-semibold text-text-primary truncate">{track.name || "Sem título"}</p>
                  <p className="text-[11px] text-text-muted truncate">{track.artists?.map((a) => a.name).join(", ") || "Artista desconhecido"}</p>
                </div>

                {/* Actions */}
                {selectedTrack?.id === track.id && (
                  <div className="flex gap-1 animate-fade-in">
                    <button className="flex h-7 w-7 items-center justify-center rounded-full" style={{ background: "var(--accent)", cursor: "pointer" }}>
                      <Play size={12} style={{ color: "var(--bg-0)", marginLeft: "1px" }} />
                    </button>
                    <button className="flex h-7 w-7 items-center justify-center rounded-full" style={{ background: "var(--panel-elevated)", cursor: "pointer" }}>
                      <Heart size={12} style={{ color: "var(--accent-pink)" }} />
                    </button>
                  </div>
                )}
              </button>
            ))}
          </div>
        )}
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
          <div className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br ${getGradient(selectedTrack.id || 0)}`}>
            <Music size={14} style={{ color: "var(--bg-0)" }} />
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-xs font-semibold text-text-primary truncate">{selectedTrack.name}</p>
            <p className="text-[10px] text-text-muted">{selectedTrack.artists?.map((a) => a.name).join(", ")}</p>
          </div>
          <button className="flex h-9 w-9 items-center justify-center rounded-full" style={{ background: "var(--accent)", cursor: "pointer" }}>
            <Play size={14} style={{ color: "var(--bg-0)", marginLeft: "1px" }} />
          </button>
        </div>
      )}
    </div>
  );
}
