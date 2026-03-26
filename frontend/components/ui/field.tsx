interface FieldProps {
  label: string;
  hint?: string;
  error?: string;
  children: React.ReactNode;
  className?: string;
}

/**
 * Form field wrapper with an optional inline hint or inline error rendered below the input.
 * When `error` is set it takes priority over `hint` and is rendered in red.
 */
export function Field({ label, hint, error, children, className }: FieldProps) {
  return (
    <div className={className}>
      <label className="mb-1.5 block text-xs font-semibold uppercase tracking-[0.08em] text-[color:var(--ink-muted)]">
        {label}
      </label>
      {children}
      {error ? (
        <p className="mt-1.5 flex items-start gap-1.5 text-[0.74rem] leading-snug text-[#fb7185]">
          <span className="mt-px shrink-0 opacity-70">↳</span>
          {error}
        </p>
      ) : hint ? (
        <p className="mt-1.5 flex items-start gap-1.5 text-[0.74rem] leading-snug text-[color:var(--ink-faint)]">
          <span className="mt-px shrink-0 text-[color:var(--accent)] opacity-70">↳</span>
          {hint}
        </p>
      ) : null}
    </div>
  );
}
