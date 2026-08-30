import { Component, useCallback, useEffect, useRef, useState } from "react";
import {
  BatteryFull,
  Bell,
  Bluetooth,
  ChevronRight,
  Clock3,
  Cloud,
  FileText,
  Grid2X2,
  MessageCircle,
  Music,
  Radio,
  Settings,
  Wifi,
  Zap,
} from "lucide-react";
import AppLauncher from "./pages/AppLauncher";
import Assistant from "./pages/Assistant";
import MusicPlayer from "./pages/MusicPlayer";
import Weather from "./pages/Weather";
import Features from "./pages/Features";
import Notes from "./pages/Notes";
import Alarm from "./pages/Alarm";
import SettingsPage from "./pages/Settings";
import SongShare from "./pages/SongShare";
import ControlCenter from "./components/ControlCenter";
import SplashScreen from "./pages/SplashScreen";
import { api } from "./lib/api";
import { usePolledApi } from "./lib/useApi";

const Screen = {
  Splash: "splash",
  Launcher: "launcher",
  Assistant: "assistant",
  Music: "music",
  Weather: "weather",
  Features: "features",
  Notes: "notes",
  Alarm: "alarm",
  Settings: "settings",
  SongShare: "songshare",
};

const NAV_ITEMS = [
  { screen: Screen.Launcher, label: "Inicio", icon: Grid2X2 },
  { screen: Screen.Assistant, label: "Assistente", icon: MessageCircle },
  { screen: Screen.Music, label: "Spotify", icon: Radio },
  { screen: Screen.SongShare, label: "SongShare", icon: Music },
  { screen: Screen.Weather, label: "Clima", icon: Cloud },
  { screen: Screen.Features, label: "Recursos", icon: Zap },
  { screen: Screen.Notes, label: "Notas", icon: FileText },
  { screen: Screen.Alarm, label: "Alarme", icon: Bell },
  { screen: Screen.Settings, label: "Definicoes", icon: Settings },
];

