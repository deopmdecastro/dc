import { Cpu, MemoryStick, HardDrive, Radio } from "lucide-react";
import Panel from "../components/Panel";
import { api } from "../lib/api";
import { usePolledApi } from "../lib/useApi";

const metrics = [
  { key: "cpu", label: "CPU", icon: Cpu, color: "text-accent" },
  { key: "memory", label: "Memória (PSRAM)", icon: MemoryStick, color: "text-accent-blue" },
  { key: "storage", label: "Flash", icon: HardDrive, color: "text-accent-violet" },
  { key: "network", label: "Wi-Fi", icon: Radio, color: "text-accent-cyan" },
];

export default function Resources() {
  const { data: health, offline } = usePolledApi(() => api.health(), { intervalMs: 10000 });

  return (
    <div className="space-y-6">
      <Panel eyebrow="Sistema" title="Recursos do dispositivo">
        {offline && (
          <p className="mb-4 rounded-s border border-warning/30 bg-warning/10 px-3 py-2 text-xs text-warning" style={{ borderRadius: "var(--radius-s)" }}>
            O backend não expõe ainda métricas detalhadas de hardware — esta página
            mostra o healthcheck de <code>/health</code> e placeholders para as
            métricas do ES3C28P.
          </p>
        )}
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
          {metrics.map(({ key, label, icon: Icon, color }) => (
            <div
              key={key}
              className="rounded-m border border-stroke-soft bg-panel-soft p-4"
              style={{ borderRadius: "var(--radius-m)" }}
            >
              <Icon className={color} size={20} />
              <p className="mt-3 text-xs text-text-muted">{label}</p>
              <p className="font-display text-xl font-semibold text-text-primary">—</p>
            </div>
          ))}
        </div>
      </Panel>

      <Panel eyebrow="Healthcheck" title="/health">
        <pre className="overflow-x-auto rounded-m border border-stroke-soft bg-bg-1 p-3 text-xs text-accent-hi" style={{ borderRadius: "var(--radius-m)" }}>
          {health ? JSON.stringify(health, null, 2) : "aguardando resposta do backend…"}
        </pre>
      </Panel>
    </div>
  );
}
