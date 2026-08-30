import { useState, useEffect } from "react";

export default function SplashScreen() {
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    const interval = setInterval(() => {
      setProgress((p) => (p >= 100 ? 0 : p + 2));
    }, 50);
    return () => clearInterval(interval);
  }, []);

  return (
    <div
      className="flex h-full flex-col items-center justify-center"
      style={{
        background: "radial-gradient(ellipse at center, var(--accent-deep) 0%, var(--bg-0) 70%)",
      }}
    >
      {/* Animated background orbs */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div
          className="absolute animate-pulse-slow"
          style={{
            width: "200px",
            height: "200px",
            borderRadius: "50%",
            background: "radial-gradient(circle, rgba(56, 189, 248, 0.15) 0%, transparent 70%)",
            top: "20%",
            left: "10%",
            filter: "blur(40px)",
          }}
        />
        <div
          className="absolute animate-pulse-slow"
          style={{
            width: "150px",
            height: "150px",
            borderRadius: "50%",
            background: "radial-gradient(circle, rgba(167, 139, 250, 0.12) 0%, transparent 70%)",
            bottom: "20%",
            right: "15%",
            filter: "blur(30px)",
            animationDelay: "1s",
          }}
        />
      </div>

      {/* Logo */}
      <img
        src="/logo.png"
        alt="DC OS"
        className="relative z-10"
        style={{ height: "60px", width: "auto", objectFit: "contain", marginBottom: "16px" }}
      />

      {/* Title */}
      <h1
        className="relative z-10 font-display font-extrabold tracking-wider"
        style={{ fontSize: "28px", color: "var(--text-primary)", marginBottom: "4px" }}
      >
        DC OS
      </h1>

      {/* Subtitle */}
      <p
        className="relative z-10 text-xs font-medium tracking-wide"
        style={{ color: "var(--accent-hi)", marginBottom: "24px" }}
      >
        Assistente Pessoal
      </p>

      {/* Progress bar */}
      <div
        className="relative z-10 overflow-hidden"
        style={{
          width: "160px",
          height: "4px",
          borderRadius: "2px",
          background: "var(--panel-soft)",
        }}
      >
        <div
          style={{
            width: `${progress}%`,
            height: "100%",
            background: "linear-gradient(90deg, var(--accent), var(--accent-hi))",
            borderRadius: "2px",
            transition: "width 50ms linear",
          }}
        />
        {/* Shimmer effect */}
        <div
          className="absolute inset-0"
          style={{
            background: "linear-gradient(90deg, transparent, rgba(255,255,255,0.3), transparent)",
            animation: "shimmer 2s infinite",
          }}
        />
      </div>

      {/* Version tag */}
      <span
        className="relative z-10 mt-4 text-[10px]"
        style={{ color: "var(--text-dim)" }}
      >
        v1.0.0 · ES3C28P
      </span>
    </div>
  );
}
