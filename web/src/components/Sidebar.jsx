import { NavLink } from "react-router-dom";
import {
  LayoutGrid,
  MessageCircle,
  Cloud,
  FileText,
  Bell,
  Settings,
  Zap,
} from "lucide-react";

const SpotifyIcon = (props) => (
  <svg viewBox="0 0 24 24" width={18} height={18} fill="currentColor" {...props}>
    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.586 14.424a.622.622 0 0 1-.857.207c-2.348-1.435-5.304-1.76-8.785-.964a.622.622 0 1 1-.277-1.215c3.809-.871 7.077-.496 9.713 1.115a.623.623 0 0 1 .206.857zm1.223-2.723a.78.78 0 0 1-1.072.257c-2.687-1.652-6.785-2.131-9.965-1.166a.78.78 0 1 1-.453-1.492c3.635-1.102 8.147-.568 11.233 1.329a.78.78 0 0 1 .257 1.072zm.105-2.835C14.692 9.128 9.375 8.95 6.297 9.883a.936.936 0 1 1-.543-1.79c3.533-1.072 9.404-.865 13.115 1.339a.936.936 0 0 1-.955 1.612z" />
  </svg>
);

const items = [
  { to: "/", label: "Início", icon: LayoutGrid, color: "text-accent" },
  { to: "/assistente", label: "Assistente", icon: MessageCircle, color: "text-accent-blue" },
  { to: "/spotify", label: "Spotify", icon: SpotifyIcon, color: "text-success" },
  { to: "/clima", label: "Clima", icon: Cloud, color: "text-accent-cyan" },
  { to: "/recursos", label: "Recursos", icon: Zap, color: "text-accent-violet" },
  { to: "/notas", label: "Notas", icon: FileText, color: "text-warning" },
  { to: "/alarme", label: "Alarme", icon: Bell, color: "text-accent-pink" },
  { to: "/definicoes", label: "Definições", icon: Settings, color: "text-text-secondary" },
];

export default function Sidebar({ open = false, onClose = () => {} }) {
  return (
    <>
      {/* Backdrop — só em mobile, quando a drawer está aberta */}
      {open && (
        <div
          onClick={onClose}
          className="fixed inset-0 z-30 bg-bg-0/70 backdrop-blur-sm md:hidden"
          aria-hidden="true"
        />
      )}

      <nav
        className={[
          "fixed inset-y-0 left-0 z-40 flex w-[240px] shrink-0 flex-col gap-1 border-r border-stroke-soft bg-bg-1 p-3 transition-transform duration-200 ease-out",
          "md:static md:z-auto md:w-[220px] md:translate-x-0 md:bg-bg-1/60",
          open ? "translate-x-0" : "-translate-x-full",
        ].join(" ")}
      >
        {items.map(({ to, label, icon: Icon, color }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            onClick={onClose}
            className={({ isActive }) =>
              [
                "group flex items-center gap-3 rounded-m px-3 py-2.5 text-sm transition-colors",
                isActive
                  ? "bg-panel-elevated text-text-primary"
                  : "text-text-secondary hover:bg-panel hover:text-text-primary",
              ].join(" ")
            }
            style={{ borderRadius: "var(--radius-m)" }}
          >
            {({ isActive }) => (
              <>
                <span
                  className={[
                    "flex h-8 w-8 items-center justify-center rounded-s border transition-colors",
                    isActive ? "border-current bg-panel-elevated" : "border-stroke-soft bg-panel",
                    color,
                  ].join(" ")}
                  style={{ borderRadius: "var(--radius-s)" }}
                >
                  <Icon size={17} />
                </span>
                <span className="font-medium">{label}</span>
              </>
            )}
          </NavLink>
        ))}

        <div
          className="mt-auto rounded-m border border-stroke-soft bg-panel/60 p-3 text-xs text-text-muted"
          style={{ borderRadius: "var(--radius-m)" }}
        >
          <p className="font-semibold text-text-secondary">ES3C28P</p>
          <p>ESP32-S3 · 320×240 IPS</p>
        </div>
      </nav>
    </>
  );
}
