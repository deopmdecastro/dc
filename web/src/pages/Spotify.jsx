import { Play, Pause, SkipBack, SkipForward, ExternalLink, Music2 } from "lucide-react";
import Panel from "../components/Panel";
import { api } from "../lib/api";
import { usePolledApi } from "../lib/useApi";

export default function Spotify() {
  const { data: status } = usePolledApi(() => api.spotifyStatus(), { intervalMs: 20000 });
  const { data: state, offline, refresh } = usePolledApi(() => api.musicState(), { intervalMs: 8000 });
  const { data: tracksResp } = usePolledApi(() => api.musicTopTracks(), { intervalMs: 60000 });

  const tracks = tracksResp?.tracks ?? tracksResp?.items ?? (Array.isArray(tracksResp) ? tracksResp : []);
  const playing = state?.is_playing ?? state?.playing ?? false;

  const sendCommand = async (action) => {
    try {
      await api.musicCommand(action);
      refresh();
    } catch (e) {
      // backend offline — sem-op na demo
    }
  };

  return (
    <div className="space-y-6">
      <Panel eyebrow="Spotify" title="Player">
        {offline && (
          <p className="mb-4 rounded-s border border-warning/30 bg-warning/10 px-3 py-2 text-xs text-warning" style={{ borderRadius: "var(--radius-s)" }}>
            Sem ligação ao <code>dc-os-core</code>. A mostrar layout de exemplo — liga o backend para dados reais.
          </p>
        )}

        <div className="flex flex-col items-center gap-5 py-4 sm:flex-row sm:items-center">
          <div className="flex h-28 w-28 shrink-0 items-center justify-center rounded-lg border border-stroke-soft bg-panel-elevated" style={{ borderRadius: "var(--radius-lg)" }}>
            <Music2 className="text-success" size={40} />
          </div>
          <div className="min-w-0 flex-1 text-center sm:text-left">
            <p className="truncate font-display text-lg font-semibold text-text-primary">
              {state?.track ?? state?.title ?? "Nada em reprodução"}
            </p>
            <p className="truncate text-sm text-text-secondary">
              {state?.artist ?? "Liga o Spotify a partir de Definições"}
            </p>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={() => sendCommand("prev")}
              className="flex h-10 w-10 items-center justify-center rounded-full border border-stroke-soft bg-panel-soft text-text-secondary transition-colors hover:text-text-primary"
            >
              <SkipBack size={17} />
            </button>
            <button
              onClick={() => sendCommand(playing ? "pause" : "play")}
              className="flex h-12 w-12 items-center justify-center rounded-full bg-success text-bg-0 transition-transform hover:scale-105"
            >
              {playing ? <Pause size={20} /> : <Play size={20} className="ml-0.5" />}
            </button>
            <button
              onClick={() => sendCommand("next")}
              className="flex h-10 w-10 items-center justify-center rounded-full border border-stroke-soft bg-panel-soft text-text-secondary transition-colors hover:text-text-primary"
            >
              <SkipForward size={17} />
            </button>
          </div>
        </div>
      </Panel>

      <Panel
        eyebrow="OAuth"
        title="Ligação Spotify"
        action={
          <a
            href={api.spotifyLoginUrl()}
            target="_blank"
            rel="noreferrer"
            className="flex items-center gap-1.5 rounded-s border border-success/40 bg-success-tint px-3 py-1.5 text-xs font-medium text-success hover:bg-success/20"
            style={{ borderRadius: "var(--radius-s)" }}
          >
            Iniciar sessão <ExternalLink size={13} />
          </a>
        }
      >
        <p className="text-sm text-text-secondary">
          {status ? JSON.stringify(status) : "Estado de ligação indisponível — verifica /spotify/status no backend."}
        </p>
      </Panel>

      <Panel eyebrow="Top tracks" title="Mais ouvidas">
        {tracks.length ? (
          <ul className="divide-y divide-stroke-soft">
            {tracks.slice(0, 8).map((t, i) => (
              <li key={t.id ?? i} className="flex items-center gap-3 py-2.5">
                <span className="w-5 shrink-0 text-right font-mono text-xs text-text-dim">{i + 1}</span>
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm text-text-primary">{t.name ?? t.title ?? "Faixa"}</p>
                  <p className="truncate text-xs text-text-muted">{t.artist ?? t.artists?.join(", ") ?? ""}</p>
                </div>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-text-muted">Sem faixas para mostrar de momento.</p>
        )}
      </Panel>
    </div>
  );
}
