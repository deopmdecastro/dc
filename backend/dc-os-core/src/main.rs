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

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    env,
    net::SocketAddr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    stt_url:    String,
    mopidy_url: String,
    spotify_token: String,
    spotify_device_id: Option<String>,
    http:       reqwest::Client,
}

#[derive(Serialize)]
struct Health { status: &'static str, service: &'static str, version: &'static str }

#[derive(Deserialize)]
struct MusicCommand { action: String }

#[derive(Deserialize)]
struct TimeQuery { offset_secs: Option<i64> }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let state = Arc::new(AppState {
        stt_url:    env::var("STT_URL").unwrap_or_else(|_| "http://stt-whisper:9000/asr".into()),
        mopidy_url: env::var("MOPIDY_URL").unwrap_or_else(|_| "http://mopidy:6680".into()),
        spotify_token: env::var("SPOTIFY_TOKEN").unwrap_or_default(),
        spotify_device_id: env::var("SPOTIFY_DEVICE_ID").ok().filter(|v| !v.trim().is_empty()),
        http:       reqwest::Client::new(),
    });

    let app = Router::new()
        .route("/health",           get(health))
        .route("/time",             get(time_now))
        .route("/ws",               get(ws_upgrade))
        .route("/voice/transcribe", post(transcribe))
        .route("/music/state",      get(music_state))
        .route("/music/devices",    get(music_devices))
        .route("/music/top-tracks", get(music_top_tracks))
        .route("/music/command",    post(music_command))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let port: u16 = env::var("DC_CORE_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("dc-os-core escutando em {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn health() -> Json<Health> {
    Json(Health { status: "ok", service: "dc-os-core", version: env!("CARGO_PKG_VERSION") })
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
            Message::Ping(p) => { let _ = socket.send(Message::Pong(p)).await; }
            Message::Close(_) => break,
            _ => {}
        }
    }
    tracing::info!("WS: firmware desconectado");
}

async fn transcribe(State(s): State<Arc<AppState>>, body: bytes::Bytes) -> impl IntoResponse {
    let form = reqwest::multipart::Form::new().part(
        "audio_file",
        reqwest::multipart::Part::bytes(body.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav").unwrap(),
    );
    match s.http.post(&s.stt_url).multipart(form).send().await {
        Ok(r) => r.text().await.unwrap_or_default(),
        Err(e) => format!("{{\"error\":\"{e}\"}}"),
    }
}

async fn music_state(State(s): State<Arc<AppState>>) -> Json<serde_json::Value> {
    if !s.spotify_token.is_empty() {
        return Json(match spotify_request(&s, reqwest::Method::GET, "/v1/me/player", None).await {
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
        });
    }

    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "core.playback.get_state"
    });
    let val = match s.http.post(format!("{}/mopidy/rpc", s.mopidy_url)).json(&body).send().await {
        Ok(response) => {
            let status = response.status();
            let body = response.json::<serde_json::Value>().await.unwrap_or_default();
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
    if s.spotify_token.is_empty() {
        return Json(serde_json::json!({
            "ok": false,
            "driver": "spotify",
            "error": "SPOTIFY_TOKEN nao configurado"
        }));
    }

    Json(match spotify_request(&s, reqwest::Method::GET, "/v1/me/player/devices", None).await {
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
    })
}

async fn music_top_tracks(State(s): State<Arc<AppState>>) -> Json<serde_json::Value> {
    if s.spotify_token.is_empty() {
        return Json(serde_json::json!({
            "ok": false,
            "driver": "spotify",
            "error": "SPOTIFY_TOKEN nao configurado"
        }));
    }

    let endpoint = "/v1/me/top/tracks?time_range=long_term&limit=5";
    Json(match spotify_request(&s, reqwest::Method::GET, endpoint, None).await {
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
    })
}

async fn music_command(
    State(s): State<Arc<AppState>>, Json(cmd): Json<MusicCommand>,
) -> Json<serde_json::Value> {
    if !s.spotify_token.is_empty() {
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
        "play"  => "core.playback.play",
        "pause" => "core.playback.pause",
        "next"  => "core.playback.next",
        "prev"  => "core.playback.previous",
        _       => return Json(serde_json::json!({ "error": "unknown action" })),
    };
    let body = serde_json::json!({ "jsonrpc":"2.0", "id":1, "method": method });
    match s.http.post(format!("{}/mopidy/rpc", s.mopidy_url)).json(&body).send().await {
        Ok(response) => {
            let status = response.status();
            let body = response.json::<serde_json::Value>().await.unwrap_or_default();
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
    let (method, path, body) = match action {
        "play" => (reqwest::Method::PUT, "/v1/me/player/play", Some(serde_json::json!({}))),
        "pause" => (reqwest::Method::PUT, "/v1/me/player/pause", None),
        "next" => (reqwest::Method::POST, "/v1/me/player/next", None),
        "prev" => (reqwest::Method::POST, "/v1/me/player/previous", None),
        _ => return Ok(serde_json::json!({ "ok": false, "error": "unknown action" })),
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

async fn spotify_request(
    s: &AppState,
    method: reqwest::Method,
    path: &str,
    body: Option<serde_json::Value>,
) -> anyhow::Result<(StatusCode, serde_json::Value)> {
    let mut url = format!("https://api.spotify.com{path}");
    if let Some(device_id) = &s.spotify_device_id {
        let sep = if url.contains('?') { '&' } else { '?' };
        url.push(sep);
        url.push_str("device_id=");
        url.push_str(device_id);
    }

    let mut request = s
        .http
        .request(method, url)
        .bearer_auth(&s.spotify_token)
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
