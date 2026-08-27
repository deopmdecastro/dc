//! Backend Slint → framebuffer software renderer sobre o SPI do ILI9341V.
//! Encaminha `flush()` linha-a-linha para o display.

use crate::display::Display;
use slint::platform::{software_renderer::MinimalSoftwareWindow, Platform};
use std::rc::Rc;

pub struct EspSlintPlatform {
    pub window: Rc<MinimalSoftwareWindow>,
    pub display: std::cell::RefCell<Display<'static>>,
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

/// Roda o loop de eventos Slint (bloqueante).
pub fn run_event_loop(platform: EspSlintPlatform) -> ! {
    slint::platform::set_platform(Box::new(platform))
        .expect("Slint platform já registrada");

    // A janela raiz (`AppWindow`) é criada e mostrada pelo main.rs.
    loop {
        slint::platform::update_timers_and_animations();
        // TODO: renderizar `window.draw_if_needed(|renderer| {...})`
        // e transferir a região suja para o ILI9341V via SPI DMA.
        unsafe { esp_idf_sys::vTaskDelay(1); }
    }
}
