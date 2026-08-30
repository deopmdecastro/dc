import { Routes, Route } from "react-router-dom";
import Sidebar from "./components/Sidebar";
import StatusBar from "./components/StatusBar";
import Home from "./pages/Home";
import Assistant from "./pages/Assistant";
import Spotify from "./pages/Spotify";
import Weather from "./pages/Weather";
import Resources from "./pages/Resources";
import Notes from "./pages/Notes";
import Alarm from "./pages/Alarm";
import Settings from "./pages/Settings";
import { api } from "./lib/api";
import { usePolledApi } from "./lib/useApi";

export default function App() {
  const { offline } = usePolledApi(() => api.health(), { intervalMs: 15000 });

  return (
    <div className="flex h-screen flex-col bg-bg-0 text-text-primary">
      <StatusBar online={!offline} />
      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <main className="min-h-0 flex-1 overflow-y-auto p-6">
          <div className="mx-auto h-full max-w-5xl">
            <Routes>
              <Route path="/" element={<Home />} />
              <Route path="/assistente" element={<Assistant />} />
              <Route path="/spotify" element={<Spotify />} />
              <Route path="/clima" element={<Weather />} />
              <Route path="/recursos" element={<Resources />} />
              <Route path="/notas" element={<Notes />} />
              <Route path="/alarme" element={<Alarm />} />
              <Route path="/definicoes" element={<Settings />} />
            </Routes>
          </div>
        </main>
      </div>
    </div>
  );
}
