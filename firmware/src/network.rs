//! Task de rede: Wi-Fi station + WebSocket cliente até o `dc-os-core`.

use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi,
};
use esp_idf_hal::modem::Modem;

pub struct NetConfig<'a> {
    pub ssid:      &'a str,
    pub password:  &'a str,
    pub server_ws: &'a str, // ex.: "ws://192.168.1.50:8080/ws"
}

/// Conecta em modo station e sobe cliente WebSocket persistente.
pub fn spawn_network_task(
    modem: Modem,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
    cfg: NetConfig<'_>,
) -> Result<()> {
    let mut wifi = EspWifi::new(modem, sysloop, Some(nvs))?;
    log::info!("Wi-Fi: conectando em SSID='{}'", cfg.ssid);

    // TODO:
    //   1. wifi.set_configuration(&Configuration::Client(ClientConfiguration {
    //         ssid: cfg.ssid.into(), password: cfg.password.into(),
    //         auth_method: AuthMethod::WPA2Personal, ..default }))?;
    //   2. wifi.start()?; wifi.connect()?;
    //   3. Cliente WS (embassy-net + ws crate) → cfg.server_ws
    //   4. Multiplex: audio in/out + comandos JSON (voice/state/music).
    let _ = wifi.start();
    log::info!("WebSocket alvo: {}", cfg.server_ws);
    Ok(())
}
