import { ArrowLeft, Wifi, Cloud, Bluetooth, Zap, Activity } from "lucide-react";
import { usePolledApi } from "../lib/useApi";
import { api } from "../lib/api";

const features = [
  { icon: Zap, title: "Spotify", detail: "Conectado · A tocar", color: "var(--success)", gradient: "from-green-500 to-emerald-400", bg: "var(--success-tint)" },
  { icon: Wifi, title: "Wi-Fi", detail: "Ligado · Rede ativa", color: "var(--accent-blue)", gradient: "from-blue-500 to-cyan-400", bg: "var(--accent-tint)" },
  { icon: Cloud, title: "Clima", detail: "Ativo · Lisboa", color: "var(--accent-cyan)", gradient: "from-teal-500 to-cyan-400", bg: "var(--accent-tint)" },
  { icon: Bluetooth, title: "Bluetooth", detail: "Desligado", color: "var(--accent-violet)", gradient: "from-violet-500 to-purple-400", bg: "var(--panel-soft)" },
];

export default function Features({ onBack }) {
  const { offline } = usePolledApi(() => api.health(), { intervalMs: 15000 });

  return (
    <div className="flex h-full flex-col bg-bg-0">
      {/* Header */}
      <div className="flex items-center gap-3" style={{ height: "56px", padding: "0 16px", background: "linear-gradient(180deg, var(--bg-1) 0%, var(--bg-0) 100%)", borderBottom: "1px solid var(--stroke-soft)" }}>
        <button onClick={onBack} className="flex h-9 w-9 items-center justify-center rounded-xl" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", cursor: "pointer" }}>
          <ArrowLeft size={16} style={{ color: "var(--text-secondary)" }} />
        </button>
        <div className="flex-1"><h1 className="font-display text-base font-bold text-text-primary">Recursos</h1></div>
        <div className="flex items-center gap-1.5">
          <Activity size={12} style={{ color: offline ? "var(--danger)" : "var(--success)" }} />
          <span className="text-[10px] font-medium" style={{ color: offline ? "var(--danger)" : "var(--success)" }}>{offline ? "Offline" : "Online"}</span>
        </div>
      </div>

      {/* Feature cards */}
      <div className="flex-1 overflow-y-auto p-4">
        <div className="grid grid-cols-2 gap-3">
          {features.map((f, i) => (
            <div key={i} className="rounded-2xl p-4 transition-all hover:scale-[1.02]" style={{ background: f.bg, border: `1px solid ${f.color}30` }}>
              <div className={`flex h-11 w-11 items-center justify-center rounded-xl bg-gradient-to-br ${f.gradient}`}>
                <f.icon size={20} style={{ color: "var(--bg-0)" }} />
              </div>
              <p className="mt-3 text-sm font-bold text-text-primary">{f.title}</p>
              <p className="text-[10px] text-text-muted mt-0.5">{f.detail}</p>
            </div>
          ))}
        </div>

        {/* System info */}
        <div className="mt-4 rounded-2xl p-4" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)" }}>
          <p className="text-xs font-semibold text-text-primary mb-2">Sistema</p>
          <div className="space-y-1.5">
            <div className="flex justify-between text-[10px]"><span className="text-text-muted">Backend</span><span className="text-text-secondary">{api.baseUrl}</span></div>
            <div className="flex justify-between text-[10px]"><span className="text-text-muted">Estado</span><span style={{ color: offline ? "var(--danger)" : "var(--success)" }}>{offline ? "Desconectado" : "Conectado"}</span></div>
            <div className="flex justify-between text-[10px]"><span className="text-text-muted">Rede</span><span className="text-text-secondary">Wi-Fi</span></div>
          </div>
        </div>
      </div>
    </div>
  );
}
