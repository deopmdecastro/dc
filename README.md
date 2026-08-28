# DC OS - DC Assistant

Firmware e backend local para o **DC Assistant**, um assistente pessoal em
hardware **ES3C28P**:

- ESP32-S3
- 8 MB PSRAM
- 16 MB flash
- Display IPS 2.8" ILI9341V 240x320, usado em landscape 320x240
- Touch FT6336G por I2C
- MEMS mic + DAC/amp I2S

## Estado Atual

- Firmware Rust/ESP-IDF compila em `release`.
- Flash usa tabela custom de 16 MB com app `factory` de 4 MB.
- Display ILI9341V inicializa e recebe frames Slint.
- Touch FT6336G inicializa em I2C `0x38` e envia eventos para a UI.
- UI Slint no estilo DC OS escuro, com launcher sem barra lateral, Definições
  por categorias, Alarme configurável, PIN persistente e região/idioma.
- A tela inicial usa gesto de deslizar para a esquerda para abrir o launcher
  de aplicacoes (o botao "Abrir Apps" foi removido).
- Player de musica integra a Spotify Web API e mostra as top tracks reais.
- Wi-Fi liga de verdade, sincroniza hora por SNTP e consome a API real por HTTP.
- Player envia `play`, `pause`, `next` e `prev` para o backend/Mopidy.
- Brilho ajusta o PWM do backlight; volume fica persistido para integração de
  áudio.
- Bluetooth/BLE está desligado no `sdkconfig.defaults` porque o firmware atual
  ainda não usa BLE e isso evita incompatibilidade com ESP-IDF 5.2.3.

## Estrutura

```text
dc/
├── firmware/                 # Firmware Rust para ESP32-S3
│   ├── .cargo/config.toml     # target Xtensa e target-dir ../../t
│   ├── Cargo.toml
│   ├── partitions.csv         # tabela 16 MB, factory app 4 MB
│   ├── sdkconfig.defaults     # flash, PSRAM, stacks, Wi-Fi
│   ├── src/                   # display, touch, audio, network, Slint platform
│   └── ui/                    # telas Slint 320x240
├── backend/                   # Docker Compose: core, Whisper, Mopidy
└── docs/                      # pinout e guia detalhado de build/flash
```

## Pré-Requisitos

No Windows/PowerShell:

```powershell
cargo install espup --locked
espup install
. $HOME/export-esp.ps1

cargo install cargo-espflash espflash ldproxy --locked
cargo install slint-viewer --version 1.17.1 --locked
```

Abre um PowerShell novo se `slint-viewer`, `cargo espflash` ou `ldproxy` não
forem encontrados logo após a instalação.

## Backend

```powershell
cd C:\DC\dc\backend
docker compose up -d --build
docker compose ps
curl.exe -s http://localhost:8080/health
```

Serviços principais:

- `dc-os-core`: HTTP + WebSocket em `localhost:8080`
- `stt-whisper`: ASR local em `localhost:9000`
- `mopidy`: música em `localhost:6680` e `localhost:6600`

Para parar:

```powershell
cd C:\DC\dc\backend
docker compose down
```

## Preview da UI

Validar sintaxe Slint sem abrir janela:

```powershell
cd C:\DC\dc\firmware
slint-viewer ui\main.slint --check
```

Abrir preview desktop:

```powershell
cd C:\DC\dc\firmware
slint-viewer ui\main.slint
```

Se o comando não estiver no `PATH`, usa o executável direto:

```powershell
cd C:\DC\dc\firmware
& "$env:USERPROFILE\.cargo\bin\slint-viewer.exe" ui\main.slint
```

## Firmware

Sempre corre os comandos a partir de `firmware/`.

```powershell
cd C:\DC\dc\firmware
. $HOME/export-esp.ps1
cargo build --release --locked
```

Flash + monitor na porta `COM6`:

```powershell
cd C:\DC\dc\firmware
cargo espflash flash --release --locked --chip esp32s3 --flash-mode dio --flash-size 16mb --partition-table partitions.csv --bootloader ..\..\t\xtensa-esp32s3-espidf\release\bootloader.bin --port COM6 --monitor --monitor-baud 115200 --skip-update-check
```

Se a porta for outra, troca `COM6` por `COM<n>`.

Com o monitor aberto:

- `CTRL+R`: reset por software
- `CTRL+C`: sair do monitor

Ao carregar no botão físico de reset, o Windows pode reiniciar a porta USB/JTAG
e o monitor pode mostrar `os error 22`. Isso normalmente é da porta série a
cair e voltar, não necessariamente erro do firmware.

## Flash Sem Monitor

Útil para validar gravação sem deixar a porta série presa:

```powershell
cd C:\DC\dc\firmware
cargo espflash flash --release --locked --chip esp32s3 --flash-mode dio --flash-size 16mb --partition-table partitions.csv --bootloader ..\..\t\xtensa-esp32s3-espidf\release\bootloader.bin --port COM6 --skip-update-check
```

## Logs Esperados

Depois do flash, o monitor deve mostrar algo nesta linha:

