import { Link } from "react-router-dom";
import { Cloud, MessageCircle, FileText, Bell, Settings, Zap, ArrowUpRight } from "lucide-react";
import Panel from "../components/Panel";
import { api } from "../lib/api";
import { usePolledApi } from "../lib/useApi";

const apps = [
  { to: "/assistente", name: "Assistente", desc: "Fala com o DC Assistant", icon: MessageCircle, color: "text-accent-blue", border: "border-accent-blue/40" },
  { to: "/spotify", name: "Spotify", desc: "Controla a música", icon: SpotifyDot, color: "text-success", border: "border-success/40" },
  { to: "/clima", name: "Clima", desc: "Previsão em tempo real", icon: Cloud, color: "text-accent-cyan", border: "border-accent-cyan/40" },
  { to: "/recursos", name: "Recursos", desc: "CPU, memória, rede", icon: Zap, color: "text-accent-violet", border: "border-accent-violet/40" },
  { to: "/notas", name: "Notas", desc: "Apontamentos rápidos", icon: FileText, color: "text-warning", border: "border-warning/40" },
  { to: "/alarme", name: "Alarme", desc: "Define alarmes", icon: Bell, color: "text-accent-pink", border: "border-accent-pink/40" },
  { to: "/definicoes", name: "Definições", desc: "Wi-Fi, região, PIN", icon: Settings, color: "text-text-secondary", border: "border-stroke" },
];

function SpotifyDot(props) {
  return (
    <svg viewBox="0 0 24 24" width={props.size ?? 18} height={props.size ?? 18} fill="currentColor" className={props.className}>
      <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.586 14.424a.622.622 0 0 1-.857.207c-2.348-1.435-5.304-1.76-8.785-.964a.622.622 0 1 1-.277-1.215c3.809-.871 7.077-.496 9.713 1.115a.623.623 0 0 1 .206.857zm1.223-2.723a.78.78 0 0 1-1.072.257c-2.687-1.652-6.785-2.131-9.965-1.166a.78.78 0 1 1-.453-1.492c3.635-1.102 8.147-.568 11.233 1.329a.78.78 0 0 1 .257 1.072zm.105-2.835C14.692 9.128 9.375 8.95 6.297 9.883a.936.936 0 1 1-.543-1.79c3.533-1.072 9.404-.865 13.115 1.339a.936.936 0 0 1-.955 1.612z" />
    </svg>
  );
}

export default function Home() {
  const { data: health, offline } = usePolledApi(() => api.health(), { intervalMs: 15000 });
  const { data: weather } = usePolledApi(() => api.weather(0), { intervalMs: 60000 });

  return (
    <div className="space-y-6">
      <Panel className="relative overflow-hidden bg-noise">
        <div className="pointer-events-none absolute -right-16 -top-20 h-64 w-64 rounded-full bg-accent/10 blur-3xl" />
        <div className="relative flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
          <div>
            <p className="mb-1 text-[11px] font-semibold uppercase tracking-wider text-accent">DC Assistant</p>
            <h1 className="font-display text-2xl font-semibold text-text-primary sm:text-3xl">
              Bem-vindo de volta.
            </h1>
            <p className="mt-1 max-w-md text-sm text-text-secondary">
              Versão web do DC OS — o mesmo assistente do teu ES3C28P, agora em
              React + Tailwind, ligado ao <code className="rounded bg-panel-elevated px-1 py-0.5 text-xs">dc-os-core</code>.
            </p>
          </div>
          <div
            className="flex items-center gap-2 self-start rounded-full border px-3 py-1.5 text-xs font-medium"
            style={{
              borderColor: offline ? "var(--color-danger)" : "var(--color-success)",
              color: offline ? "var(--color-danger)" : "var(--color-success)",
              background: offline ? "var(--color-danger-tint)" : "var(--color-success-tint)",
            }}
          >
            <span className={`h-1.5 w-1.5 rounded-full ${offline ? "bg-danger" : "bg-success animate-pulse-slow"}`} />
            {offline ? "Backend offline" : `Ligado · ${health?.status ?? "ok"}`}
          </div>
        </div>
      </Panel>

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
        <Panel eyebrow="Agora" title="Clima" className="lg:col-span-1">
          {weather ? (
            <div className="flex items-end justify-between">
              <div>
                <p className="font-display text-4xl font-semibold text-text-primary">
                  {Math.round(weather.temperature ?? weather.temp ?? 0)}°
                </p>
                <p className="text-sm text-text-secondary">{weather.city ?? weather.summary ?? "—"}</p>
              </div>
              <Cloud className="text-accent-cyan" size={40} />
            </div>
          ) : (
            <p className="text-sm text-text-muted">
              {offline ? "Sem ligação ao backend (usa dados de exemplo em /clima)." : "A carregar…"}
            </p>
          )}
        </Panel>

        <Panel eyebrow="7 apps" title="Aplicações" className="lg:col-span-2">
          <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
            {apps.map(({ to, name, icon: Icon, color, border }) => (
              <Link
                key={to}
                to={to}
                className={`group flex flex-col items-start gap-3 rounded-m border bg-panel-soft p-3 transition-colors hover:bg-panel-elevated ${border}`}
                style={{ borderRadius: "var(--radius-m)" }}
              >
                <span className={`flex h-9 w-9 items-center justify-center rounded-s border border-current/40 bg-panel-elevated ${color}`} style={{ borderRadius: "var(--radius-s)" }}>
                  <Icon size={18} />
                </span>
                <span className="text-sm font-medium text-text-primary">{name}</span>
                <ArrowUpRight size={13} className="text-text-dim transition-colors group-hover:text-accent" />
              </Link>
            ))}
          </div>
        </Panel>
      </div>
    </div>
  );
}
