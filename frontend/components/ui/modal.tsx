'use client';
import { useEffect } from 'react';

interface ModalProps {
  title: string;
  open: boolean;
  onClose: () => void;
  children: React.ReactNode;
}

export function Modal({ title, open, onClose, children }: ModalProps) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [onClose]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div className="absolute inset-0 bg-[#08120f]/45 backdrop-blur-[2px]" onClick={onClose} />
      <div className="relative w-full max-w-md rounded-2xl border border-white/55 bg-[linear-gradient(170deg,rgba(255,255,255,0.98),rgba(242,249,245,0.96))] p-6 shadow-[0_28px_60px_rgba(10,24,19,0.3)]">
        <div className="mb-4 flex items-start justify-between gap-3">
          <div>
            <p className="section-kicker">Quick Create</p>
            <h2 className="mt-1 text-xl font-semibold text-[color:var(--ink-strong)]">{title}</h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="inline-flex h-8 w-8 items-center justify-center rounded-lg border border-[color:var(--line)] text-lg text-[color:var(--ink-muted)] hover:bg-white"
            aria-label="Close"
          >
            &times;
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}
