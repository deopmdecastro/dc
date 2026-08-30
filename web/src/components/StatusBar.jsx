import { useEffect, useState } from "react";
import { Wifi, WifiOff, BatteryFull, Menu } from "lucide-react";
import { api } from "../lib/api";

export default function StatusBar({ online, onMenuClick }) {
  const [now, setNow] = useState(new Date());

  useEffect(() => {
    const id = setInterval(() => setNow(new Date()), 1000 * 30);
    return () => clearInterval(id);
  }, []);

  const time = now.toLocaleTimeString("pt-PT", { hour: "2-digit", minute: "2-digit" });
  const date = now.toLocaleDateString("pt-PT", { weekday: "short", day: "numeric", month: "short" });

  return (
    <div className="flex items-center justify-between border-b border-stroke-soft bg-bg-1/80 px-4 py-3 backdrop-blur-sm sm:px-5">
      <div className="flex items-center gap-2">
        <button
          onClick={onMenuClick}
          className="mr-1 flex h-8 w-8 items-center justify-center rounded-s text-text-secondary hover:bg-panel hover:text-text-primary md:hidden"
          style={{ borderRadius: "var(--radius-s)" }}
          aria-label="Abrir menu"
        >
          <Menu size={18} />
        </button>
        <div className="h-2 w-2 rounded-full bg-accent shadow-[0_0_8px_2px_var(--color-accent)]" />
        <span className="font-display text-sm font-semibold tracking-wide text-text-primary">
          DC OS
        </span>
        <span className="hidden text-xs text-text-muted sm:inline">— DC Assistant</span>
      </div>

      <div className="flex items-center gap-4 text-text-secondary">
        <span className="hidden text-xs capitalize text-text-muted sm:inline">{date}</span>
        <span className="font-mono text-sm text-text-primary">{time}</span>
        <div className="flex items-center gap-1.5" title={online ? "Backend ligado" : "Backend offline"}>
          {online ? (
            <Wifi size={15} className="text-success" />
          ) : (
            <WifiOff size={15} className="text-danger" />
          )}
        </div>
        <BatteryFull size={17} className="text-text-secondary" />
      </div>
    </div>
  );
}
