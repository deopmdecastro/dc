//! Task de rede: Wi-Fi station + SNTP + probe HTTP do dc-os-core.

use crate::system::{NetworkCommand, SystemEvent, WeatherInfo, WifiNetworkInfo};
use anyhow::{anyhow, Result};
use core::convert::TryInto;
use embedded_svc::{
    http::{client::Client as HttpClient, Method},
    io::Write as EmbeddedWrite,
    utils::io,
    wifi::{AuthMethod, ClientConfiguration, Configuration},
};
use esp_idf_hal::modem::Modem;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::client::EspHttpConnection,
    nvs::EspDefaultNvsPartition,
    sntp::EspSntp,
    wifi::{BlockingWifi, EspWifi},
};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct NetConfig {
    pub enabled: bool,
    pub ssid: String,
    pub password: String,
    pub bluetooth_enabled: bool,
    pub api_health_url: String,
    pub region_index: u8,
    pub timezone_offset_secs: i32,
}

const NET_TASK_STACK_SIZE: usize = 16 * 1024;
const API_PROBE_INTERVAL_SECS: u64 = 30;
const CLOCK_INTERVAL_SECS: u64 = 1;
const API_TIME_INTERVAL_SECS: u64 = 30;
const WIFI_SCAN_INTERVAL_SECS: u64 = 45;
const SPOTIFY_RETRY_INTERVAL_SECS: u64 = 60;
const SONGSHARE_RETRY_INTERVAL_SECS: u64 = 120;
const WEATHER_INTERVAL_SECS: u64 = 600;
const MAX_WIFI_NETWORKS: usize = 10;

pub fn spawn_network_task(
    modem: Modem<'static>,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
    cfg: NetConfig,
    cmd_rx: Receiver<NetworkCommand>,
    event_tx: Sender<SystemEvent>,
) -> Result<()> {
    std::thread::Builder::new()
        .name("dc-net".into())
        .stack_size(NET_TASK_STACK_SIZE)
        .spawn(move || {
            if let Err(e) = run_network(modem, sysloop, nvs, cfg, cmd_rx, event_tx) {
                log::error!("network: falhou: {e:?}");
            }
        })?;
    Ok(())
}

