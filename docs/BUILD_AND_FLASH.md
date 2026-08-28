# Build & Flash — Firmware DC OS (ESP32-S3 / ES3C28P)

Este guia cobre as duas formas de compilar e gravar o firmware Rust
(`firmware/`): via **PlatformIO** (recomendado, já configurado no
`platformio.ini`) ou via **cargo + espflash** diretamente.

## 1. Pré-requisitos

### Comuns às duas vias
- Python 3.9+ e `pip`
- Git
- ~2 GB livres (toolchain Xtensa + ESP-IDF)
- Cabo USB-C com dados (não só carga) ligado ao ES3C28P

### Toolchain Rust para Xtensa (ESP32-S3)
O projeto usa `rust-toolchain.toml` com `channel = "esp"` e o target
`xtensa-esp32s3-espidf`. Instala-se com o **espup**:

```bash
cargo install espup --locked
espup install
# Carrega as variáveis de ambiente do toolchain Xtensa em cada shell novo:
source $HOME/export-esp.sh        # Linux/macOS
# ou, no Windows (PowerShell):
# . $HOME/export-esp.ps1
```

Instala também as ferramentas de flash/monitor:

```bash
cargo install espflash ldproxy --locked
```

## 2A. Build + flash via PlatformIO (recomendado)

O `platformio.ini` já delega o build para o `cargo` através de
`scripts/rust_build.py`, portanto o PlatformIO trata do ESP-IDF e da
gravação/monitor.

```bash
cd firmware

# Compilar (chama automaticamente `cargo build --release --locked`)
pio run -e dc_assistant

# Gravar no ES3C28P (ajusta a porta se necessário)
pio run -e dc_assistant -t upload --upload-port /dev/ttyUSB0

# Monitor série (115200 bps, já com exception decoder configurado)
pio device monitor -e dc_assistant
```

No Windows a porta costuma ser `COM<n>` (ex.: `COM5`); no macOS,
`/dev/cu.usbserial-XXXX` ou `/dev/cu.usbmodemXXXX`.

Para descobrir a porta antes de gravar:

```bash
pio device list
```

## 2B. Build + flash apenas com cargo/espflash

Alternativa sem PlatformIO — útil para iteração rápida durante o
debug do display.

> ⚠️ **Atenção à versão do `espflash`.** A partir da v3, o binário
> `espflash` sozinho **não** aceita `--release` — esse flag só existe
> no subcomando `cargo espflash` (do pacote `cargo-espflash`), que
> invoca o `cargo build` por ti. O `espflash flash` "puro" espera
> sempre o **caminho do ELF já compilado** como argumento posicional.
> Repara também que o `.cargo/config.toml` deste projeto redireciona
> o `target-dir` para `../../t` (para encurtar caminhos no Windows),
> por isso o ELF **não** fica em `firmware/target/...`.

**Opção 1 — `cargo-espflash` (recomendada, um único comando):**

```bash
cargo install cargo-espflash --locked
cd firmware
source $HOME/export-esp.sh        # Linux/macOS; no Windows: . $HOME/export-esp.ps1

# Compila e grava (chama o cargo build internamente) e abre o monitor
cargo espflash flash --release --monitor
# Se não detetar a porta sozinho: --port /dev/ttyUSB0 (Linux) ou --port COM5 (Windows)
```

**Opção 2 — `espflash` "puro" (sem instalar mais nada):**

```bash
cd firmware
source $HOME/export-esp.sh
cargo build --release --locked

# Linux/macOS:
espflash flash --monitor ../../t/xtensa-esp32s3-espidf/release/dc-os-firmware

# Windows (PowerShell):
espflash flash --monitor ..\..\t\xtensa-esp32s3-espidf\release\dc-os-firmware
```

Se não detetar a porta automaticamente, acrescenta `--port <porta>`
(`/dev/ttyUSB0` no Linux, `COM5` por exemplo no Windows).

Se não souberes a porta:

```bash
espflash board-info
```

## 3. Erros comuns

