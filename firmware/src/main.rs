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
mod display;
mod network;
mod pinout;
mod slint_platform;
mod touch;

use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};

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
    audio::spawn_audio_task(
        |_lvl| { /* atualizar audio-level da UI via .invoke_from_event_loop */ },
        |_pcm| { /* enviar buffer PCM ao WebSocket */ },
    )?;
    network::spawn_network_task(
        modem, sysloop, nvs,
        network::NetConfig {
            ssid:      option_env!("DC_WIFI_SSID").unwrap_or("DC_Network"),
            password:  option_env!("DC_WIFI_PASS").unwrap_or(""),
            server_ws: option_env!("DC_CORE_WS").unwrap_or("ws://192.168.1.50:8080/ws"),
        },
    )?;

    // ---- Cria a AppWindow (definida em ui/main.slint) ----
    let app = AppWindow::new()
        .map_err(|e| anyhow::anyhow!("falha ao criar AppWindow Slint: {e:?}"))?;
    // Boot animation → home. 2200ms deixa o fade-in + hold do logo completos
    // antes de descer para a tela de definicao de senha.
    let weak = app.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(2200), move || {
        if let Some(w) = weak.upgrade() {
            w.set_current_screen(Screen::FirstBootPasscode);
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
    app.on_set_rotation(|on| {
        log::info!("UI: rotacao automatica {}", if on { "ligada" } else { "desligada" });
        slint_platform::apply_display_rotation(on);
    });

    app.show()
        .map_err(|e| anyhow::anyhow!("falha ao exibir AppWindow Slint: {e:?}"))?;
    window.request_redraw();
    // ---- Event loop bloqueante ----
    slint_platform::run_event_loop(touch_rx);
}
