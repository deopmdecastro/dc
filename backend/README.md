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
| GET  | `/ws`                 | WebSocket bidirecional com o firmware |
| POST | `/voice/transcribe`   | Encaminha WAV/PCM ao Whisper |
| GET  | `/music/state`        | Estado do player (proxy Mopidy) |
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
| `ASR_MODEL`    | small | stt-whisper |
