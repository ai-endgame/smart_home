import { clsx } from 'clsx';
import { ButtonHTMLAttributes } from 'react';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md';
}

const variants = {
  primary:
    'border border-transparent bg-[linear-gradient(160deg,#1f8f6a,#2c6da6)] text-white shadow-[0_10px_22px_rgba(31,109,84,0.34)] hover:brightness-105',
  secondary:
    'border border-[color:var(--line)] bg-white text-[color:var(--ink-strong)] hover:bg-[color:var(--bg-soft)]',
  danger:
    'border border-transparent bg-[linear-gradient(160deg,#b33f4a,#cf5d6b)] text-white shadow-[0_10px_20px_rgba(164,53,66,0.3)] hover:brightness-105',
  ghost:
    'border border-transparent bg-transparent text-[color:var(--ink-muted)] hover:bg-white/80 hover:text-[color:var(--ink-strong)]',
};

const sizes = {
  sm: 'h-8 px-3 text-xs',
  md: 'h-10 px-4 text-sm',
};

export function Button({ variant = 'primary', size = 'md', className, ...props }: ButtonProps) {
  return (
    <button
      className={clsx(
        'inline-flex items-center justify-center gap-1.5 rounded-xl font-semibold transition-all duration-200 disabled:cursor-not-allowed disabled:opacity-55',
        variants[variant],
        sizes[size],
        className,
      )}
      {...props}
    />
  );
}