fn run_network(
    modem: Modem<'static>,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
    cfg: NetConfig,
    cmd_rx: Receiver<NetworkCommand>,
    event_tx: Sender<SystemEvent>,
) -> Result<()> {
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sysloop.clone(), Some(nvs.clone()))?,
        sysloop,
    )?;

    let mut desired_wifi = cfg.enabled;
    let mut ssid = cfg.ssid;
    let mut password = cfg.password;
    let mut region_index = cfg.region_index.min(4);
    let mut timezone_offset_secs = cfg.timezone_offset_secs;
    let mut connected = false;
    let mut sntp: Option<EspSntp<'static>> = None;
    let mut next_probe_in = 0_u64;
    let mut next_clock_in = 0_u64;
    let mut next_api_time_in = 0_u64;
    let mut next_scan_in = 0_u64;
    let mut next_spotify_in = 0_u64;
    let mut next_songshare_in = 0_u64;
    let mut next_weather_in = 0_u64;
    let mut last_api_time = "--:--".to_owned();

    log::info!(
        "Wi-Fi: config inicial enabled={} ssid='{}' api='{}'",
        desired_wifi,
        ssid,
        cfg.api_health_url
    );
    let _ = event_tx.send(SystemEvent::BluetoothChanged(cfg.bluetooth_enabled));
    let _ = event_tx.send(SystemEvent::BluetoothDevicesChanged(Vec::new()));

    loop {
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                NetworkCommand::SetWifiEnabled(enabled) => {
                    desired_wifi = enabled;
                    next_scan_in = 0;
                    log::info!("Wi-Fi: pedido da UI enabled={enabled}");
                    if !enabled {
                        let _ = event_tx.send(SystemEvent::WifiNetworksChanged(Vec::new()));
                    }
                }
                NetworkCommand::ScanWifi => {
                    next_scan_in = 0;
                    log::info!("Wi-Fi: scan pedido pela UI");
                }
                NetworkCommand::SetWifiCredentials {
                    ssid: new_ssid,
                    password: new_password,
                } => {
                    log::info!("Wi-Fi: nova rede selecionada '{}'", new_ssid);
                    ssid = new_ssid;
                    password = new_password;
                    desired_wifi = true;
                    next_scan_in = 0;
                    if connected {
                        let _ = wifi.disconnect();
                        let _ = wifi.stop();
                        connected = false;
                        let _ = event_tx.send(SystemEvent::WifiChanged(false));
                        let _ = event_tx.send(SystemEvent::ApiHealthChanged(false));
                    }
                }
                NetworkCommand::SetBluetoothEnabled(enabled) => {
                    log::info!(
                        "Bluetooth: estado {} guardado; scan BLE real desativado nesta combinacao ESP-IDF/esp-idf-svc",
                        if enabled { "ligado" } else { "desligado" }
                    );
                    let _ = event_tx.send(SystemEvent::BluetoothChanged(enabled));
                    let _ = event_tx.send(SystemEvent::BluetoothDevicesChanged(Vec::new()));
                }
                NetworkCommand::ScanBluetooth => {
                    log::info!(
                        "Bluetooth: scan BLE ignorado; Bluedroid do esp-idf-svc 0.52.1 nao compila com ESP-IDF 5.2.3"
                    );
                    let _ = event_tx.send(SystemEvent::BluetoothDevicesChanged(Vec::new()));
                }
                NetworkCommand::SetLocale {
                    region_index: next_region,
                    timezone_offset_secs: offset_secs,
                } => {
                    region_index = next_region.min(4);
                    timezone_offset_secs = offset_secs;
                    next_weather_in = 0;
                    let _ =
                        event_tx.send(SystemEvent::TimeChanged(current_hhmm(timezone_offset_secs)));
                    log::info!(
                        "SNTP: region={} timezone offset={}s",
                        region_index,
                        offset_secs
                    );
                }
                NetworkCommand::RefreshWeather => {
                    next_weather_in = 0;
                    log::info!("Clima: atualizacao imediata pedida pela UI");
                }
                NetworkCommand::VoiceCommand {
                    text,
                    language_index,
                } => {
                    handle_voice_text(
                        connected,
                        &cfg.api_health_url,
                        text,
                        language_index,
                        &event_tx,
                    );
                }
                NetworkCommand::VoiceAudio {
                    wav,
                    language_index,
                } => {
                    if !connected {
                        log::warn!("Voz: transcricao ignorada sem Wi-Fi");
                        let _ = event_tx.send(SystemEvent::VoiceCommandResult {
                            text: "Voz indisponivel: sem Wi-Fi".to_owned(),
                            app_index: None,
                            app_name: None,
                        });
                    } else if wav.len() <= 44 {
                        log::warn!("Voz: audio vazio");
                        let _ = event_tx.send(SystemEvent::VoiceCommandResult {
                            text: "Nao ouvi audio suficiente".to_owned(),
                            app_index: None,
                            app_name: None,
                        });
                    } else {
                        if !probe_api_fast(&cfg.api_health_url) {
                            log::warn!("Voz: backend inacessivel (probe falhou)");
                            let _ = event_tx.send(SystemEvent::VoiceCommandResult {
                                text: "Backend inacessivel".to_owned(),
                                app_index: None,
                                app_name: None,
                            });
                        } else {
                            match post_voice_transcribe(&cfg.api_health_url, &wav, language_index) {
                                Ok(text) => handle_voice_text(
                                    true,
                                    &cfg.api_health_url,
                                    text,
                                    language_index,
                                    &event_tx,
                                ),
                                Err(e) => {
                                    log::warn!("Voz: /voice/transcribe falhou: {e:?}");
                                    let _ = event_tx.send(SystemEvent::VoiceCommandResult {
                                        text: "Transcricao indisponivel".to_owned(),
                                        app_index: None,
                                        app_name: None,
                                    });
                                }
                            }
                        }
                    }
                }
                NetworkCommand::CreateNote { text } => {
                    if connected {
                        if let Err(e) = post_note(&cfg.api_health_url, &text) {
                            log::warn!("API: criar nota falhou: {e:?}");
                        }
                    } else {
                        log::warn!("API: criar nota ignorada sem Wi-Fi");
                    }
                }
                NetworkCommand::DeleteNote { id } => {
                    if connected {
                        if let Err(e) = delete_note(&cfg.api_health_url, id) {
                            log::warn!("API: apagar nota falhou: {e:?}");
                        }
                    }
                }
                NetworkCommand::MusicCommand(action) => {
                    if connected {
                        if let Err(e) = post_music_command(&cfg.api_health_url, &action) {
                            log::warn!("API: comando de musica falhou: {e:?}");
                        }
                    } else {
                        log::warn!("API: comando de musica ignorado sem Wi-Fi");
                    }
                }
            }
        }

        if desired_wifi {
            if next_scan_in == 0 {
                let connected_ssid = if connected { ssid.as_str() } else { "" };
                match scan_wifi(&mut wifi, connected_ssid) {
                    Ok(networks) => {
                        log::info!("Wi-Fi: scan encontrou {} redes", networks.len());
                        let _ = event_tx.send(SystemEvent::WifiNetworksChanged(networks));
                    }
                    Err(e) => log::warn!("Wi-Fi: scan falhou: {e:?}"),
                }
                next_scan_in = WIFI_SCAN_INTERVAL_SECS;
            } else {
                next_scan_in = next_scan_in.saturating_sub(1);
            }
        }

        if desired_wifi && !connected {
            match connect_wifi(&mut wifi, &ssid, &password) {
                Ok(()) => {
                    connected = true;
                    let _ = event_tx.send(SystemEvent::WifiChanged(true));
                    sntp = match EspSntp::new_default() {
                        Ok(service) => {
                            log::info!("SNTP: iniciado");
                            Some(service)
                        }
                        Err(e) => {
                            log::warn!("SNTP: falha ao iniciar: {e:?}");
                            None
                        }
                    };
                    next_probe_in = 0;
                    next_clock_in = 0;
                    next_api_time_in = 0;
                    next_spotify_in = 0;
                    next_songshare_in = 0;
                    next_weather_in = 0;
                }
                Err(e) => {
                    log::warn!("Wi-Fi: falha ao conectar: {e:?}");
                    let _ = event_tx.send(SystemEvent::WifiChanged(false));
                    std::thread::sleep(Duration::from_secs(5));
                }
            }
        }

        if !desired_wifi && connected {
            log::info!("Wi-Fi: desligando");
            let _ = wifi.disconnect();
            let _ = wifi.stop();
            sntp = None;
            connected = false;
            let _ = event_tx.send(SystemEvent::WifiChanged(false));
            let _ = event_tx.send(SystemEvent::ApiHealthChanged(false));
            let _ = event_tx.send(SystemEvent::WifiNetworksChanged(Vec::new()));
            next_spotify_in = 0;
            next_songshare_in = 0;
        }

        if connected {
            if next_clock_in == 0 {
                let local_time = current_hhmm(timezone_offset_secs);
                if local_time == "--:--" && last_api_time != "--:--" {
                    let _ = event_tx.send(SystemEvent::TimeChanged(last_api_time.clone()));
                } else {
                    let _ = event_tx.send(SystemEvent::TimeChanged(local_time));
                }
                next_clock_in = CLOCK_INTERVAL_SECS;
            } else {
                next_clock_in = next_clock_in.saturating_sub(1);
            }

            if next_api_time_in == 0 {
                if let Ok(api_time) = fetch_api_time(&cfg.api_health_url, timezone_offset_secs) {
                    last_api_time = api_time.clone();
                    let _ = event_tx.send(SystemEvent::TimeChanged(api_time));
                }
                next_api_time_in = API_TIME_INTERVAL_SECS;
            } else {
                next_api_time_in = next_api_time_in.saturating_sub(1);
            }

            if next_probe_in == 0 {
                let ok = probe_api(&cfg.api_health_url).unwrap_or_else(|e| {
                    log::warn!(
                        "API: health falhou em '{}': {e:?}; confirma DC_CORE_HTTP com o IP do PC na mesma rede",
                        cfg.api_health_url
                    );
                    false
                });
                let _ = event_tx.send(SystemEvent::ApiHealthChanged(ok));
                next_probe_in = API_PROBE_INTERVAL_SECS;
            } else {
                next_probe_in = next_probe_in.saturating_sub(1);
            }

            if next_spotify_in == 0 {
                let spotify_token = crate::spotify::SPOTIFY_TOKEN;
                log::info!("Spotify: a pedir top tracks via {}", cfg.api_health_url);
                crate::spotify::fetch_top_tracks(&cfg.api_health_url, spotify_token, &event_tx);
                next_spotify_in = SPOTIFY_RETRY_INTERVAL_SECS;
            } else {
                next_spotify_in = next_spotify_in.saturating_sub(1);
            }

            if next_songshare_in == 0 {
                log::info!("SongShare: a pedir musicas via {}", cfg.api_health_url);
                crate::songshare::fetch_tracks(&cfg.api_health_url, &event_tx);
                next_songshare_in = SONGSHARE_RETRY_INTERVAL_SECS;
            } else {
                next_songshare_in = next_songshare_in.saturating_sub(1);
            }

            if next_weather_in == 0 {
                match fetch_weather(&cfg.api_health_url, region_index) {
                    Ok(weather) => {
                        log::info!(
                            "Clima: {} {}C {}",
                            weather.city,
                            weather.temperature_c,
                            weather.summary
                        );
                        let _ = event_tx.send(SystemEvent::WeatherChanged(weather));
                    }
                    Err(e) => log::warn!("Clima: falha ao obter clima real: {e:?}"),
                }
                next_weather_in = WEATHER_INTERVAL_SECS;
            } else {
                next_weather_in = next_weather_in.saturating_sub(1);
            }
        }

        let _keep_sntp_alive = sntp.as_ref();
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn connect_wifi(
    wifi: &mut BlockingWifi<EspWifi<'static>>,
    ssid: &str,
    password: &str,
) -> Result<()> {
    if ssid.is_empty() {
        return Err(anyhow!("SSID vazio"));
    }

    let wifi_configuration = Configuration::Client(ClientConfiguration {
        ssid: ssid
            .try_into()
            .map_err(|_| anyhow!("SSID demasiado longo para Wi-Fi"))?,
        bssid: None,
        auth_method: if password.is_empty() {
            AuthMethod::None
        } else {
            AuthMethod::WPA2Personal
        },
        password: password
            .try_into()
            .map_err(|_| anyhow!("password Wi-Fi demasiado longa"))?,
        channel: None,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;
    if !wifi.is_started()? {
        wifi.start()?;
        log::info!("Wi-Fi: started");
    }
    wifi.connect()?;
    log::info!("Wi-Fi: connected");
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    log::info!("Wi-Fi: DHCP {ip_info:?}");
    Ok(())
}

fn probe_api(url: &str) -> Result<bool> {
    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config())?);
    let headers = [("accept", "application/json")];
    let request = client.request(Method::Get, url, &headers)?;
    log::info!("API: GET {url}");
    let mut response = request.submit()?;
    let status = response.status();
    let mut buf = [0_u8; 256];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
    let body = core::str::from_utf8(&buf[..bytes_read]).unwrap_or("");
    log::info!("API: health status={} body={:?}", status, body);
    Ok((200..300).contains(&status))
}

