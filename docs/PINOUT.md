# ES3C28P — Mapeamento de GPIO

Base: ESP32-S3 · 8 MB PSRAM · 16 MB Flash · 2.8" IPS 240×320 · FT6336G · MEMS mic.

## Display ILI9341V (SPI)

| Sinal | GPIO | Observação |
|-------|------|------------|
| SCLK  | 12   | LCD SPI clock |
| MOSI  | 13   | LCD SPI data |
| CS    | 46   | Chip select |
| DC    | 11   | Data/Command (reservado — confirmar em bring-up) |
| RST   | CHIP_PU | Compartilhado com reset do ESP32-S3 |
| BL    | 45   | PWM backlight (LEDC canal 0) |

## Touch FT6336G (I2C0)

| Sinal | GPIO |
|-------|------|
| SDA   | 16   |
| SCL   | 15   |
| INT   | 17   |
| RST   | 18   |

## Áudio I2S (mic MEMS + DAC/amp)

| Sinal | GPIO |
|-------|------|
| MCLK / BCLK | 4 |
| WS (LRCLK)  | 8 |
| DIN  (mic → S3) | 6 |
| DOUT (S3 → spk) | 7 |

## Diversos

| Recurso | GPIO |
|---------|------|
| RGB LED | 42 |
| Battery ADC | 9 |
| microSD CLK/CMD/D0..D3 | 38 / 40 / 39 / 41 / 48 / 47 |
| USB D− / D+ | 19 / 20 |
| UART0 RX / TX | 44 / 43 |
| Expansão GPIO livre | 2, 3, 14, 21 |
