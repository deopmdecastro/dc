import { useState, useEffect } from "react";
import { ArrowLeft, Wifi, Bluetooth, Shield, Globe, Sliders, Cpu, ChevronRight, Volume2, Sun, Power, RotateCw } from "lucide-react";
import { api } from "../lib/api";

const STORAGE_KEY = "dcos.settings.web";
function loadSettings() { try { return JSON.parse(localStorage.getItem(STORAGE_KEY) || "{}"); } catch { return {}; } }
function saveSettings(s) { localStorage.setItem(STORAGE_KEY, JSON.stringify(s)); }

function Toggle({ checked, onChange, color = "var(--accent)" }) {
  return (
    <button onClick={() => onChange(!checked)} style={{ width: "52px", height: "28px", borderRadius: "14px", background: checked ? color : "var(--bg-2)", border: `1.5px solid ${checked ? color : "var(--stroke)"}`, position: "relative", cursor: "pointer", transition: "all 150ms ease", padding: 0 }}>
      <div style={{ position: "absolute", top: "3px", left: checked ? "26px" : "3px", width: "20px", height: "20px", borderRadius: "10px", background: checked ? "var(--bg-0)" : "var(--text-secondary)", transition: "all 150ms ease" }} />
    </button>
  );
}

function SliderRow({ icon: Icon, label, value, onChange, color = "var(--accent)" }) {
  return (
    <div className="flex items-center gap-3" style={{ width: "100%", height: "56px", borderRadius: "16px", background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", padding: "0 16px" }}>
      <div className="flex h-9 w-9 items-center justify-center rounded-xl" style={{ background: `${color}20` }}><Icon size={16} style={{ color }} /></div>
      <span className="text-sm font-semibold text-text-primary" style={{ width: "70px" }}>{label}</span>
      <div className="relative flex-1" style={{ height: "6px", borderRadius: "3px", background: "var(--bg-2)" }}>
        <div style={{ width: `${value}%`, height: "100%", borderRadius: "3px", background: `linear-gradient(90deg, ${color}, ${color}cc)` }} />
        <input type="range" min={0} max={100} value={value} onChange={(e) => onChange(Number(e.target.value))} className="absolute inset-0 cursor-pointer opacity-0" />
      </div>
      <span className="font-mono text-xs text-text-muted w-10 text-right">{value}%</span>
    </div>
  );
}

function SettingsCard({ icon: Icon, title, desc, onClick, color = "var(--accent)", children }) {
  return (
    <div className="rounded-2xl p-4 transition-all" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", marginBottom: "12px" }}>
      <div className="flex items-center gap-3">
        <div className="flex h-11 w-11 items-center justify-center rounded-xl" style={{ background: `linear-gradient(135deg, ${color}, ${color}aa)` }}><Icon size={18} style={{ color: "var(--bg-0)" }} /></div>
        <div className="flex-1"><p className="text-sm font-semibold text-text-primary">{title}</p><p className="text-[10px] text-text-muted">{desc}</p></div>
        {children || (onClick && <ChevronRight size={16} style={{ color: "var(--text-dim)" }} />)}
      </div>
    </div>
  );
}

const LANGS = ["Português (PT)", "Português (BR)", "English", "Español"];
const REGIONS = ["Lisboa", "Porto", "São Paulo", "Rio de Janeiro"];

export default function Settings({ onBack }) {
  const [settings, setSettings] = useState(loadSettings);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    const def = { volume: 60, brightness: 80, lang: 0, region: 0, wifiOn: true, btOn: false, notifications: true, pin: "" };
    setSettings((s) => ({ ...def, ...s }));
  }, []);

  const update = (key, value) => { setSettings((p) => { const n = { ...p, [key]: value }; saveSettings(n); return n; }); setSaved(true); setTimeout(() => setSaved(false), 1500); };

  return (
    <div className="flex h-full flex-col bg-bg-0">
      {/* Header */}
      <div className="flex items-center gap-3" style={{ height: "56px", padding: "0 16px", background: "linear-gradient(180deg, var(--bg-1) 0%, var(--bg-0) 100%)", borderBottom: "1px solid var(--stroke-soft)" }}>
        <button onClick={onBack} className="flex h-9 w-9 items-center justify-center rounded-xl" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", cursor: "pointer" }}>
          <ArrowLeft size={16} style={{ color: "var(--text-secondary)" }} />
        </button>
        <div className="flex-1"><h1 className="font-display text-base font-bold text-text-primary">Definições</h1></div>
        {saved && <span className="text-xs font-medium animate-fade-in" style={{ color: "var(--success)" }}>✓ Guardado</span>}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {/* Connections */}
        <SettingsCard icon={Wifi} title="Ligações" desc="Wi-Fi, Bluetooth" color="var(--accent-blue)">
          <div className="mt-3 space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2"><Wifi size={14} style={{ color: "var(--accent-blue)" }} /><span className="text-xs text-text-primary">Wi-Fi</span></div>
              <Toggle checked={settings.wifiOn} onChange={(v) => update("wifiOn", v)} color="var(--accent-blue)" />
            </div>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2"><Bluetooth size={14} style={{ color: "var(--accent)" }} /><span className="text-xs text-text-primary">Bluetooth</span></div>
              <Toggle checked={settings.btOn} onChange={(v) => update("btOn", v)} />
            </div>
          </div>
        </SettingsCard>

        {/* Security */}
        <SettingsCard icon={Shield} title="Segurança" desc={settings.pin ? "PIN definido" : "Sem PIN"} color="var(--accent-violet)" />

        {/* Region */}
        <SettingsCard icon={Globe} title="Região e Idioma" desc={LANGS[settings.lang]} color="var(--accent-cyan)">
          <div className="mt-3 space-y-2">
            <select value={settings.lang} onChange={(e) => update("lang", Number(e.target.value))} className="w-full rounded-xl px-3 py-2 text-sm" style={{ background: "var(--bg-0)", border: "1px solid var(--stroke-soft)", color: "var(--text-primary)", outline: "none" }}>
              {LANGS.map((l, i) => <option key={i} value={i} style={{ background: "var(--bg-1)" }}>{l}</option>)}
            </select>
          </div>
        </SettingsCard>

        {/* Sound & Display */}
        <SettingsCard icon={Sliders} title="Som e Ecrã" desc="Volume, brilho" color="var(--accent)">
          <div className="mt-3 space-y-2">
            <SliderRow icon={Volume2} label="Volume" value={settings.volume} onChange={(v) => update("volume", v)} color="var(--accent)" />
            <SliderRow icon={Sun} label="Brilho" value={settings.brightness} onChange={(v) => update("brightness", v)} color="var(--warning)" />
          </div>
        </SettingsCard>

        {/* System */}
        <SettingsCard icon={Cpu} title="Sistema" desc={api.baseUrl} color="var(--accent-pink)">
          <div className="mt-3 flex gap-2">
            <button className="flex-1 flex items-center justify-center gap-2 rounded-xl py-2.5" style={{ background: "var(--danger-tint)", border: "1px solid var(--danger)/30", cursor: "pointer", fontSize: "11px", color: "var(--danger)" }}><Power size={14} />Desligar</button>
            <button className="flex-1 flex items-center justify-center gap-2 rounded-xl py-2.5" style={{ background: "var(--panel-soft)", border: "1px solid var(--warning)/30", cursor: "pointer", fontSize: "11px", color: "var(--warning)" }}><RotateCw size={14} />Repor</button>
          </div>
        </SettingsCard>
      </div>
    </div>
  );
}
