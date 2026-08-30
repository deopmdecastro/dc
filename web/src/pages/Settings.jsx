import { useState } from "react";
import Panel from "../components/Panel";
import { api } from "../lib/api";

const LANGS = ["Português (PT)", "Português (BR)", "English", "Español"];

function Toggle({ checked, onChange }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={`relative h-6 w-11 rounded-full transition-colors ${checked ? "bg-accent" : "bg-panel-elevated"}`}
    >
      <span
        className={`absolute top-0.5 h-5 w-5 rounded-full bg-text-primary transition-transform ${
          checked ? "translate-x-5" : "translate-x-0.5"
        }`}
      />
    </button>
  );
}

export default function Settings() {
  const [lang, setLang] = useState(0);
  const [brightness, setBrightness] = useState(80);
  const [volume, setVolume] = useState(60);
  const [notifications, setNotifications] = useState(true);
  const [pin, setPin] = useState("");

  return (
    <div className="space-y-6">
      <Panel eyebrow="Ligação" title="Backend">
        <div className="flex items-center justify-between text-sm">
          <span className="text-text-secondary">URL da API</span>
          <code className="rounded-s bg-panel-soft px-2 py-1 text-xs text-accent-hi" style={{ borderRadius: "var(--radius-s)" }}>
            {api.baseUrl}
          </code>
        </div>
        <p className="mt-2 text-xs text-text-muted">
          Define <code>VITE_API_BASE_URL</code> num ficheiro <code>.env</code> para apontar a outro host.
        </p>
      </Panel>

      <Panel eyebrow="Região / idioma" title="Preferências">
        <div className="space-y-4">
          <label className="block">
            <span className="mb-1.5 block text-sm text-text-secondary">Idioma</span>
            <select
              value={lang}
              onChange={(e) => setLang(Number(e.target.value))}
              className="w-full rounded-s border border-stroke-soft bg-panel-soft px-3 py-2 text-sm text-text-primary outline-none focus:border-accent sm:w-72"
              style={{ borderRadius: "var(--radius-s)" }}
            >
              {LANGS.map((l, i) => (
                <option key={l} value={i}>
                  {l}
                </option>
              ))}
            </select>
          </label>

          <label className="block">
            <span className="mb-1.5 block text-sm text-text-secondary">PIN (opcional)</span>
            <input
              type="password"
              inputMode="numeric"
              maxLength={6}
              value={pin}
              onChange={(e) => setPin(e.target.value.replace(/\D/g, ""))}
              placeholder="••••"
              className="w-full rounded-s border border-stroke-soft bg-panel-soft px-3 py-2 text-sm text-text-primary outline-none focus:border-accent sm:w-72"
              style={{ borderRadius: "var(--radius-s)" }}
            />
          </label>
        </div>
      </Panel>

      <Panel eyebrow="Ecrã e som" title="Ajustes">
        <div className="space-y-5">
          <div>
            <div className="mb-1.5 flex justify-between text-sm">
              <span className="text-text-secondary">Brilho</span>
              <span className="text-text-muted">{brightness}%</span>
            </div>
            <input
              type="range"
              min={10}
              max={100}
              value={brightness}
              onChange={(e) => setBrightness(Number(e.target.value))}
              className="w-full accent-[var(--color-accent)]"
            />
          </div>
          <div>
            <div className="mb-1.5 flex justify-between text-sm">
              <span className="text-text-secondary">Volume</span>
              <span className="text-text-muted">{volume}%</span>
            </div>
            <input
              type="range"
              min={0}
              max={100}
              value={volume}
              onChange={(e) => setVolume(Number(e.target.value))}
              className="w-full accent-[var(--color-accent)]"
            />
          </div>
          <div className="flex items-center justify-between">
            <span className="text-sm text-text-secondary">Notificações</span>
            <Toggle checked={notifications} onChange={setNotifications} />
          </div>
        </div>
      </Panel>
    </div>
  );
}
