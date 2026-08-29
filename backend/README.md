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
| GET | `/weather?region=0..4` | Clima atual via Open-Meteo |

## Subir Localmente

```bash
docker compose up -d --build
curl http://localhost:8081/health
curl "http://localhost:8081/weather?region=1"
```

## Variaveis

| Var | Default | Servico |
|-----|---------|---------|
| `DC_CORE_HOST_PORT` | 8081 | docker compose |
| `DC_CORE_PORT` | 8080 | dc-os-core |
| `STT_URL` | http://stt-whisper:9000/asr | dc-os-core |
| `MOPIDY_URL` | http://mopidy:6680 | dc-os-core |
| `SPOTIFY_TOKEN` | vazio | dc-os-core |
| `SPOTIFY_REFRESH_TOKEN` | vazio | dc-os-core |
| `SPOTIFY_CLIENT_ID` | vazio | dc-os-core |
| `SPOTIFY_CLIENT_SECRET` | vazio | dc-os-core |
| `SPOTIFY_DEVICE_ID` | vazio | dc-os-core |
| `ASR_MODEL` | small | stt-whisper |

Para tocar musica pelo Spotify Web API, cria `backend/.env` a partir de
`.env.example` e coloca o access token, refresh token, client id e client
secret gerados pelo OAuth. O access token expira, mas o core renova
automaticamente quando `SPOTIFY_REFRESH_TOKEN`, `SPOTIFY_CLIENT_ID` e
`SPOTIFY_CLIENT_SECRET` estao configurados.

Scopes recomendados: `user-top-read`, `user-read-playback-state`,
`user-modify-playback-state` e `user-read-currently-playing`. Playback exige
conta Premium e um dispositivo Spotify ativo; usa `/music/devices` para
descobrir o `SPOTIFY_DEVICE_ID` quando necessario.
