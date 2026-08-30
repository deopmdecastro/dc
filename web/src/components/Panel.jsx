export default function Panel({ title, eyebrow, action, children, className = "" }) {
  return (
    <div
      className={`rounded-lg border border-stroke-soft bg-panel p-5 ${className}`}
      style={{ borderRadius: "var(--radius-lg)" }}
    >
      {(title || action) && (
        <div className="mb-4 flex items-center justify-between">
          <div>
            {eyebrow && (
              <p className="mb-1 text-[11px] font-semibold uppercase tracking-wider text-text-muted">
                {eyebrow}
              </p>
            )}
            {title && <h2 className="font-display text-base font-semibold text-text-primary">{title}</h2>}
          </div>
          {action}
        </div>
      )}
      {children}
    </div>
  );
}