fn fetch_api_time(health_url: &str, offset_secs: i32) -> Result<String> {
    let url = api_time_url(health_url, offset_secs);
    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config())?);
    let headers = [("accept", "application/json")];
    let request = client.request(Method::Get, &url, &headers)?;
    log::info!("API: GET {url}");
    let mut response = request.submit()?;
    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(anyhow!("API /time retornou HTTP {status}"));
    }

    let mut buf = [0_u8; 192];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
    let body = core::str::from_utf8(&buf[..bytes_read]).unwrap_or("");
    let value: serde_json::Value = serde_json::from_str(body)?;
    let hhmm = value
        .get("hhmm")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("API /time sem campo hhmm"))?;

    Ok(hhmm.to_owned())
}

fn api_time_url(health_url: &str, offset_secs: i32) -> String {
    let base = health_url
        .strip_suffix("/health")
        .unwrap_or_else(|| health_url.trim_end_matches('/'));
    format!("{base}/time?offset_secs={offset_secs}")
}

fn fetch_weather(health_url: &str, region_index: u8) -> Result<WeatherInfo> {
    // Try IP geolocation first for GPS-based weather
    let coords = ip_geolocation();

    let url = match &coords {
        Some(loc) => {
            log::info!("Clima: a usar localização GPS ({}, {})", loc.lat, loc.lon);
            format!(
                "{}/weather?lat={}&lon={}",
                api_base(health_url),
                loc.lat, loc.lon
            )
        }
        None => {
            format!(
                "{}/weather?region={}",
                api_base(health_url),
                region_index.min(4)
            )
        }
    };

    log::info!("Clima: GET {url}");
    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config())?);
    let headers = [("accept", "application/json")];
    let request = client.request(Method::Get, &url, &headers)?;
    let mut response = request.submit()?;
    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(anyhow!("dc-os-core /weather retornou HTTP {status}"));
    }
    let mut buf = [0_u8; 1024];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
    let body = core::str::from_utf8(&buf[..bytes_read]).unwrap_or("");
    let value: CoreWeatherResponse = serde_json::from_str(body)?;
    if !value.ok {
        return Err(anyhow!("dc-os-core /weather ok=false: {}", value.summary));
    }

    Ok(WeatherInfo {
        city: value.city,
        temperature_c: value.temperature_c as i32,
        summary: value.summary,
    })
}

