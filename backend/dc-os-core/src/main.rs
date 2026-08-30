//! DC OS Core — hub central.
//!
//! Rotas:
//!   GET  /health              healthcheck
//!   GET  /ws                  WebSocket bidirecional com o firmware
//!   POST /voice/transcribe    encaminha PCM ao Whisper
//!   GET  /music/state         estado atual do player (proxy Mopidy)
//!   GET  /music/devices       dispositivos Spotify disponiveis
//!   GET  /music/top-tracks    top tracks reais via Spotify Web API
//!   POST /music/command       play/pause/next/prev (proxy Mopidy JSON-RPC)
//!   GET  /songshare/tracks    catalogo Songstats/RapidAPI compacto
//!   GET  /weather             clima atual por regiao via Open-Meteo

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

/// Tempo que a resposta de `/music/top-tracks` fica em cache antes de voltar
/// a chamar a Spotify Web API. As "top tracks" (long_term) mudam muito
/// raramente, mas o firmware sonda este endpoint a cada 60s; sem cache isso
/// gasta a quota da API e aumenta o risco de HTTP 429 (rate limit).
const TOP_TRACKS_CACHE_TTL: Duration = Duration::from_secs(300);
const SONGSHARE_CACHE_TTL: Duration = Duration::from_secs(300);

const SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";
const SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
const SPOTIFY_SCOPES: &str =
    "user-top-read user-read-playback-state user-modify-playback-state user-read-currently-playing";

struct AppState {
    stt_url: String,
    mopidy_url: String,
    spotify: SpotifyAuth,
    http: reqwest::Client,
    notes: RwLock<Vec<Note>>,
    top_tracks_cache: RwLock<Option<CachedTopTracks>>,
    songshare: SongShareConfig,
    songshare_cache: RwLock<Option<CachedTopTracks>>,
}

struct CachedTopTracks {
    fetched_at: Instant,
    value: serde_json::Value,
}

struct SpotifyAuth {
    access_token: String,
    refresh_token: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    device_id: Option<String>,
    runtime_token: RwLock<String>,
    /// Refresh token obtido via `/spotify/login` -> `/spotify/callback`.
    /// Tem prioridade sobre `refresh_token` (vindo do .env) porque pode ser
    /// mais recente (a Spotify por vezes roda o refresh token a cada uso).
    runtime_refresh_token: RwLock<String>,
    /// Caminho opcional (SPOTIFY_TOKEN_STORE) onde o refresh token e
    /// persistido em disco, para sobreviver a um restart do container.
    token_store_path: Option<String>,
    /// Valor de `state` do ultimo pedido `/spotify/login`, para validar o
    /// callback e mitigar CSRF.
    oauth_state: RwLock<Option<String>>,
}

struct SongShareConfig {
    rapidapi_key: String,
    rapidapi_host: String,
    songstats_label_id: String,
    beatport_label_id: String,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

#[derive(Clone, Serialize)]
struct Note {
    id: u64,
    text: String,
    created_at: u64,
}

#[derive(Deserialize)]
struct NoteInput {
    text: String,
}

#[derive(Deserialize)]
struct MusicCommand {
    action: String,
}

#[derive(Deserialize)]
struct TimeQuery {
    offset_secs: Option<i64>,
}

#[derive(Deserialize)]
struct MusicTopTracksQuery {
    compact: Option<String>,
}

#[derive(Deserialize)]
struct SongShareTracksQuery {
    compact: Option<String>,
}

#[derive(Deserialize)]
struct WeatherQuery {
    region: Option<u8>,
}

#[derive(Deserialize)]
struct VoiceCommandInput {
    text: String,
    language: Option<u8>,
}

#[derive(Deserialize)]
struct VoiceTranscribeQuery {
    language: Option<u8>,
}

#[derive(Deserialize)]
struct SpotifyTokenRefreshResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
}

