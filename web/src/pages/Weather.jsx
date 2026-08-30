import { useState } from "react";
import { Cloud, Droplets, Wind } from "lucide-react";
import Panel from "../components/Panel";
import { api } from "../lib/api";
import { usePolledApi } from "../lib/useApi";

const REGIONS = ["Padrão", "Norte", "Centro", "Sul", "Ilhas"];

export default function Weather() {
  const [region, setRegion] = useState(0);
  const { data, offline, loading } = usePolledApi(() => api.weather(region), {
    intervalMs: 60000,
    deps: [region],
  });

  return (
    <div className="space-y-6">
      <Panel
        eyebrow="Open-Meteo"
        title="Clima atual"
        action={
          <select
            value={region}
            onChange={(e) => setRegion(Number(e.target.value))}
            className="rounded-s border border-stroke-soft bg-panel-soft px-2 py-1.5 text-xs text-text-secondary outline-none"
            style={{ borderRadius: "var(--radius-s)" }}
          >
            {REGIONS.map((r, i) => (
              <option key={r} value={i}>
                {r}
              </option>
            ))}
          </select>
        }
      >
        {offline && (
          <p className="mb-4 rounded-s border border-warning/30 bg-warning/10 px-3 py-2 text-xs text-warning" style={{ borderRadius: "var(--radius-s)" }}>
            Sem ligação ao backend — liga o <code>dc-os-core</code> (porta 8081) para dados reais.
          </p>
        )}

        {loading && !data ? (
          <p className="text-sm text-text-muted">A carregar…</p>
        ) : (
          <div className="flex flex-col items-center gap-6 py-4 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex items-center gap-4">
              <Cloud className="text-accent-cyan" size={56} />
              <div>
                <p className="font-display text-5xl font-semibold text-text-primary">
                  {data ? Math.round(data.temperature ?? data.temp ?? 0) : "--"}°
                </p>
                <p className="text-sm text-text-secondary">{data?.city ?? "Cidade indisponível"}</p>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4 text-sm text-text-secondary">
              <div className="flex items-center gap-2">
                <Droplets size={16} className="text-accent-blue" />
                {data?.humidity != null ? `${data.humidity}% humidade` : "—"}
              </div>
              <div className="flex items-center gap-2">
                <Wind size={16} className="text-accent-cyan" />
                {data?.wind_speed != null ? `${data.wind_speed} km/h` : "—"}
              </div>
            </div>
          </div>
        )}
        {data?.summary && <p className="mt-4 text-sm text-text-secondary">{data.summary}</p>}
      </Panel>
    </div>
  );
}