/// Reverse geocode coordinates to city name using BigDataFree API
fn reverse_geocode(lat: f64, lon: f64) -> Option<String> {
    let url = format!(
        "https://api.bigdatacloud.net/data/reverse-geocode-client?latitude={}&longitude={}&localityLanguage=pt",
        lat, lon
    );
    log::info!("Geocoding: buscando cidade para {}, {}", lat, lon);
    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config()).ok()?);
    let headers = [("accept", "application/json")];
    let request = client.request(Method::Get, &url, &headers).ok()?;
    let mut response = request.submit().ok()?;
    let status = response.status();
    if !(200..300).contains(&status) {
        return None;
    }
    let mut buf = [0_u8; 1024];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0).ok()?;
    let body = core::str::from_utf8(&buf[..bytes_read]).unwrap_or("");

    // Parse JSON manually to avoid serde issues on embedded
    let city = extract_json_string(body, "\"city\":\"").or_else(|| extract_json_string(body, "\"locality\":\""));
    let country = extract_json_string(body, "\"countryName\":\"");

    city.map(|name| format!("{}{}", name, country.map(|c| format!(", {}", c)).unwrap_or_default()))
}

/// Simple JSON string extractor for embedded (no serde needed)
fn extract_json_string<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let start = json.find(key)? + key.len();
    let end = json[start..].find('"')?;
    Some(&json[start..start + end])
}

