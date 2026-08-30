import { MessageCircle, Cloud, FileText, Bell, Settings, Zap, Music, Radio } from "lucide-react";
import { usePolledApi } from "../lib/useApi";
import { api } from "../lib/api";

const SpotifyIcon = (props) => (
  <svg viewBox="0 0 24 24" width={22} height={22} fill="currentColor" {...props}>
    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.586 14.424a.622.622 0 0 1-.857.207c-2.348-1.435-5.304-1.76-8.785-.964a.622.622 0 1 1-.277-1.215c3.809-.871 7.077-.496 9.713 1.115a.623.623 0 0 1 .206.857zm1.223-2.723a.78.78 0 0 1-1.072.257c-2.687-1.652-6.785-2.131-9.965-1.166a.78.78 0 1 1-.453-1.492c3.635-1.102 8.147-.568 11.233 1.329a.78.78 0 0 1 .257 1.072zm.105-2.835C14.692 9.128 9.375 8.95 6.297 9.883a.936.936 0 1 1-.543-1.79c3.533-1.072 9.404-.865 13.115 1.339a.936.936 0 0 1-.955 1.612z" />
  </svg>
);

const SongShareIcon = (props) => (
  <svg viewBox="0 0 24 24" width={22} height={22} fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" {...props}>
    <path d="M9 18V5l12-2v13" /><circle cx="6" cy="18" r="3" /><circle cx="18" cy="16" r="3" />
  </svg>
);

const apps = [
  { screen: "assistant", name: "Assistente", desc: "Chat & Voz", icon: MessageCircle, gradient: "from-blue-500 to-cyan-400", bg: "bg-blue-500/10", border: "border-blue-500/30" },
  { screen: "music", name: "Spotify", desc: "Música", icon: SpotifyIcon, gradient: "from-green-500 to-emerald-400", bg: "bg-green-500/10", border: "border-green-500/30" },
  { screen: "songshare", name: "SongShare", desc: "Descobrir", icon: SongShareIcon, gradient: "from-orange-500 to-amber-400", bg: "bg-orange-500/10", border: "border-orange-500/30" },
  { screen: "weather", name: "Clima", desc: "Previsão", icon: Cloud, gradient: "from-teal-500 to-cyan-400", bg: "bg-teal-500/10", border: "border-teal-500/30" },
  { screen: "features", name: "Recursos", desc: "Sistema", icon: Zap, gradient: "from-violet-500 to-purple-400", bg: "bg-violet-500/10", border: "border-violet-500/30" },
  { screen: "notes", name: "Notas", desc: "Apontamentos", icon: FileText, gradient: "from-yellow-500 to-orange-400", bg: "bg-yellow-500/10", border: "border-yellow-500/30" },
  { screen: "alarm", name: "Alarme", desc: "Despertar", icon: Bell, gradient: "from-pink-500 to-rose-400", bg: "bg-pink-500/10", border: "border-pink-500/30" },
  { screen: "settings", name: "Definições", desc: "Config", icon: Settings, gradient: "from-slate-400 to-slate-500", bg: "bg-slate-500/10", border: "border-slate-500/30" },
];

export default function AppLauncher({ onNavigate }) {
  const { data: weather, loading: weatherLoading } = usePolledApi(() => api.weather(1), { intervalMs: 60000 });
  const { data: tracks, loading: tracksLoading } = usePolledApi(() => api.musicTopTracks(true), { intervalMs: 30000 });
  const { offline } = usePolledApi(() => api.health(), { intervalMs: 15000 });

  const temp = Math.round(weather?.temperature_c ?? 0);
  const city = weather?.city ?? "Lisboa";
  const topTracks = tracks?.body?.items?.slice(0, 3) || [];

  return (
    <div className="flex h-full flex-col overflow-y-auto bg-bg-0" style={{ padding: "24px" }}>
      {/* Welcome section */}
      <div className="mb-6">
        <p className="text-[11px] font-semibold uppercase tracking-wider text-accent mb-1">DC Assistant</p>
        <h1 className="font-display text-2xl font-bold text-text-primary sm:text-3xl">Bem-vindo de volta.</h1>
        <p className="mt-1 text-sm text-text-secondary">
          8 aplicações · {offline ? <span className="text-danger">Offline</span> : <span className="text-success">Online</span>}
        </p>
      </div>

      {/* Quick info cards */}
      <div className="grid grid-cols-2 gap-3 mb-6">
        {/* Weather mini card */}
        <div
          className="rounded-2xl p-4"
          style={{
            background: "linear-gradient(135deg, var(--accent-tint), var(--panel-soft))",
            border: "1px solid var(--accent)/20",
          }}
        >
          <div className="flex items-center gap-2 mb-2">
            <div className="flex h-8 w-8 items-center justify-center rounded-lg" style={{ background: "linear-gradient(135deg, var(--accent), var(--accent-cyan))" }}>
              <Cloud size={14} style={{ color: "var(--bg-0)" }} />
            </div>
            <span className="text-[10px] font-semibold text-text-muted uppercase">Clima</span>
          </div>
          {weatherLoading && !weather ? (
            <div className="h-8 w-16 rounded bg-panel-soft animate-pulse-slow" />
          ) : (
            <>
              <p className="font-display text-2xl font-bold text-text-primary">{temp}°</p>
              <p className="text-[10px] text-text-muted">{city}</p>
            </>
          )}
        </div>

        {/* Music mini card */}
        <div
          className="rounded-2xl p-4"
          style={{
            background: "linear-gradient(135deg, var(--success-tint), var(--panel-soft))",
            border: "1px solid var(--success)/20",
          }}
        >
          <div className="flex items-center gap-2 mb-2">
            <div className="flex h-8 w-8 items-center justify-center rounded-lg" style={{ background: "linear-gradient(135deg, var(--success), #22c55e)" }}>
              <Music size={14} style={{ color: "var(--bg-0)" }} />
            </div>
            <span className="text-[10px] font-semibold text-text-muted uppercase">Música</span>
          </div>
          {tracksLoading && !tracks ? (
            <div className="h-8 w-16 rounded bg-panel-soft animate-pulse-slow" />
          ) : topTracks.length > 0 ? (
            <p className="text-xs font-semibold text-text-primary truncate">{topTracks[0].name}</p>
          ) : (
            <p className="text-xs text-text-muted">Sem faixas</p>
          )}
        </div>
      </div>

      {/* App grid */}
      <h2 className="text-sm font-semibold text-text-secondary mb-3">Aplicações</h2>
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
        {apps.map((app) => (
          <button
            key={app.screen}
            onClick={() => onNavigate(app.screen)}
            className={`group relative flex flex-col items-start gap-3 rounded-2xl border p-4 transition-all duration-150 hover-lift ${app.bg} ${app.border}`}
          >
            <div className={`flex h-11 w-11 items-center justify-center rounded-xl bg-gradient-to-br ${app.gradient}`}>
              <app.icon size={20} style={{ color: "var(--bg-0)" }} />
            </div>
            <div className="text-left">
              <span className="block text-sm font-semibold text-text-primary">{app.name}</span>
              <span className="block text-[10px] text-text-muted">{app.desc}</span>
            </div>
            <div className="absolute right-3 top-3 opacity-0 transition-opacity group-hover:opacity-100">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" style={{ color: "var(--text-dim)" }}>
                <path d="M7 17L17 7M17 7H7M17 7v10" />
              </svg>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
