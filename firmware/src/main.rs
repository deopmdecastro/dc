//! DC OS — Firmware do DC Assistant (ESP32-S3 · ES3C28P)
//!
//! Bootstrap:
//!   1. link_patches + nvs                    (esp-idf-svc)
//!   2. Display ILI9341V + backlight PWM     (SPI2 + LEDC)
//!   3. Touch FT6336G task                    (I2C0 + INT)
//!   4. Audio I2S task                        (mic MEMS + spk)
//!   5. Network task                          (Wi-Fi STA + WS client)
//!   6. Slint event loop                      (renderiza AppWindow)

#![allow(clippy::needless_return)]

mod audio;
mod config;
mod display;
mod network;
mod pinout;
mod slint_platform;
mod system;
mod touch;

use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use std::{cell::RefCell, rc::Rc, sync::mpsc};
use system::{NetworkCommand, SystemEvent};

// Módulo gerado a partir de ui/main.slint.
slint::include_modules!();

fn main() -> Result<()> {
    // ---- Espressif runtime ----
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("DC OS boot — DC Assistant firmware v{}", env!("CARGO_PKG_VERSION"));

    let peripherals = Peripherals::take()?;
    let sysloop     = EspSystemEventLoop::take()?;
    let nvs         = EspDefaultNvsPartition::take()?;
    let config_store = Rc::new(RefCell::new(config::ConfigStore::new(nvs.clone())?));
    let app_config = config_store.borrow().load();

    // ---- Display + backlight ----
    // Só se passam os periféricos concretos de que o Display precisa
    // (SPI2 + timer0/channel0 do LEDC) — assim `peripherals.modem` fica
    // livre, com o lifetime normal ('static), para ir para a thread de
    // rede sem qualquer truque unsafe de duplicação da struct inteira.
    let display = display::Display::init(
        peripherals.spi2,
        peripherals.ledc.timer0,
        peripherals.ledc.channel0,
    )?;
    let modem = peripherals.modem;
    log::info!("Display OK — {}×{}", display.width, display.height);

    // ---- Slint window (software renderer) ----
    let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
    window.set_size(slint::PhysicalSize::new(pinout::DISPLAY_W, pinout::DISPLAY_H));

    let display_static = unsafe {
        core::mem::transmute::<display::Display<'_>, display::Display<'static>>(display)
    };
    slint_platform::init_platform(window.clone(), display_static);

    // ---- Tasks periféricas ----
    let touch_rx = touch::spawn_touch_task(peripherals.i2c0)?;
    let (network_cmd_tx, network_cmd_rx) = mpsc::channel();
    let (system_event_tx, system_event_rx) = mpsc::channel();
    audio::spawn_audio_task(
        |_lvl| { /* atualizar audio-level da UI via .invoke_from_event_loop */ },
        |_pcm| { /* enviar buffer PCM ao WebSocket */ },
    )?;
    network::spawn_network_task(
        modem, sysloop, nvs,
        network::NetConfig {
            enabled: app_config.wifi_enabled,
            ssid: app_config.wifi_ssid.clone(),
            password: app_config.wifi_password.clone(),
            api_health_url: app_config.api_health_url.clone(),
        },
        network_cmd_rx,
        system_event_tx.clone(),
    )?;

    // ---- Cria a AppWindow (definida em ui/main.slint) ----
    let app = AppWindow::new()
        .map_err(|e| anyhow::anyhow!("falha ao criar AppWindow Slint: {e:?}"))?;
    app.set_wifi_on(app_config.wifi_enabled);
    app.set_bluetooth_on(app_config.bluetooth_enabled);
    app.set_current_time("--:--".into());
    slint_platform::set_app_window(&app);
    // Boot animation → home. 2200ms deixa o fade-in + hold do logo completos
    // antes de descer para a tela de definicao de senha.
    let weak = app.as_weak();
    let has_passcode = app_config.passcode.is_some();
    slint::Timer::single_shot(std::time::Duration::from_millis(2200), move || {
        if let Some(w) = weak.upgrade() {
            w.set_current_screen(if has_passcode {
                Screen::Home
            } else {
                Screen::FirstBootPasscode
            });
        }
    });

    // Callbacks Rust ↔ Slint
    app.on_set_brightness(|level| {
        log::info!("UI: brilho ajustado para {:.0}%", level * 100.0);
        // TODO: platform.display.borrow_mut().set_brightness(level)
    });
    app.on_set_volume(|v| log::info!("UI: volume {:.0}%", v * 100.0));
    app.on_wake_word_triggered(|| log::info!("UI: wake-word acionada"));
    app.on_launch_app(|idx|      log::info!("UI: launch_app({})", idx));
    {
        let store = config_store.clone();
        app.on_passcode_created(move |pass| {
            if let Err(e) = store.borrow().save_passcode(pass.as_str()) {
                log::warn!("NVS: falha ao guardar PIN inicial: {e:?}");
            } else {
                log::info!("NVS: PIN inicial guardado");
            }
        });
    }
    {
        let store = config_store.clone();
        app.on_passcode_updated(move |pass| {
            if let Err(e) = store.borrow().save_passcode(pass.as_str()) {
                log::warn!("NVS: falha ao atualizar PIN: {e:?}");
            } else {
                log::info!("NVS: PIN atualizado");
            }
        });
    }
    {
        let store = config_store.clone();
        let tx = network_cmd_tx.clone();
        app.on_wifi_enabled_changed(move |enabled| {
            if let Err(e) = store.borrow().save_wifi_enabled(enabled) {
                log::warn!("NVS: falha ao guardar estado Wi-Fi: {e:?}");
            }
            let _ = tx.send(NetworkCommand::SetWifiEnabled(enabled));
        });
    }
    {
        let store = config_store.clone();
        let tx = network_cmd_tx.clone();
        let wifi_password = app_config.wifi_password.clone();
        app.on_wifi_network_selected(move |ssid| {
            if let Err(e) = store.borrow().save_wifi_credentials(ssid.as_str(), &wifi_password) {
                log::warn!("NVS: falha ao guardar rede Wi-Fi: {e:?}");
            }
            let _ = tx.send(NetworkCommand::SetWifiCredentials {
                ssid: ssid.to_string(),
                password: wifi_password.clone(),
            });
        });
    }
    {
        let store = config_store.clone();
        let event_tx = system_event_tx.clone();
        app.on_bluetooth_enabled_changed(move |enabled| {
            if let Err(e) = store.borrow().save_bluetooth_enabled(enabled) {
                log::warn!("NVS: falha ao guardar estado Bluetooth: {e:?}");
            }
            let _ = event_tx.send(SystemEvent::BluetoothChanged(enabled));
            log::info!(
                "Bluetooth: estado {} guardado; inicializacao BLE fica pendente de driver dedicado",
                if enabled { "ligado" } else { "desligado" }
            );
        });
    }
    app.on_set_rotation(|on| {
        log::info!("UI: rotacao automatica {}", if on { "ligada" } else { "desligada" });
        slint_platform::apply_display_rotation(on);
    });

    app.show()
        .map_err(|e| anyhow::anyhow!("falha ao exibir AppWindow Slint: {e:?}"))?;
    window.request_redraw();
    // ---- Event loop bloqueante ----
    slint_platform::run_event_loop(touch_rx, system_event_rx);
}
