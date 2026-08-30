import { useState } from "react";
import { ArrowLeft, Play, Pause, SkipBack, SkipForward, Shuffle, Repeat, Heart, Loader2, Music, ListMusic, Clock, Disc3 } from "lucide-react";
import { api } from "../lib/api";
import { usePolledApi } from "../lib/useApi";

function IconButton({ children, onClick, size = 44, gradient }) {
  const [pressed, setPressed] = useState(false);
  return (
    <button
      onClick={onClick}
      onMouseDown={() => setPressed(true)}
      onMouseUp={() => setPressed(false)}
      onMouseLeave={() => setPressed(false)}
      className="flex items-center justify-center transition-all duration-150"
      style={{
        width: `${size}px`,
        height: `${size}px`,
        borderRadius: `${size / 2}px`,
        background: pressed ? "var(--panel-elevated)" : gradient || "transparent",
        border: gradient ? "none" : "1px solid var(--stroke-soft)",
        cursor: "pointer",
        boxShadow: pressed ? "none" : gradient ? `0 4px 15px -5px ${gradient.includes("accent") ? "var(--accent)" : "var(--success)"}` : "none",
      }}
    >
      {children}
    </button>
  );
}

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

const TABS = [
  { id: "top", label: "Top", icon: Disc3 },
  { id: "recent", label: "Recentes", icon: Clock },
  { id: "saved", label: "Guardadas", icon: Heart },
  { id: "playlists", label: "Playlists", icon: ListMusic },
];

