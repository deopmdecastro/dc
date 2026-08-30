//! Fish Audio API: TTS e efeitos sonoros.
//! Docs: https://fish.audio/pt/app/developers/

use anyhow::{anyhow, Result};
use embedded_svc::{
    http::{client::Client as HttpClient, Method},
    io::Write as EmbeddedWrite,
    utils::io,
};
use esp_idf_svc::http::client::EspHttpConnection;
use std::fs;
use std::io::Write;
use std::path::Path;

const API_BASE: &str = "https://api.fish.audio";
mod generated {
    include!(concat!(env!("OUT_DIR"), "/fish_audio_key.rs"));
}
const API_KEY: &str = generated::FISH_AUDIO_API_KEY;
const SOUND_DIR: &str = "sound";

fn http_config() -> esp_idf_svc::http::client::Configuration {
    esp_idf_svc::http::client::Configuration {
        timeout: Some(std::time::Duration::from_secs(15)),
        buffer_size: Some(2048),
        ..Default::default()
    }
}

/// Testa conectividade com um dominio especifico.
/// Retorna true se conseguiu resolver DNS e conectar.
pub fn test_connection(url: &str) -> bool {
    log::info!("Teste de conectividade: {}", url);
    let conn = match EspHttpConnection::new(&http_config()) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Falha ao criar conexao HTTP: {:?}", e);
            return false;
        }
    };
    let mut client = HttpClient::wrap(conn);
    let headers = [("accept", "application/json")];
    let request = match client.request(Method::Get, url, &headers) {
        Ok(r) => r,
        Err(e) => {
            log::warn!("Falha ao criar request: {:?}", e);
            return false;
        }
    };
    match request.submit() {
        Ok(response) => {
            let status = response.status();
            log::info!("Conectividade OK - HTTP {}", status);
            (200..300).contains(&status)
        }
        Err(e) => {
            log::warn!("Falha na conexao: {:?}", e);
            false
        }
    }
}

/// Testa a conectividade com a API Fish Audio.
pub fn test_fish_audio() -> bool {
    log::info!("Teste Fish Audio: iniciando...");
    test_connection("https://api.fish.audio")
}

/// Informacoes de diagnostico da rede.
pub fn network_diagnostics() {
    log::info!("=== Diagnostico de Rede ===");
    log::info!("Teste 1: Open-Meteo (HTTPS)...");
    let openmeteo = check_internet();
    log::info!("Resultado: {}", if openmeteo { "OK" } else { "FALHA" });

    log::info!("Teste 2: Fish Audio (HTTPS)...");
    let fish = test_fish_audio();
    log::info!("Resultado: {}", if fish { "OK" } else { "FALHA" });

    log::info!("===========================");
}

/// Verifica se ha conectividade com a internet (usando Open-Meteo que ja funciona).
pub fn check_internet() -> bool {
    test_connection(
        "https://api.open-meteo.com/v1/forecast?latitude=0&longitude=0&current=temperature_2m",
    )
}

/// Gera TTS (texto para voz) e guarda o ficheiro de audio.
/// Retorna o caminho do ficheiro gerado.
pub fn tts(text: &str, voice_id: Option<&str>) -> Result<String> {
    if !check_internet() {
        return Err(anyhow!("Sem conexao a internet"));
    }
    let voice = voice_id.unwrap_or("7f2844ae83a14f5682cf382304f1a7fc");
    let url = format!("{}/v1/tts", API_BASE);
    let payload = format!(
        r#"{{"text":{},"reference_id":{},"model":"s2.1-pro-free","format":"wav","sample_rate":16000}}"#,
        serde_json::to_string(text)?,
        serde_json::to_string(voice)?
    );
    log::info!("Fish Audio TTS: {} caracteres", text.len());

    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config())?);
    let content_length = payload.len().to_string();
    let headers = [
        ("accept", "audio/wav"),
        ("content-type", "application/json"),
        ("content-length", content_length.as_str()),
        ("authorization", &format!("Bearer {}", API_KEY)),
    ];
    let mut request = client.request(Method::Post, &url, &headers)?;
    request.write_all(payload.as_bytes())?;
    request.flush()?;
    let mut response = request.submit()?;

    let status = response.status();
    log::info!("Fish Audio TTS: HTTP {}", status);
    if !(200..300).contains(&status) {
        return Err(anyhow!("Fish Audio TTS retornou HTTP {status}"));
    }

    let filename = format!("{}.wav", generate_id());
    let path = format!("{}/{}", SOUND_DIR, filename);
    ensure_sound_dir()?;

    let mut buf = [0_u8; 1024];
    let mut total_bytes = 0;
    loop {
        let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
        if bytes_read == 0 {
            break;
        }
        let data = &buf[..bytes_read];
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(total_bytes == 0)
            .append(true)
            .open(&path)?;
        file.write_all(data)?;
        total_bytes += bytes_read;
    }

    log::info!(
        "Fish Audio TTS: {} bytes guardados em {}",
        total_bytes,
        path
    );
    Ok(path)
}

