import { useEffect, useState } from "react";
import { Bell, BellOff, Trash2 } from "lucide-react";
import Panel from "../components/Panel";

const STORAGE_KEY = "dcos.alarms";
const DAYS = ["D", "S", "T", "Q", "Q", "S", "S"];

export default function Alarm() {
  const [alarms, setAlarms] = useState([]);
  const [time, setTime] = useState("07:30");

  useEffect(() => {
    try {
      setAlarms(JSON.parse(localStorage.getItem(STORAGE_KEY) || "[]"));
    } catch {
      setAlarms([]);
    }
  }, []);

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(alarms));
  }, [alarms]);

  const addAlarm = () => {
    setAlarms([...alarms, { id: crypto.randomUUID(), time, enabled: true, days: [1, 2, 3, 4, 5] }]);
  };

  const toggle = (id) =>
    setAlarms(alarms.map((a) => (a.id === id ? { ...a, enabled: !a.enabled } : a)));

  const remove = (id) => setAlarms(alarms.filter((a) => a.id !== id));

  const toggleDay = (id, dayIdx) =>
    setAlarms(
      alarms.map((a) =>
        a.id === id
          ? {
              ...a,
              days: a.days.includes(dayIdx) ? a.days.filter((d) => d !== dayIdx) : [...a.days, dayIdx].sort(),
            }
          : a
      )
    );

  return (
    <div className="space-y-6">
      <Panel eyebrow="Alarme" title="Adicionar alarme">
        <div className="flex flex-wrap items-center gap-3">
          <input
            type="time"
            value={time}
            onChange={(e) => setTime(e.target.value)}
            className="rounded-s border border-stroke-soft bg-panel-soft px-3 py-2 text-sm text-text-primary outline-none focus:border-accent"
            style={{ borderRadius: "var(--radius-s)" }}
          />
          <button
            onClick={addAlarm}
            className="rounded-s bg-accent-pink px-4 py-2 text-sm font-medium text-bg-0 transition-opacity hover:opacity-90"
            style={{ borderRadius: "var(--radius-s)" }}
          >
            Adicionar alarme
          </button>
        </div>
      </Panel>

      <Panel eyebrow={`${alarms.length} alarme${alarms.length === 1 ? "" : "s"}`} title="Os teus alarmes">
        {alarms.length === 0 ? (
          <p className="text-sm text-text-muted">Sem alarmes configurados.</p>
        ) : (
          <ul className="space-y-3">
            {alarms
              .slice()
              .sort((a, b) => a.time.localeCompare(b.time))
              .map((a) => (
                <li
                  key={a.id}
                  className="flex flex-col gap-3 rounded-m border border-stroke-soft bg-panel-soft p-4 sm:flex-row sm:items-center sm:justify-between"
                  style={{ borderRadius: "var(--radius-m)" }}
                >
                  <div className="flex items-center gap-3">
                    <button
                      onClick={() => toggle(a.id)}
                      className={`flex h-9 w-9 items-center justify-center rounded-full ${
                        a.enabled ? "bg-accent-tint text-accent-pink" : "bg-panel-elevated text-text-dim"
                      }`}
                    >
                      {a.enabled ? <Bell size={16} /> : <BellOff size={16} />}
                    </button>
                    <span className={`font-mono text-2xl ${a.enabled ? "text-text-primary" : "text-text-dim"}`}>
                      {a.time}
                    </span>
                  </div>
                  <div className="flex items-center gap-4">
                    <div className="flex gap-1">
                      {DAYS.map((d, i) => (
                        <button
                          key={i}
                          onClick={() => toggleDay(a.id, i)}
                          className={`flex h-7 w-7 items-center justify-center rounded-full text-[11px] font-semibold transition-colors ${
                            a.days.includes(i)
                              ? "bg-accent-pink text-bg-0"
                              : "bg-panel-elevated text-text-muted"
                          }`}
                        >
                          {d}
                        </button>
                      ))}
                    </div>
                    <button onClick={() => remove(a.id)} className="text-text-dim hover:text-danger">
                      <Trash2 size={16} />
                    </button>
                  </div>
                </li>
              ))}
          </ul>
        )}
      </Panel>
    </div>
  );
}