| Sintoma | Causa provável | Solução |
|---|---|---|
| `error: linker 'xtensa-esp32s3-elf-gcc' not found` | `export-esp.sh` não foi carregado nesta shell | Corre `source $HOME/export-esp.sh` antes do build |
| `Failed to connect to ESP32-S3` no flash | Placa não entrou em modo bootloader | Mantém **BOOT** premido, toca em **RESET**, solta **BOOT** e repete o comando de flash |
| Build demora muito na primeira vez | Está a compilar o ESP-IDF completo | Normal na 1ª build (10–20 min); as seguintes usam cache |
| Ecrã liga o backlight mas fica branco | Sequência de bring-up do ILI9341V incompleta/pixels não enviados | Ver secção "Debug do ecrã branco" abaixo |
| Build passa (`pio run` / `cargo build` OK) mas a placa reinicia em boot-loop / crasha logo a seguir ao flash, com `Guru Meditation Error: ... StoreProhibited` ou `stack overflow in task main` no monitor série | `sdkconfig.defaults` não estava a ser lido pelo `esp-idf-sys` (faltava a tabela `[package.metadata.esp-idf-sys]` no `Cargo.toml`), pelo que a stack da task `main` ficava no valor de fábrica do ESP-IDF (3584 bytes) em vez dos 32 KB necessários para o renderer do Slint + Wi-Fi | Já corrigido no `Cargo.toml` (commit `5645bb4`). Se voltares a ver isto depois de mexer no `Cargo.toml`/`sdkconfig.defaults`, força a regeneração: `cargo clean` + apaga a pasta de build do `esp-idf-sys` (normalmente em `target/**/esp-idf-sys-*/out` ou `~/.espressif`/cache do embuild) antes de rebuildar |
| `esptool` liga, mas falha a meio do write com `Timed out waiting for packet content` ou timeouts repetidos | Cabo USB só de carga, hub USB sem alimentação suficiente, ou `upload_speed`/baudrate de flash demasiado alto para o cabo/porta | Usa cabo de dados direto ao PC (não hub), ou baixa `upload_speed` no `platformio.ini` (ex.: `460800`) / usa `espflash flash --baud 460800 ...` |

## 4. Debug do ecrã branco (contexto)

O ecrã ficar só com luz branca (backlight aceso, sem imagem) tem duas
causas típicas neste projeto, ambas relacionadas com pontos do código
marcados como stub/TODO:

1. **Bring-up do controlador nunca enviado** — se `Display::init` liga
   o backlight PWM mas não envia `SWRESET` → `SLPOUT` → `MADCTL` →
   `COLMOD` → `DISPON` ao ILI9341V, o painel fica iluminado mas sem
   nenhum comando de configuração, e nunca sai do estado de arranque.
2. **Loop de render nunca faz flush para o SPI** — mesmo com o
   controlador inicializado, se o loop de eventos não chamar
   `window.draw_if_needed(...)` e não transferir a região "suja" do
   framebuffer via SPI, nenhum pixel chega ao painel.

Após corrigir isto (ver commit associado a este guia em
`firmware/src/display.rs` e `firmware/src/slint_platform.rs`),
confirma no monitor série que aparecem os logs:

```
Display: ILI9341V bring-up (320×240)
Display: ILI9341V pronto
Display OK — 320×240
```

Se o ecrã continuar branco após estes logs aparecerem, verifica por
ordem:

- **MADCTL (0x36)** — valor errado inverte cores/orientação, mas não
  costuma dar ecrã branco liso.
- **COLMOD (0x3A)** — se não corresponder ao formato de pixel enviado
  (aqui RGB565 = `0x55`), a imagem pode aparecer distorcida ou em
  branco.
- **Alimentação/RST** — no ES3C28P o RST do painel está ligado ao
  `CHIP_PU` do ESP32-S3 (ver `docs/PINOUT.md`); um brownout no boot
  pode deixar o painel num estado inconsistente — testa alimentar por
  USB de PC/hub com boa corrente, não só um cabo de carregador.
- **Baudrate do SPI** — 40 MHz é o valor configurado; em fios longos ou
  breadboard, baixar para 20 MHz (`SpiConfig::new().baudrate(20.MHz().into())`)
  ajuda a descartar problemas de integridade de sinal.

## 5. Checklist rápida antes de regravar

Depois de qualquer correção no firmware (ecrã, stack, Wi-Fi, etc.), segue
esta ordem para evitar reaproveitar um build/sdkconfig desatualizado:

1. `cd firmware`
2. `cargo clean` (garante que o `sdkconfig` é regenerado a partir do
   `sdkconfig.defaults` atual — importante depois de mexer nesse ficheiro
   ou no `[package.metadata.esp-idf-sys]` do `Cargo.toml`)
3. `cargo build --release --locked` (ou `pio run -e dc_assistant`) —
   confirma que termina sem erros
4. Coloca o ES3C28P em modo bootloader se o `esptool` não conseguir ligar
   sozinho: mantém **BOOT** premido, toca em **RESET**, solta **BOOT**
5. Grava com o `cargo espflash flash --release --monitor` (recomendado)
   ou `espflash flash --monitor <caminho-do-ELF>` — ver secção 2B para o
   caminho exato do ELF e por que `espflash flash --release` sozinho
   dá erro nas versões recentes — ou `pio run -t upload` seguido de
   `pio device monitor`
6. No monitor série, confirma pela ordem:
   ```
   DC OS boot — DC Assistant firmware v0.1.0
   Display: ILI9341V bring-up (320×240)
   Display: ILI9341V pronto
   Display OK — 320×240
   ```
   Sem "Guru Meditation Error" nem "stack overflow in task main" a seguir.
7. Se o ecrã continuar branco com estes logs a aparecer, segue a secção 4
   acima (é já um problema de hardware/fiação, não de firmware).
