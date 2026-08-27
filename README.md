# DC OS — DC Assistant

Firmware e sistema operacional para o **DC Assistant**, um assistente virtual pessoal
(estilo Alexa) construído sobre o módulo hardware **ES3C28P**
(ESP32-S3 · 8 MB PSRAM · 16 MB Flash · IPS 2.8" ILI9341V 240×320 · Touch FT6336G · MEMS mic + I2S DAC).

## Stack

| Camada | Tecnologia |
|--------|------------|
| Firmware | **Rust** (`esp-idf-hal`, `esp-idf-svc`) + PlatformIO |
| GUI | **Slint** (horizontal 320×240) |
| Backend local | **Docker Compose** (Rust · `axum` + Whisper.cpp + Mopidy/Librespot) |
| Áudio | I2S (mic MEMS in / DAC out) → streaming WebSocket → STT local |

## Estrutura do repositório

```
dc/
├── firmware/                # Firmware Rust para ESP32-S3
│   ├── platformio.ini
│   ├── Cargo.toml
│   ├── src/                 # main.rs + tasks (display, touch, audio, net)
│   └── ui/                  # Módulos Slint (.slint)
├── backend/                 # Servidor local (Docker Compose)
│   ├── docker-compose.yml
│   ├── dc-os-core/          # Hub em Rust (axum + tokio + WS)
│   ├── stt-whisper/         # Container Whisper.cpp
│   └── music-mopidy/        # Player + Librespot (Spotify Connect)
└── docs/
    └── PINOUT.md            # Mapeamento de GPIO do ES3C28P
```

## Quick start

### Backend
```bash
cd backend
docker compose up -d
# API HTTP em http://localhost:8080
# WebSocket em  ws://localhost:8080/ws
```

### Firmware
```bash
cd firmware
cargo build --release --locked
cargo espflash flash --release --locked --chip esp32s3 --flash-mode dio --flash-size 16mb --partition-table partitions.csv --bootloader ..\..\t\xtensa-esp32s3-espidf\release\bootloader.bin --port COM6 --monitor --monitor-baud 115200 --skip-update-check
```

O firmware usa Rust/ESP-IDF com a toolchain `esp` e `cargo-espflash`. A
configuração do PlatformIO permanece no projeto para o ambiente ESP-IDF, mas o
build/upload reprodutível do binário Rust é feito pelo Cargo com `Cargo.lock`.

Consulte [`docs/PINOUT.md`](docs/PINOUT.md) para o mapeamento de GPIO.
