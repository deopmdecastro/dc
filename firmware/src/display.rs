//! Driver do display ILI9341V (SPI) e integração com o backend software
//! renderer do Slint. Mantém um framebuffer em PSRAM.

use crate::pinout::{pins, DISPLAY_H, DISPLAY_W};
use anyhow::Result;
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{AnyIOPin, Output, PinDriver},
    ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, Resolution},
    peripherals::Peripherals,
    spi::{config::Config as SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig},
    units::*,
};

// Comandos ILI9341V usados no bring-up e no envio de pixels.
const CMD_SWRESET: u8 = 0x01;
const CMD_SLPOUT:  u8 = 0x11;
const CMD_DISPON:  u8 = 0x29;
const CMD_CASET:   u8 = 0x2A;
const CMD_PASET:   u8 = 0x2B;
const CMD_RAMWR:   u8 = 0x2C;
const CMD_MADCTL:  u8 = 0x36;
const CMD_COLMOD:  u8 = 0x3A;

pub struct Display<'d> {
    spi:  SpiDeviceDriver<'d, SpiDriver<'d>>,
    dc:   PinDriver<'d, Output>,
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
            spi,
            dc,
            backlight,
            width:  DISPLAY_W,
            height: DISPLAY_H,
        };
        disp.bring_up_ili9341v()?;
        Ok(disp)
    }

    /// Envia um byte de comando (linha DC em nível baixo).
    fn cmd(&mut self, cmd: u8) -> Result<()> {
        self.dc.set_low()?;
        self.spi.write(&[cmd])?;
        Ok(())
    }

    /// Envia bytes de dados associados ao último comando (DC em nível alto).
    fn data(&mut self, data: &[u8]) -> Result<()> {
        self.dc.set_high()?;
        self.spi.write(data)?;
        Ok(())
    }

    /// Sequência de comandos do ILI9341V para 320×240 horizontal (MADCTL 0x28,
    /// BGR, sem mirror) e 16 bpp (RGB565). Sem esta sequência o painel fica
    /// com o backlight ligado mas sem imagem — ecrã aparenta "luz branca".
    fn bring_up_ili9341v(&mut self) -> Result<()> {
        log::info!("Display: ILI9341V bring-up ({}×{})", self.width, self.height);

        self.cmd(CMD_SWRESET)?;
        FreeRtos::delay_ms(120);

        self.cmd(CMD_SLPOUT)?;
        FreeRtos::delay_ms(120);

        self.cmd(CMD_MADCTL)?;
        self.data(&[0x28])?; // rotação horizontal, BGR

        self.cmd(CMD_COLMOD)?;
        self.data(&[0x55])?; // 16 bpp (RGB565)
        FreeRtos::delay_ms(10);

        self.cmd(CMD_DISPON)?;
        FreeRtos::delay_ms(20);

        log::info!("Display: ILI9341V pronto");
        Ok(())
    }

    /// Define a janela de escrita (endereço de coluna/linha) e prepara o
    /// controlador para receber dados de pixel via RAMWR.
    fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<()> {
        self.cmd(CMD_CASET)?;
        self.data(&[(x0 >> 8) as u8, x0 as u8, (x1 >> 8) as u8, x1 as u8])?;

        self.cmd(CMD_PASET)?;
        self.data(&[(y0 >> 8) as u8, y0 as u8, (y1 >> 8) as u8, y1 as u8])?;

        self.cmd(CMD_RAMWR)?;
        Ok(())
    }

    /// Escreve uma linha de pixels RGB565 (big-endian, como o ILI9341V
    /// espera) na região [x0, x1) da linha `y`. Usado pelo backend do Slint
    /// (`slint_platform.rs`) para fazer flush do framebuffer.
    pub fn write_line_rgb565(&mut self, y: u16, x0: u16, pixels_be: &[u8]) -> Result<()> {
        let x1 = x0 + (pixels_be.len() / 2) as u16;
        self.set_window(x0, y, x1.saturating_sub(1), y)?;
        self.data(pixels_be)
    }

    /// Ajusta brilho 0.0..1.0 via PWM no GPIO45.
    pub fn set_brightness(&mut self, level: f32) -> Result<()> {
        let level = level.clamp(0.0, 1.0);
        let duty = (self.backlight.get_max_duty() as f32 * level) as u32;
        self.backlight.set_duty(duty)?;
        Ok(())
    }
}
