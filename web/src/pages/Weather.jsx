import { useState, useEffect, useCallback } from "react";
import { ArrowLeft, RefreshCw, MapPin, Navigation, Loader2, Droplets, Wind } from "lucide-react";
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
  const [locationMode, setLocationMode] = useState("gps"); // "gps" or "manual"
  const [gpsCoords, setGpsCoords] = useState(null);
  const [gpsError, setGpsError] = useState(null);
  const [regionIdx, setRegionIdx] = useState(1);
  const [loadingGps, setLoadingGps] = useState(false);

  // Get GPS location
  const getGpsLocation = useCallback(() => {
    if (!navigator.geolocation) {
      setGpsError("Geolocalização não suportada");
      setLocationMode("manual");
      return;
    }

    setLoadingGps(true);
    setGpsError(null);

    navigator.geolocation.getCurrentPosition(
      (position) => {
        setGpsCoords({
          lat: position.coords.latitude,
          lon: position.coords.longitude,
        });
        setLocationMode("gps");
        setLoadingGps(false);
      },
      (error) => {
        setGpsError(error.message === "User denied Geolocation" ? "Permissão negada" : "Erro ao obter localização");
        setLocationMode("manual");
        setLoadingGps(false);
      },
      { enableHighAccuracy: true, timeout: 10000, maximumAge: 300000 }
    );
  }, []);

  // Try GPS on mount
  useEffect(() => {
    getGpsLocation();
  }, [getGpsLocation]);

  // Fetch weather based on mode
  const weatherFetcher = useCallback(() => {
    if (locationMode === "gps" && gpsCoords) {
      return api.weatherByCoords(gpsCoords.lat, gpsCoords.lon);
    }
    return api.weather(regionIdx);
  }, [locationMode, gpsCoords, regionIdx]);

  const { data: weather, loading, refresh } = usePolledApi(weatherFetcher, {
    intervalMs: 60000,
    deps: [locationMode, gpsCoords, regionIdx],
  });

  const temp = Math.round(weather?.temperature_c ?? 0);
  const city = weather?.city ?? "A localizar...";
  const summary = weather?.summary ?? "—";
  const humidity = weather?.humidity_percent ?? weather?.humidity;
  const wind = weather?.wind_kmh ?? weather?.wind_speed_kmh;
  const weatherOk = weather?.ok !== false;

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

      {/* Location mode toggle */}
      <div className="px-4 py-3">
        <div className="flex gap-2">
          <button
            onClick={() => { setLocationMode("gps"); if (!gpsCoords) getGpsLocation(); }}
            className="flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium transition-all"
            style={{
              background: locationMode === "gps" ? "linear-gradient(135deg, var(--accent), var(--accent-blue))" : "var(--panel-soft)",
              border: `1px solid ${locationMode === "gps" ? "var(--accent)" : "var(--stroke-soft)"}`,
              color: locationMode === "gps" ? "var(--bg-0)" : "var(--text-secondary)",
            }}
          >
            <Navigation size={12} />
            GPS
          </button>
          <button
            onClick={() => setLocationMode("manual")}
            className="flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium transition-all"
            style={{
              background: locationMode === "manual" ? "linear-gradient(135deg, var(--accent), var(--accent-blue))" : "var(--panel-soft)",
              border: `1px solid ${locationMode === "manual" ? "var(--accent)" : "var(--stroke-soft)"}`,
              color: locationMode === "manual" ? "var(--bg-0)" : "var(--text-secondary)",
            }}
          >
            <MapPin size={12} />
            Manual
          </button>
        </div>

        {/* GPS error / loading */}
        {locationMode === "gps" && loadingGps && (
          <div className="mt-2 flex items-center gap-2 text-xs text-text-muted">
            <Loader2 size={12} className="animate-spin" />
            A obter localização GPS...
          </div>
        )}
        {locationMode === "gps" && gpsError && (
          <div className="mt-2 flex items-center gap-2 text-xs" style={{ color: "var(--warning)" }}>
            <MapPin size={12} />
            {gpsError} — a usar localização manual
          </div>
        )}
        {locationMode === "gps" && gpsCoords && (
          <div className="mt-2 flex items-center gap-2 text-xs text-text-muted">
            <Navigation size={12} style={{ color: "var(--success)" }} />
            {gpsCoords.lat.toFixed(4)}, {gpsCoords.lon.toFixed(4)}
          </div>
        )}
      </div>

      {/* Region selector (manual mode) */}
      {locationMode === "manual" && (
        <div className="px-4 pb-3">
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
      )}

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
            <p
              className="font-display font-extrabold"
              style={{ fontSize: "64px", color: "var(--accent-hi)", lineHeight: 1.1 }}
            >
              {temp}°
            </p>
          )}

          {/* Summary */}
          <p className="text-sm text-text-secondary font-medium">{summary}</p>

          <div className="mt-5 grid grid-cols-2 gap-3">
            <div className="rounded-2xl p-3" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)" }}>
              <Droplets size={16} className="mx-auto mb-1" style={{ color: "var(--accent)" }} />
              <p className="text-[10px] text-text-muted">Humidade</p>
              <p className="text-sm font-bold text-text-primary">{humidity != null ? `${humidity}%` : "--"}</p>
            </div>
            <div className="rounded-2xl p-3" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)" }}>
              <Wind size={16} className="mx-auto mb-1" style={{ color: "var(--accent)" }} />
              <p className="text-[10px] text-text-muted">Vento</p>
              <p className="text-sm font-bold text-text-primary">{wind != null ? `${wind} km/h` : "--"}</p>
            </div>
          </div>

          {/* Status badge */}
          {!weatherOk && (
            <div
              className="mt-3 inline-flex items-center gap-1 rounded-full px-2.5 py-1 text-[10px]"
              style={{ background: "var(--warning-tint)", color: "var(--warning)" }}
            >
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
