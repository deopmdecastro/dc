// pinout.rs — Mapeamento centralizado dos GPIOs do módulo ES3C28P.
// Fonte: docs/PINOUT.md (extraído de ES3C28P_ES2N28P_Specification_V1.0.pdf).

pub mod pins {
    // Display ILI9341V (SPI2 / HSPI)
    pub const DISP_SCLK: u8 = 12;
    pub const DISP_MOSI: u8 = 11;
    pub const DISP_CS: u8 = 10;
    pub const DISP_DC: u8 = 46;
    pub const DISP_BL: u8 = 45; // PWM backlight (LEDC canal 0)

    // Touch FT6336G (I2C0)
    pub const TOUCH_SDA: u8 = 16;
    pub const TOUCH_SCL: u8 = 15;
    pub const TOUCH_INT: u8 = 17;
    pub const TOUCH_RST: u8 = 18;

    // Áudio I2S
    pub const AMP_EN: u8 = 1;   // Amplificador enable (LOW = ligado)
    pub const I2S_MCLK: u8 = 4; // Master clock do codec
    pub const I2S_BCLK: u8 = 5; // Bit clock
    pub const I2S_WS: u8 = 7;   // Word select (L/R channel)
    pub const I2S_DOUT: u8 = 8; // Dados de saída (ESP32 → codec)
    pub const I2S_DIN: u8 = 6;  // Dados de entrada (codec → ESP32)

    // I²C (codec, touch e expansão compartilham)
    pub const I2C_SDA: u8 = 16;
    pub const I2C_SCL: u8 = 15;
}

/// Resolução da UI (Slint) — display em orientação horizontal.
pub const DISPLAY_W: u32 = 320;
pub const DISPLAY_H: u32 = 240;