#[derive(Deserialize)]
struct SpotifyCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let token_store_path = env::var("SPOTIFY_TOKEN_STORE")
        .ok()
        .filter(|v| !v.trim().is_empty());
    let persisted_refresh_token = token_store_path
        .as_ref()
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    if let Some(path) = &token_store_path {
        tracing::info!("Spotify: token store configurado em {path}");
    }

    let state = Arc::new(AppState {
        stt_url: env::var("STT_URL").unwrap_or_else(|_| "http://stt-whisper:9000/asr".into()),
        mopidy_url: env::var("MOPIDY_URL").unwrap_or_else(|_| "http://mopidy:6680".into()),
        spotify: SpotifyAuth {
            access_token: env::var("SPOTIFY_TOKEN").unwrap_or_default(),
            // Um refresh token ja persistido em disco (obtido via
            // /spotify/login) e mais fresco do que o valor fixo do .env.
            refresh_token: persisted_refresh_token
                .unwrap_or_else(|| env::var("SPOTIFY_REFRESH_TOKEN").unwrap_or_default()),
            client_id: env::var("SPOTIFY_CLIENT_ID").unwrap_or_default(),
            client_secret: env::var("SPOTIFY_CLIENT_SECRET").unwrap_or_default(),
            redirect_uri: env::var("SPOTIFY_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:8081/spotify/callback".to_owned()),
            device_id: env::var("SPOTIFY_DEVICE_ID")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            runtime_token: RwLock::new(String::new()),
            runtime_refresh_token: RwLock::new(String::new()),
            token_store_path,
            oauth_state: RwLock::new(None),
        },
        http: reqwest::Client::new(),
        notes: RwLock::new(Vec::new()),
        top_tracks_cache: RwLock::new(None),
        songshare: SongShareConfig {
            rapidapi_key: env::var("SONGSTATS_RAPIDAPI_KEY").unwrap_or_default(),
            rapidapi_host: env::var("SONGSTATS_RAPIDAPI_HOST")
                .unwrap_or_else(|_| "songstats.p.rapidapi.com".to_owned()),
            songstats_label_id: env::var("SONGSTATS_LABEL_ID")
                .unwrap_or_else(|_| "7gk4yfc9".to_owned()),
            beatport_label_id: env::var("BEATPORT_LABEL_ID").unwrap_or_else(|_| "74932".to_owned()),
        },
        songshare_cache: RwLock::new(None),
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/time", get(time_now))
        .route("/ws", get(ws_upgrade))
        .route("/voice/transcribe", post(transcribe))
        .route("/music/state", get(music_state))
        .route("/music/devices", get(music_devices))
        .route("/music/top-tracks", get(music_top_tracks))
        .route("/music/command", post(music_command))
        .route("/songshare/tracks", get(songshare_tracks))
        .route("/spotify/login", get(spotify_login))
        .route("/spotify/callback", get(spotify_callback))
        .route("/spotify/status", get(spotify_status))
        .route("/weather", get(weather))
        .route("/notes", get(list_notes).post(create_note))
        .route("/notes/:id", axum::routing::delete(delete_note))
        .route("/voice/command", post(voice_command))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let port: u16 = env::var("DC_CORE_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("dc-os-core escutando em {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        service: "dc-os-core",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn time_now(Query(query): Query<TimeQuery>) -> Json<serde_json::Value> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_secs() as i64)
        .unwrap_or_default();
    let offset = query.offset_secs.unwrap_or(0).clamp(-12 * 3600, 14 * 3600);
    let local = now.saturating_add(offset);
    let day = local.rem_euclid(86_400);
    let hour = day / 3600;
    let minute = (day % 3600) / 60;

    Json(serde_json::json!({
        "ok": true,
        "unix": now,
        "offset_secs": offset,
        "hhmm": format!("{hour:02}:{minute:02}")
    }))
}

async fn list_notes(State(s): State<Arc<AppState>>) -> Json<Vec<Note>> {
    Json(s.notes.read().await.clone())
}

async fn create_note(
    State(s): State<Arc<AppState>>,
    Json(input): Json<NoteInput>,
) -> (StatusCode, Json<Note>) {
    let mut notes = s.notes.write().await;
    let note = Note {
        id: notes.last().map(|n| n.id + 1).unwrap_or(1),
        text: input.text.trim().to_owned(),
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|v| v.as_secs())
            .unwrap_or_default(),
    };
    notes.push(note.clone());
    (StatusCode::CREATED, Json(note))
}

async fn delete_note(
    State(s): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
) -> StatusCode {
    let mut notes = s.notes.write().await;
    let old_len = notes.len();
    notes.retain(|note| note.id != id);
    if notes.len() == old_len {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::NO_CONTENT
    }
}

async fn voice_command(Json(input): Json<VoiceCommandInput>) -> Json<serde_json::Value> {
    let normalized = normalize_voice_text(&input.text);
    let app = classify_voice_command(&normalized);
    Json(serde_json::json!({
        "ok": app.is_some(),
        "app_index": app.map(|app| app.index),
        "app_name": app.map(|app| app.name),
        "language": input.language,
        "text": input.text.clone(),
        "normalized": normalized
    }))
}

#[derive(Clone, Copy)]
struct VoiceApp {
    index: u8,
    name: &'static str,
}

fn classify_voice_command(text: &str) -> Option<VoiceApp> {
    if contains_any(
        text,
        &["songshare", "song share", "songstats", "song stats"],
    ) {
        return Some(VoiceApp {
            index: 7,
            name: "SongShare",
        });
    }
    if contains_any(
        text,
        &["spotify", "musica", "music", "musique", "cancion", "cancao"],
    ) {
        return Some(VoiceApp {
            index: 1,
            name: "Spotify",
        });
    }
    if contains_any(
        text,
        &[
            "clima", "tempo", "previsao", "weather", "forecast", "meteo", "tiempo",
        ],
    ) {
        return Some(VoiceApp {
            index: 2,
            name: "Clima",
        });
    }
    if contains_any(text, &["nota", "notas", "note", "notes"]) {
        return Some(VoiceApp {
            index: 4,
            name: "Notas",
        });
    }
    if contains_any(
        text,
        &["alarme", "alarma", "alarm", "reveil", "despertador"],
    ) {
        return Some(VoiceApp {
            index: 5,
            name: "Alarme",
        });
    }
    if contains_any(
        text,
        &[
            "config",
            "definicoes",
            "definicao",
            "settings",
            "setup",
            "reglages",
            "parametres",
            "ajustes",
            "configuracion",
        ],
    ) {
        return Some(VoiceApp {
            index: 6,
            name: "Definicoes",
        });
    }
    if contains_any(
        text,
        &[
            "app",
            "apps",
            "aplicacao",
            "aplicacoes",
            "application",
            "applications",
            "aplicacion",
            "aplicaciones",
            "menu",
        ],
    ) {
        return Some(VoiceApp {
            index: 0,
            name: "Launcher",
        });
    }
    None
}

fn contains_any(text: &str, terms: &[&str]) -> bool {
    terms.iter().any(|term| text.contains(term))
}

fn normalize_voice_text(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            '\u{00e1}' | '\u{00e0}' | '\u{00e3}' | '\u{00e2}' | '\u{00e4}' => 'a',
            '\u{00e9}' | '\u{00e8}' | '\u{00ea}' | '\u{00eb}' => 'e',
            '\u{00ed}' | '\u{00ec}' | '\u{00ee}' | '\u{00ef}' => 'i',
            '\u{00f3}' | '\u{00f2}' | '\u{00f5}' | '\u{00f4}' | '\u{00f6}' => 'o',
            '\u{00fa}' | '\u{00f9}' | '\u{00fb}' | '\u{00fc}' => 'u',
            '\u{00e7}' => 'c',
            '\u{00f1}' => 'n',
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{classify_voice_command, normalize_voice_text};

    #[test]
    fn classifies_commands_in_configured_languages() {
        let cases = [
            ("abrir clima", Some(2)),
            ("abrir defini\u{00e7}\u{00f5}es", Some(6)),
            ("open spotify", Some(1)),
            ("ouvrir les notes", Some(4)),
            ("abrir alarma", Some(5)),
        ];

        for (text, expected) in cases {
            let app = classify_voice_command(&normalize_voice_text(text));
            assert_eq!(app.map(|app| app.index), expected, "text={text}");
        }
    }
}

async fn weather(
    Query(query): Query<WeatherQuery>,
    State(s): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let region = weather_region(query.region.unwrap_or(0));
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code&timezone=auto",
        region.latitude, region.longitude
    );

    match s.http.get(url).send().await {
        Ok(response) => {
            let status = response.status();
            let body = response
                .json::<serde_json::Value>()
                .await
                .unwrap_or_default();
            let temp = body
                .get("current")
                .and_then(|current| current.get("temperature_2m"))
                .and_then(|value| value.as_f64())
                .unwrap_or_default()
                .round() as i64;
            let code = body
                .get("current")
                .and_then(|current| current.get("weather_code"))
                .and_then(|value| value.as_i64())
                .unwrap_or_default();

            Json(serde_json::json!({
                "ok": status.is_success(),
                "provider": "open-meteo",
                "city": region.city,
                "temperature_c": temp,
                "summary": weather_summary(code),
                "status": status.as_u16()
            }))
        }
        Err(e) => Json(serde_json::json!({
            "ok": false,
            "provider": "open-meteo",
            "city": region.city,
            "temperature_c": 0,
            "summary": "Indisponivel",
            "error": e.to_string()
        })),
    }
}

