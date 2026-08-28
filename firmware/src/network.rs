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

// Stack dedicada à task de rede, fora da stack da task `main`.
//
// Causa raiz confirmada em hardware: esta função chamava-se
// `spawn_network_task` mas nunca chegava a criar uma task/thread — o
// bring-up do EspWifi (init + start()) corria de forma síncrona dentro
// da própria `main`, por cima do que o Display::init + Slint já tinham
// consumido da stack de 32 KB (ver sdkconfig.defaults). Como a função
// devolvia logo a seguir ao `wifi.start()` (o SSID/password nunca
// chegavam a ser configurados), o `wifi`/`nvs`/`sysloop` locais saíam
// de scope e o teardown do Wi-Fi — rotinas C do ESP-IDF pesadas em
// stack — acontecia ainda dentro da `main`, estourando os 32 KB logo a
// seguir ao "wifi:lmac stop hw txq" (`stack overflow in task main`).
const NET_TASK_STACK_SIZE: usize = 12 * 1024;

/// Cria uma task/thread própria (stack dedicada) que faz o bring-up do
/// Wi-Fi station e, no futuro, sobe o cliente WebSocket persistente.
/// `cfg` tem de ser `'static` porque a closure é movida para outra
/// thread (`option_env!` já dá `&'static str`, por isso os chamadores
/// não precisam de mudar nada).
pub fn spawn_network_task(
    modem: Modem,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
    cfg: NetConfig<'static>,
) -> Result<()> {
    std::thread::Builder::new()
        .name("dc-net".into())
        .stack_size(NET_TASK_STACK_SIZE)
        .spawn(move || {
            if let Err(e) = run_network(modem, sysloop, nvs, cfg) {
                log::error!("network: falhou a inicializar: {e:?}");
            }
        })?;
    Ok(())
}

/// Corpo da task de rede — corre na thread própria criada acima, nunca
/// na `main`.
fn run_network(
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

    // Mantém `wifi`/`nvs`/`sysloop` vivos (e a task a existir) em vez de
    // devolver logo a seguir ao start() — devolver aqui era o que
    // disparava o teardown imediato do Wi-Fi visto nos logs
    // ("EspWifi dropped" / "NvsDefault dropped" a seguir ao start()).
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
