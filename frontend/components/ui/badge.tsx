import { clsx } from 'clsx';

interface BadgeProps {
  label: string;
  variant?: 'default' | 'success' | 'warning' | 'error' | 'info';
}

const variants = {
  default: 'border border-[color:var(--line)] bg-white/80 text-[color:var(--ink-muted)]',
  success: 'border border-[#8bdec0] bg-[#e8faf2] text-[#1d7355]',
  warning: 'border border-[#eccb84] bg-[#fff6df] text-[#8b5f10]',
  error: 'border border-[#e5a0a6] bg-[#feecee] text-[#8f2f38]',
  info: 'border border-[#9ec4e5] bg-[#ecf6ff] text-[#235988]',
};

export function Badge({ label, variant = 'default' }: BadgeProps) {
  return (
    <span
      className={clsx(
        'inline-flex items-center rounded-full px-2.5 py-1 text-[0.68rem] font-semibold uppercase tracking-[0.06em]',
        variants[variant],
      )}
    >
      {label}
    </span>
  );
}
