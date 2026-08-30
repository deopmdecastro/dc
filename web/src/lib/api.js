const BASE_URL = import.meta.env.VITE_API_BASE_URL || "http://localhost:8081";

function query(params) {
  const search = new URLSearchParams();
  Object.entries(params).forEach(([key, value]) => {
    if (value !== undefined && value !== null && value !== false) {
      search.set(key, value === true ? "true" : String(value));
    }
  });
  const text = search.toString();
  return text ? `?${text}` : "";
}

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
  weather: (region = 0) => request(`/weather${query({ region })}`),
  weatherByCoords: (lat, lon) => request(`/weather${query({ lat, lon })}`),

  // Music
  musicState: () => request("/music/state"),
  musicDevices: () => request("/music/devices"),
  musicTopTracks: (compact = true, limit = 20) => request(`/music/top-tracks${query({ compact, limit })}`),
  musicPlaylists: (compact = true) => request(`/music/playlists${query({ compact })}`),
  musicSavedTracks: (compact = true, limit = 50) => request(`/music/saved-tracks${query({ compact, limit })}`),
  musicRecentlyPlayed: (compact = true, limit = 20) => request(`/music/recently-played${query({ compact, limit })}`),
  musicCommand: (action) => request("/music/command", { method: "POST", body: JSON.stringify({ action }) }),

  // SongShare
  songshareTracks: (compact = true) => request(`/songshare/tracks${query({ compact })}`),

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
