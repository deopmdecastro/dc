# DC OS Backend

Ambiente de servidor local do DC Assistant, orquestrado com **Docker Compose**.

## Serviços

| Serviço | Porta | Descrição |
|---------|-------|-----------|
| `dc-os-core`   | 8080 | Hub em Rust (`axum`) — HTTP + WebSocket com o firmware |
| `stt-whisper`  | 9000 | Whisper.cpp ASR REST (offline PT-BR) |
| `mopidy`       | 6680 / 6600 | Player + Librespot (Spotify Connect) |

## Endpoints do `dc-os-core`

| Método | Rota | Uso |
|--------|------|-----|
| GET  | `/health`             | Healthcheck |
| GET  | `/time`               | Hora atual para o firmware |
| GET  | `/ws`                 | WebSocket bidirecional com o firmware |
| POST | `/voice/transcribe`   | Encaminha WAV/PCM ao Whisper |
| GET  | `/music/state`        | Estado do player (proxy Mopidy) |
| GET  | `/music/devices`      | Dispositivos Spotify disponiveis |
| GET  | `/music/top-tracks`   | Top tracks reais via Spotify Web API |
| POST | `/music/command`      | `{ "action": "play\|pause\|next\|prev" }` |

## Subir localmente

```bash
docker compose up -d --build
curl http://localhost:8080/health
```

## Variáveis de ambiente

| Var | Default | Serviço |
|-----|---------|---------|
| `DC_CORE_PORT` | 8080 | dc-os-core |
| `STT_URL`      | http://stt-whisper:9000/asr | dc-os-core |
| `MOPIDY_URL`   | http://mopidy:6680 | dc-os-core |
| `SPOTIFY_TOKEN` | vazio | dc-os-core |
| `SPOTIFY_DEVICE_ID` | vazio | dc-os-core |
| `ASR_MODEL`    | small | stt-whisper |

Para tocar musica pelo Spotify Web API, cria `backend/.env` a partir de
`.env.example` e coloca um token com scopes `user-top-read`,
`user-read-playback-state` e `user-modify-playback-state`. Playback exige conta
Premium e um dispositivo ativo; usa `/music/devices` para descobrir o
`SPOTIFY_DEVICE_ID` quando necessario.
