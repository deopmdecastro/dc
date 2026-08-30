import { useState, useRef, useEffect } from "react";
import { X, Wifi, Bluetooth, Moon, RotateCw, Volume2, Sun, Music, Cloud } from "lucide-react";

function ToggleTile({ icon: Icon, label, checked, onChange, color, bgActive }) {
  return (
    <div className="flex flex-col items-center gap-1.5">
      <button
        onClick={() => onChange(!checked)}
        className="flex items-center justify-center transition-all duration-150"
        style={{
          width: "64px",
          height: "56px",
          borderRadius: "14px",
          background: checked ? bgActive || color : "var(--panel-soft)",
          border: `1.5px solid ${checked ? color : "var(--stroke)"}`,
          boxShadow: checked ? `0 0 20px -5px ${color}` : "none",
        }}
      >
        <Icon size={22} style={{ color: checked ? "var(--bg-0)" : "var(--text-secondary)" }} />
      </button>
      <span className="text-[10px] font-medium" style={{ color: checked ? color : "var(--text-muted)" }}>
        {label}
      </span>
    </div>
  );
}

function SliderControl({ icon: Icon, value, onChange, color = "var(--accent)" }) {
  return (
    <div
      className="flex items-center gap-3"
      style={{
        width: "100%",
        height: "44px",
        borderRadius: "14px",
        background: "var(--panel-soft)",
        border: "1px solid var(--stroke-soft)",
        padding: "0 16px",
      }}
    >
      <Icon size={16} style={{ color }} />
      <span className="font-mono text-xs" style={{ color: "var(--text-muted)", width: "32px" }}>
        {value}%
      </span>
      <div className="relative flex-1" style={{ height: "6px", borderRadius: "3px", background: "var(--bg-2)" }}>
        <div
          style={{
            width: `${value}%`,
            height: "100%",
            borderRadius: "3px",
            background: `linear-gradient(90deg, ${color}, ${color}dd)`,
          }}
        />
        <input
          type="range"
          min={0}
          max={100}
          value={value}
          onChange={(e) => onChange(Number(e.target.value))}
          className="absolute inset-0 cursor-pointer opacity-0"
        />
      </div>
    </div>
  );
}

export default function ControlCenter({ open, dragging, dragY, onClose, onDragStart }) {
  const [wifiOn, setWifiOn] = useState(true);
  const [btOn, setBtOn] = useState(false);
  const [dndOn, setDndOn] = useState(false);
  const [rotateOn, setRotateOn] = useState(false);
  const [volume, setVolume] = useState(60);
  const [brightness, setBrightness] = useState(80);
  const panelRef = useRef(null);

  const now = new Date();
  const timeStr = now.toLocaleTimeString("pt-PT", { hour: "2-digit", minute: "2-digit" });

  if (!open && !dragging) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex flex-col"
      style={{
        background: "rgba(3, 5, 14, 0.85)",
        backdropFilter: "blur(20px)",
        opacity: open ? 1 : 0,
        transition: "opacity 0.2s ease",
      }}
      onClick={onClose}
    >
      {/* Drag handle area at top */}
      <div
        className="flex justify-center pt-3 pb-2 cursor-grab active:cursor-grabbing"
        onMouseDown={(e) => { e.stopPropagation(); onDragStart(e.clientY); }}
        onTouchStart={(e) => { e.stopPropagation(); onDragStart(e.touches[0].clientY); }}
      >
        <div style={{ width: "40px", height: "4px", borderRadius: "2px", background: "var(--stroke)" }} />
      </div>

      {/* Panel content */}
      <div
        ref={panelRef}
        className="mx-auto w-full max-w-lg flex-1 overflow-y-auto rounded-t-3xl p-6 animate-slide-up"
        style={{
          background: "linear-gradient(180deg, var(--bg-1) 0%, var(--bg-0) 100%)",
          border: "1px solid var(--stroke-soft)",
          borderBottom: "none",
          transform: dragging ? `translateY(${Math.min(dragY, 100)}px)` : "translateY(0)",
          transition: dragging ? "none" : "transform 0.3s ease-out",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <h2 className="font-display text-lg font-bold text-text-primary">Centro de Controlo</h2>
          <div className="flex items-center gap-4">
            <span className="font-mono text-sm font-bold" style={{ color: "var(--accent)" }}>{timeStr}</span>
            <button
              onClick={onClose}
              className="flex h-8 w-8 items-center justify-center rounded-full"
              style={{ background: "var(--panel-soft)", border: "1px solid var(--stroke-soft)" }}
            >
              <X size={14} style={{ color: "var(--text-muted)" }} />
            </button>
          </div>
        </div>

        {/* Toggle tiles */}
        <div className="grid grid-cols-4 gap-3 mb-6">
          <ToggleTile icon={Wifi} label="Wi-Fi" checked={wifiOn} onChange={setWifiOn} color="var(--accent-blue)" bgActive="linear-gradient(135deg, var(--accent-blue), #60a5fa)" />
          <ToggleTile icon={Bluetooth} label="Bluetooth" checked={btOn} onChange={setBtOn} color="var(--accent)" bgActive="linear-gradient(135deg, var(--accent), var(--accent-hi))" />
          <ToggleTile icon={Moon} label="DND" checked={dndOn} onChange={setDndOn} color="var(--accent-violet)" bgActive="linear-gradient(135deg, var(--accent-violet), #c4b5fd)" />
          <ToggleTile icon={RotateCw} label="Rotação" checked={rotateOn} onChange={setRotateOn} color="var(--accent-cyan)" bgActive="linear-gradient(135deg, var(--accent-cyan), #5eead4)" />
        </div>

        {/* Sliders */}
        <div className="space-y-3 mb-6">
          <SliderControl icon={Volume2} value={volume} onChange={setVolume} color="var(--accent)" />
          <SliderControl icon={Sun} value={brightness} onChange={setBrightness} color="var(--warning)" />
        </div>

        {/* Quick actions */}
        <div className="grid grid-cols-3 gap-3">
          <button
            className="flex flex-col items-center gap-2 rounded-2xl p-4 transition-all hover:scale-105"
            style={{ background: "linear-gradient(135deg, var(--success-tint), var(--panel-soft))", border: "1px solid var(--success)/30" }}
          >
            <Music size={20} style={{ color: "var(--success)" }} />
            <span className="text-[10px] font-medium text-text-secondary">Música</span>
          </button>
          <button
            className="flex flex-col items-center gap-2 rounded-2xl p-4 transition-all hover:scale-105"
            style={{ background: "linear-gradient(135deg, var(--accent-tint), var(--panel-soft))", border: "1px solid var(--accent)/30" }}
          >
            <Cloud size={20} style={{ color: "var(--accent)" }} />
            <span className="text-[10px] font-medium text-text-secondary">Clima</span>
          </button>
          <button
            className="flex flex-col items-center gap-2 rounded-2xl p-4 transition-all hover:scale-105"
            style={{ background: "linear-gradient(135deg, var(--danger-tint), var(--panel-soft))", border: "1px solid var(--danger)/30" }}
          >
            <X size={20} style={{ color: "var(--danger)" }} />
            <span className="text-[10px] font-medium text-text-secondary">Desligar</span>
          </button>
        </div>
      </div>
    </div>
  );
}
