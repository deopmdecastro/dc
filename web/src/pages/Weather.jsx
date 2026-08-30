import { ArrowLeft, Cloud, RefreshCw, Droplets, Wind } from "lucide-react";
import { api } from "../lib/api";
import { usePolledApi } from "../lib/useApi";

export default function Weather({ onBack }) {
  const { data: weather, loading, offline, refresh } = usePolledApi(() => api.weather(0), { intervalMs: 60000 });

  const temp = Math.round(weather?.temperature ?? weather?.temp ?? 0);
  const city = weather?.city ?? "Lisboa";
  const summary = weather?.summary ?? "Parcialmente nublado";
  const humidity = weather?.humidity ?? 65;
  const wind = weather?.wind_speed ?? 12;

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
        <div className="flex-1">
          <h1 className="font-display text-base font-bold text-text-primary">Clima</h1>
        </div>
        <button
          onClick={refresh}
          className="flex h-9 w-9 items-center justify-center rounded-xl"
          style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", cursor: "pointer" }}
        >
          <RefreshCw size={14} style={{ color: "var(--text-muted)" }} />
        </button>
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
            <Cloud size={40} style={{ color: "var(--bg-0)" }} />
          </div>

          {/* City */}
          <p className="font-display text-lg font-bold text-text-primary">{city}</p>

          {/* Temperature */}
          <p
            className="font-display font-extrabold"
            style={{ fontSize: "64px", color: "var(--accent-hi)", lineHeight: 1.1 }}
          >
            {temp}°
          </p>

          {/* Summary */}
          <p className="text-sm text-text-secondary font-medium">{summary}</p>
        </div>

        {/* Stats row */}
        <div className="grid grid-cols-2 gap-3 w-full mt-4">
          <div
            className="flex items-center gap-3 rounded-2xl p-4"
            style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)" }}
          >
            <div
              className="flex h-10 w-10 items-center justify-center rounded-xl"
              style={{ background: "linear-gradient(135deg, var(--accent-blue), #60a5fa)" }}
            >
              <Droplets size={18} style={{ color: "var(--bg-0)" }} />
            </div>
            <div>
              <p className="text-lg font-bold text-text-primary">{humidity}%</p>
              <p className="text-[10px] text-text-muted">Humidade</p>
            </div>
          </div>
          <div
            className="flex items-center gap-3 rounded-2xl p-4"
            style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)" }}
          >
            <div
              className="flex h-10 w-10 items-center justify-center rounded-xl"
              style={{ background: "linear-gradient(135deg, var(--accent-cyan), #5eead4)" }}
            >
              <Wind size={18} style={{ color: "var(--bg-0)" }} />
            </div>
            <div>
              <p className="text-lg font-bold text-text-primary">{wind} km/h</p>
              <p className="text-[10px] text-text-muted">Vento</p>
            </div>
          </div>
        </div>
      </div>

      {/* Attribution */}
      <p className="text-center text-[10px] text-text-dim py-3">Open-Meteo</p>
    </div>
  );
}
