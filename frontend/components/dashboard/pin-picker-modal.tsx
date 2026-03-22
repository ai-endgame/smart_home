'use client';
import { useState } from 'react';
import { useScenes } from '@/lib/hooks/use-scenes';
import { useScripts } from '@/lib/hooks/use-scripts';
import { useAutomation } from '@/lib/hooks/use-automation';
import type { QuickAction } from '@/lib/hooks/use-quick-actions';

interface PinPickerModalProps {
  open: boolean;
  onClose: () => void;
  onPin: (action: QuickAction) => void;
}

export function PinPickerModal({ open, onClose, onPin }: PinPickerModalProps) {
  const [query, setQuery] = useState('');
  const { scenes } = useScenes();
  const { scripts } = useScripts();
  const { rules } = useAutomation();

  if (!open) return null;

  const q = query.toLowerCase();

  const sceneItems = scenes
    .filter(s => s.name.toLowerCase().includes(q))
    .map(s => ({ type: 'scene' as const, id: s.id, label: s.name }));

  const scriptItems = scripts
    .filter(s => s.name.toLowerCase().includes(q))
    .map(s => ({ type: 'script' as const, id: s.id, label: s.name }));

  const automationItems = rules
    .filter(r => r.name.toLowerCase().includes(q))
    .map(r => ({ type: 'automation' as const, id: r.name, label: r.name }));

  const groups = [
    { label: 'Scenes', icon: '🎬', items: sceneItems },
    { label: 'Scripts', icon: '📜', items: scriptItems },
    { label: 'Automations', icon: '⚡', items: automationItems },
  ].filter(g => g.items.length > 0);

  const handlePick = (item: QuickAction) => {
    onPin(item);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center overflow-y-auto p-4 sm:items-center">
      <div className="fixed inset-0 bg-black/50 backdrop-blur-sm" onClick={onClose} aria-hidden="true" />
      <div className="relative z-10 flex w-full max-w-md flex-col rounded-2xl border border-[var(--line-strong)] bg-[var(--bg-modal)] shadow-[var(--shadow-modal)]">
        {/* Header */}
        <div className="flex shrink-0 items-center justify-between border-b border-[var(--line)] px-5 py-4">
          <h2 className="text-sm font-semibold text-[color:var(--ink-strong)]">Pin a Quick Action</h2>
          <button
            type="button"
            onClick={onClose}
            className="inline-flex h-7 w-7 items-center justify-center rounded-lg text-[color:var(--ink-muted)] transition hover:bg-[var(--surface-hover)] hover:text-[color:var(--ink-strong)]"
          >
            ✕
          </button>
        </div>

        {/* Search */}
        <div className="shrink-0 border-b border-[var(--line)] px-5 py-3">
          <input
            type="text"
            value={query}
            onChange={e => setQuery(e.target.value)}
            placeholder="Search scenes, scripts, automations…"
            autoFocus
            className="w-full rounded-lg border border-[var(--line-strong)] bg-[var(--input-bg)] px-3 py-2 text-sm text-[color:var(--ink-strong)] placeholder:text-[color:var(--ink-faint)] focus:border-[color:var(--accent)] focus:outline-none"
          />
        </div>

        {/* List */}
        <div className="max-h-80 overflow-y-auto">
          {groups.length === 0 ? (
            <p className="py-10 text-center text-sm text-[color:var(--ink-muted)]">
              {query ? 'No matches found' : 'No scenes, scripts, or automations yet'}
            </p>
          ) : (
            groups.map(group => (
              <div key={group.label}>
                <p className="sticky top-0 bg-[var(--bg-modal)] px-5 py-2 text-[0.7rem] font-semibold uppercase tracking-[0.08em] text-[color:var(--ink-faint)]">
                  {group.icon} {group.label}
                </p>
                {group.items.map(item => (
                  <button
                    key={item.id}
                    type="button"
                    onClick={() => handlePick(item)}
                    className="flex w-full items-center gap-3 px-5 py-2.5 text-left text-sm text-[color:var(--ink-strong)] transition hover:bg-[var(--surface-hover)]"
                  >
                    {item.label}
                  </button>
                ))}
              </div>
            ))
          )}
        </div>

        {/* Footer note */}
        <div className="shrink-0 border-t border-[var(--line)] px-5 py-3">
          <p className="text-center text-[0.7rem] text-[color:var(--ink-faint)]">
            Actions are saved on this device only
          </p>
        </div>
      </div>
    </div>
  );
}
