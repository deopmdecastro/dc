# DC OS Backend

Ambiente local do DC Assistant, orquestrado com Docker Compose.

## Servicos

| Servico | Porta | Descricao |
|---------|-------|-----------|
| `dc-os-core` | 8081 -> 8080 | Hub Rust/Axum, HTTP + WebSocket com o firmware |
| `stt-whisper` | 9000 | Whisper ASR REST |
| `mopidy` | 6680 / 6600 | Player + MPD |

## Endpoints

| Metodo | Rota | Uso |
|--------|------|-----|
| GET | `/health` | Healthcheck |
| GET | `/time` | Hora atual para o firmware |
| GET | `/ws` | WebSocket com o firmware |
| POST | `/voice/transcribe` | Encaminha audio ao Whisper |
| GET | `/music/state` | Estado do player |
| GET | `/music/devices` | Dispositivos Spotify disponiveis |
| GET | `/music/top-tracks` | Top tracks reais via Spotify Web API |
| POST | `/music/command` | `{ "action": "play|pause|next|prev" }` |

## Subir Localmente

```bash
docker compose up -d --build
curl http://localhost:8081/health
```

## Variaveis

| Var | Default | Servico |
|-----|---------|---------|
| `DC_CORE_HOST_PORT` | 8081 | docker compose |
| `DC_CORE_PORT` | 8080 | dc-os-core |
| `STT_URL` | http://stt-whisper:9000/asr | dc-os-core |
| `MOPIDY_URL` | http://mopidy:6680 | dc-os-core |
| `SPOTIFY_TOKEN` | vazio | dc-os-core |
| `SPOTIFY_DEVICE_ID` | vazio | dc-os-core |
| `ASR_MODEL` | small | stt-whisper |

Para tocar musica pelo Spotify Web API, cria `backend/.env` a partir de
`.env.example` e coloca um token com scopes `user-top-read`,
`user-read-playback-state` e `user-modify-playback-state`. Playback exige conta
Premium e um dispositivo ativo; usa `/music/devices` para descobrir o
`SPOTIFY_DEVICE_ID` quando necessario.