/// Procura efeitos sonoros por nome/tag.
/// Retorna lista de IDs e nomes de sons.
pub fn search_sound_effects(query: &str) -> Result<Vec<SoundEffect>> {
    let url = format!(
        "{}/v1/sound-effects/search?q={}",
        API_BASE,
        url_encode(query)
    );
    log::info!("Fish Audio: procurando efeitos '{}'", query);

    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config())?);
    let headers = [
        ("accept", "application/json"),
        ("authorization", &format!("Bearer {}", API_KEY)),
    ];
    let request = client.request(Method::Get, &url, &headers)?;
    let mut response = request.submit()?;

    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(anyhow!("Fish Audio search retornou HTTP {status}"));
    }

    let mut buf = [0_u8; 2048];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
    let body = core::str::from_utf8(&buf[..bytes_read]).unwrap_or("[]");

    let results: Vec<SoundEffect> = serde_json::from_str(body).map_err(|e| {
        log::warn!("Fish Audio: erro na resposta: {}", body);
        anyhow!("Fish Audio: erro ao parsear resposta: {e}")
    })?;

    log::info!("Fish Audio: {} efeitos encontrados", results.len());
    Ok(results)
}

/// Faz download de um efeito sonoro pelo ID.
/// Retorna o caminho do ficheiro descarregado.
pub fn download_sound_effect(effect_id: &str, name: Option<&str>) -> Result<String> {
    let url = format!("{}/v1/sound-effects/{}/download", API_BASE, effect_id);
    log::info!("Fish Audio: descarregando efeito {}", effect_id);

    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config())?);
    let headers = [
        ("accept", "audio/mpeg"),
        ("authorization", &format!("Bearer {}", API_KEY)),
    ];
    let request = client.request(Method::Get, &url, &headers)?;
    let mut response = request.submit()?;

    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(anyhow!("Fish Audio download retornou HTTP {status}"));
    }

    let raw_name = name.unwrap_or(effect_id);
    let filename = format!("{}.mp3", raw_name.replace('/', "_").replace('\\', "_"));
    let path = format!("{}/{}", SOUND_DIR, filename);
    ensure_sound_dir()?;

    let mut buf = [0_u8; 1024];
    let mut total_bytes = 0;
    loop {
        let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
        if bytes_read == 0 {
            break;
        }
        let data = &buf[..bytes_read];
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(total_bytes == 0)
            .append(true)
            .open(&path)?;
        file.write_all(data)?;
        total_bytes += bytes_read;
    }

    log::info!(
        "Fish Audio: {} bytes descarregados para {}",
        total_bytes,
        path
    );
    Ok(path)
}

/// Lista ficheiros de som locais.
pub fn list_local_sounds() -> Vec<String> {
    let mut sounds = Vec::new();
    if let Ok(entries) = fs::read_dir(SOUND_DIR) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                sounds.push(name.to_string());
            }
        }
    }
    sounds
}

/// Remove um ficheiro de som local.
pub fn delete_sound(filename: &str) -> Result<()> {
    let path = format!("{}/{}", SOUND_DIR, filename);
    fs::remove_file(&path)?;
    log::info!("Fish Audio: removido {}", path);
    Ok(())
}

fn ensure_sound_dir() -> Result<()> {
    let path = Path::new(SOUND_DIR);
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", nanos)
}

fn url_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push('+'),
            _ => {
                let bytes = c.to_string().as_bytes().to_vec();
                for b in bytes {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    result
}

#[derive(Debug, serde::Deserialize)]
pub struct SoundEffect {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub duration: Option<f64>,
    pub tags: Option<Vec<String>>,
}