class ErrorBoundary extends Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError() {
    return { hasError: true };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex h-full items-center justify-center p-4">
          <div className="text-center">
            <div className="mx-auto mb-3 flex h-12 w-12 items-center justify-center rounded-full bg-danger-tint text-xl">
              !
            </div>
            <p className="font-display text-sm font-semibold text-text-primary">Erro</p>
            <button
              onClick={() => window.location.reload()}
              className="mt-3 rounded bg-accent px-3 py-1.5 text-xs font-medium text-bg-0"
            >
              Recarregar
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

function BrandMark() {
  return (
    <div className="dc-brand">
      <div className="dc-brand-logo">DC</div>
      <div className="dc-brand-subtitle">ASSISTANT</div>
    </div>
  );
}

function Sidebar({ screen, onNavigate, online }) {
  return (
    <aside className="dc-sidebar">
      <BrandMark />

      <nav className="dc-nav">
        {NAV_ITEMS.map((item) => {
          const active = screen === item.screen;
          return (
            <button
              key={item.screen}
              className={`dc-nav-item ${active ? "is-active" : ""}`}
              onClick={() => onNavigate(item.screen)}
              type="button"
            >
              <item.icon size={15} />
              <span>{item.label}</span>
            </button>
          );
        })}
      </nav>

      <button className="dc-profile" type="button">
        <div className="dc-profile-avatar">DC</div>
        <div className="min-w-0 flex-1 text-left">
          <div className="truncate text-[10px] font-semibold text-text-primary">
            Deogracia Castro
          </div>
          <div className="mt-1 flex items-center gap-1.5 text-[10px] text-text-muted">
            <span className={`dc-dot ${online ? "is-online" : "is-offline"}`} />
            {online ? "Online" : "Offline"}
          </div>
        </div>
        <ChevronRight size={13} className="text-text-dim" />
      </button>
    </aside>
  );
}

function TopBar({ online, onStatusClick }) {
  const [now, setNow] = useState(new Date());

  useEffect(() => {
    const id = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(id);
  }, []);

  const time = now.toLocaleTimeString("pt-PT", { hour: "2-digit", minute: "2-digit" });
  const date = now.toLocaleDateString("pt-PT", {
    weekday: "long",
    day: "numeric",
    month: "long",
  });

  return (
    <header className="dc-topbar">
      <div className="dc-time-block">
        <div className="dc-time-icon">
          <Clock3 size={18} />
        </div>
        <div>
          <div className="dc-time">{time}</div>
          <div className="dc-date">{date}</div>
        </div>
      </div>

      <div className="dc-status-chips">
        <button className="dc-chip" type="button" onClick={onStatusClick}>
          <Wifi size={13} />
          <span className={`dc-dot ${online ? "is-online" : "is-offline"}`} />
          <span>{online ? "Online" : "Offline"}</span>
        </button>
        <button className="dc-icon-chip" type="button" onClick={onStatusClick}>
          <Bluetooth size={14} />
        </button>
        <button className="dc-chip" type="button" onClick={onStatusClick}>
          <BatteryFull size={14} />
          <span>100%</span>
        </button>
      </div>
    </header>
  );
}

function AppContent() {
  const [screen, setScreen] = useState(Screen.Splash);
  const [controlCenterOpen, setControlCenterOpen] = useState(false);
  const [controlCenterDragging, setControlCenterDragging] = useState(false);
  const [dragY, setDragY] = useState(0);
  const [fade, setFade] = useState(true);
  const dragStartY = useRef(0);
  const { offline } = usePolledApi(() => api.health(), { intervalMs: 15000 });
  const online = !offline;

  useEffect(() => {
    if (screen === Screen.Splash) {
      const timer = setTimeout(() => {
        setFade(false);
        setTimeout(() => {
          setScreen(Screen.Launcher);
          setFade(true);
        }, 160);
      }, 1800);
      return () => clearTimeout(timer);
    }
  }, [screen]);

  const navigate = useCallback(
    (next) => {
      if (!next || next === screen) return;
      setFade(false);
      setTimeout(() => {
        setScreen(next);
        setFade(true);
      }, 140);
    },
    [screen],
  );

  const handleDragStart = (y) => {
    setControlCenterDragging(true);
    dragStartY.current = y;
  };

  const handleDragMove = (y) => {
    if (!controlCenterDragging) return;
    const delta = Math.max(0, y - dragStartY.current);
    setDragY(delta);
    if (delta > 60 && !controlCenterOpen) {
      setControlCenterOpen(true);
    }
  };

  const handleDragEnd = () => {
    setControlCenterDragging(false);
    setControlCenterOpen(dragY > 80 ? true : controlCenterOpen);
    setDragY(0);
  };

  const renderScreen = () => {
    switch (screen) {
      case Screen.Splash:
        return <SplashScreen />;
      case Screen.Launcher:
        return <AppLauncher onNavigate={navigate} online={online} />;
      case Screen.Assistant:
        return <Assistant onBack={() => navigate(Screen.Launcher)} />;
      case Screen.Music:
        return <MusicPlayer onBack={() => navigate(Screen.Launcher)} />;
      case Screen.SongShare:
        return <SongShare onBack={() => navigate(Screen.Launcher)} />;
      case Screen.Weather:
        return <Weather onBack={() => navigate(Screen.Launcher)} />;
      case Screen.Features:
        return <Features onBack={() => navigate(Screen.Launcher)} />;
      case Screen.Notes:
        return <Notes onBack={() => navigate(Screen.Launcher)} />;
      case Screen.Alarm:
        return <Alarm onBack={() => navigate(Screen.Launcher)} />;
      case Screen.Settings:
        return <SettingsPage onBack={() => navigate(Screen.Launcher)} />;
      default:
        return <AppLauncher onNavigate={navigate} online={online} />;
    }
  };

  if (screen === Screen.Splash) {
    return (
      <div className="h-screen w-full bg-bg-0">
        <div style={{ opacity: fade ? 1 : 0.35, transition: "opacity 160ms ease-out" }}>
          <ErrorBoundary>{renderScreen()}</ErrorBoundary>
        </div>
      </div>
    );
  }

  return (
    <div
      className="dc-web-frame"
      onMouseMove={(e) => handleDragMove(e.clientY)}
      onMouseUp={handleDragEnd}
      onTouchMove={(e) => handleDragMove(e.touches[0].clientY)}
      onTouchEnd={handleDragEnd}
    >
      <div className="dc-web-shell">
        <Sidebar screen={screen} onNavigate={navigate} online={online} />

        <main className="dc-main-panel">
          <TopBar
            online={online}
            onStatusClick={() => {
              handleDragStart(0);
              setControlCenterOpen(true);
            }}
          />

          <section
            className={`dc-content ${screen === Screen.Launcher ? "is-dashboard" : "is-app"}`}
            style={{ opacity: fade ? 1 : 0.25, transition: "opacity 160ms ease-out" }}
          >
            <ErrorBoundary>{renderScreen()}</ErrorBoundary>
          </section>
        </main>
      </div>

      <ControlCenter
        open={controlCenterOpen}
        dragging={controlCenterDragging}
        dragY={dragY}
        onClose={() => setControlCenterOpen(false)}
        onDragStart={handleDragStart}
      />
    </div>
  );
}

export default function App() {
  return <AppContent />;
}
