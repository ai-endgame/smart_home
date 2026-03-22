interface EmptyStateProps {
  icon: string;
  title: string;
  subtitle?: string;
  action?: React.ReactNode;
}

export function EmptyState({ icon, title, subtitle, action }: EmptyStateProps) {
  return (
    <div className="surface-card flex flex-col items-center gap-4 px-8 py-16 text-center">
      <span
        className="flex h-16 w-16 items-center justify-center rounded-2xl text-3xl"
        style={{ background: 'var(--surface-hover)' }}
        aria-hidden
      >
        {icon}
      </span>
      <div className="space-y-1.5">
        <p className="font-semibold text-[color:var(--ink-strong)]">{title}</p>
        {subtitle && (
          <p className="max-w-xs text-sm text-[color:var(--ink-muted)]">{subtitle}</p>
        )}
      </div>
      {action && <div className="mt-2">{action}</div>}
    </div>
  );
}
