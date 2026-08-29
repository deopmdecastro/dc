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
- UI Slint no estilo DC OS escuro ("Midnight Cyan"), com launcher sem barra
  lateral, Definições por categorias, Alarme configurável, PIN persistente e
  região/idioma.
- Design system centralizado em `ui/theme.slint`: paleta ciano-azul, superfícies
  em camadas, tokens de raio/espaçamento/tipografia. Ícones em SVG line-icon
  (estilo Lucide) tingidos por `colorize`, com estados accent/hover.
- A tela inicial usa gesto de deslizar para a esquerda para abrir o launcher
  de aplicacoes (o botao "Abrir Apps" foi removido).
- Player de musica consome top tracks reais pelo backend e usa Spotify Web API
  para `play`, `pause`, `next` e `prev` quando o OAuth Spotify esta configurado.
- Wi-Fi liga de verdade, sincroniza hora por SNTP e tambem consome `/time` da
  API real por HTTP.
- Brilho ajusta o PWM do backlight; volume fica persistido para integração de
  áudio.
- Bluetooth fica configurado em modo BLE/NimBLE no `sdkconfig.defaults`, sem
  ativar Bluedroid, porque Bluedroid quebra o build com ESP-IDF 5.2.3.

## Estrutura

```text
dc/
├── firmware/                 # Firmware Rust para ESP32-S3
│   ├── .cargo/config.toml     # target Xtensa e target-dir ../../t
│   ├── Cargo.toml
│   ├── partitions.csv         # tabela 16 MB, factory app 4 MB
│   ├── sdkconfig.defaults     # flash, PSRAM, stacks, Wi-Fi e BLE
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
curl.exe -s http://localhost:8081/health
curl.exe -s "http://localhost:8081/time?offset_secs=3600"
curl.exe -s http://localhost:8081/music/devices
```

Serviços principais:

- `dc-os-core`: HTTP + WebSocket em `localhost:8081`
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

## Icones

O conjunto de UI usa icones line-icon no estilo **Lucide** (stroke 2px,
cantos arredondados), guardados como SVG locais em
`firmware/ui/assets/icons/` porque o Slint embarcado precisa embutir os
assets no binario. Os icones sao monocromaticos e recebem cor em runtime
via a propriedade `colorize` da `Image` do Slint (accent quando ativo,
`text-secondary`/`text-muted` quando inativo), evitando manter varias copias
coloridas do mesmo simbolo.

Icones adicionados nesta iteracao de UI/UX:
`shield`, `globe`, `sliders`, `cpu`, `zap`, `bell`, `clock`, `battery`,
`activity`, `grid`, `play`, `pause`. Substituiram os antigos simbolos
Unicode (`⌁ ◆ 文 ♪ ⚙`) que renderizavam mal no painel pequeno.

Os icones de marca (ex.: Spotify) continuam a vir da biblioteca Simple Icons:

- Biblioteca: https://iconstack.io/library/simple
- Pacote SVG: https://github.com/simple-icons/simple-icons

O icone do Spotify usa `firmware/ui/assets/icons/spotify.svg`.

## Tema / Design System

Todas as telas importam `ui/theme.slint`, o unico ponto de verdade de estilo:

- **Fundos** em camadas (`bg-0 → bg-2`) e **paineis** (`panel`,
  `panel-soft`, `panel-elevated`) para dar profundidade sem sombras (o
  software renderer nao faz blur).
- **Accent** ciano-azul (`#38BDF8`) com variacoes `accent-hi`, `accent-deep`,
  `accent-cyan`, `accent-blue`, `accent-violet`, `accent-pink`.
- **Superficies tonais** pre-mescladas (`accent-tint`, `success-tint`,
  `danger-tint`) para chips/badges sem alpha em tempo real.
- **Tokens**: `radius-xs..xl`, `space-1..4` (grelha 4px), `font-xs..xl`.

Para mudar a identidade visual do DC OS basta editar `theme.slint`.

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
o firmware pede ao backend as 5 faixas mais ouvidas (`/music/top-tracks`).
Os titulos e artistas reais aparecem no ecrã do music player.

Para incluir o token OAuth do Spotify no firmware, usa uma destas opcoes antes
de compilar.

Opcao recomendada: criar um ficheiro local ignorado pelo Git:

```powershell
cd C:\DC\dc\firmware
Copy-Item .env.example .env.local
notepad .env.local
cargo build --release --locked
```

Dentro de `.env.local`:

```text
SPOTIFY_TOKEN="BQ..."
```

Tambem podes usar variavel de ambiente so nessa sessao:

```powershell
cd C:\DC\dc\firmware
$env:SPOTIFY_TOKEN = "BQ..."
cargo build --release --locked
```

O token e incorporado no binario em build-time. Sempre que trocares o token,
recompila e faz flash novamente. Sem token, o player mostra "A carregar..." e
funciona em modo offline.

