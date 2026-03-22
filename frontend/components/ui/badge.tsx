import { clsx } from 'clsx';

interface BadgeProps {
  label: string;
  variant?: 'default' | 'success' | 'warning' | 'error' | 'info';
}

const variants = {
  default:  'border-[color:var(--line-strong)] bg-[var(--surface)] text-[color:var(--ink-muted)]',
  success:  'border-[color:var(--success-soft)] bg-[var(--success-soft)] text-[color:var(--success)]',
  warning:  'border-[color:var(--warn-soft)] bg-[var(--warn-soft)] text-[color:var(--warn)]',
  error:    'border-[color:var(--danger-soft)] bg-[var(--danger-soft)] text-[color:var(--danger)]',
  info:     'border-[color:var(--info-soft)] bg-[var(--info-soft)] text-[color:var(--info)]',
};

export function Badge({ label, variant = 'default' }: BadgeProps) {
  return (
    <span
      className={clsx(
        'inline-flex items-center rounded-full border px-2.5 py-0.5 text-[0.68rem] font-semibold uppercase tracking-[0.06em]',
        variants[variant],
      )}
    >
      {label}
    </span>
  );
}
