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
    <div className="fixed inset-0 z-50 flex items-start justify-center overflow-y-auto p-4 sm:items-center">
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-[rgba(5,7,15,0.7)] backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Dialog */}
      <div className="relative my-auto w-full max-w-md flex flex-col max-h-[calc(100dvh-2rem)] rounded-2xl border border-[var(--line-strong)] bg-[var(--bg-modal)] shadow-[var(--shadow-modal)]">
        {/* Header — sticky */}
        <div className="flex shrink-0 items-start justify-between gap-3 border-b border-[var(--line)] px-6 py-4">
          <div>
            <p className="section-kicker">Quick Create</p>
            <h2 className="mt-1 text-lg font-semibold text-[color:var(--ink-strong)]">{title}</h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="mt-0.5 inline-flex h-7 w-7 items-center justify-center rounded-lg text-lg text-[color:var(--ink-muted)] transition hover:bg-[var(--surface-hover)] hover:text-[color:var(--ink-strong)]"
            aria-label="Close"
          >
            ×
          </button>
        </div>

        {/* Content — scrollable */}
        <div className="overflow-y-auto px-6 py-5">{children}</div>
      </div>
    </div>
  );
}
