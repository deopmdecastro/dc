import { useEffect, useState, useRef } from "react";
import { Wifi, WifiOff, Bluetooth, BatteryFull, Activity, ChevronDown } from "lucide-react";

export default function StatusBar({ online, onDragStart, onTap }) {
  const [now, setNow] = useState(new Date());
  const barRef = useRef(null);

  useEffect(() => {
    const id = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(id);
  }, []);

  const timeStr = now.toLocaleTimeString("pt-PT", { hour: "2-digit", minute: "2-digit" });
  const dateStr = now.toLocaleDateString("pt-PT", { weekday: "short", day: "numeric", month: "short" });

  return (
    <div
      ref={barRef}
      className="relative flex items-center justify-between px-4 py-2 cursor-pointer select-none"
      style={{
        background: "linear-gradient(180deg, var(--bg-1) 0%, rgba(15, 20, 28, 0.95) 100%)",
        borderBottom: "1px solid var(--stroke-soft)",
        backdropFilter: "blur(12px)",
      }}
      onClick={onTap}
      onMouseDown={(e) => onDragStart(e.clientY)}
      onTouchStart={(e) => onDragStart(e.touches[0].clientY)}
    >
      {/* Left: Logo + time */}
      <div className="flex items-center gap-3">
        <div
          className="flex h-7 w-7 items-center justify-center"
          style={{
            background: "linear-gradient(135deg, var(--accent), var(--accent-blue))",
            borderRadius: "var(--radius-s)",
          }}
        >
          <span style={{ fontSize: "10px", fontWeight: 800, color: "var(--bg-0)" }}>DC</span>
        </div>
        <div className="flex flex-col">
          <span className="font-display text-sm font-bold text-text-primary">{timeStr}</span>
          <span className="text-[10px] capitalize text-text-muted">{dateStr}</span>
        </div>
      </div>

      {/* Center: Drag hint */}
      <div className="absolute left-1/2 -translate-x-1/2 flex flex-col items-center gap-0.5 opacity-40">
        <div style={{ width: "32px", height: "3px", borderRadius: "2px", background: "var(--stroke)" }} />
      </div>

      {/* Right: Status icons */}
      <div className="flex items-center gap-3">
        {/* API status */}
        <div className="flex items-center gap-1.5">
          <Activity size={12} style={{ color: online ? "var(--success)" : "var(--danger)" }} />
          <span className="text-[10px] font-medium" style={{ color: online ? "var(--success)" : "var(--danger)" }}>
            {online ? "Online" : "Offline"}
          </span>
        </div>

        {/* Wifi */}
        <Wifi size={15} style={{ color: online ? "var(--accent-blue)" : "var(--text-dim)" }} />

        {/* Bluetooth */}
        <Bluetooth size={14} style={{ color: "var(--accent)", opacity: 0.5 }} />

        {/* Battery */}
        <div className="flex items-center gap-1">
          <BatteryFull size={16} style={{ color: "var(--success)" }} />
          <span className="text-[10px] text-text-muted">100%</span>
        </div>
      </div>
    </div>
  );
}
