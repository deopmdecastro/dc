//! Spotify Web API integration — fetches the user's top tracks.
//!
//! Uses the hardcoded OAuth token provided at build time. The ESP32-S3
//! makes an HTTPS GET to `api.spotify.com/v1/me/top/tracks`, parses the
//! JSON response with serde_json, and emits `SystemEvent::SpotifyTracksLoaded`
//! so the Slint UI can display real track data in the music player.

use crate::system::SystemEvent;
use anyhow::{anyhow, Result};
use embedded_svc::http::client::{Client as HttpClient, Method};
use esp_idf_svc::http::client::EspHttpConnection;
use std::sync::mpsc::Sender;

const SPOTIFY_API_BASE: &str = "https://api.spotify.com/v1/me/top/tracks?time_range=long_term&limit=5";

pub fn fetch_top_tracks(token: &str, event_tx: &Sender<SystemEvent>) {
    match fetch_top_tracks_inner(token) {
        Ok(tracks) => {
            log::info!("Spotify: {} faixas carregadas", tracks.len());
            let _ = event_tx.send(SystemEvent::SpotifyTracksLoaded(tracks));
        }
        Err(e) => {
            log::warn!("Spotify: falha ao obter top tracks: {e:?}");
        }
    }
}

fn fetch_top_tracks_inner(token: &str) -> Result<Vec<SpotifyTrack>> {
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

    let mut buf = Vec::new();
    let mut chunk = [0u8; 512];
    loop {
        let n = embedded_svc::io::Read::read(&mut response, &mut chunk)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
    }

    let body = std::str::from_utf8(&buf).map_err(|e| anyhow!("body nao e UTF-8: {e}"))?;
    log::info!("Spotify: resposta {} bytes", body.len());

    let top: TopTracksResponse = serde_json::from_str(body)
        .map_err(|e| anyhow!("parse JSON: {e}"))?;

    Ok(top.items)
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct SpotifyTrack {
    pub name: String,
    pub artists: Vec<Artist>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct Artist {
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
struct TopTracksResponse {
    items: Vec<SpotifyTrack>,
}

impl SpotifyTrack {
    pub fn artist_names(&self) -> String {
        self.artists
            .iter()
            .map(|a| a.name.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }
}
