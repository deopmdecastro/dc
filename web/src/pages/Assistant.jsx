import { useState, useRef, useEffect, useCallback } from "react";
import { Mic, ArrowLeft, Volume2, Loader2, Sparkles, Trash2 } from "lucide-react";
import { api } from "../lib/api";

function AudioVisualizer({ state }) {
  const bars = 40;
  const [heights, setHeights] = useState(Array(bars).fill(6));

  useEffect(() => {
    if (state === "listening" || state === "speaking") {
      const interval = setInterval(() => { setHeights((prev) => prev.map(() => Math.random() * 32 + 6)); }, 80);
      return () => clearInterval(interval);
    } else {
      setHeights(Array(bars).fill(6));
    }
  }, [state]);

  const barColor = state === "listening" ? "var(--accent)" : state === "speaking" ? "var(--success)" : "var(--stroke-soft)";

  return (
    <div className="flex items-end justify-center gap-[2px]" style={{ height: "40px" }}>
      {heights.map((h, i) => (
        <div key={i} style={{ width: "5px", height: `${h}px`, background: barColor, borderRadius: "2px", transition: "height 80ms ease-out" }} />
      ))}
    </div>
  );
}

export default function Assistant({ onBack }) {
  const [state, setState] = useState("idle");
  const [capturedText, setCapturedText] = useState("");
  const [messages, setMessages] = useState([]);
  const bottomRef = useRef(null);

  const processCommand = useCallback(async (text) => {
    const lower = text.toLowerCase();
    if (lower.includes("nota") || lower.includes("anota")) {
      const noteText = text.replace(/^(nota|anota|adiciona nota|cria nota)\s*/i, "").trim();
      if (noteText) { try { await api.createNote(noteText); return `✓ Nota: "${noteText}"`; } catch { return "✗ Falha ao criar nota"; } }
      return "O que queres anotar?";
    }
    if (lower.includes("clima") || lower.includes("tempo")) { try { const w = await api.weather(0); return `🌤️ ${w.city ?? ""}: ${Math.round(w.temperature ?? w.temp ?? 0)}°C`; } catch { return "✗ Sem dados de clima"; } }
    if (lower.includes("música") || lower.includes("musica")) { return "🎵 Usa o Spotify para controlar a música"; }
    if (lower.includes("hora") || lower.includes("horas")) { const now = new Date(); return `🕐 São ${now.getHours()}:${String(now.getMinutes()).padStart(2, "0")}.`; }
    return "Comandos: nota, clima, música, hora";
  }, []);

  const handleMic = useCallback(async () => {
    if (state === "listening") return;
    setState("listening"); setCapturedText(""); setMessages((m) => [...m, { role: "system", text: "A ouvir..." }]);
    setTimeout(async () => {
      const simText = "nota Teste do assistente";
      setCapturedText(simText); setState("speaking"); setMessages((m) => [...m, { role: "user", text: simText }]);
      const response = await processCommand(simText);
      setMessages((m) => [...m, { role: "assistant", text: response }]);
      setTimeout(() => { setState("idle"); setCapturedText(""); }, 2500);
    }, 3000);
  }, [state, processCommand]);

  useEffect(() => { bottomRef.current?.scrollIntoView({ behavior: "smooth" }); }, [messages]);

  const statusText = state === "listening" ? "A ouvir..." : state === "speaking" ? "A falar..." : "Assistente pessoal";
  const statusColor = state === "listening" ? "var(--accent)" : state === "speaking" ? "var(--success)" : "var(--text-secondary)";

  return (
    <div className="flex h-full flex-col bg-bg-0">
      {/* Header */}
      <div className="flex items-center gap-3" style={{ height: "56px", padding: "0 16px", background: "linear-gradient(180deg, var(--bg-1) 0%, var(--bg-0) 100%)", borderBottom: "1px solid var(--stroke-soft)" }}>
        <button onClick={onBack} className="flex h-9 w-9 items-center justify-center rounded-xl" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", cursor: "pointer" }}>
          <ArrowLeft size={16} style={{ color: "var(--text-secondary)" }} />
        </button>
        <div className="flex-1"><h1 className="font-display text-base font-bold text-text-primary">Assistente</h1></div>
        <button onClick={() => setMessages([])} className="flex h-9 w-9 items-center justify-center rounded-xl" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", cursor: "pointer" }}>
          <Trash2 size={14} style={{ color: "var(--text-muted)" }} />
        </button>
      </div>

      {/* Main content */}
      <div className="flex flex-1 flex-col items-center justify-center px-6">
        {/* DC OS badge */}
        <div className="flex items-center justify-center" style={{ width: "70px", height: "22px", borderRadius: "11px", background: state === "listening" ? "var(--accent-tint)" : "var(--panel-soft)", border: `1.5px solid ${state === "listening" ? "var(--accent)" : "var(--stroke)"}`, fontSize: "9px", fontWeight: 700, letterSpacing: "1px", color: state === "listening" ? "var(--accent-hi)" : "var(--text-muted)", marginBottom: "12px" }}>
          DC OS
        </div>

        {/* Title */}
        <span className="font-display text-2xl font-extrabold text-text-primary text-center mb-2">{statusText}</span>

        {/* Captured text */}
        {capturedText && (
          <div className="animate-fade-in" style={{ marginTop: "12px", padding: "8px 16px", borderRadius: "12px", background: state === "speaking" ? "var(--success-tint)" : "var(--panel-soft)", border: `1px solid ${state === "speaking" ? "var(--success)" : "var(--stroke)"}`, fontSize: "13px", fontWeight: 600, color: state === "speaking" ? "var(--success)" : "var(--text-primary)" }}>
            {capturedText}
          </div>
        )}

        {/* Visualizer */}
        <div className="w-full mt-4"><AudioVisualizer state={state} /></div>

        {/* Messages */}
        {messages.length > 0 && (
          <div className="w-full mt-4 max-h-32 overflow-y-auto space-y-2">
            {messages.slice(-3).map((m, i) => (
              <div key={i} className="text-xs" style={{ color: m.role === "user" ? "var(--accent-hi)" : m.role === "assistant" ? "var(--success)" : "var(--text-muted)", textAlign: m.role === "user" ? "right" : "left" }}>{m.text}</div>
            ))}
            <div ref={bottomRef} />
          </div>
        )}
      </div>

      {/* Mic button */}
      <div className="flex justify-center py-6">
        <button onClick={handleMic} disabled={state !== "idle"} className="flex items-center justify-center transition-all duration-200" style={{ width: "80px", height: "80px", borderRadius: "40px", background: state === "listening" ? "linear-gradient(135deg, var(--accent-deep), var(--accent))" : state === "speaking" ? "linear-gradient(135deg, var(--success), #22c55e)" : "linear-gradient(135deg, var(--accent-hi), var(--accent-blue))", border: state === "idle" ? "2px solid var(--accent)" : "2px solid transparent", cursor: state === "idle" ? "pointer" : "default", boxShadow: state === "idle" ? "0 8px 30px -5px var(--accent)" : "0 8px 30px -5px " + (state === "listening" ? "var(--accent)" : "var(--success)") }}>
          {state === "speaking" ? <Volume2 size={28} style={{ color: "var(--bg-0)" }} /> : state === "listening" ? <Loader2 size={28} className="animate-spin" style={{ color: "var(--bg-0)" }} /> : <Mic size={28} style={{ color: "var(--bg-0)" }} />}
        </button>
      </div>
    </div>
  );
}