#[allow(dead_code)]
fn weather_summary(code: i32, temp: i32, wind: i32) -> String {
    let condition = match code {
        0 => "Ceu limpo",
        1 | 2 | 3 => "Parcialmente nublado",
        45 | 48 => "Nevoeiro",
        51 | 53 | 55 => "Chuvisco",
        61 | 63 | 65 => "Chuva",
        71 | 73 | 75 => "Neve",
        80 | 81 | 82 => "Aguaceiros",
        95 | 96 | 99 => "Trovoada",
        _ => "Nublado",
    };
    format!("{} | {}°C | Vento {} km/h", condition, temp, wind)
}

#[derive(Debug, serde::Deserialize)]
struct CoreWeatherResponse {
    ok: bool,
    city: String,
    temperature_c: i64,
    summary: String,
}

/// IP geolocation result
#[derive(Debug)]
struct IpLocation {
    lat: f64,
    lon: f64,
    city: String,
}

/// Get approximate location based on IP address using ipapi.co (free, no key needed)
fn ip_geolocation() -> Option<IpLocation> {
    let url = "https://ipapi.co/json/";
    log::info!("Clima: a obter localização por IP...");
    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config()).ok()?);
    let headers = [("accept", "application/json")];
    let request = client.request(Method::Get, url, &headers).ok()?;
    let mut response = request.submit().ok()?;
    let status = response.status();
    if !(200..300).contains(&status) {
        log::warn!("Clima: IP geolocation retornou HTTP {status}");
        return None;
    }
    let mut buf = [0_u8; 1024];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0).ok()?;
    let body = core::str::from_utf8(&buf[..bytes_read]).unwrap_or("");

    #[derive(serde::Deserialize)]
    struct IpApiResponse {
        latitude: Option<f64>,
        longitude: Option<f64>,
        city: Option<String>,
        region: Option<String>,
        country_name: Option<String>,
    }

    let value: IpApiResponse = serde_json::from_str(body).ok()?;
    let lat = value.latitude?;
    let lon = value.longitude?;
    let city = format!(
        "{}, {}",
        value.city.unwrap_or_default(),
        value.region.unwrap_or_default()
    );
    log::info!("Clima: localização IP -> {} ({}, {})", city, lat, lon);
    Some(IpLocation { lat, lon, city })
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct OpenMeteoResponse {
    current: OpenMeteoCurrent,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct OpenMeteoCurrent {
    temperature_2m: f64,
    wind_speed_10m: f64,
    weather_code: i32,
}

fn api_base(health_url: &str) -> String {
    health_url
        .strip_suffix("/health")
        .unwrap_or_else(|| health_url.trim_end_matches('/'))
        .to_owned()
}

fn post_music_command(health_url: &str, action: &str) -> Result<()> {
    let url = music_command_url(health_url);
    let payload = format!("{{\"action\":\"{}\"}}", action);
    let content_length = payload.len().to_string();
    let headers = [
        ("content-type", "application/json"),
        ("content-length", content_length.as_str()),
    ];

    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config())?);
    let mut request = client.request(Method::Post, &url, &headers)?;
    request.write_all(payload.as_bytes())?;
    request.flush()?;
    let mut response = request.submit()?;
    let status = response.status();
    let mut buf = [0_u8; 512];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
    let body = core::str::from_utf8(&buf[..bytes_read]).unwrap_or("");
    log::info!(
        "API: POST {} action={} status={} body={:?}",
        url,
        action,
        status,
        body
    );

    if !(200..300).contains(&status) || body.contains("\"ok\":false") {
        return Err(anyhow!(
            "API music command falhou status={status} body={body}"
        ));
    }

    Ok(())
}

