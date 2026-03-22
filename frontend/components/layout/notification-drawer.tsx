'use client';
import { useEffect } from 'react';
import { useNotifications } from '@/lib/context/notification-context';
import type { EventKind } from '@/lib/api/types';

const EVENT_META: Record<EventKind, { icon: string; color: string }> = {
  device_updated:      { icon: '↕',  color: 'var(--accent)' },
  device_connected:    { icon: '●',  color: 'var(--success)' },
  device_disconnected: { icon: '○',  color: 'var(--ink-faint)' },
  device_error:        { icon: '⚠',  color: 'var(--danger)' },
  automation:          { icon: '⚡', color: 'var(--warn)' },
  request:             { icon: '↗',  color: 'var(--ink-muted)' },
  client_connected:    { icon: '↙',  color: 'var(--ink-muted)' },
  client_disconnected: { icon: '↙',  color: 'var(--ink-faint)' },
  server:              { icon: '◈',  color: 'var(--info)' },
};

function formatTime(ts: string): string {
  try {
    return new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } catch {
    return '';
  }
}

export function NotificationDrawer() {
  const { events, isOpen, close } = useNotifications();

  // Close on Escape key
  useEffect(() => {
    if (!isOpen) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') close();
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [isOpen, close]);

  return (
    <>
      {/* Backdrop */}
      {isOpen && (
        <div
          className="fixed inset-0 z-30 bg-black/40 backdrop-blur-sm transition-opacity"
          onClick={close}
          aria-hidden="true"
        />
      )}

      {/* Drawer panel */}
      <div
        className={`fixed right-0 top-0 z-40 flex h-full w-full max-w-sm flex-col border-l border-[var(--line-strong)] bg-[var(--bg-modal)] shadow-[var(--shadow-modal)] transition-transform duration-300 ease-in-out ${
          isOpen ? 'translate-x-0' : 'translate-x-full'
        }`}
        role="dialog"
        aria-label="Notifications"
        aria-modal="true"
      >
        {/* Header */}
        <div className="flex shrink-0 items-center justify-between border-b border-[var(--line)] px-5 py-4">
          <h2 className="text-sm font-semibold text-[color:var(--ink-strong)]">Notifications</h2>
          <button
            type="button"
            onClick={close}
            aria-label="Close notifications"
            className="inline-flex h-7 w-7 items-center justify-center rounded-lg text-[color:var(--ink-muted)] transition hover:bg-[var(--surface-hover)] hover:text-[color:var(--ink-strong)]"
          >
            ✕
          </button>
        </div>

        {/* Event list */}
        <div className="flex-1 overflow-y-auto">
          {events.length === 0 ? (
            <div className="flex flex-col items-center justify-center gap-3 py-16 text-center">
              <span className="text-3xl opacity-30">🔔</span>
              <p className="text-sm text-[color:var(--ink-muted)]">No notifications yet</p>
              <p className="text-xs text-[color:var(--ink-faint)]">Events will appear here as they arrive</p>
            </div>
          ) : (
            <ol className="divide-y divide-[color:var(--line)]">
              {events.map((ev) => {
                const meta = EVENT_META[ev.kind] ?? { icon: '·', color: 'var(--ink-faint)' };
                const isError = ev.kind === 'device_error';
                return (
                  <li
                    key={ev.event_id}
                    className="flex items-start gap-3 px-5 py-3"
                    style={isError ? { background: 'var(--danger-soft)' } : undefined}
                  >
                    <span
                      className="mt-0.5 inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-md text-xs font-bold"
                      style={{ background: `color-mix(in srgb, ${meta.color} 15%, transparent)`, color: meta.color }}
                    >
                      {meta.icon}
                    </span>
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm text-[color:var(--ink-strong)]">{ev.message}</p>
                      {ev.device_name && (
                        <p className="text-xs text-[color:var(--ink-muted)]">{ev.device_name}</p>
                      )}
                    </div>
                    <time className="shrink-0 text-[0.68rem] tabular-nums text-[color:var(--ink-faint)]">
                      {formatTime(ev.timestamp)}
                    </time>
                  </li>
                );
              })}
            </ol>
          )}
        </div>
      </div>
    </>
  );
}
