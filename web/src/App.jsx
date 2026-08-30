import { Component, useState, useEffect, useCallback, useRef } from "react";
import StatusBar from "./components/StatusBar";
import AppLauncher from "./pages/AppLauncher";
import Assistant from "./pages/Assistant";
import MusicPlayer from "./pages/MusicPlayer";
import Weather from "./pages/Weather";
import Features from "./pages/Features";
import Notes from "./pages/Notes";
import Alarm from "./pages/Alarm";
import Settings from "./pages/Settings";
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
            <div className="mx-auto mb-3 flex h-12 w-12 items-center justify-center rounded-full bg-danger-tint text-xl">⚠️</div>
            <p className="font-display text-sm font-semibold text-text-primary">Erro</p>
            <button onClick={() => window.location.reload()} className="mt-3 rounded bg-accent px-3 py-1.5 text-xs font-medium text-bg-0">Recarregar</button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

function AppContent() {
  const [screen, setScreen] = useState(Screen.Splash);
  const [prevScreen, setPrevScreen] = useState(null);
  const [controlCenterOpen, setControlCenterOpen] = useState(false);
  const [controlCenterDragging, setControlCenterDragging] = useState(false);
  const [dragY, setDragY] = useState(0);
  const [fade, setFade] = useState(true);
  const dragStartY = useRef(0);
  const { offline } = usePolledApi(() => api.health(), { intervalMs: 15000 });

  useEffect(() => {
    if (screen === Screen.Splash) {
      const timer = setTimeout(() => {
        setFade(false);
        setTimeout(() => { setScreen(Screen.Launcher); setFade(true); }, 160);
      }, 2500);
      return () => clearTimeout(timer);
    }
  }, [screen]);

  const navigate = useCallback((next) => {
    setFade(false);
    setTimeout(() => { setPrevScreen(screen); setScreen(next); setFade(true); }, 160);
  }, [screen]);

  // Control center drag handlers
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
    if (dragY > 80) {
      setControlCenterOpen(true);
    } else {
      setControlCenterOpen(false);
    }
    setDragY(0);
  };

  const renderScreen = () => {
    switch (screen) {
      case Screen.Splash: return <SplashScreen />;
      case Screen.Launcher: return <AppLauncher onNavigate={navigate} />;
      case Screen.Assistant: return <Assistant onBack={() => navigate(Screen.Launcher)} />;
      case Screen.Music: return <MusicPlayer onBack={() => navigate(Screen.Launcher)} />;
      case Screen.SongShare: return <SongShare onBack={() => navigate(Screen.Launcher)} />;
      case Screen.Weather: return <Weather onBack={() => navigate(Screen.Launcher)} />;
      case Screen.Features: return <Features onBack={() => navigate(Screen.Launcher)} />;
      case Screen.Notes: return <Notes onBack={() => navigate(Screen.Launcher)} />;
      case Screen.Alarm: return <Alarm onBack={() => navigate(Screen.Launcher)} />;
      case Screen.Settings: return <Settings onBack={() => navigate(Screen.Launcher)} />;
      default: return <AppLauncher onNavigate={navigate} />;
    }
  };

  const showStatusBar = screen !== Screen.Splash;

  return (
    <div
      className="flex h-screen w-full flex-col bg-bg-0 gradient-mesh"
      onMouseMove={(e) => handleDragMove(e.clientY)}
      onMouseUp={handleDragEnd}
      onTouchMove={(e) => handleDragMove(e.touches[0].clientY)}
      onTouchEnd={handleDragEnd}
    >
      {showStatusBar && (
        <StatusBar
          online={!offline}
          onDragStart={() => handleDragStart(window.event?.clientY || 0)}
          onTap={() => setControlCenterOpen(true)}
        />
      )}

      <div
        className="flex-1 overflow-hidden"
        style={{ opacity: fade ? 1 : 0.35, transition: "opacity 160ms ease-out" }}
      >
        <ErrorBoundary>{renderScreen()}</ErrorBoundary>
      </div>

      {/* Control Center overlay */}
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
