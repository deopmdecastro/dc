import { useState, useEffect, useCallback } from "react";
import { ArrowLeft, Plus, Trash2, FileText, WifiOff, Loader2, X } from "lucide-react";
import { api } from "../lib/api";

const FALLBACK_KEY = "dcos.notes.fallback";

function loadFallback() {
  try { return JSON.parse(localStorage.getItem(FALLBACK_KEY) || "[]"); } catch { return []; }
}
function saveFallback(notes) { localStorage.setItem(FALLBACK_KEY, JSON.stringify(notes)); }

export default function Notes({ onBack }) {
  const [notes, setNotes] = useState([]);
  const [loading, setLoading] = useState(true);
  const [online, setOnline] = useState(true);
  const [showEditor, setShowEditor] = useState(false);
  const [draft, setDraft] = useState("");
  const [toast, setToast] = useState(null);

  const showToast = useCallback((text) => { setToast(text); setTimeout(() => setToast(null), 2000); }, []);

  const fetchNotes = useCallback(async () => {
    try {
      const data = await api.notes();
      const remoteNotes = (Array.isArray(data) ? data : data.notes || []).map((n) => ({
        id: n.id ?? n._id ?? crypto.randomUUID(), text: n.text ?? n.content ?? "", created: n.created_at ?? n.created ?? Date.now(),
      }));
      setNotes(remoteNotes); setOnline(true); saveFallback(remoteNotes);
    } catch { setOnline(false); setNotes(loadFallback()); } finally { setLoading(false); }
  }, []);

  useEffect(() => { fetchNotes(); }, [fetchNotes]);

  const addNote = async () => {
    if (!draft.trim()) return;
    const tempId = crypto.randomUUID();
    const newNote = { id: tempId, text: draft.trim(), created: Date.now() };
    setNotes((prev) => [newNote, ...prev]); setDraft(""); setShowEditor(false);
    if (!online) { saveFallback([newNote, ...notes]); return; }
    try { const saved = await api.createNote(newNote.text); setNotes((prev) => prev.map((n) => (n.id === tempId ? { ...n, id: saved?.id ?? saved?._id ?? tempId } : n))); }
    catch { showToast("Falha ao guardar"); }
  };

  const removeNote = async (id) => {
    setNotes((prev) => prev.filter((n) => n.id !== id));
    if (!online) { saveFallback(notes.filter((n) => n.id !== id)); return; }
    try { await api.deleteNote(id); } catch { showToast("Falha ao remover"); }
  };

  return (
    <div className="flex h-full flex-col bg-bg-0">
      {/* Header */}
      <div className="flex items-center gap-3" style={{ height: "56px", padding: "0 16px", background: "linear-gradient(180deg, var(--bg-1) 0%, var(--bg-0) 100%)", borderBottom: "1px solid var(--stroke-soft)" }}>
        <button onClick={onBack} className="flex h-9 w-9 items-center justify-center rounded-xl" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", cursor: "pointer" }}>
          <ArrowLeft size={16} style={{ color: "var(--text-secondary)" }} />
        </button>
        <div className="flex-1">
          <h1 className="font-display text-base font-bold text-text-primary">Notas</h1>
          <p className="text-[10px] text-text-muted">{notes.length} notas</p>
        </div>
        <button onClick={() => setShowEditor(true)} className="flex h-9 items-center gap-1.5 rounded-xl px-3" style={{ background: "linear-gradient(135deg, var(--warning), #f59e0b)", border: "none", cursor: "pointer", fontSize: "12px", fontWeight: 600, color: "var(--bg-0)" }}>
          <Plus size={14} /> Nova
        </button>
      </div>

      {/* Toast */}
      {toast && (
        <div className="absolute left-1/2 -translate-x-1/2 z-50 animate-slide-up" style={{ top: "64px", padding: "8px 16px", borderRadius: "12px", background: "var(--danger-tint)", border: "1px solid var(--danger)", fontSize: "12px", color: "var(--danger)" }}>
          {toast}
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4">
        {loading ? (
          <div className="flex items-center justify-center h-full"><Loader2 size={20} className="animate-spin" style={{ color: "var(--text-muted)" }} /></div>
        ) : notes.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full" style={{ gap: "12px" }}>
            <div className="flex h-16 w-16 items-center justify-center rounded-2xl" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)" }}>
              <FileText size={28} style={{ color: "var(--text-dim)" }} />
            </div>
            <p className="text-sm text-text-muted">Ainda não tens notas</p>
          </div>
        ) : (
          <div className="space-y-2">
            {notes.map((n) => (
              <div key={n.id} className="group flex items-start gap-3 rounded-2xl p-4 transition-all duration-150" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)" }}>
                <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl" style={{ background: "linear-gradient(135deg, var(--warning), #f59e0b)" }}>
                  <FileText size={16} style={{ color: "var(--bg-0)" }} />
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-text-primary break-words">{n.text}</p>
                  <p className="text-[10px] text-text-dim mt-1">{new Date(n.created).toLocaleString("pt-PT")}</p>
                </div>
                <button onClick={() => removeNote(n.id)} className="shrink-0 opacity-0 group-hover:opacity-100 transition-opacity flex h-8 w-8 items-center justify-center rounded-lg" style={{ background: "var(--danger-tint)", cursor: "pointer" }}>
                  <Trash2 size={14} style={{ color: "var(--danger)" }} />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Offline */}
      {!online && (
        <div className="flex items-center justify-center gap-1.5 py-2" style={{ background: "var(--warning-tint)", borderTop: "1px solid var(--warning)/30", fontSize: "10px", color: "var(--warning)" }}>
          <WifiOff size={10} /> Offline — notas guardadas localmente
        </div>
      )}

      {/* Editor modal */}
      {showEditor && (
        <div className="absolute inset-0 z-50 flex items-center justify-center p-4" style={{ background: "rgba(3, 5, 14, 0.8)", backdropFilter: "blur(10px)" }} onClick={() => setShowEditor(false)}>
          <div className="w-full max-w-md rounded-2xl p-6 animate-slide-up" style={{ background: "var(--bg-1)", border: "1px solid var(--stroke)", boxShadow: "0 25px 80px -20px rgba(0,0,0,0.6)" }} onClick={(e) => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-display text-base font-bold text-text-primary">Nova nota</h3>
              <button onClick={() => setShowEditor(false)} className="flex h-7 w-7 items-center justify-center rounded-lg" style={{ background: "var(--panel-soft)", cursor: "pointer" }}>
                <X size={14} style={{ color: "var(--text-muted)" }} />
              </button>
            </div>
            <textarea
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              placeholder="Escreve aqui…"
              autoFocus
              rows={3}
              className="w-full rounded-xl p-3 text-sm text-text-primary outline-none resize-none"
              style={{ background: "var(--bg-0)", border: "1px solid var(--stroke-soft)" }}
            />
            <div className="flex justify-end gap-2 mt-4">
              <button onClick={() => setShowEditor(false)} className="rounded-xl px-4 py-2 text-sm" style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)", cursor: "pointer", color: "var(--text-secondary)" }}>Cancelar</button>
              <button onClick={addNote} disabled={!draft.trim()} className="rounded-xl px-4 py-2 text-sm font-semibold disabled:opacity-40" style={{ background: "linear-gradient(135deg, var(--accent), var(--accent-blue))", border: "none", cursor: draft.trim() ? "pointer" : "default", color: "var(--bg-0)" }}>Guardar</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
