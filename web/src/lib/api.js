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

  // System
  health: () => request("/health"),
  time: (offsetSecs) => request(`/time${offsetSecs != null ? `?offset_secs=${offsetSecs}` : ""}`),

  // Weather
  weather: (region = 0) => request(`/weather?region=${region}`),
  weatherByCoords: (lat, lon) => request(`/weather?lat=${lat}&lon=${lon}`),

  // Music
  musicState: () => request("/music/state"),
  musicDevices: () => request("/music/devices"),
  musicTopTracks: (compact = true, limit = 20) => request(`/music/top-tracks${compact ? "?compact=true" : ""}${limit ? `&limit=${limit}` : ""}`),
  musicPlaylists: (compact = true) => request(`/music/playlists${compact ? "?compact=true" : ""}`),
  musicSavedTracks: (compact = true, limit = 50) => request(`/music/saved-tracks${compact ? "?compact=true" : ""}${limit ? `&limit=${limit}` : ""}`),
  musicRecentlyPlayed: (compact = true, limit = 20) => request(`/music/recently-played${compact ? "?compact=true" : ""}${limit ? `&limit=${limit}` : ""}`),
  musicCommand: (action) => request("/music/command", { method: "POST", body: JSON.stringify({ action }) }),

  // SongShare
  songshareTracks: (compact = true) => request(`/songshare/tracks${compact ? "?compact=true" : ""}`),

  // Voice
  voiceCommand: (text, language = 0) => request("/voice/command", { method: "POST", body: JSON.stringify({ text, language }) }),

  // Spotify
  spotifyStatus: () => request("/spotify/status"),
  spotifyLoginUrl: () => `${BASE_URL}/spotify/login`,

  // Notes
  notes: () => request("/notes"),
  createNote: (text) => request("/notes", { method: "POST", body: JSON.stringify({ text }) }),
  deleteNote: (id) => request(`/notes/${id}`, { method: "DELETE" }),
};

export default api;