async fn ws_upgrade(ws: WebSocketUpgrade, State(_s): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// Loop de mensagens vindas do firmware.
/// Formato JSON: `{ "type": "audio"|"state"|"command", ... }`
/// Frames binários = PCM 16-bit mono @16 kHz para STT em tempo real.
async fn handle_socket(mut socket: WebSocket) {
    tracing::info!("WS: firmware conectado");
    while let Some(Ok(msg)) = futures::StreamExt::next(&mut socket).await {
        match msg {
            Message::Text(t) => tracing::debug!("WS text: {t}"),
            Message::Binary(_b) => { /* TODO: bufferizar e enviar ao Whisper */ }
            Message::Ping(p) => {
                let _ = socket.send(Message::Pong(p)).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    tracing::info!("WS: firmware desconectado");
}

async fn transcribe(
    Query(query): Query<VoiceTranscribeQuery>,
    State(s): State<Arc<AppState>>,
    body: bytes::Bytes,
) -> Json<serde_json::Value> {
    let language = stt_language(query.language.unwrap_or(0));
    let form = reqwest::multipart::Form::new().part(
        "audio_file",
        reqwest::multipart::Part::bytes(body.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .unwrap(),
    );
    Json(match s
        .http
        .post(&s.stt_url)
        .query(&[
            ("task", "transcribe"),
            ("language", language),
            ("output", "json"),
        ])
        .multipart(form)
        .send()
        .await
    {
        Ok(r) => {
            let status = r.status();
            let raw = r.text().await.unwrap_or_default();
            let text = extract_transcript_text(&raw);
            serde_json::json!({
                "ok": status.is_success() && !text.trim().is_empty(),
                "provider": "whisper",
                "language": language,
                "text": text,
                "status": status.as_u16(),
                "raw": raw
            })
        }
        Err(e) => serde_json::json!({
            "ok": false,
            "provider": "whisper",
            "language": language,
            "text": "",
            "error": e.to_string()
        }),
    })
}

fn stt_language(language_index: u8) -> &'static str {
    match language_index.min(4) {
        0 | 1 => "pt",
        2 => "en",
        3 => "fr",
        4 => "es",
        _ => "pt",
    }
}

fn extract_transcript_text(raw: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) {
        if let Some(text) = find_transcript_text(&value) {
            return text;
        }
    }
    raw.trim().to_owned()
}

fn find_transcript_text(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(text) => Some(text.trim().to_owned()).filter(|v| !v.is_empty()),
        serde_json::Value::Array(items) => items.iter().find_map(find_transcript_text),
        serde_json::Value::Object(map) => {
            for key in ["text", "transcription", "transcript"] {
                if let Some(text) = map
                    .get(key)
                    .and_then(|value| value.as_str())
                    .map(str::trim)
                    .filter(|text| !text.is_empty())
                {
                    return Some(text.to_owned());
                }
            }
            map.values().find_map(find_transcript_text)
        }
        _ => None,
    }
}

async fn music_state(State(s): State<Arc<AppState>>) -> Json<serde_json::Value> {
    if spotify_configured(&s).await {
        return Json(
            match spotify_request(&s, reqwest::Method::GET, "/v1/me/player", None).await {
                Ok((StatusCode::NO_CONTENT, _)) => serde_json::json!({
                    "ok": false,
                    "driver": "spotify",
                    "reason": "no_active_playback"
                }),
                Ok((status, body)) => serde_json::json!({
                    "ok": status.is_success(),
                    "driver": "spotify",
                    "status": status.as_u16(),
                    "body": body
                }),
                Err(e) => serde_json::json!({
                    "ok": false,
                    "driver": "spotify",
                    "error": e.to_string()
                }),
            },
        );
    }

    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "core.playback.get_state"
    });
    let val = match s
        .http
        .post(format!("{}/mopidy/rpc", s.mopidy_url))
        .json(&body)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let body = response
                .json::<serde_json::Value>()
                .await
                .unwrap_or_default();
            serde_json::json!({
                "ok": status.is_success() && body.get("error").is_none(),
                "driver": "mopidy",
                "status": status.as_u16(),
                "body": body
            })
        }
        Err(e) => serde_json::json!({
            "ok": false,
            "driver": "mopidy",
            "error": e.to_string()
        }),
    };
    Json(val)
}