fn scan_wifi(
    wifi: &mut BlockingWifi<EspWifi<'static>>,
    connected_ssid: &str,
) -> Result<Vec<WifiNetworkInfo>> {
    if !wifi.is_started()? {
        let station_config = Configuration::Client(ClientConfiguration {
            ssid: "".try_into().map_err(|_| anyhow!("SSID vazio invalido"))?,
            bssid: None,
            auth_method: AuthMethod::None,
            password: ""
                .try_into()
                .map_err(|_| anyhow!("password vazia invalida"))?,
            channel: None,
            ..Default::default()
        });
        wifi.set_configuration(&station_config)?;
        wifi.start()?;
        log::info!("Wi-Fi: started para scan");
    }

    let mut networks: Vec<WifiNetworkInfo> = wifi
        .scan()?
        .into_iter()
        .filter_map(|ap| {
            let ssid = ap.ssid.as_str().trim();
            if ssid.is_empty() {
                return None;
            }
            Some(WifiNetworkInfo {
                ssid: ssid.to_owned(),
                secured: ap.auth_method.is_some() && ap.auth_method != Some(AuthMethod::None),
                connected: ssid == connected_ssid,
                signal_strength: ap.signal_strength,
            })
        })
        .collect();

    networks.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));
    let mut unique = Vec::new();
    for network in networks {
        if unique
            .iter()
            .any(|n: &WifiNetworkInfo| n.ssid == network.ssid)
        {
            continue;
        }
        unique.push(network);
        if unique.len() == MAX_WIFI_NETWORKS {
            break;
        }
    }

    Ok(unique)
}

