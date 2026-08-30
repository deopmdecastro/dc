import { useState, useEffect } from "react";
import { ArrowLeft, Play, Pause, SkipBack, SkipForward, Shuffle, Repeat, Heart, Loader2 } from "lucide-react";
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

export default function MusicPlayer({ onBack }) {
  const { data: tracks, loading: tracksLoading } = usePolledApi(() => api.musicTopTracks(true), { intervalMs: 30000 });
  const [currentIdx, setCurrentIdx] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [progress, setProgress] = useState(30);
  const [controlling, setControlling] = useState(false);

  const trackList = tracks?.body?.items || [];
  const track = trackList[currentIdx] || null;
  const hasTracks = trackList.length > 0;

  const sendCommand = async (action) => {
    if (controlling) return;
    setControlling(true);
    try {
      await api.musicCommand(action);
      if (action === "play") setPlaying(true);
      if (action === "pause") setPlaying(false);
      if (action === "next") { setCurrentIdx((i) => (i + 1) % Math.max(trackList.length, 1)); setProgress(0); }
      if (action === "prev") { setCurrentIdx((i) => (i - 1 + Math.max(trackList.length, 1)) % Math.max(trackList.length, 1)); setProgress(0); }
    } catch {
      // silent
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
        <div className="flex-1 text-center"><h1 className="font-display text-base font-bold text-text-primary">A tocar</h1></div>
        <Heart size={18} style={{ color: "var(--success)" }} />
      </div>

      {/* Content */}
      <div className="flex flex-1 flex-col items-center justify-center px-8">
        {tracksLoading && !tracks ? (
          <div className="flex items-center justify-center">
            <Loader2 size={24} className="animate-spin" style={{ color: "var(--accent)" }} />
          </div>
        ) : hasTracks ? (
          <>
            {/* Album art */}
            <div
              className="flex items-center justify-center"
              style={{
                width: "140px",
                height: "140px",
                borderRadius: "20px",
                background: "linear-gradient(135deg, var(--success-tint), var(--panel-elevated))",
                border: "1px solid var(--stroke-soft)",
                marginBottom: "24px",
                boxShadow: "0 20px 60px -15px rgba(74, 222, 128, 0.3)",
              }}
            >
              <div className="flex items-center justify-center" style={{ width: "60px", height: "60px", borderRadius: "30px", background: "linear-gradient(135deg, var(--success), #22c55e)" }}>
                <span style={{ fontSize: "24px", color: "var(--bg-0)" }}>♪</span>
              </div>
            </div>

            {/* Track info */}
            <div className="text-center w-full">
              <p className="font-display text-lg font-bold text-text-primary truncate">{track?.name || "Faixa"}</p>
              <p className="text-sm text-text-secondary truncate" style={{ marginTop: "2px" }}>{track?.artists?.map((a) => a.name).join(", ") || "Artista"}</p>
              {track?.album?.name && <p className="text-[11px] text-text-muted truncate" style={{ marginTop: "1px" }}>{track.album.name}</p>}
            </div>
          </>
        ) : (
          <div className="text-center">
            <div className="mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-2xl" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)" }}>
              <span style={{ fontSize: "32px" }}>🎵</span>
            </div>
            <p className="text-sm text-text-secondary">Sem faixas disponíveis</p>
            <p className="text-[11px] text-text-dim mt-1">Configura Spotify para ver as tuas faixas</p>
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

      {/* Progress bar */}
      {hasTracks && (
        <div className="px-6">
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
      {hasTracks && (
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