async fn music_devices(State(s): State<Arc<AppState>>) -> Json<serde_json::Value> {
    if !spotify_configured(&s).await {
        return Json(serde_json::json!({
            "ok": false,
            "driver": "spotify",
            "error": "Spotify nao configurado"
        }));
    }

    Json(
        match spotify_request(&s, reqwest::Method::GET, "/v1/me/player/devices", None).await {
            Ok((status, body)) => serde_json::json!({
                "ok": status.is_success(),
                "driver": "spotify",
                "status": status.as_u16(),
                "body": body
            }),
            Err(e) => serde_json::json!({
                "ok": false,
                "driver": "spotify",
                "error": e.to_string()
            }),
        },
    )
}

async fn music_top_tracks(
    Query(query): Query<MusicTopTracksQuery>,
    State(s): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    if !spotify_configured(&s).await {
        return Json(serde_json::json!({
            "ok": false,
            "driver": "spotify",
            "error": "Spotify nao configurado. Visita /spotify/login para autorizar a conta."
        }));
    }

    if let Some(cached) = read_top_tracks_cache(&s).await {
        return Json(top_tracks_response(cached, query.compact(), true));
    }

    let endpoint = "/v1/me/top/tracks?time_range=long_term&limit=5";
    Json(
        match spotify_request(&s, reqwest::Method::GET, endpoint, None).await {
            Ok((status, body)) if status.is_success() => {
                write_top_tracks_cache(&s, body.clone()).await;
                top_tracks_response(body, query.compact(), false)
            }
            Ok((StatusCode::TOO_MANY_REQUESTS, body)) => serde_json::json!({
                "ok": false,
                "driver": "spotify",
                "status": 429,
                "error": "rate_limited",
                "body": body
            }),
            Ok((status, body)) => serde_json::json!({
                "ok": false,
                "driver": "spotify",
                "status": status.as_u16(),
                "body": body
            }),
            Err(e) => serde_json::json!({
                "ok": false,
                "driver": "spotify",
                "error": e.to_string()
            }),
        },
    )
}