fn post_note(health_url: &str, text: &str) -> Result<()> {
    let url = api_base(health_url) + "/notes";
    let body = serde_json::json!({"text": text});
    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config())?);
    let mut request =
        client.request(Method::Post, &url, &[("Content-Type", "application/json")])?;
    request.write_all(body.to_string().as_bytes())?;
    let response = request.submit()?;
    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(anyhow!("POST /notes retornou HTTP {status}"));
    }
    Ok(())
}

fn delete_note(health_url: &str, id: u64) -> Result<()> {
    let url = format!("{}/notes/{id}", api_base(health_url));
    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config())?);
    let response = client.request(Method::Delete, &url, &[])?.submit()?;
    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(anyhow!("DELETE /notes/{id} retornou HTTP {status}"));
    }
    Ok(())
}

fn handle_voice_text(
    connected: bool,
    api_health_url: &str,
    text: String,
    language_index: u8,
    event_tx: &Sender<SystemEvent>,
) {
    if !connected {
        log::warn!("Voz: comando ignorado sem Wi-Fi");
        let _ = event_tx.send(SystemEvent::VoiceCommandResult {
            text: "Voz indisponivel: sem Wi-Fi".to_owned(),
            app_index: None,
            app_name: None,
        });
        return;
    }

    match post_voice_command(api_health_url, &text, language_index) {
        Ok(result) => {
            log::info!("Voz: comando '{}' -> {:?}", text, result.app_name);
            let _ = event_tx.send(SystemEvent::VoiceCommandResult {
                text,
                app_index: result.app_index,
                app_name: result.app_name,
            });
        }
        Err(e) => {
            log::warn!("Voz: /voice/command falhou: {e:?}");
            let _ = event_tx.send(SystemEvent::VoiceCommandResult {
                text,
                app_index: None,
                app_name: None,
            });
        }
    }
}

fn post_voice_transcribe(health_url: &str, wav: &[u8], language_index: u8) -> Result<String> {
    let url = format!(
        "{}/voice/transcribe?language={}",
        api_base(health_url),
        language_index.min(4)
    );
    let content_length = wav.len().to_string();
    let headers = [
        ("content-type", "audio/wav"),
        ("accept", "application/json"),
        ("content-length", content_length.as_str()),
    ];

    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config())?);
    let mut request = client.request(Method::Post, &url, &headers)?;
    request.write_all(wav)?;
    request.flush()?;
    let mut response = request.submit()?;
    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(anyhow!("/voice/transcribe retornou HTTP {status}"));
    }

    let body = read_response_body_limit(&mut response, 4096)?;
    let result: VoiceTranscribeResponse =
        serde_json::from_str(&body).map_err(|e| anyhow!("parse /voice/transcribe: {e}"))?;
    let text = result.text.unwrap_or_default();
    if !result.ok || text.trim().is_empty() {
        return Err(anyhow!(
            "transcricao vazia: {}",
            result.error.unwrap_or_else(|| body)
        ));
    }
    Ok(text)
}

