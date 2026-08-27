//! DC OS Core — hub central.
//!
//! Rotas:
//!   GET  /health              healthcheck
//!   GET  /ws                  WebSocket bidirecional com o firmware
//!   POST /voice/transcribe    encaminha PCM ao Whisper
//!   GET  /music/state         estado atual do player (proxy Mopidy)
//!   POST /music/command       play/pause/next/prev (proxy Mopidy JSON-RPC)

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    stt_url:    String,
    mopidy_url: String,
    http:       reqwest::Client,
}

#[derive(Serialize)]
struct Health { status: &'static str, service: &'static str, version: &'static str }

#[derive(Deserialize)]
struct MusicCommand { action: String }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let state = Arc::new(AppState {
        stt_url:    env::var("STT_URL").unwrap_or_else(|_| "http://stt-whisper:9000/asr".into()),
        mopidy_url: env::var("MOPIDY_URL").unwrap_or_else(|_| "http://mopidy:6680".into()),
        http:       reqwest::Client::new(),
    });

    let app = Router::new()
        .route("/health",           get(health))
        .route("/ws",               get(ws_upgrade))
        .route("/voice/transcribe", post(transcribe))
        .route("/music/state",      get(music_state))
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
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "core.playback.get_state"
    });
    let val = s.http.post(format!("{}/mopidy/rpc", s.mopidy_url))
        .json(&body).send().await
        .and_then(|r| Ok(r))
        .map(|_| serde_json::json!({ "ok": true }))
        .unwrap_or(serde_json::json!({ "ok": false }));
    Json(val)
}

async fn music_command(
    State(s): State<Arc<AppState>>, Json(cmd): Json<MusicCommand>,
) -> Json<serde_json::Value> {
    let method = match cmd.action.as_str() {
        "play"  => "core.playback.play",
        "pause" => "core.playback.pause",
        "next"  => "core.playback.next",
        "prev"  => "core.playback.previous",
        _       => return Json(serde_json::json!({ "error": "unknown action" })),
    };
    let body = serde_json::json!({ "jsonrpc":"2.0", "id":1, "method": method });
    let _ = s.http.post(format!("{}/mopidy/rpc", s.mopidy_url))
        .json(&body).send().await;
    Json(serde_json::json!({ "ok": true, "action": cmd.action }))
}
