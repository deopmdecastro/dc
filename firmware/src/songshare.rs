//! SongShare/Songstats catalog via dc-os-core.

use crate::{spotify::SpotifyTrack, system::SystemEvent};
use anyhow::{anyhow, Result};
use embedded_svc::http::client::{Client as HttpClient, Method};
use embedded_svc::utils::io;
use esp_idf_svc::http::client::EspHttpConnection;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub fn fetch_tracks(api_health_url: &str, event_tx: &Sender<SystemEvent>) {
    match fetch_tracks_inner(api_health_url) {
        Ok(tracks) if !tracks.is_empty() => {
            log::info!("SongShare: {} faixas carregadas", tracks.len());
            let _ = event_tx.send(SystemEvent::SongShareTracksLoaded(tracks));
        }
        Ok(_) => {
            log::warn!("SongShare: API respondeu sem faixas reconheciveis");
            let _ = event_tx.send(SystemEvent::SongShareTracksLoaded(vec![status_track(
                "Sem musicas",
                "Songstats sem itens",
            )]));
        }
        Err(e) => {
            log::warn!("SongShare: falha ao obter faixas: {e:?}");
            let _ = event_tx.send(SystemEvent::SongShareTracksLoaded(vec![status_track(
                "SongShare indisponivel",
                &short_error(&e.to_string()),
            )]));
        }
    }
}

fn fetch_tracks_inner(api_health_url: &str) -> Result<Vec<SpotifyTrack>> {
    let url = tracks_url(api_health_url);
    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config())?);
    let headers = [("accept", "application/json")];
    let request = client.request(Method::Get, &url, &headers)?;
    let mut response = request.submit()?;
    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(anyhow!(
            "dc-os-core /songshare/tracks retornou HTTP {status}"
        ));
    }

    let body = read_response_body(&mut response)?;
    let top: CoreTracksResponse =
        serde_json::from_str(&body).map_err(|e| anyhow!("parse JSON songshare: {e}"))?;
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

fn tracks_url(api_health_url: &str) -> String {
    let base = api_health_url
        .strip_suffix("/health")
        .unwrap_or_else(|| api_health_url.trim_end_matches('/'));
    format!("{base}/songshare/tracks?compact=true")
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

fn http_config() -> esp_idf_svc::http::client::Configuration {
    esp_idf_svc::http::client::Configuration {
        timeout: Some(Duration::from_secs(6)),
        ..Default::default()
    }
}

fn status_track(name: &str, detail: &str) -> SpotifyTrack {
    SpotifyTrack {
        name: name.to_owned(),
        artists: vec![crate::spotify::Artist {
            name: detail.to_owned(),
        }],
        album: None,
    }
}

fn short_error(error: &str) -> String {
    if error.contains("not subscribed") {
        "RapidAPI sem subscricao".to_owned()
    } else if error.contains("403") {
        "RapidAPI recusou 403".to_owned()
    } else {
        "A repetir em breve".to_owned()
    }
}

#[derive(Debug, serde::Deserialize)]
struct CoreTracksResponse {
    ok: bool,
    #[serde(default)]
    body: Option<TracksBody>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct TracksBody {
    items: Vec<SpotifyTrack>,
}
