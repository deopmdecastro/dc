import { useState, useEffect } from "react";
import { ArrowLeft, Bell, BellOff, ChevronUp, ChevronDown, Plus, Trash2 } from "lucide-react";

const STORAGE_KEY = "dcos.alarms.web";

function loadAlarms() {
  try { return JSON.parse(localStorage.getItem(STORAGE_KEY) || "[]"); }
  catch { return [{ id: "default", time: "07:00", enabled: true, days: [1, 2, 3, 4, 5] }]; }
}
function saveAlarms(alarms) { localStorage.setItem(STORAGE_KEY, JSON.stringify(alarms)); }

function Toggle({ checked, onChange, color = "var(--accent-pink)" }) {
  return (
    <button onClick={() => onChange(!checked)} style={{ width: "52px", height: "28px", borderRadius: "14px", background: checked ? color : "var(--bg-2)", border: `1.5px solid ${checked ? color : "var(--stroke)"}`, position: "relative", cursor: "pointer", transition: "all 150ms ease", padding: 0 }}>
      <div style={{ position: "absolute", top: "3px", left: checked ? "26px" : "3px", width: "20px", height: "20px", borderRadius: "10px", background: checked ? "var(--bg-0)" : "var(--text-secondary)", transition: "all 150ms ease" }} />
    </button>
  );
}

const DAYS = ["D", "S", "T", "Q", "Q", "S", "S"];

export default function Alarm({ onBack }) {
  const [alarms, setAlarms] = useState(loadAlarms);
  const [editingId, setEditingId] = useState(null);

  useEffect(() => { saveAlarms(alarms); }, [alarms]);

  const toggleAlarm = (id) => setAlarms((p) => p.map((a) => (a.id === id ? { ...a, enabled: !a.enabled } : a)));
  const adjustTime = (id, deltaMin) => {
    setAlarms((p) => p.map((a) => { if (a.id !== id) return a; const [h, m] = a.time.split(":").map(Number); let t = ((h * 60 + m + deltaMin) % (24 * 60) + 24 * 60) % (24 * 60); return { ...a, time: `${String(Math.floor(t / 60)).padStart(2, "0")}:${String(t % 60).padStart(2, "0")}` }; }));
  };
  const addAlarm = () => { const n = { id: crypto.randomUUID(), time: "08:00", enabled: true, days: [1, 2, 3, 4, 5] }; setAlarms((p) => [...p, n]); setEditingId(n.id); };
  const removeAlarm = (id) => setAlarms((p) => p.filter((a) => a.id !== id));
  const toggleDay = (id, d) => setAlarms((p) => p.map((a) => a.id === id ? { ...a, days: a.days.includes(d) ? a.days.filter((x) => x !== d) : [...a.days, d].sort() } : a));

  return (
    <div className="flex h-full flex-col bg-bg-0">
      {/* Header */}
      <div className="flex items-center gap-3" style={{ height: "56px", padding: "0 16px", background: "linear-gradient(180deg, var(--bg-1) 0%, var(--bg-0) 100%)", borderBottom: "1px solid var(--stroke-soft)" }}>
        <button onClick={onBack} className="flex h-9 w-9 items-center justify-center rounded-xl" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", cursor: "pointer" }}>
          <ArrowLeft size={16} style={{ color: "var(--text-secondary)" }} />
        </button>
        <div className="flex-1"><h1 className="font-display text-base font-bold text-text-primary">Alarme</h1></div>
        <button onClick={addAlarm} className="flex h-9 w-9 items-center justify-center rounded-xl" style={{ background: "linear-gradient(135deg, var(--accent-pink), #ec4899)", border: "none", cursor: "pointer" }}>
          <Plus size={16} style={{ color: "var(--bg-0)" }} />
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {alarms.length === 0 ? (
          <div className="flex items-center justify-center h-full"><p className="text-sm text-text-muted">Sem alarmes</p></div>
        ) : alarms.map((alarm) => (
          <div key={alarm.id} className="rounded-2xl p-4 transition-all" style={{ background: alarm.enabled ? "linear-gradient(135deg, var(--accent-pink)/10, var(--panel-soft))" : "var(--panel-soft)", border: `1px solid ${alarm.enabled ? "var(--accent-pink)/30" : "var(--stroke-soft)"}` }}>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="flex h-11 w-11 items-center justify-center rounded-xl" style={{ background: alarm.enabled ? "linear-gradient(135deg, var(--accent-pink), #ec4899)" : "var(--bg-2)" }}>
                  {alarm.enabled ? <Bell size={18} style={{ color: "var(--bg-0)" }} /> : <BellOff size={18} style={{ color: "var(--text-dim)" }} />}
                </div>
                <span className="font-mono text-3xl font-extrabold" style={{ color: alarm.enabled ? "var(--text-primary)" : "var(--text-dim)" }}>{alarm.time}</span>
              </div>
              <Toggle checked={alarm.enabled} onChange={() => toggleAlarm(alarm.id)} />
            </div>

            {editingId === alarm.id && (
              <div className="flex justify-between mt-3 animate-fade-in">
                <div className="flex gap-2">
                  <button onClick={() => adjustTime(alarm.id, -60)} className="flex h-8 items-center gap-1 rounded-lg px-3" style={{ background: "var(--bg-2)", cursor: "pointer", fontSize: "11px", color: "var(--text-secondary)" }}><ChevronDown size={12} />H</button>
                  <button onClick={() => adjustTime(alarm.id, 60)} className="flex h-8 items-center gap-1 rounded-lg px-3" style={{ background: "var(--accent-deep)", border: "1px solid var(--accent)", cursor: "pointer", fontSize: "11px", color: "var(--accent-hi)" }}><ChevronUp size={12} />H</button>
                </div>
                <div className="flex gap-2">
                  <button onClick={() => adjustTime(alarm.id, -5)} className="flex h-8 items-center gap-1 rounded-lg px-3" style={{ background: "var(--bg-2)", cursor: "pointer", fontSize: "11px", color: "var(--text-secondary)" }}><ChevronDown size={12} />M</button>
                  <button onClick={() => adjustTime(alarm.id, 5)} className="flex h-8 items-center gap-1 rounded-lg px-3" style={{ background: "var(--accent-deep)", border: "1px solid var(--accent)", cursor: "pointer", fontSize: "11px", color: "var(--accent-hi)" }}><ChevronUp size={12} />M</button>
                </div>
              </div>
            )}

            <div className="flex items-center justify-between mt-3">
              <div className="flex gap-1">
                {DAYS.map((d, i) => (
                  <button key={i} onClick={() => toggleDay(alarm.id, i)} className="flex h-7 w-7 items-center justify-center rounded-full" style={{ background: alarm.days.includes(i) ? "var(--accent-pink)" : "var(--bg-2)", cursor: "pointer", fontSize: "10px", fontWeight: 600, color: alarm.days.includes(i) ? "var(--bg-0)" : "var(--text-muted)" }}>{d}</button>
                ))}
              </div>
              <div className="flex gap-2">
                <button onClick={() => setEditingId(editingId === alarm.id ? null : alarm.id)} className="text-[10px]" style={{ color: "var(--text-dim)", background: "transparent", border: "none", cursor: "pointer" }}>{editingId === alarm.id ? "OK" : "Editar"}</button>
                <button onClick={() => removeAlarm(alarm.id)} className="flex h-6 w-6 items-center justify-center rounded-md" style={{ background: "var(--danger-tint)", cursor: "pointer" }}><Trash2 size={10} style={{ color: "var(--danger)" }} /></button>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