fn top_tracks_response(body: serde_json::Value, compact: bool, cached: bool) -> serde_json::Value {
    if compact {
        serde_json::json!({
            "ok": true,
            "driver": "spotify",
            "cached": cached,
            "body": {
                "items": compact_spotify_tracks(&body)
            }
        })
    } else {
        serde_json::json!({
            "ok": true,
            "driver": "spotify",
            "cached": cached,
            "body": body
        })
    }
}

async fn read_top_tracks_cache(s: &AppState) -> Option<serde_json::Value> {
    let cache = s.top_tracks_cache.read().await;
    let entry = cache.as_ref()?;
    (entry.fetched_at.elapsed() < TOP_TRACKS_CACHE_TTL).then(|| entry.value.clone())
}

async fn write_top_tracks_cache(s: &AppState, value: serde_json::Value) {
    let mut cache = s.top_tracks_cache.write().await;
    *cache = Some(CachedTopTracks {
        fetched_at: Instant::now(),
        value,
    });
}

fn compact_spotify_tracks(body: &serde_json::Value) -> Vec<serde_json::Value> {
    body.get("items")
        .and_then(|items| items.as_array())
        .map(|items| {
            items
                .iter()
                .take(5)
                .map(|item| {
                    let artists = item
                        .get("artists")
                        .and_then(|artists| artists.as_array())
                        .map(|artists| {
                            artists
                                .iter()
                                .filter_map(|artist| {
                                    artist.get("name").and_then(|name| name.as_str())
                                })
                                .map(|name| serde_json::json!({ "name": name }))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();

                    serde_json::json!({
                        "name": item.get("name").and_then(|name| name.as_str()).unwrap_or("Sem titulo"),
                        "artists": artists,
                        "album": {
                            "name": item
                                .get("album")
                                .and_then(|album| album.get("name"))
                                .and_then(|name| name.as_str())
                                .unwrap_or("")
                        }
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

impl MusicTopTracksQuery {
    fn compact(&self) -> bool {
        self.compact
            .as_deref()
            .map(|value| matches!(value, "1" | "true" | "yes" | "sim"))
            .unwrap_or(false)
    }
}

impl SongShareTracksQuery {
    fn compact(&self) -> bool {
        self.compact
            .as_deref()
            .map(|value| matches!(value, "1" | "true" | "yes" | "sim"))
            .unwrap_or(false)
    }
}

async fn songshare_tracks(
    Query(query): Query<SongShareTracksQuery>,
    State(s): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    if s.songshare.rapidapi_key.trim().is_empty() {
        return Json(serde_json::json!({
            "ok": false,
            "driver": "songshare",
            "error": "SONGSTATS_RAPIDAPI_KEY nao configurado no backend/.env"
        }));
    }

    if let Some(cached) = read_songshare_cache(&s).await {
        return Json(songshare_response(cached, query.compact(), true));
    }

    let url = format!(
        "https://{}/labels/songshare?songstats_label_id={}&beatport_label_id={}",
        s.songshare.rapidapi_host, s.songshare.songstats_label_id, s.songshare.beatport_label_id
    );

    Json(
        match s
            .http
            .get(url)
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("x-rapidapi-host", &s.songshare.rapidapi_host)
            .header("x-rapidapi-key", &s.songshare.rapidapi_key)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                let body = response
                    .json::<serde_json::Value>()
                    .await
                    .unwrap_or_default();
                if status.is_success() {
                    write_songshare_cache(&s, body.clone()).await;
                    songshare_response(body, query.compact(), false)
                } else {
                    let error = body
                        .get("message")
                        .and_then(|message| message.as_str())
                        .unwrap_or("Songstats/RapidAPI recusou o pedido");
                    serde_json::json!({
                        "ok": false,
                        "driver": "songshare",
                        "status": status.as_u16(),
                        "error": error,
                        "body": body
                    })
                }
            }
            Err(e) => serde_json::json!({
                "ok": false,
                "driver": "songshare",
                "error": e.to_string()
            }),
        },
    )
}

fn songshare_response(body: serde_json::Value, compact: bool, cached: bool) -> serde_json::Value {
    if compact {
        serde_json::json!({
            "ok": true,
            "driver": "songshare",
            "cached": cached,
            "body": {
                "items": compact_songshare_tracks(&body)
            }
        })
    } else {
        serde_json::json!({
            "ok": true,
            "driver": "songshare",
            "cached": cached,
            "body": body
        })
    }
}

async fn read_songshare_cache(s: &AppState) -> Option<serde_json::Value> {
    let cache = s.songshare_cache.read().await;
    let entry = cache.as_ref()?;
    (entry.fetched_at.elapsed() < SONGSHARE_CACHE_TTL).then(|| entry.value.clone())
}

async fn write_songshare_cache(s: &AppState, value: serde_json::Value) {
    let mut cache = s.songshare_cache.write().await;
    *cache = Some(CachedTopTracks {
        fetched_at: Instant::now(),
        value,
    });
}

fn compact_songshare_tracks(body: &serde_json::Value) -> Vec<serde_json::Value> {
    let mut tracks = Vec::new();
    collect_songshare_tracks(body, &mut tracks);
    dedupe_tracks(tracks).into_iter().take(8).collect()
}

fn collect_songshare_tracks(value: &serde_json::Value, out: &mut Vec<serde_json::Value>) {
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                collect_songshare_tracks(item, out);
            }
        }
        serde_json::Value::Object(map) => {
            let title = string_field(
                map,
                &["title", "name", "track_name", "song_name", "release_title"],
            );
            let artist = string_field(
                map,
                &[
                    "artist",
                    "artist_name",
                    "artists",
                    "primary_artist",
                    "creator_name",
                ],
            );
            let album = string_field(map, &["album", "album_name", "release", "release_name"]);
            let looks_like_track = map.contains_key("isrc")
                || map.contains_key("track")
                || map.contains_key("song")
                || artist.is_some();

            if let Some(title) = title {
                if looks_like_track {
                    out.push(serde_json::json!({
                        "name": title,
                        "artists": [{ "name": artist.unwrap_or_else(|| "Songstats".to_owned()) }],
                        "album": { "name": album.unwrap_or_default() }
                    }));
                }
            }

            for child in map.values() {
                collect_songshare_tracks(child, out);
            }
        }
        _ => {}
    }
}

fn string_field(map: &serde_json::Map<String, serde_json::Value>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(value) = map.get(*key) {
            if let Some(text) = json_string(value) {
                return Some(text);
            }
        }
    }
    None
}

fn json_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(text) => Some(text.trim().to_owned()).filter(|v| !v.is_empty()),
        serde_json::Value::Array(items) => items.first().and_then(json_string),
        serde_json::Value::Object(map) => string_field(map, &["name", "title"]),
        _ => None,
    }
}

fn dedupe_tracks(tracks: Vec<serde_json::Value>) -> Vec<serde_json::Value> {
    let mut unique = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for track in tracks {
        let key = track
            .get("name")
            .and_then(|name| name.as_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if seen.insert(key) {
            unique.push(track);
        }
    }
    unique
}

struct WeatherRegion {
    city: &'static str,
    latitude: f64,
    longitude: f64,
}

fn weather_region(region: u8) -> WeatherRegion {
    match region.min(4) {
        0 => WeatherRegion {
            city: "Brasilia",
            latitude: -15.7939,
            longitude: -47.8828,
        },
        1 => WeatherRegion {
            city: "Lisboa",
            latitude: 38.7223,
            longitude: -9.1393,
        },
        2 => WeatherRegion {
            city: "Luanda",
            latitude: -8.8390,
            longitude: 13.2894,
        },
        3 => WeatherRegion {
            city: "Maputo",
            latitude: -25.9692,
            longitude: 32.5732,
        },
        _ => WeatherRegion {
            city: "New York",
            latitude: 40.7128,
            longitude: -74.0060,
        },
    }
}

fn weather_summary(code: i64) -> &'static str {
    match code {
        0 => "Limpo",
        1..=3 => "Parcial",
        45 | 48 => "Nevoeiro",
        51..=67 => "Chuvisco",
        71..=77 => "Neve",
        80..=82 => "Chuva",
        95..=99 => "Trovoada",
        _ => "Nublado",
    }
}

async fn music_command(
    State(s): State<Arc<AppState>>,
    Json(cmd): Json<MusicCommand>,
) -> Json<serde_json::Value> {
    if spotify_configured(&s).await {
        return Json(match spotify_music_command(&s, &cmd.action).await {
            Ok(value) => value,
            Err(e) => serde_json::json!({
                "ok": false,
                "driver": "spotify",
                "action": cmd.action,
                "error": e.to_string()
            }),
        });
    }

    let method = match cmd.action.as_str() {
        "play" => "core.playback.play",
        "pause" => "core.playback.pause",
        "next" => "core.playback.next",
        "prev" => "core.playback.previous",
        _ => return Json(serde_json::json!({ "error": "unknown action" })),
    };
    let body = serde_json::json!({ "jsonrpc":"2.0", "id":1, "method": method });
    match s
        .http
        .post(format!("{}/mopidy/rpc", s.mopidy_url))
        .json(&body)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let body = response
                .json::<serde_json::Value>()
                .await
                .unwrap_or_default();
            Json(serde_json::json!({
                "ok": status.is_success() && body.get("error").is_none(),
                "driver": "mopidy",
                "action": cmd.action,
                "status": status.as_u16(),
                "body": body
            }))
        }
        Err(e) => Json(serde_json::json!({
            "ok": false,
            "driver": "mopidy",
            "action": cmd.action,
            "error": e.to_string()
        })),
    }
}

