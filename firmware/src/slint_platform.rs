//! Backend Slint → framebuffer software renderer sobre o SPI do ILI9341V.
//! Encaminha `flush()` linha-a-linha para o display.

use crate::display::Display;
use crate::pinout::DISPLAY_W;
use slint::platform::{
    software_renderer::{LineBufferProvider, MinimalSoftwareWindow, Rgb565Pixel},
    Platform,
};
use std::cell::RefCell;
use std::rc::Rc;

// O display e a janela ficam em thread-locals porque o `Platform` registado
// no Slint (via `set_platform`) só é acedido internamente pela lib — mas o
// nosso loop de eventos, aqui no mesmo módulo, também precisa de lhes aceder
// para desenhar e fazer flush a cada frame.
thread_local! {
    static WINDOW:  RefCell<Option<Rc<MinimalSoftwareWindow>>> = RefCell::new(None);
    static DISPLAY: RefCell<Option<Display<'static>>>          = RefCell::new(None);
}

pub struct EspSlintPlatform {
    pub window: Rc<MinimalSoftwareWindow>,
}

impl Platform for EspSlintPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> core::time::Duration {
        core::time::Duration::from_millis(
            unsafe { esp_idf_sys::esp_timer_get_time() as u64 } / 1000,
        )
    }
}

/// Regista a plataforma Slint e guarda `window`/`display` (em thread-locals
/// deste módulo) para uso no `run_event_loop`. O `display` não faz parte da
/// struct `EspSlintPlatform` porque o trait `Platform` não precisa dele — só
/// o loop de eventos, aqui ao lado, é que faz o flush dos pixels.
pub fn init_platform(window: Rc<MinimalSoftwareWindow>, display: Display<'static>) {
    DISPLAY.with(|d| *d.borrow_mut() = Some(display));
    WINDOW.with(|w| *w.borrow_mut() = Some(window.clone()));

    slint::platform::set_platform(Box::new(EspSlintPlatform { window }))
        .expect("Slint platform já registrada");
}

/// Adaptador que traduz cada linha renderizada pelo software renderer do
/// Slint (buffer RGB565 em RAM) para um `write_line_rgb565` no SPI.
struct SpiLineBuffer<'a> {
    display: &'a mut Display<'static>,
}

impl<'a> LineBufferProvider for SpiLineBuffer<'a> {
    type TargetPixel = Rgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        // Buffer de linha no stack: largura máxima do painel × 2 bytes/pixel.
        let mut px_buf = [Rgb565Pixel(0); DISPLAY_W as usize];
        let slice = &mut px_buf[..range.len()];
        render_fn(slice);

        // Converte para bytes big-endian, como o ILI9341V espera.
        let mut bytes = [0u8; (DISPLAY_W as usize) * 2];
        for (i, px) in slice.iter().enumerate() {
            let be = px.0.to_be_bytes();
            bytes[i * 2] = be[0];
            bytes[i * 2 + 1] = be[1];
        }

        if let Err(e) = self.display.write_line_rgb565(
            line as u16,
            range.start as u16,
            &bytes[..slice.len() * 2],
        ) {
            log::warn!("Display: falha ao enviar linha {line}: {e:?}");
        }
    }
}

/// Roda o loop de eventos Slint (bloqueante).
pub fn run_event_loop() -> ! {
    // A janela raiz (`AppWindow`) é criada e mostrada pelo main.rs.
    loop {
        slint::platform::update_timers_and_animations();

        WINDOW.with(|w| {
            if let Some(window) = w.borrow().as_ref() {
                window.draw_if_needed(|renderer| {
                    DISPLAY.with(|d| {
                        if let Some(display) = d.borrow_mut().as_mut() {
                            renderer.render_by_line(SpiLineBuffer { display });
                        }
                    });
                });
            }
        });

        unsafe { esp_idf_sys::vTaskDelay(1); }
    }
}