export default function MusicPlayer({ onBack }) {
  const [activeTab, setActiveTab] = useState("top");
  const [currentIdx, setCurrentIdx] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [progress, setProgress] = useState(30);
  const [controlling, setControlling] = useState(false);
  const [commandError, setCommandError] = useState("");

  const { data: topTracks, loading: topLoading } = usePolledApi(() => api.musicTopTracks(true, 20), { intervalMs: 30000 });
  const { data: recentlyPlayed, loading: recentLoading } = usePolledApi(() => api.musicRecentlyPlayed(true, 20), { intervalMs: 60000 });
  const { data: savedTracks, loading: savedLoading } = usePolledApi(() => api.musicSavedTracks(true, 50), { intervalMs: 60000 });
  const { data: playlists, loading: playlistsLoading } = usePolledApi(() => api.musicPlaylists(true), { intervalMs: 120000 });
  const { data: devices } = usePolledApi(() => api.musicDevices(), { intervalMs: 15000 });

  const trackLists = {
    top: topTracks?.body?.items || [],
    recent: recentlyPlayed?.body?.items || [],
    saved: savedTracks?.body?.items || [],
    playlists: playlists?.body?.items || [],
  };

  const currentTracks = trackLists[activeTab] || [];
  const currentTrack = currentTracks[currentIdx] || null;
  const hasTracks = currentTracks.length > 0;
  const isLoading = (activeTab === "top" && topLoading) || (activeTab === "recent" && recentLoading) || (activeTab === "saved" && savedLoading) || (activeTab === "playlists" && playlistsLoading);
  const deviceCount = devices?.body?.devices?.length ?? 0;
  const deviceWarning = devices?.ok && deviceCount === 0
    ? "Nenhum dispositivo Spotify ativo. Abre o Spotify no PC ou telemovel e deixa uma faixa pronta."
    : "";
  const visibleError = commandError || deviceWarning;

  const sendCommand = async (action) => {
    if (controlling) return;
    setControlling(true);
    setCommandError("");
    try {
      const result = await api.musicCommand(action);
      if (!result?.ok) {
        const message = result?.hint || result?.body?.error?.message || result?.error || "Comando recusado pelo Spotify";
        setCommandError(message);
        return;
      }
      if (action === "play") setPlaying(true);
      if (action === "pause") setPlaying(false);
      if (action === "next") { setCurrentIdx((i) => (i + 1) % Math.max(currentTracks.length, 1)); setProgress(0); }
      if (action === "prev") { setCurrentIdx((i) => (i - 1 + Math.max(currentTracks.length, 1)) % Math.max(currentTracks.length, 1)); setProgress(0); }
    } catch (error) {
      setCommandError(error.message || "Falha ao contactar o backend");
    } finally {
      setTimeout(() => setControlling(false), 300);
    }
  };

  return (
    <div className="flex h-full flex-col bg-bg-0">
      {/* Header */}
      <div className="flex items-center gap-3" style={{ height: "56px", padding: "0 16px", background: "linear-gradient(180deg, var(--bg-1) 0%, var(--bg-0) 100%)", borderBottom: "1px solid var(--stroke-soft)" }}>
        <button onClick={onBack} className="flex h-9 w-9 items-center justify-center rounded-xl" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", cursor: "pointer" }}>
          <ArrowLeft size={16} style={{ color: "var(--text-secondary)" }} />
        </button>
        <div className="flex-1 text-center"><h1 className="font-display text-base font-bold text-text-primary">Spotify</h1></div>
        <Heart size={18} style={{ color: "var(--success)" }} />
      </div>

      {/* Tabs */}
      <div className="flex gap-1 px-4 py-2 overflow-x-auto">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            onClick={() => { setActiveTab(tab.id); setCurrentIdx(0); }}
            className="flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium whitespace-nowrap transition-all"
            style={{
              background: activeTab === tab.id ? "linear-gradient(135deg, var(--success), #22c55e)" : "var(--panel-soft)",
              border: `1px solid ${activeTab === tab.id ? "var(--success)" : "var(--stroke-soft)"}`,
              color: activeTab === tab.id ? "var(--bg-0)" : "var(--text-secondary)",
            }}
          >
            <tab.icon size={12} />
            <span>{tab.label}</span>
          </button>
        ))}
      </div>

      {visibleError && (
        <div className="mx-4 mb-2 rounded-xl px-3 py-2 text-[11px]" style={{ background: "var(--warning-tint)", color: "var(--warning)" }}>
          {visibleError}
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-4 pb-4">
        {isLoading && !currentTracks.length ? (
          <div className="flex items-center justify-center h-32">
            <Loader2 size={20} className="animate-spin" style={{ color: "var(--success)" }} />
          </div>
        ) : activeTab === "playlists" ? (
          /* Playlists grid */
          <div className="grid grid-cols-2 gap-3">
            {currentTracks.map((playlist, idx) => (
              <div
                key={playlist.id || idx}
                className="flex flex-col rounded-xl p-3 transition-all cursor-pointer hover:scale-[1.02]"
                style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)" }}
              >
                <div className={`flex h-16 w-full items-center justify-center rounded-lg bg-gradient-to-br ${getGradient(playlist.id || idx)}`}>
                  <Music size={24} style={{ color: "var(--bg-0)" }} />
                </div>
                <p className="mt-2 text-xs font-semibold text-text-primary truncate">{playlist.name}</p>
                <p className="text-[10px] text-text-muted">{playlist.tracks} faixas</p>
              </div>
            ))}
          </div>
        ) : hasTracks ? (
          /* Track list */
          <div className="space-y-2">
            {currentTracks.map((track, idx) => (
              <div
                key={track.id || idx}
                onClick={() => setCurrentIdx(idx)}
                className="flex items-center gap-3 rounded-xl p-3 transition-all duration-150 cursor-pointer"
                style={{
                  background: currentIdx === idx ? "var(--success-tint)" : "var(--panel-soft)",
                  border: `1px solid ${currentIdx === idx ? "var(--success)/40" : "var(--stroke-soft)"}`,
                }}
              >
                <div className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br ${getGradient(track.id || idx)}`}>
                  <Music size={16} style={{ color: "var(--bg-0)" }} />
                </div>
                <div className="flex-1 min-w-0 text-left">
                  <p className="text-sm font-semibold text-text-primary truncate">{track.name}</p>
                  <p className="text-[11px] text-text-muted truncate">{track.artists?.map((a) => a.name).join(", ") || "Artista"}</p>
                </div>
                {currentIdx === idx && (
                  <div className="flex h-6 w-6 items-center justify-center rounded-full" style={{ background: "var(--success)" }}>
                    <div className="h-2 w-2 animate-pulse rounded-full bg-bg-0" />
                  </div>
                )}
              </div>
            ))}
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center h-32 text-center">
            <Music size={24} style={{ color: "var(--text-dim)" }} />
            <p className="text-sm text-text-muted mt-2">Sem faixas</p>
            <a
              href={api.spotifyLoginUrl()}
              target="_blank"
              rel="noopener noreferrer"
              className="mt-3 inline-flex items-center gap-1.5 rounded-xl px-4 py-2 text-sm font-medium"
              style={{ background: "linear-gradient(135deg, var(--success), #22c55e)", color: "var(--bg-0)", textDecoration: "none" }}
            >
              Ligar ao Spotify
            </a>
          </div>
        )}
      </div>

      {/* Now playing bar */}
      {currentTrack && activeTab !== "playlists" && (
        <div
          className="flex items-center gap-3 px-4 py-3 animate-slide-up"
          style={{
            background: "linear-gradient(90deg, var(--success-tint), var(--panel-soft))",
            borderTop: "1px solid var(--success)/30",
          }}
        >
          <div className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br ${getGradient(currentTrack.id || 0)}`}>
            <Music size={14} style={{ color: "var(--bg-0)" }} />
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-xs font-semibold text-text-primary truncate">{currentTrack.name}</p>
            <p className="text-[10px] text-text-muted">{currentTrack.artists?.map((a) => a.name).join(", ")}</p>
          </div>
          <button className="flex h-9 w-9 items-center justify-center rounded-full" style={{ background: "var(--success)", cursor: "pointer" }}>
            <Play size={14} style={{ color: "var(--bg-0)", marginLeft: "1px" }} />
          </button>
        </div>
      )}

      {/* Progress bar */}
      {currentTrack && activeTab !== "playlists" && (
        <div className="px-4">
          <div className="overflow-hidden" style={{ width: "100%", height: "6px", borderRadius: "3px", background: "var(--bg-2)" }}>
            <div style={{ width: `${progress}%`, height: "100%", borderRadius: "3px", background: "linear-gradient(90deg, var(--success), #22c55e)", transition: "width 0.3s ease" }} />
          </div>
          <div className="flex justify-between mt-1.5">
            <span className="font-mono text-[10px] text-text-muted">1:23</span>
            <span className="font-mono text-[10px] text-text-muted">3:45</span>
          </div>
        </div>
      )}

      {/* Controls */}
      {currentTrack && activeTab !== "playlists" && (
        <div className="flex items-center justify-between px-6 py-4" style={{ marginTop: "8px" }}>
          <IconButton size={36} onClick={() => {}}><Shuffle size={16} style={{ color: "var(--text-muted)" }} /></IconButton>
          <IconButton size={44} onClick={() => sendCommand("prev")}><SkipBack size={18} style={{ color: "var(--text-primary)" }} /></IconButton>
          <IconButton size={56} onClick={() => sendCommand(playing ? "pause" : "play")} gradient={playing ? "linear-gradient(135deg, var(--success), #22c55e)" : "linear-gradient(135deg, var(--accent-hi), var(--accent))"}>
            {controlling ? <Loader2 size={22} className="animate-spin" style={{ color: "var(--bg-0)" }} /> : playing ? <Pause size={22} style={{ color: "var(--bg-0)" }} /> : <Play size={22} style={{ color: "var(--bg-0)", marginLeft: "2px" }} />}
          </IconButton>
          <IconButton size={44} onClick={() => sendCommand("next")}><SkipForward size={18} style={{ color: "var(--text-primary)" }} /></IconButton>
          <IconButton size={36} onClick={() => {}}><Repeat size={16} style={{ color: "var(--text-muted)" }} /></IconButton>
        </div>
      )}
    </div>
  );
}