async fn spotify_music_command(s: &AppState, action: &str) -> anyhow::Result<serde_json::Value> {
    let (method, path, body) = if action == "play" {
        let uris = spotify_top_track_uris(s).await.unwrap_or_default();
        let body = if uris.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::json!({ "uris": uris })
        };
        (reqwest::Method::PUT, "/v1/me/player/play", Some(body))
    } else {
        match action {
            "pause" => (reqwest::Method::PUT, "/v1/me/player/pause", None),
            "next" => (reqwest::Method::POST, "/v1/me/player/next", None),
            "prev" => (reqwest::Method::POST, "/v1/me/player/previous", None),
            _ => return Ok(serde_json::json!({ "ok": false, "error": "unknown action" })),
        }
    };

    let (status, response_body) = spotify_request(s, method, path, body).await?;
    Ok(serde_json::json!({
        "ok": status.is_success() || status == StatusCode::NO_CONTENT,
        "driver": "spotify",
        "action": action,
        "status": status.as_u16(),
        "body": response_body
    }))
}

async fn spotify_top_track_uris(s: &AppState) -> anyhow::Result<Vec<String>> {
    let endpoint = "/v1/me/top/tracks?time_range=long_term&limit=5";
    let (status, body) = spotify_request(s, reqwest::Method::GET, endpoint, None).await?;
    if !status.is_success() {
        anyhow::bail!("Spotify top tracks HTTP {}", status.as_u16());
    }

    let uris = body
        .get("items")
        .and_then(|items| items.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("uri").and_then(|uri| uri.as_str()))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(uris)
}