fn post_voice_command(
    health_url: &str,
    text: &str,
    language_index: u8,
) -> Result<VoiceCommandResponse> {
    let url = api_base(health_url) + "/voice/command";
    let payload = serde_json::json!({
        "text": text,
        "language": language_index,
    })
    .to_string();
    let content_length = payload.len().to_string();
    let headers = [
        ("content-type", "application/json"),
        ("accept", "application/json"),
        ("content-length", content_length.as_str()),
    ];

    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config())?);
    let mut request = client.request(Method::Post, &url, &headers)?;
    request.write_all(payload.as_bytes())?;
    request.flush()?;
    let mut response = request.submit()?;
    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(anyhow!("/voice/command retornou HTTP {status}"));
    }

    let body = read_response_body_limit(&mut response, 1024)?;
    let result: VoiceCommandResponse =
        serde_json::from_str(&body).map_err(|e| anyhow!("parse /voice/command: {e}"))?;
    if !result.ok {
        return Err(anyhow!("comando nao reconhecido: {body}"));
    }
    Ok(result)
}

fn music_command_url(health_url: &str) -> String {
    if let Some(base) = health_url.strip_suffix("/health") {
        format!("{base}/music/command")
    } else {
        format!("{}/music/command", health_url.trim_end_matches('/'))
    }
}

fn http_config() -> esp_idf_svc::http::client::Configuration {
    esp_idf_svc::http::client::Configuration {
        timeout: Some(Duration::from_secs(20)),
        use_global_ca_store: false,
        ..Default::default()
    }
}

fn http_config_fast() -> esp_idf_svc::http::client::Configuration {
    esp_idf_svc::http::client::Configuration {
        timeout: Some(Duration::from_secs(3)),
        use_global_ca_store: false,
        ..Default::default()
    }
}

fn probe_api_fast(url: &str) -> bool {
    let conn = match EspHttpConnection::new(&http_config_fast()) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let mut client = HttpClient::wrap(conn);
    let headers = [("accept", "application/json")];
    let request = match client.request(Method::Get, url, &headers) {
        Ok(r) => r,
        Err(_) => return false,
    };
    match request.submit() {
        Ok(mut response) => (200..300).contains(&response.status()),
        Err(_) => false,
    }
}

fn read_response_body_limit<R>(response: &mut R, limit: usize) -> Result<String>
where
    R: embedded_svc::io::Read,
    <R as embedded_svc::io::ErrorType>::Error: core::fmt::Debug + Send + Sync + 'static,
{
    let mut body = Vec::new();
    let mut chunk = [0_u8; 512];
    loop {
        let bytes_read = io::try_read_full(&mut *response, &mut chunk)
            .map_err(|e| anyhow!("read error: {:?}", e.0))?;
        if bytes_read == 0 {
            break;
        }
        let remaining = limit.saturating_sub(body.len());
        body.extend_from_slice(&chunk[..bytes_read.min(remaining)]);
        if bytes_read < chunk.len() || body.len() >= limit {
            break;
        }
    }
    String::from_utf8(body).map_err(|e| anyhow!("body nao e UTF-8: {e}"))
}

#[derive(Debug, serde::Deserialize)]
struct VoiceTranscribeResponse {
    ok: bool,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct VoiceCommandResponse {
    ok: bool,
    #[serde(default)]
    app_index: Option<u8>,
    #[serde(default)]
    app_name: Option<String>,
}

fn current_hhmm(offset_secs: i32) -> String {
    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(value) => value.as_secs(),
        Err(_) => return "--:--".to_owned(),
    };

    if now < 1_600_000_000 {
        return "--:--".to_owned();
    }

    let local = if offset_secs >= 0 {
        now.saturating_add(offset_secs as u64)
    } else {
        now.saturating_sub(offset_secs.unsigned_abs() as u64)
    };
    let seconds_of_day = local % 86_400;
    let hour = seconds_of_day / 3600;
    let minute = (seconds_of_day % 3600) / 60;
    format!("{hour:02}:{minute:02}")
}
