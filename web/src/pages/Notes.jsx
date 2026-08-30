import { useEffect, useState } from "react";
import { Plus, Trash2 } from "lucide-react";
import Panel from "../components/Panel";

const STORAGE_KEY = "dcos.notes";

export default function Notes() {
  const [notes, setNotes] = useState([]);
  const [draft, setDraft] = useState("");

  useEffect(() => {
    try {
      const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) || "[]");
      setNotes(saved);
    } catch {
      setNotes([]);
    }
  }, []);

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(notes));
  }, [notes]);

  const addNote = () => {
    if (!draft.trim()) return;
    setNotes([{ id: crypto.randomUUID(), text: draft.trim(), created: Date.now() }, ...notes]);
    setDraft("");
  };

  const removeNote = (id) => setNotes(notes.filter((n) => n.id !== id));

  return (
    <div className="space-y-6">
      <Panel eyebrow="Notas" title="Novo apontamento">
        <div className="flex gap-2">
          <input
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && addNote()}
            placeholder="Escreve uma nota rápida…"
            className="flex-1 rounded-s border border-stroke-soft bg-panel-soft px-3 py-2 text-sm text-text-primary outline-none placeholder:text-text-dim focus:border-accent"
            style={{ borderRadius: "var(--radius-s)" }}
          />
          <button
            onClick={addNote}
            className="flex items-center gap-1.5 rounded-s bg-accent px-3 py-2 text-sm font-medium text-bg-0 transition-opacity hover:opacity-90"
            style={{ borderRadius: "var(--radius-s)" }}
          >
            <Plus size={16} /> Adicionar
          </button>
        </div>
      </Panel>

      <Panel eyebrow={`${notes.length} nota${notes.length === 1 ? "" : "s"}`} title="As tuas notas">
        {notes.length === 0 ? (
          <p className="text-sm text-text-muted">Ainda não tens notas. Escreve a primeira acima.</p>
        ) : (
          <ul className="space-y-2">
            {notes.map((n) => (
              <li
                key={n.id}
                className="flex items-start justify-between gap-3 rounded-m border border-stroke-soft bg-panel-soft p-3"
                style={{ borderRadius: "var(--radius-m)" }}
              >
                <div>
                  <p className="text-sm text-text-primary">{n.text}</p>
                  <p className="mt-1 text-[11px] text-text-dim">
                    {new Date(n.created).toLocaleString("pt-PT")}
                  </p>
                </div>
                <button
                  onClick={() => removeNote(n.id)}
                  className="shrink-0 text-text-dim transition-colors hover:text-danger"
                >
                  <Trash2 size={15} />
                </button>
              </li>
            ))}
          </ul>
        )}
      </Panel>
    </div>
  );
}