async fn spotify_request(
    s: &AppState,
    method: reqwest::Method,
    path: &str,
    body: Option<serde_json::Value>,
) -> anyhow::Result<(StatusCode, serde_json::Value)> {
    let mut url = format!("https://api.spotify.com{path}");
    if let Some(device_id) = &s.spotify.device_id {
        let sep = if url.contains('?') { '&' } else { '?' };
        url.push(sep);
        url.push_str("device_id=");
        url.push_str(device_id);
    }

    let token = spotify_token(s).await?;
    let result = send_spotify_request(s, method.clone(), &url, body.clone(), &token).await?;
    if result.0 != StatusCode::UNAUTHORIZED || !spotify_can_refresh(s).await {
        return Ok(result);
    }

    let token = refresh_spotify_token(s).await?;
    send_spotify_request(s, method, &url, body, &token).await
}

async fn send_spotify_request(
    s: &AppState,
    method: reqwest::Method,
    url: &str,
    body: Option<serde_json::Value>,
    token: &str,
) -> anyhow::Result<(StatusCode, serde_json::Value)> {
    let mut request = s
        .http
        .request(method, url)
        .bearer_auth(token)
        .header("accept", "application/json");

    if let Some(body) = body {
        request = request.json(&body);
    }

    let response = request.send().await?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    let body = if text.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({ "raw": text }))
    };

    Ok((status, body))
}

async fn spotify_configured(s: &AppState) -> bool {
    !spotify_token_value(s).await.is_empty() || spotify_can_refresh(s).await
}

async fn spotify_token(s: &AppState) -> anyhow::Result<String> {
    let token = spotify_token_value(s).await;
    if !token.is_empty() {
        return Ok(token);
    }
    refresh_spotify_token(s).await
}

async fn spotify_token_value(s: &AppState) -> String {
    let runtime_token = s.spotify.runtime_token.read().await;
    if !runtime_token.is_empty() {
        return runtime_token.clone();
    }
    s.spotify.access_token.clone()
}

/// Refresh token "ativo": o obtido dinamicamente via /spotify/login tem
/// prioridade sobre o valor fixo carregado do .env, porque a Spotify pode
/// ter rodado (rotacionado) o refresh token num pedido anterior.
async fn active_refresh_token(s: &AppState) -> String {
    let runtime = s.spotify.runtime_refresh_token.read().await;
    if !runtime.is_empty() {
        return runtime.clone();
    }
    s.spotify.refresh_token.clone()
}

async fn spotify_can_refresh(s: &AppState) -> bool {
    !active_refresh_token(s).await.is_empty()
        && !s.spotify.client_id.is_empty()
        && !s.spotify.client_secret.is_empty()
}

async fn refresh_spotify_token(s: &AppState) -> anyhow::Result<String> {
    let refresh_token = active_refresh_token(s).await;
    if refresh_token.is_empty()
        || s.spotify.client_id.is_empty()
        || s.spotify.client_secret.is_empty()
    {
        anyhow::bail!(
            "Spotify token expirado e refresh token nao configurado. Visita /spotify/login."
        );
    }

    let response = s
        .http
        .post(SPOTIFY_TOKEN_URL)
        .basic_auth(&s.spotify.client_id, Some(&s.spotify.client_secret))
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
        ])
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        anyhow::bail!("Spotify refresh HTTP {}: {}", status.as_u16(), text);
    }

    let body = serde_json::from_str::<SpotifyTokenRefreshResponse>(&text)?;
    {
        let mut runtime_token = s.spotify.runtime_token.write().await;
        *runtime_token = body.access_token.clone();
    }

    // A Spotify por vezes devolve um novo refresh_token (rotacao); se vier,
    // substitui o antigo em memoria e em disco para nao ficar desatualizado.
    if let Some(new_refresh_token) = &body.refresh_token {
        store_refresh_token(s, new_refresh_token).await;
    }

    Ok(body.access_token)
}

