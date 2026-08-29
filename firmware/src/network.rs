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
    let url = api_weather_url(health_url, region_index);
    let mut client = HttpClient::wrap(EspHttpConnection::new(&http_config())?);
    let headers = [("accept", "application/json")];
    let request = client.request(Method::Get, &url, &headers)?;
    log::info!("API: GET {url}");
    let mut response = request.submit()?;
    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(anyhow!("API /weather retornou HTTP {status}"));
    }

    let mut buf = [0_u8; 512];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
    let body = core::str::from_utf8(&buf[..bytes_read]).unwrap_or("");
    let value: WeatherApiResponse = serde_json::from_str(body)?;
    if !value.ok {
        return Err(anyhow!("API /weather respondeu ok=false"));
    }

    Ok(WeatherInfo {
        city: value.city,
        temperature_c: value.temperature_c,
        summary: value.summary,
    })
}

fn api_weather_url(health_url: &str, region_index: u8) -> String {
    let base = health_url
        .strip_suffix("/health")
        .unwrap_or_else(|| health_url.trim_end_matches('/'));
    format!("{base}/weather?region={}", region_index.min(4))
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

fn music_command_url(health_url: &str) -> String {
    if let Some(base) = health_url.strip_suffix("/health") {
        format!("{base}/music/command")
    } else {
        format!("{}/music/command", health_url.trim_end_matches('/'))
    }
}

#[derive(Debug, serde::Deserialize)]
struct WeatherApiResponse {
    ok: bool,
    city: String,
    temperature_c: i32,
    summary: String,
}

fn http_config() -> esp_idf_svc::http::client::Configuration {
    esp_idf_svc::http::client::Configuration {
        timeout: Some(Duration::from_secs(6)),
        ..Default::default()
    }
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
