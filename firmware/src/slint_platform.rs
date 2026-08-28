//! Backend Slint → framebuffer software renderer sobre o SPI do ILI9341V.
//! Encaminha `flush()` linha-a-linha para o display.

use crate::display::{Display, DisplayRotation};
use crate::pinout::DISPLAY_W;
use crate::touch::TouchEvent;
use slint::LogicalPosition;
use slint::platform::{
    software_renderer::{LineBufferProvider, MinimalSoftwareWindow, Rgb565Pixel},
    Platform, PointerEventButton, WindowEvent,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

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

/// Chamado a partir do callback `set-rotation` do Slint. Executa dentro da
/// thread do event loop, pelo que pode aceder ao `DISPLAY` thread-local em
/// seguranca. Se `autorotate` estiver `true`, mantem landscape (0deg); caso
/// contrario roda 180deg. Um dia isto pode ler o acelerometro.
pub fn apply_display_rotation(autorotate: bool) {
    let rot = if autorotate {
        DisplayRotation::Landscape
    } else {
        DisplayRotation::Landscape180
    };
    DISPLAY.with(|d| {
        if let Some(display) = d.borrow_mut().as_mut() {
            if let Err(e) = display.set_rotation(rot) {
                log::warn!("Display: falha ao aplicar rotacao: {e:?}");
            }
        }
    });
    WINDOW.with(|w| {
        if let Some(window) = w.borrow().as_ref() {
            window.request_redraw();
        }
    });
}

/// Adaptador que traduz cada linha renderizada pelo software renderer do
/// Slint (buffer RGB565 em RAM) para um `write_line_rgb565` no SPI.
struct SpiLineBuffer<'a> {
    display: &'a mut Display<'static>,
}

// Buffers de linha do renderer — ficam fora da stack porque `process_line`
// é chamado de dentro da recursão do software renderer do Slint, no ponto
// em que a stack da task `main` está mais ocupada. Dois arrays de 640 bytes
// alocados localmente nessa profundidade foram identificados como
// contribuinte de um stack overflow observado em testes em hardware.
thread_local! {
    static LINE_PIXELS: RefCell<[Rgb565Pixel; DISPLAY_W as usize]> =
        RefCell::new([Rgb565Pixel(0); DISPLAY_W as usize]);
    static LINE_BYTES: RefCell<[u8; DISPLAY_W as usize * 2]> =
        RefCell::new([0u8; DISPLAY_W as usize * 2]);
}

impl<'a> LineBufferProvider for SpiLineBuffer<'a> {
    type TargetPixel = Rgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        LINE_PIXELS.with(|px_cell| {
            let mut px_buf = px_cell.borrow_mut();
            let slice = &mut px_buf[..range.len()];
            render_fn(slice);

            // Converte para bytes big-endian, como o ILI9341V espera.
            LINE_BYTES.with(|bytes_cell| {
                let mut bytes = bytes_cell.borrow_mut();
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
            });
        });
    }
}

/// Roda o loop de eventos Slint (bloqueante).
pub fn run_event_loop(touch_rx: Receiver<TouchEvent>) -> ! {
    // A janela raiz (`AppWindow`) é criada e mostrada pelo main.rs.
    let mut frame_count: u32 = 0;
    let mut idle_ticks: u32 = 0;

    loop {
        dispatch_pending_touch_events(&touch_rx);
        slint::platform::update_timers_and_animations();

        WINDOW.with(|w| {
            if let Some(window) = w.borrow().as_ref() {
                if window.draw_if_needed(|renderer| {
                    DISPLAY.with(|d| {
                        if let Some(display) = d.borrow_mut().as_mut() {
                            renderer.render_by_line(SpiLineBuffer { display });
                        }
                    });
                }) {
                    frame_count = frame_count.wrapping_add(1);
                    idle_ticks = 0;

                    if frame_count <= 3 || frame_count % 60 == 0 {
                        log::info!("Slint: frame {} enviado ao display", frame_count);
                    }
                } else {
                    idle_ticks = idle_ticks.wrapping_add(1);
                    if idle_ticks == 1000 {
                        log::warn!("Slint: 1000 ciclos sem redraw pendente");
                    }
                }
            }
        });

        unsafe { esp_idf_sys::vTaskDelay(1); }
    }
}

fn dispatch_pending_touch_events(touch_rx: &Receiver<TouchEvent>) {
    while let Ok(event) = touch_rx.try_recv() {
        WINDOW.with(|w| {
            if let Some(window) = w.borrow().as_ref() {
                let window_event = match event {
                    TouchEvent::Down(point) => WindowEvent::PointerPressed {
                        position: point_to_position(point),
                        button: PointerEventButton::Left,
                    },
                    TouchEvent::Move(point) => WindowEvent::PointerMoved {
                        position: point_to_position(point),
                    },
                    TouchEvent::Up(point) => WindowEvent::PointerReleased {
                        position: point_to_position(point),
                        button: PointerEventButton::Left,
                    },
                    TouchEvent::SwipeX { .. } | TouchEvent::SwipeY { .. } => return,
                };

                if let Err(e) = window.try_dispatch_event(window_event) {
                    log::warn!("Slint: falha ao entregar touch: {e:?}");
                }
                window.request_redraw();
            }
        });
    }
}

fn point_to_position(point: crate::touch::Point) -> LogicalPosition {
    LogicalPosition::new(point.x as f32, point.y as f32)
}
