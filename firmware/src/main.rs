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
mod spotify;
mod system;
mod touch;

use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use std::{cell::RefCell, rc::Rc, sync::mpsc};
use system::NetworkCommand;

// Módulo gerado a partir de ui/main.slint.
slint::include_modules!();

fn main() -> Result<()> {
    // ---- Espressif runtime ----
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!(
        "DC OS boot — DC Assistant firmware v{}",
        env!("CARGO_PKG_VERSION")
    );

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
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
    window.set_size(slint::PhysicalSize::new(
        pinout::DISPLAY_W,
        pinout::DISPLAY_H,
    ));

    let display_static =
        unsafe { core::mem::transmute::<display::Display<'_>, display::Display<'static>>(display) };
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
        modem,
        sysloop,
        nvs,
        network::NetConfig {
            enabled: app_config.wifi_enabled,
            ssid: app_config.wifi_ssid.clone(),
            password: app_config.wifi_password.clone(),
            bluetooth_enabled: app_config.bluetooth_enabled,
            api_health_url: app_config.api_health_url.clone(),
            region_index: app_config.region_index,
            timezone_offset_secs: timezone_offset_secs(app_config.region_index),
        },
        network_cmd_rx,
        system_event_tx.clone(),
    )?;

    // ---- Cria a AppWindow (definida em ui/main.slint) ----
    let app =
        AppWindow::new().map_err(|e| anyhow::anyhow!("falha ao criar AppWindow Slint: {e:?}"))?;
    app.set_wifi_on(app_config.wifi_enabled);
    app.set_bluetooth_on(app_config.bluetooth_enabled);
    app.set_volume(app_config.volume as f32 / 100.0);
    app.set_brightness(app_config.brightness as f32 / 100.0);
    app.set_region_index(app_config.region_index as i32);
    app.set_language_index(app_config.language_index as i32);
    app.set_alarm_enabled(app_config.alarm_enabled);
    app.set_alarm_hour(app_config.alarm_hour as i32);
    app.set_alarm_minute(app_config.alarm_minute as i32);
    app.set_alarm_day_mode(app_config.alarm_day_mode as i32);
    app.set_alarm_tone(app_config.alarm_tone as i32);
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
    {
        let store = config_store.clone();
        app.on_set_brightness(move |level| {
            let level = level.clamp(0.0, 1.0);
            log::info!("UI: brilho ajustado para {:.0}%", level * 100.0);
            slint_platform::set_display_brightness(level);
            if let Err(e) = store.borrow().save_brightness(percent_u8(level)) {
                log::warn!("NVS: falha ao guardar brilho: {e:?}");
            }
        });
    }
    {
        let store = config_store.clone();
        app.on_set_volume(move |v| {
            let v = v.clamp(0.0, 1.0);
            log::info!("UI: volume {:.0}%", v * 100.0);
            if let Err(e) = store.borrow().save_volume(percent_u8(v)) {
                log::warn!("NVS: falha ao guardar volume: {e:?}");
            }
        });
    }
    app.on_wake_word_triggered(|| log::info!("UI: wake-word acionada"));
    app.on_launch_app(|idx| log::info!("UI: launch_app({})", idx));
    {
        let tx = network_cmd_tx.clone();
        app.on_music_command(move |action| {
            let _ = tx.send(NetworkCommand::MusicCommand(action.to_string()));
        });
    }
    {
        let store = config_store.clone();
        let tx = network_cmd_tx.clone();
        app.on_locale_changed(move |region, language| {
            let region = region.clamp(0, 4) as u8;
            let language = language.clamp(0, 4) as u8;
            if let Err(e) = store.borrow().save_locale(region, language) {
                log::warn!("NVS: falha ao guardar idioma/regiao: {e:?}");
            }
            let _ = tx.send(NetworkCommand::SetLocale {
                region_index: region,
                timezone_offset_secs: timezone_offset_secs(region),
            });
        });
    }
    {
        let store = config_store.clone();
        app.on_alarm_changed(move |enabled, hour, minute, day_mode, tone| {
            let hour = hour.clamp(0, 23) as u8;
            let minute = minute.clamp(0, 59) as u8;
            let day_mode = day_mode.clamp(0, 3) as u8;
            let tone = tone.clamp(0, 2) as u8;
            if let Err(e) = store
                .borrow()
                .save_alarm(enabled, hour, minute, day_mode, tone)
            {
                log::warn!("NVS: falha ao guardar alarme: {e:?}");
            } else {
                log::info!(
                    "Alarme: {} {:02}:{:02} dias={} toque={}",
                    if enabled { "ligado" } else { "desligado" },
                    hour,
                    minute,
                    day_mode,
                    tone
                );
            }
        });
    }
    app.on_test_alarm_tone(|tone| {
        log::info!("Alarme: testar toque {}", tone.clamp(0, 2));
        // TODO: quando audio.rs deixar de ser stub, encaminhar esta acao para I2S TX.
    });
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
        let tx = network_cmd_tx.clone();
        app.on_scan_wifi_networks(move || {
            let _ = tx.send(NetworkCommand::ScanWifi);
        });
    }
    {
        // A linguagem .slint nao tem funcao de substring/remocao de
        // caracteres em strings, por isso o backspace da senha Wi-Fi e
        // resolvido aqui: le o valor atual, remove o ultimo caractere
        // (respeitando fronteiras UTF-8) e escreve de volta na UI.
        let app_weak = app.as_weak();
        app.on_wifi_password_backspace(move || {
            if let Some(app) = app_weak.upgrade() {
                let current = app.get_wifi_password();
                let mut chars: Vec<char> = current.chars().collect();
                chars.pop();
                app.set_wifi_password(chars.into_iter().collect::<String>().into());
            }
        });
    }
    {
        let store = config_store.clone();
        let tx = network_cmd_tx.clone();
        app.on_wifi_network_selected(move |ssid, password| {
            if let Err(e) = store
                .borrow()
                .save_wifi_credentials(ssid.as_str(), password.as_str())
            {
                log::warn!("NVS: falha ao guardar rede Wi-Fi: {e:?}");
            }
            let _ = tx.send(NetworkCommand::SetWifiCredentials {
                ssid: ssid.to_string(),
                password: password.to_string(),
            });
        });
    }
    {
        let store = config_store.clone();
        let tx = network_cmd_tx.clone();
        app.on_bluetooth_enabled_changed(move |enabled| {
            if let Err(e) = store.borrow().save_bluetooth_enabled(enabled) {
                log::warn!("NVS: falha ao guardar estado Bluetooth: {e:?}");
            }
            let _ = tx.send(NetworkCommand::SetBluetoothEnabled(enabled));
        });
    }
    app.on_set_rotation(|on| {
        log::info!("UI: orientacao {}", if on { "normal" } else { "invertida" });
        slint_platform::apply_display_rotation(on);
    });

    app.show()
        .map_err(|e| anyhow::anyhow!("falha ao exibir AppWindow Slint: {e:?}"))?;
    window.request_redraw();
    // ---- Event loop bloqueante ----
    slint_platform::run_event_loop(touch_rx, system_event_rx);
}

fn timezone_offset_secs(region_index: u8) -> i32 {
    match region_index {
        0 => -3 * 3600, // Brasilia
        1 => 3600,      // Portugal continental em horario de verao
        2 => 3600,      // Angola
        3 => 2 * 3600,  // Mocambique
        4 => -4 * 3600, // EUA Eastern em horario de verao
        _ => 0,
    }
}

fn percent_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 100.0).round() as u8
}
