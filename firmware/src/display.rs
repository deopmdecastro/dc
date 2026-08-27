//! Driver do display ILI9341V (SPI) e integração com o backend software
//! renderer do Slint. Mantém um framebuffer em PSRAM.

use crate::pinout::{pins, DISPLAY_H, DISPLAY_W};
use anyhow::Result;
use esp_idf_hal::{
    gpio::{AnyIOPin, Output, PinDriver},
    ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, Resolution},
    peripherals::Peripherals,
    spi::{config::Config as SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig},
    units::*,
};

pub struct Display<'d> {
    _spi: SpiDeviceDriver<'d, SpiDriver<'d>>,
    _dc:  PinDriver<'d, Output>,
    pub backlight: LedcDriver<'d>,
    pub width:  u32,
    pub height: u32,
}

impl<'d> Display<'d> {
    /// Inicializa o SPI, DC, BL (PWM) e envia a sequência de bring-up do
    /// ILI9341V para orientação **horizontal** (Memory Access Control = 0x28).
    pub fn init(p: Peripherals) -> Result<Self> {
        // --- SPI2 ---
        let sclk = unsafe { AnyIOPin::steal(pins::DISP_SCLK as _) };
        let mosi = unsafe { AnyIOPin::steal(pins::DISP_MOSI as _) };
        let cs   = unsafe { AnyIOPin::steal(pins::DISP_CS   as _) };
        let dc   = unsafe { AnyIOPin::steal(pins::DISP_DC   as _) };

        let spi_drv = SpiDriver::new(
            p.spi2, sclk, mosi, Option::<AnyIOPin>::None, &SpiDriverConfig::new(),
        )?;
        let spi = SpiDeviceDriver::new(
            spi_drv, Some(cs),
            &SpiConfig::new().baudrate(40.MHz().into()),
        )?;

        let dc = PinDriver::output(dc)?;

        // --- Backlight PWM (LEDC canal 0) ---
        let timer = LedcTimerDriver::new(
            p.ledc.timer0,
            &TimerConfig::default()
                .frequency(5.kHz().into())
                .resolution(Resolution::Bits10),
        )?;
        let mut backlight = LedcDriver::new(p.ledc.channel0, timer,
            unsafe { AnyIOPin::steal(pins::DISP_BL as _) })?;
        backlight.set_duty(backlight.get_max_duty() * 60 / 100)?; // 60% brilho inicial

        let mut disp = Self {
            _spi: spi,
            _dc:  dc,
            backlight,
            width:  DISPLAY_W,
            height: DISPLAY_H,
        };
        disp.bring_up_ili9341v()?;
        Ok(disp)
    }

    /// Sequência de comandos do ILI9341V para 320×240 horizontal.
    /// (Stubs — a implementação completa usa os helpers privados
    /// `cmd`/`data` sobre o SpiDeviceDriver.)
    fn bring_up_ili9341v(&mut self) -> Result<()> {
        log::info!("Display: ILI9341V bring-up ({}×{})", self.width, self.height);
        // TODO: SWRESET (0x01) → delay 120 ms
        //       SLPOUT  (0x11) → delay 120 ms
        //       MADCTL  (0x36) ← 0x28  (rotação horizontal, BGR)
        //       COLMOD  (0x3A) ← 0x55  (16 bpp)
        //       DISPON  (0x29)
        Ok(())
    }

    /// Ajusta brilho 0.0..1.0 via PWM no GPIO45.
    pub fn set_brightness(&mut self, level: f32) -> Result<()> {
        let level = level.clamp(0.0, 1.0);
        let duty = (self.backlight.get_max_duty() as f32 * level) as u32;
        self.backlight.set_duty(duty)?;
        Ok(())
    }
}
