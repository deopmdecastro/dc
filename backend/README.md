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
| GET | `/music/top-tracks` | Top tracks reais via Spotify Web API (cache de 5 min) |
| POST | `/music/command` | `{ "action": "play|pause|next|prev" }` |
| GET | `/songshare/tracks` | Catalogo Songstats/RapidAPI em formato compacto para o firmware |
| GET | `/spotify/login` | Inicia o login OAuth da Spotify (visitar num browser) |
| GET | `/spotify/callback` | Callback OAuth; troca o `code` por tokens |
| GET | `/spotify/status` | Diagnostico: o que esta configurado e o que falta |
| GET | `/weather?region=0..4` | Clima atual via Open-Meteo |

## Subir Localmente

```bash
docker compose up -d --build
curl http://localhost:8081/health
curl "http://localhost:8081/weather?region=1"
curl "http://localhost:8081/songshare/tracks?compact=true"
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
| `SPOTIFY_REDIRECT_URI` | `http://localhost:8081/spotify/callback` | dc-os-core |
| `SPOTIFY_TOKEN_STORE` | `/data/spotify_refresh_token` (no container) | dc-os-core |
| `SONGSTATS_RAPIDAPI_KEY` | vazio | dc-os-core |
| `SONGSTATS_RAPIDAPI_HOST` | `songstats.p.rapidapi.com` | dc-os-core |
| `SONGSTATS_LABEL_ID` | `7gk4yfc9` | dc-os-core |
| `BEATPORT_LABEL_ID` | `74932` | dc-os-core |
| `ASR_MODEL` | small | stt-whisper |

## Configurar o Spotify (login OAuth em 3 passos)

1. Cria uma app em https://developer.spotify.com/dashboard, copia o
   **Client ID** e o **Client Secret**, e adiciona
   `http://localhost:8081/spotify/callback` (ou o valor que definires em
   `SPOTIFY_REDIRECT_URI`) como Redirect URI da app.
2. Copia `backend/.env.example` para `backend/.env` e preenche
   `SPOTIFY_CLIENT_ID` e `SPOTIFY_CLIENT_SECRET` (nunca commites este
   ficheiro — ja esta no `.gitignore`). Sobe os servicos:
   ```bash
   docker compose up -d --build
   ```
3. No PC (ou em qualquer browser na mesma rede), visita
   `http://localhost:8081/spotify/login`, autoriza a tua conta Spotify e
   pronto: o `dc-os-core` guarda o `access_token`/`refresh_token`
   automaticamente (em memoria e em disco, no volume `spotify-data`), e
   passa a renovar sozinho quando o token expira — nao precisas de gerar
   nem colar tokens manualmente. Confirma com:
   ```bash
   curl http://localhost:8081/spotify/status
   ```

Se preferires nao usar o login interativo, continua a poder colocar um
`SPOTIFY_TOKEN`/`SPOTIFY_REFRESH_TOKEN` ja gerados manualmente em
`backend/.env` — o comportamento antigo continua a funcionar como
alternativa.

Scopes pedidos automaticamente no `/spotify/login`: `user-top-read`,
`user-read-playback-state`, `user-modify-playback-state` e
`user-read-currently-playing`. Playback exige conta Premium e um
dispositivo Spotify ativo; usa `/music/devices` para descobrir o
`SPOTIFY_DEVICE_ID` quando necessario.

`/music/top-tracks` mantem agora uma cache de 5 minutos por processo, para
reduzir o numero de chamadas feitas a Spotify Web API (o firmware sonda este
endpoint a cada 60s) e diminuir o risco de HTTP 429 (rate limit). Um pedido
que devolva 429 e reportado como `{"ok": false, "error": "rate_limited"}` em
vez de tentar renovar o token (o 429 nao tem nada a ver com token expirado).

## SongShare / Songstats

`/songshare/tracks?compact=true` usa a API Songstats via RapidAPI e devolve as
faixas num formato igual ao compact do Spotify:

```json
{
  "ok": true,
  "driver": "songshare",
  "body": { "items": [{ "name": "...", "artists": [{ "name": "..." }] }] }
}
```

A chave deve ficar apenas em `backend/.env`:

```text
SONGSTATS_RAPIDAPI_KEY=...
SONGSTATS_LABEL_ID=7gk4yfc9
BEATPORT_LABEL_ID=74932
```

Esta API fornece catalogo/metadados/songshare; ela nao e um stream de audio.
No firmware, a app SongShare reutiliza a UI do player para listar/navegar as
faixas recebidas.