```text
Flash size:        16MB
App/part. size:    .../4,194,304 bytes
Display: ILI9341V bring-up (320x240)
Display: ILI9341V pronto
Display OK - 320x240
Touch FT6336G: I2C OK, chip/vendor id=0x11
Touch FT6336G task: polling 20 ms, SDA=16, SCL=15, RST=18, INT=17
Slint: frame 1 enviado ao display
```

Quando tocares no ecrã, devem aparecer logs como:

```text
Touch: down raw=(..., ...) ui=(..., ...)
Touch: up ui=(..., ...)
```

## Spotify

O player de musica agora integra a Spotify Web API. Apos ligar o Wi-Fi,
o firmware pede as 5 faixas mais ouvidas (`v1/me/top/tracks?time_range=long_term`).
Os titulos e artistas reais aparecem no ecrã do music player.

Para incluir o token OAuth do Spotify no firmware, define a variavel de
ambiente antes de compilar:

```powershell
cd C:\DC\dc\firmware
$env:SPOTIFY_TOKEN = "BQ..."
cargo build --release --locked
```

O token e incorporado no binario em build-time via `option_env!("SPOTIFY_TOKEN")`.
Sem token, o player mostra "A carregar..." e funciona em modo offline.

## Configuração Wi-Fi

Por omissão, o firmware tenta:

- SSID: `DC_Network`
- Password: vazia
- API health: `http://192.168.1.50:8080/health`

O ESP não consegue usar `localhost` para falar com o backend no PC. Usa o IP
LAN da máquina onde corre o `dc-os-core`, por exemplo `192.168.1.50`.

Podes alterar em build-time com variáveis de ambiente:

```powershell
cd C:\DC\dc\firmware
$env:DC_WIFI_SSID = "MinhaRede"
$env:DC_WIFI_PASS = "MinhaSenha"
$env:DC_CORE_HTTP = "http://192.168.1.50:8080/health"
cargo build --release --locked
```

Depois grava normalmente com `cargo espflash flash ...`. O estado Wi-Fi, a
rede selecionada e o PIN de 5 dígitos ficam gravados no NVS; no arranque
seguinte o firmware carrega esses valores e salta a configuração inicial se já
houver PIN.

## Hora, API e Bluetooth

Quando o Wi-Fi liga, o firmware inicia SNTP e atualiza o relógio da status bar.
O fuso é escolhido pela região configurada:

- Brasil: UTC-3
- Portugal: UTC+1
- Angola: UTC+1
- Moçambique: UTC+2
- Estados Unidos: UTC-4

Em seguida faz `GET` periódico ao endpoint `DC_CORE_HTTP` para confirmar se a
API real está acessível. Os botões do player chamam `POST /music/command` no
mesmo backend.

As Definições abrem primeiro em categorias:

- Conexões: Wi-Fi, Bluetooth e redes conhecidas
- Segurança: alterar PIN
- Idioma e Região: 5 regiões e 5 idiomas
- Som e Ecrã: volume e brilho
- Sistema: resumo do firmware

O Bluetooth já aparece e fica persistido nas Definições/Control Center. O stack
BLE real continua desligado em `sdkconfig.defaults` porque este projeto fixou
ESP-IDF 5.2.3 + `esp-idf-svc 0.52.x`, combinação que anteriormente quebrou o
build ao ativar BLE. Para ativar rádio BLE de verdade, primeiro será preciso
trocar para um driver BLE compatível e reabrir `CONFIG_BT_ENABLED`.

A rotação agora é manual entre paisagem normal e paisagem invertida. Este módulo
não tem acelerómetro mapeado no pinout atual, portanto ainda não existe
auto-rotação real por posição física.

## Limpeza / Rebuild Completo

Usa quando mudares `sdkconfig.defaults`, partições ou metadata do
`esp-idf-sys`:

```powershell
cd C:\DC\dc\firmware
cargo clean
cargo build --release --locked
```

O projeto usa `target-dir = "../../t"`, então os artefatos principais ficam em:

```text
C:\DC\t\xtensa-esp32s3-espidf\release\
```

## Pinout Principal

Display ILI9341V:

- SCLK: GPIO12
- MOSI: GPIO11
- MISO: GPIO13, não usado no driver atual
- CS: GPIO10
- DC: GPIO46
- BL: GPIO45
- RST: `CHIP_PU`

Touch FT6336G:

- SDA: GPIO16
- SCL: GPIO15
- INT: GPIO17
- RST: GPIO18

Ver [docs/PINOUT.md](docs/PINOUT.md) para a tabela completa.

## Troubleshooting Rápido

- `slint-viewer` não reconhecido: instala com `cargo install slint-viewer --version 1.17.1 --locked` e abre um PowerShell novo.
- `image_too_big`: confirma que estás em `firmware/` e que o comando inclui `--partition-table partitions.csv --flash-size 16mb`.
- `linker 'xtensa-esp32s3-elf-gcc' not found`: corre `. $HOME/export-esp.ps1`.
- Flash não conecta: mantém `BOOT` premido, toca em `RESET`, solta `BOOT` e repete o flash.
- Ecrã branco/sem UI: confirma os logs `Display OK` e o pinout em `docs/PINOUT.md`.
- Touch não reage: confirma `Touch FT6336G: I2C OK` no monitor.

Guia detalhado: [docs/BUILD_AND_FLASH.md](docs/BUILD_AND_FLASH.md).