Para o backend tocar musica de verdade pelo Spotify Web API, configura tambem
`backend/.env`:

```powershell
cd C:\DC\dc\backend
Copy-Item .env.example .env
notepad .env
docker compose up -d --build
```

Dentro de `backend/.env`:

```text
SPOTIFY_TOKEN="BQ..."
SPOTIFY_REFRESH_TOKEN="AQ..."
SPOTIFY_CLIENT_ID="..."
SPOTIFY_CLIENT_SECRET="..."
SPOTIFY_DEVICE_ID=
```

O token precisa dos scopes `user-top-read`, `user-read-playback-state`,
`user-modify-playback-state` e `user-read-currently-playing`. O access token
expira, mas o backend renova automaticamente quando `SPOTIFY_REFRESH_TOKEN`,
`SPOTIFY_CLIENT_ID` e `SPOTIFY_CLIENT_SECRET` estao configurados. Para tocar, a
conta precisa de Spotify Premium e um dispositivo ativo. Usa `/music/devices`
para descobrir o `device_id` caso o Spotify responda que nao existe player
ativo.

## Configuração Wi-Fi

Por omissão, o firmware tenta:

- SSID: `DC_Network`
- Password: vazia
- API health: `http://192.168.1.50:8081/health`

O ESP não consegue usar `localhost` para falar com o backend no PC. Usa o IP
LAN da máquina onde corre o `dc-os-core`, por exemplo `192.168.1.50`.

Podes alterar em build-time com variáveis de ambiente:

```powershell
cd C:\DC\dc\firmware
$env:DC_WIFI_SSID = "MinhaRede"
$env:DC_WIFI_PASS = "MinhaSenha"
$env:DC_CORE_HTTP = "http://192.168.1.50:8081/health"
cargo build --release --locked
```

Depois grava normalmente com `cargo espflash flash ...`. O estado Wi-Fi, a
rede selecionada, a senha digitada na UI e o PIN de 5 digitos ficam gravados
no NVS; no arranque seguinte o firmware carrega esses valores e salta a
configuracao inicial se ja houver PIN.

## Hora, API e Bluetooth

Quando o Wi-Fi liga, o firmware inicia SNTP e atualiza o relógio da status bar.
O fuso é escolhido pela região configurada:

- Brasil: UTC-3
- Portugal: UTC+1
- Angola: UTC+1
- Moçambique: UTC+2
- Estados Unidos: UTC-4

Em seguida faz `GET` periodico ao endpoint `DC_CORE_HTTP` para confirmar se a
API real esta acessivel. A hora tambem pode vir de `/time?offset_secs=...` no
mesmo backend quando o SNTP ainda nao atualizou o relogio do ESP. Os botoes do
player chamam `POST /music/command` no backend.

Nas Definicoes, a pagina `Conexoes` faz scan real das redes Wi-Fi visiveis com
o radio do ESP32-S3. A lista e atualizada ao abrir a pagina, ao tocar no botao
de atualizar e periodicamente enquanto o Wi-Fi esta ligado. As redes sao
ordenadas pelo sinal, mostram RSSI em dBm e redes duplicadas ficam ocultas.

Nas redes protegidas, tocar no SSID abre um teclado compacto para inserir a
senha antes de ligar. As Definicoes tambem aceitam arrasto horizontal para
navegar entre as paginas, alem dos botoes de voltar.

As Definições abrem primeiro em categorias:

- Conexoes: Wi-Fi, Bluetooth e redes disponiveis
- Segurança: alterar PIN
- Idioma e Região: 5 regiões e 5 idiomas
- Som e Ecrã: volume e brilho
- Sistema: resumo do firmware

O Bluetooth usa configuracao NimBLE no SDK, mas o firmware ainda nao faz scan
BLE real. O toggle atualiza o estado no NVS e na UI. Bluedroid ficou
explicitamente desligado porque `esp-idf-svc 0.52.1` tenta compilar APIs GATT
que nao existem nos bindings do ESP-IDF 5.2.3, causando erros como
`esp_ble_conn_params_t` e `esp_ble_gattc_enh_open` nao encontrados. O ESP32-S3
nao suporta Bluetooth Classic/A2DP neste caminho, por isso audio por
coluna/fone Bluetooth nao esta incluido aqui.

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
- `localhost:8080` retorna 404 ou a API nao responde: este projeto usa
  `dc-os-core` em `localhost:8081`, porque a porta 8080 pode estar ocupada por
  outro container.
- Spotify retorna `401`: token expirado, invalido ou sem scopes. Gera OAuth com
  `user-top-read`, `user-read-playback-state`, `user-modify-playback-state` e
  `user-read-currently-playing`; confirma tambem `SPOTIFY_REFRESH_TOKEN`,
  `SPOTIFY_CLIENT_ID` e `SPOTIFY_CLIENT_SECRET` no `backend/.env`.

Guia detalhado: [docs/BUILD_AND_FLASH.md](docs/BUILD_AND_FLASH.md).
