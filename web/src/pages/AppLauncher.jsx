import {
  ArrowUpRight,
  Bell,
  Cloud,
  Droplets,
  FileText,
  MessageCircle,
  Music,
  Play,
  Settings,
  SkipBack,
  SkipForward,
  SlidersHorizontal,
  Wind,
  Zap,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { api } from "../lib/api";
import { usePolledApi } from "../lib/useApi";

const SpotifyIcon = (props) => (
  <svg viewBox="0 0 24 24" width={22} height={22} fill="currentColor" {...props}>
    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.586 14.424a.622.622 0 0 1-.857.207c-2.348-1.435-5.304-1.76-8.785-.964a.622.622 0 1 1-.277-1.215c3.809-.871 7.077-.496 9.713 1.115a.623.623 0 0 1 .206.857zm1.223-2.723a.78.78 0 0 1-1.072.257c-2.687-1.652-6.785-2.131-9.965-1.166a.78.78 0 1 1-.453-1.492c3.635-1.102 8.147-.568 11.233 1.329a.78.78 0 0 1 .257 1.072zm.105-2.835C14.692 9.128 9.375 8.95 6.297 9.883a.936.936 0 1 1-.543-1.79c3.533-1.072 9.404-.865 13.115 1.339a.936.936 0 0 1-.955 1.612z" />
  </svg>
);

const SongShareIcon = (props) => (
  <svg
    viewBox="0 0 24 24"
    width={22}
    height={22}
    fill="none"
    stroke="currentColor"
    strokeLinecap="round"
    strokeLinejoin="round"
    strokeWidth={2}
    {...props}
  >
    <path d="M9 18V5l12-2v13" />
    <circle cx="6" cy="18" r="3" />
    <circle cx="18" cy="16" r="3" />
  </svg>
);

const apps = [
  {
    screen: "assistant",
    name: "Assistente",
    desc: "Chat & Voz",
    icon: MessageCircle,
    color: "#13a8ff",
    tint: "rgba(19, 168, 255, 0.14)",
  },
  {
    screen: "music",
    name: "Spotify",
    desc: "Musica",
    icon: SpotifyIcon,
    color: "#24e07a",
    tint: "rgba(36, 224, 122, 0.13)",
  },
  {
    screen: "songshare",
    name: "SongShare",
    desc: "Descobrir",
    icon: SongShareIcon,
    color: "#fb9709",
    tint: "rgba(251, 151, 9, 0.14)",
  },
  {
    screen: "weather",
    name: "Clima",
    desc: "Previsao",
    icon: Cloud,
    color: "#16d0d5",
    tint: "rgba(22, 208, 213, 0.13)",
  },
  {
    screen: "features",
    name: "Recursos",
    desc: "Sistema",
    icon: Zap,
    color: "#9a58ff",
    tint: "rgba(154, 88, 255, 0.14)",
  },
  {
    screen: "notes",
    name: "Notas",
    desc: "Apontamentos",
    icon: FileText,
    color: "#ffb30a",
    tint: "rgba(255, 179, 10, 0.14)",
  },
  {
    screen: "alarm",
    name: "Alarme",
    desc: "Despertar",
    icon: Bell,
    color: "#f23c91",
    tint: "rgba(242, 60, 145, 0.14)",
  },
  {
    screen: "settings",
    name: "Definicoes",
    desc: "Config",
    icon: Settings,
    color: "#91a5be",
    tint: "rgba(145, 165, 190, 0.13)",
  },
];

function InfoMetric({ icon: Icon, label, value }) {
  return (
    <div className="dc-info-metric">
      <Icon size={15} />
      <div>
        <span>{label}</span>
        <strong>{value}</strong>
      </div>
    </div>
  );
}

function AppCard({ app, onNavigate }) {
  return (
    <button
      className="dc-app-card"
      type="button"
      onClick={() => onNavigate(app.screen)}
      style={{
        "--app-color": app.color,
        "--app-tint": app.tint,
      }}
    >
      <div className="dc-app-icon">
        <app.icon size={25} />
      </div>
      <div className="dc-app-copy">
        <strong>{app.name}</strong>
        <span>{app.desc}</span>
      </div>
      <div className="dc-app-action">
        <ArrowUpRight size={16} />
      </div>
    </button>
  );
}

export default function AppLauncher({ onNavigate, online = true }) {
  const [gpsCoords, setGpsCoords] = useState(null);

  useEffect(() => {
    if (!navigator.geolocation) return;

    navigator.geolocation.getCurrentPosition(
      (position) => {
        setGpsCoords({
          lat: position.coords.latitude,
          lon: position.coords.longitude,
        });
      },
      () => {
        setGpsCoords(null);
      },
      { enableHighAccuracy: true, maximumAge: 300000, timeout: 10000 },
    );
  }, []);

  const weatherFetcher = useCallback(() => {
    if (gpsCoords) {
      return api.weatherByCoords(gpsCoords.lat, gpsCoords.lon);
    }
    return api.weather(1);
  }, [gpsCoords]);

  const { data: weather, loading: weatherLoading } = usePolledApi(weatherFetcher, {
    intervalMs: 60000,
    deps: [gpsCoords?.lat, gpsCoords?.lon],
  });
  const { data: tracks, loading: tracksLoading } = usePolledApi(() => api.songshareTracks(true), {
    intervalMs: 60000,
  });

  const temp = Math.round(weather?.temperature_c ?? 22);
  const city = weather?.city ?? "Lisboa";
  const humidity = weather?.humidity_percent ?? weather?.humidity;
  const wind = weather?.wind_kmh ?? weather?.wind_speed_kmh;
  const topTrack = tracks?.body?.items?.[0];
  const title = topTrack?.name || "Sem faixa";
  const subtitle = topTrack?.artists?.map((artist) => artist.name).join(", ") || "SongShare API key";

  return (
    <div className="dc-dashboard">
      <div className="dc-hero">
        <div className="dc-hero-wave" aria-hidden="true">
          <span />
          <span />
          <span />
        </div>

        <div className="dc-hero-heading">
          <h1>Bem-vindo de volta.</h1>
          <p>
            8 aplicacoes <span>•</span>{" "}
            <strong className={online ? "text-success" : "text-danger"}>
              {online ? "Online" : "Offline"}
            </strong>
          </p>
        </div>

        <div className="dc-quick-grid">
          <button className="dc-quick-card dc-weather-card" type="button" onClick={() => onNavigate("weather")}>
            <div className="dc-weather-main">
              <div className="dc-weather-icon">
                <Cloud size={30} />
              </div>
              <div>
                <span>Clima atual</span>
                {weatherLoading && !weather ? (
                  <div className="dc-skeleton" />
                ) : (
                  <>
                    <strong>{temp}&deg;</strong>
                    <small>{city}</small>
                  </>
                )}
              </div>
            </div>
            <div className="dc-weather-divider" />
            <div className="dc-weather-metrics">
              <InfoMetric icon={Droplets} label="Humidade" value={humidity != null ? `${humidity}%` : "--"} />
              <InfoMetric icon={Wind} label="Vento" value={wind != null ? `${wind} km/h` : "--"} />
            </div>
          </button>

          <button className="dc-quick-card dc-music-card" type="button" onClick={() => onNavigate("songshare")}>
            <div className="dc-music-icon">
              <Music size={31} />
            </div>
            <div className="dc-music-copy">
              <span>Musica</span>
              {tracksLoading && !tracks ? (
                <div className="dc-skeleton is-wide" />
              ) : (
                <>
                  <strong>{title}</strong>
                  <small>{subtitle}</small>
                </>
              )}
            </div>
            <div className="dc-mini-controls">
              <span aria-hidden="true">
                <SkipBack size={17} />
              </span>
              <span className="is-play" aria-hidden="true">
                <Play size={18} fill="currentColor" />
              </span>
              <span aria-hidden="true">
                <SkipForward size={17} />
              </span>
            </div>
          </button>
        </div>
      </div>

      <div className="dc-section-head">
        <h2>Aplicacoes</h2>
        <button type="button">
          <SlidersHorizontal size={14} />
          Personalizar
        </button>
      </div>

      <div className="dc-app-grid">
        {apps.map((app) => (
          <AppCard key={app.screen} app={app} onNavigate={onNavigate} />
        ))}
      </div>

      <footer className="dc-footer">
        <span>DC Assistant</span> • Simplifique o seu dia.
      </footer>
    </div>
  );
}