/// Guarda o refresh token em memoria e, se SPOTIFY_TOKEN_STORE estiver
/// configurado, tambem em disco para sobreviver a um restart do container.
async fn store_refresh_token(s: &AppState, refresh_token: &str) {
    {
        let mut runtime_refresh_token = s.spotify.runtime_refresh_token.write().await;
        *runtime_refresh_token = refresh_token.to_owned();
    }

    if let Some(path) = &s.spotify.token_store_path {
        match fs::write(path, refresh_token) {
            Ok(()) => tracing::info!("Spotify: refresh token persistido em {path}"),
            Err(e) => tracing::warn!("Spotify: falha ao gravar refresh token em {path}: {e}"),
        }
    }
}

/// Redireciona o navegador para a pagina de autorizacao da Spotify. Visitar
/// isto uma vez (a partir de um browser na mesma rede) e o suficiente para
/// o dc-os-core obter e guardar o refresh token — sem isso, so client_id e
/// client_secret nao autorizam nada, porque a Spotify exige consentimento
/// explicito do utilizador para escopos como `user-top-read`.
async fn spotify_login(State(s): State<Arc<AppState>>) -> impl IntoResponse {
    if s.spotify.client_id.is_empty() || s.spotify.client_secret.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Html(
                "<p>SPOTIFY_CLIENT_ID / SPOTIFY_CLIENT_SECRET nao configurados. \
                 Define-os em backend/.env e reinicia o dc-os-core.</p>"
                    .to_owned(),
            ),
        )
            .into_response();
    }

    let state_value = format!(
        "{:x}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    {
        let mut oauth_state = s.spotify.oauth_state.write().await;
        *oauth_state = Some(state_value.clone());
    }

    let url = format!(
        "{SPOTIFY_AUTH_URL}?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}",
        urlencode(&s.spotify.client_id),
        urlencode(&s.spotify.redirect_uri),
        urlencode(SPOTIFY_SCOPES),
        urlencode(&state_value),
    );

    Redirect::temporary(&url).into_response()
}

/// Recebe o `code` de volta da Spotify, troca-o por access_token +
/// refresh_token e guarda-os (ver `store_refresh_token`).
async fn spotify_callback(
    Query(query): Query<SpotifyCallbackQuery>,
    State(s): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Some(error) = query.error {
        return Html(format!("<p>Autorizacao Spotify falhou: {error}</p>")).into_response();
    }

    let Some(code) = query.code else {
        return (
            StatusCode::BAD_REQUEST,
            Html("<p>Pedido invalido: falta o parametro code.</p>".to_owned()),
        )
            .into_response();
    };

    let expected_state = s.spotify.oauth_state.write().await.take();
    if expected_state.is_some() && expected_state != query.state {
        return (
            StatusCode::BAD_REQUEST,
            Html(
                "<p>State invalido ou expirado; volta a iniciar em /spotify/login.</p>".to_owned(),
            ),
        )
            .into_response();
    }

    let response = match s
        .http
        .post(SPOTIFY_TOKEN_URL)
        .basic_auth(&s.spotify.client_id, Some(&s.spotify.client_secret))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("redirect_uri", s.spotify.redirect_uri.as_str()),
        ])
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => return Html(format!("<p>Falha ao contactar a Spotify: {e}</p>")).into_response(),
    };

    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        tracing::warn!(
            "Spotify: troca do code falhou HTTP {}: {}",
            status.as_u16(),
            text
        );
        return Html(format!(
            "<p>A Spotify recusou a troca do codigo (HTTP {}). Confirma client id/secret \
             e se o Redirect URI <code>{}</code> esta registado na app do Spotify Developer \
             Dashboard.</p>",
            status.as_u16(),
            s.spotify.redirect_uri
        ))
        .into_response();
    }

    let body: SpotifyTokenRefreshResponse = match serde_json::from_str(&text) {
        Ok(body) => body,
        Err(e) => {
            return Html(format!("<p>Resposta inesperada da Spotify: {e}</p>")).into_response()
        }
    };

    {
        let mut runtime_token = s.spotify.runtime_token.write().await;
        *runtime_token = body.access_token.clone();
    }

    match &body.refresh_token {
        Some(refresh_token) => store_refresh_token(&s, refresh_token).await,
        None => tracing::warn!("Spotify: callback nao devolveu refresh_token"),
    }

    Html(
        "<p>Spotify autorizado com sucesso. Ja podes fechar esta janela — \
         o dc-os-core vai manter a sessao e renovar o token sozinho.</p>"
            .to_owned(),
    )
    .into_response()
}

async fn spotify_status(State(s): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ok": true,
        "client_id_configured": !s.spotify.client_id.is_empty(),
        "client_secret_configured": !s.spotify.client_secret.is_empty(),
        "refresh_token_configured": !active_refresh_token(&s).await.is_empty(),
        "device_id": s.spotify.device_id,
        "redirect_uri": s.spotify.redirect_uri,
        "login_url": "/spotify/login"
    }))
}

fn urlencode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
