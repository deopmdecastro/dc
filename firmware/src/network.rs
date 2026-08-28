//! Task de rede: Wi-Fi station + SNTP + probe HTTP do dc-os-core.

use crate::system::{NetworkCommand, SystemEvent};
use anyhow::{anyhow, Result};
use core::convert::TryInto;
use embedded_svc::{
    io::Write as EmbeddedWrite,
    http::{client::Client as HttpClient, Method},
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
    pub api_health_url: String,
    pub timezone_offset_secs: i32,
}

const NET_TASK_STACK_SIZE: usize = 16 * 1024;
const API_PROBE_INTERVAL_SECS: u64 = 30;
const CLOCK_INTERVAL_SECS: u64 = 1;

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
        EspWifi::new(modem, sysloop.clone(), Some(nvs))?,
        sysloop,
    )?;

    let mut desired_wifi = cfg.enabled;
    let mut ssid = cfg.ssid;
    let mut password = cfg.password;
    let mut timezone_offset_secs = cfg.timezone_offset_secs;
    let mut connected = false;
    let mut sntp: Option<EspSntp<'static>> = None;
    let mut next_probe_in = 0_u64;
    let mut next_clock_in = 0_u64;

    log::info!(
        "Wi-Fi: config inicial enabled={} ssid='{}' api='{}'",
        desired_wifi,
        ssid,
        cfg.api_health_url
    );

    loop {
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                NetworkCommand::SetWifiEnabled(enabled) => {
                    desired_wifi = enabled;
                    log::info!("Wi-Fi: pedido da UI enabled={enabled}");
                }
                NetworkCommand::SetWifiCredentials {
                    ssid: new_ssid,
                    password: new_password,
                } => {
                    log::info!("Wi-Fi: nova rede selecionada '{}'", new_ssid);
                    ssid = new_ssid;
                    password = new_password;
                    desired_wifi = true;
                    if connected {
                        let _ = wifi.disconnect();
                        let _ = wifi.stop();
                        connected = false;
                        let _ = event_tx.send(SystemEvent::WifiChanged(false));
                        let _ = event_tx.send(SystemEvent::ApiHealthChanged(false));
                    }
                }
                NetworkCommand::SetTimezoneOffset(offset_secs) => {
                    timezone_offset_secs = offset_secs;
                    let _ = event_tx.send(SystemEvent::TimeChanged(current_hhmm(timezone_offset_secs)));
                    log::info!("SNTP: timezone offset={}s", offset_secs);
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

                    // Fetch Spotify top tracks once after connecting.
                    let spotify_token = option_env!("SPOTIFY_TOKEN").unwrap_or("");
                    if !spotify_token.is_empty() {
                        log::info!("Spotify: a pedir top tracks apos Wi-Fi");
                        crate::spotify::fetch_top_tracks(spotify_token, &event_tx);
                    }
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
        }

        if connected {
            if next_clock_in == 0 {
                let _ = event_tx.send(SystemEvent::TimeChanged(current_hhmm(timezone_offset_secs)));
                next_clock_in = CLOCK_INTERVAL_SECS;
            } else {
                next_clock_in = next_clock_in.saturating_sub(1);
            }

            if next_probe_in == 0 {
                let ok = probe_api(&cfg.api_health_url).unwrap_or_else(|e| {
                    log::warn!("API: health falhou: {e:?}");
                    false
                });
                let _ = event_tx.send(SystemEvent::ApiHealthChanged(ok));
                next_probe_in = API_PROBE_INTERVAL_SECS;
            } else {
                next_probe_in = next_probe_in.saturating_sub(1);
            }
        }

        let _keep_sntp_alive = sntp.as_ref();
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>, ssid: &str, password: &str) -> Result<()> {
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
    wifi.start()?;
    log::info!("Wi-Fi: started");
    wifi.connect()?;
    log::info!("Wi-Fi: connected");
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    log::info!("Wi-Fi: DHCP {ip_info:?}");
    Ok(())
}

fn probe_api(url: &str) -> Result<bool> {
    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);
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

fn post_music_command(health_url: &str, action: &str) -> Result<()> {
    let url = music_command_url(health_url);
    let payload = format!("{{\"action\":\"{}\"}}", action);
    let content_length = payload.len().to_string();
    let headers = [
        ("content-type", "application/json"),
        ("content-length", content_length.as_str()),
    ];

    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);
    let mut request = client.request(Method::Post, &url, &headers)?;
    request.write_all(payload.as_bytes())?;
    request.flush()?;
    let response = request.submit()?;
    log::info!("API: POST {} action={} status={}", url, action, response.status());
    Ok(())
}

fn music_command_url(health_url: &str) -> String {
    if let Some(base) = health_url.strip_suffix("/health") {
        format!("{base}/music/command")
    } else {
        format!("{}/music/command", health_url.trim_end_matches('/'))
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
