import { useState } from "react";
import { ArrowLeft, RefreshCw, Droplets, Wind, MapPin } from "lucide-react";
import { api } from "../lib/api";
import { usePolledApi } from "../lib/useApi";

const REGIONS = [
  { index: 0, name: "Brasília", country: "🇧🇷" },
  { index: 1, name: "Lisboa", country: "🇵🇹" },
  { index: 2, name: "Luanda", country: "🇦🇴" },
  { index: 3, name: "Maputo", country: "🇲🇿" },
  { index: 4, name: "New York", country: "🇺🇸" },
];

export default function Weather({ onBack }) {
  const [regionIdx, setRegionIdx] = useState(1); // Default: Lisboa
  const { data: weather, loading, offline, refresh } = usePolledApi(
    () => api.weather(regionIdx),
    { intervalMs: 60000, deps: [regionIdx] }
  );

  const region = REGIONS.find((r) => r.index === regionIdx) || REGIONS[1];
  const temp = Math.round(weather?.temperature_c ?? 0);
  const city = weather?.city ?? region.name;
  const summary = weather?.summary ?? "—";
  const weatherOk = weather?.ok !== false;

  return (
    <div className="flex h-full flex-col bg-bg-0">
      {/* Header */}
      <div className="flex items-center gap-3" style={{ height: "56px", padding: "0 16px", background: "linear-gradient(180deg, var(--bg-1) 0%, var(--bg-0) 100%)", borderBottom: "1px solid var(--stroke-soft)" }}>
        <button onClick={onBack} className="flex h-9 w-9 items-center justify-center rounded-xl" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", cursor: "pointer" }}>
          <ArrowLeft size={16} style={{ color: "var(--text-secondary)" }} />
        </button>
        <div className="flex-1"><h1 className="font-display text-base font-bold text-text-primary">Clima</h1></div>
        <button onClick={refresh} className="flex h-9 w-9 items-center justify-center rounded-xl" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", cursor: "pointer" }}>
          <RefreshCw size={14} style={{ color: "var(--text-muted)" }} />
        </button>
      </div>

      {/* Region selector */}
      <div className="px-4 py-3">
        <div className="flex gap-2 overflow-x-auto pb-1">
          {REGIONS.map((r) => (
            <button
              key={r.index}
              onClick={() => setRegionIdx(r.index)}
              className="flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium whitespace-nowrap transition-all"
              style={{
                background: regionIdx === r.index ? "linear-gradient(135deg, var(--accent), var(--accent-blue))" : "var(--panel-soft)",
                border: `1px solid ${regionIdx === r.index ? "var(--accent)" : "var(--stroke-soft)"}`,
                color: regionIdx === r.index ? "var(--bg-0)" : "var(--text-secondary)",
              }}
            >
              <span>{r.country}</span>
              <span>{r.name}</span>
            </button>
          ))}
        </div>
      </div>

      {/* Weather card */}
      <div className="flex-1 flex flex-col items-center justify-center px-6">
        <div
          className="w-full rounded-3xl p-6 text-center"
          style={{
            background: "linear-gradient(135deg, var(--accent-tint), rgba(56, 189, 248, 0.05))",
            border: "1px solid var(--accent)/20",
            boxShadow: "0 20px 60px -15px rgba(56, 189, 248, 0.2)",
          }}
        >
          {/* Icon */}
          <div
            className="mx-auto flex items-center justify-center"
            style={{
              width: "80px",
              height: "80px",
              borderRadius: "24px",
              background: "linear-gradient(135deg, var(--accent), var(--accent-cyan))",
              marginBottom: "16px",
            }}
          >
            <span style={{ fontSize: "36px" }}>{temp > 25 ? "☀️" : temp > 15 ? "⛅" : "🌧️"}</span>
          </div>

          {/* City */}
          <div className="flex items-center justify-center gap-1.5 mb-2">
            <MapPin size={14} style={{ color: "var(--text-muted)" }} />
            <p className="font-display text-lg font-bold text-text-primary">{city}</p>
          </div>

          {/* Temperature */}
          {loading && !weather ? (
            <div className="flex items-center justify-center py-4">
              <div className="h-12 w-24 rounded-lg bg-panel-soft animate-pulse-slow" />
            </div>
          ) : (
            <p className="font-display font-extrabold" style={{ fontSize: "64px", color: "var(--accent-hi)", lineHeight: 1.1 }}>
              {temp}°
            </p>
          )}

          {/* Summary */}
          <p className="text-sm text-text-secondary font-medium">{summary}</p>

          {/* Status badge */}
          {!weatherOk && (
            <div className="mt-3 inline-flex items-center gap-1 rounded-full px-2.5 py-1 text-[10px]" style={{ background: "var(--warning-tint)", color: "var(--warning)" }}>
              Dados indisponíveis
            </div>
          )}
        </div>

        {/* Attribution */}
        <p className="mt-4 text-[10px] text-text-dim">Open-Meteo · Atualizado a cada 60s</p>
      </div>
    </div>
  );
}
