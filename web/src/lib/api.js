// Cliente para o backend `dc-os-core` (ver backend/README.md).
// Base URL configurável via VITE_API_BASE_URL (default: http://localhost:8081).
const BASE_URL = import.meta.env.VITE_API_BASE_URL || "http://localhost:8081";

async function request(path, options = {}) {
  const res = await fetch(`${BASE_URL}${path}`, {
    headers: { "Content-Type": "application/json" },
    ...options,
  });
  if (!res.ok) {
    throw new Error(`${options.method || "GET"} ${path} -> HTTP ${res.status}`);
  }
  const contentType = res.headers.get("content-type") || "";
  return contentType.includes("application/json") ? res.json() : res.text();
}

export const api = {
  baseUrl: BASE_URL,
  health: () => request("/health"),
  time: (offsetSecs) =>
    request(`/time${offsetSecs != null ? `?offset_secs=${offsetSecs}` : ""}`),
  weather: (region = 0) => request(`/weather?region=${region}`),
  musicState: () => request("/music/state"),
  musicDevices: () => request("/music/devices"),
  musicTopTracks: () => request("/music/top-tracks"),
  musicCommand: (action) =>
    request("/music/command", { method: "POST", body: JSON.stringify({ action }) }),
  spotifyStatus: () => request("/spotify/status"),
  spotifyLoginUrl: () => `${BASE_URL}/spotify/login`,
  notes: () => request("/notes"),
  createNote: (text) =>
    request("/notes", { method: "POST", body: JSON.stringify({ text }) }),
  deleteNote: (id) => request(`/notes/${id}`, { method: "DELETE" }),
};

export default api;
