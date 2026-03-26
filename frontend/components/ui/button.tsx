import { clsx } from 'clsx';
import { ButtonHTMLAttributes } from 'react';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md';
}

const variants = {
  primary:
    'border border-transparent text-white hover:brightness-110',
  secondary:
    'border border-[color:var(--line-strong)] bg-[var(--surface)] text-[color:var(--ink-strong)] hover:bg-[var(--surface-hover)] hover:border-[color:var(--accent)]',
  danger:
    'border border-transparent text-white hover:brightness-110',
  ghost:
    'border border-transparent bg-transparent text-[color:var(--ink-muted)] hover:bg-[var(--surface-hover)] hover:text-[color:var(--ink-strong)]',
};

const sizes = {
  sm: 'h-8 px-3 text-xs',
  md: 'h-9 px-4 text-sm',
};

// Variant-specific inline styles that use CSS vars (gradients can't be expressed as Tailwind classes)
const variantStyles: Record<string, React.CSSProperties> = {
  primary: {
    background: 'var(--btn-primary-bg)',
    boxShadow: 'var(--btn-primary-shadow)',
  },
  danger: {
    background: 'var(--btn-danger-bg)',
    boxShadow: 'var(--btn-danger-shadow)',
  },
};

export function Button({ variant = 'primary', size = 'md', className, style, ...props }: ButtonProps) {
  return (
    <button
      className={clsx(
        'inline-flex items-center justify-center gap-1.5 rounded-xl font-semibold transition-all duration-200 disabled:cursor-not-allowed disabled:opacity-40',
        variants[variant],
        sizes[size],
        className,
      )}
      style={{ ...variantStyles[variant], ...style }}
      {...props}
    />
  );
}
