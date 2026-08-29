//! Spotify integration - fetches the user's top tracks.
//!
//! Prefer the local dc-os-core endpoint over HTTP. If that is unavailable and
//! a build-time token exists, fall back to Spotify Web API directly.

use crate::system::SystemEvent;
use anyhow::{anyhow, Result};
use embedded_svc::http::client::{Client as HttpClient, Method};
use embedded_svc::utils::io;
use esp_idf_svc::http::client::EspHttpConnection;
use std::sync::mpsc::Sender;

include!(concat!(env!("OUT_DIR"), "/spotify_token.rs"));

const SPOTIFY_API_BASE: &str =
    "https://api.spotify.com/v1/me/top/tracks?time_range=long_term&limit=5";

pub fn fetch_top_tracks(api_health_url: &str, token: &str, event_tx: &Sender<SystemEvent>) {
    match fetch_top_tracks_inner(api_health_url, token) {
        Ok(tracks) => {
            log::info!("Spotify: {} faixas carregadas", tracks.len());
            let _ = event_tx.send(SystemEvent::SpotifyTracksLoaded(tracks));
        }
        Err(e) => {
            log::warn!("Spotify: falha ao obter top tracks: {e:?}");
            let _ = event_tx.send(SystemEvent::SpotifyTracksLoaded(vec![
                SpotifyTrack::status("Spotify indisponivel", "A repetir em breve"),
            ]));
        }
    }
}

fn fetch_top_tracks_inner(api_health_url: &str, token: &str) -> Result<Vec<SpotifyTrack>> {
    match fetch_top_tracks_from_core(api_health_url) {
        Ok(tracks) => return Ok(tracks),
        Err(e) => log::warn!("Spotify: dc-os-core /music/top-tracks falhou: {e:?}"),
    }

    if token.is_empty() {
        return Err(anyhow!("SPOTIFY_TOKEN vazio e dc-os-core indisponivel"));
    }

    fetch_top_tracks_from_spotify(token)
}

fn fetch_top_tracks_from_core(api_health_url: &str) -> Result<Vec<SpotifyTrack>> {
    let url = core_top_tracks_url(api_health_url);
    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);
    let headers = [("accept", "application/json")];
    let request = client.request(Method::Get, &url, &headers)?;
    let mut response = request.submit()?;
    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(anyhow!("dc-os-core retornou HTTP {status}"));
    }

    let body = read_response_body(&mut response)?;
    let top: CoreTopTracksResponse =
        serde_json::from_str(&body).map_err(|e| anyhow!("parse JSON dc-os-core: {e}"))?;
    if !top.ok {
        return Err(anyhow!(
            "dc-os-core reportou ok=false: {}",
            top.error.unwrap_or_else(|| "sem detalhe".to_owned())
        ));
    }

    Ok(top
        .body
        .ok_or_else(|| anyhow!("dc-os-core respondeu sem body.items"))?
        .items)
}

fn fetch_top_tracks_from_spotify(token: &str) -> Result<Vec<SpotifyTrack>> {
    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);
    let headers = [
        ("accept", "application/json"),
        ("authorization", &format!("Bearer {token}")),
    ];
    let request = client.request(Method::Get, SPOTIFY_API_BASE, &headers)?;
    let mut response = request.submit()?;

    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(anyhow!("Spotify API retornou HTTP {status}"));
    }

    let body = read_response_body(&mut response)?;
    log::info!("Spotify: resposta {} bytes", body.len());

    let top: TopTracksResponse =
        serde_json::from_str(&body).map_err(|e| anyhow!("parse JSON: {e}"))?;

    Ok(top.items)
}

fn read_response_body<R>(response: &mut R) -> Result<String>
where
    R: embedded_svc::io::Read,
    <R as embedded_svc::io::ErrorType>::Error: core::fmt::Debug,
{
    let mut buf = Vec::new();
    let mut chunk = [0u8; 512];
    loop {
        let n = io::try_read_full(&mut *response, &mut chunk)
            .map_err(|e| anyhow!("read body: {:?}", e.0))?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        if n < chunk.len() {
            break;
        }
    }

    String::from_utf8(buf).map_err(|e| anyhow!("body nao e UTF-8: {e}"))
}

fn core_top_tracks_url(api_health_url: &str) -> String {
    let base = api_health_url
        .strip_suffix("/health")
        .unwrap_or_else(|| api_health_url.trim_end_matches('/'));
    format!("{base}/music/top-tracks?compact=true")
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct SpotifyTrack {
    pub name: String,
    #[serde(default)]
    pub artists: Vec<Artist>,
    #[serde(default)]
    pub album: Option<Album>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct Artist {
    pub name: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct Album {
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
struct TopTracksResponse {
    items: Vec<SpotifyTrack>,
}

#[derive(Debug, serde::Deserialize)]
struct CoreTopTracksResponse {
    ok: bool,
    #[serde(default)]
    body: Option<TopTracksResponse>,
    #[serde(default)]
    error: Option<String>,
}

impl SpotifyTrack {
    fn status(name: &str, detail: &str) -> Self {
        Self {
            name: name.to_owned(),
            artists: vec![Artist {
                name: detail.to_owned(),
            }],
            album: None,
        }
    }

    pub fn artist_names(&self) -> String {
        self.artists
            .iter()
            .map(|a| a.name.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn album_name(&self) -> &str {
        self.album
            .as_ref()
            .map(|album| album.name.as_str())
            .unwrap_or("")
    }
}
