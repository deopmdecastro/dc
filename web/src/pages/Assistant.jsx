import { useEffect, useRef, useState } from "react";
import { Send, Sparkles } from "lucide-react";
import Panel from "../components/Panel";

const WELCOME = {
  role: "assistant",
  text: "Olá! Sou o DC Assistant. Esta janela de chat está pronta para ligar ao teu backend de voz/LLM — por agora funciona em modo local.",
};

export default function Assistant() {
  const [messages, setMessages] = useState([WELCOME]);
  const [input, setInput] = useState("");
  const bottomRef = useRef(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const send = () => {
    const text = input.trim();
    if (!text) return;
    setMessages((m) => [
      ...m,
      { role: "user", text },
      {
        role: "assistant",
        text: "Ainda não há um endpoint de chat no dc-os-core — liga /voice/transcribe ou o teu LLM aqui para respostas reais.",
      },
    ]);
    setInput("");
  };

  return (
    <div className="flex h-full flex-col">
      <Panel eyebrow="Assistente" title="Conversa" className="flex flex-1 flex-col overflow-hidden">
        <div className="flex-1 space-y-3 overflow-y-auto pr-1">
          {messages.map((m, i) => (
            <div key={i} className={`flex ${m.role === "user" ? "justify-end" : "justify-start"}`}>
              <div
                className={`max-w-[75%] rounded-m px-3.5 py-2.5 text-sm ${
                  m.role === "user"
                    ? "bg-accent text-bg-0"
                    : "border border-stroke-soft bg-panel-soft text-text-primary"
                }`}
                style={{ borderRadius: "var(--radius-m)" }}
              >
                {m.role === "assistant" && (
                  <div className="mb-1 flex items-center gap-1.5 text-[11px] font-semibold text-accent-hi">
                    <Sparkles size={12} /> DC Assistant
                  </div>
                )}
                {m.text}
              </div>
            </div>
          ))}
          <div ref={bottomRef} />
        </div>

        <div className="mt-4 flex gap-2">
          <input
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && send()}
            placeholder="Escreve uma mensagem…"
            className="flex-1 rounded-s border border-stroke-soft bg-panel-soft px-3 py-2.5 text-sm text-text-primary outline-none placeholder:text-text-dim focus:border-accent"
            style={{ borderRadius: "var(--radius-s)" }}
          />
          <button
            onClick={send}
            className="flex h-10 w-10 items-center justify-center rounded-full bg-accent-blue text-bg-0 transition-opacity hover:opacity-90"
          >
            <Send size={16} />
          </button>
        </div>
      </Panel>
    </div>
  );
}
