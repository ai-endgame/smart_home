'use client';
import { useState } from 'react';
import { useQuickActions, type QuickAction } from '@/lib/hooks/use-quick-actions';
import { useScenes } from '@/lib/hooks/use-scenes';
import { useScripts } from '@/lib/hooks/use-scripts';
import { useAutomation } from '@/lib/hooks/use-automation';
import { PinPickerModal } from './pin-picker-modal';

const TYPE_ICON: Record<QuickAction['type'], string> = {
  scene:      '🎬',
  script:     '📜',
  automation: '⚡',
};

type ChipState = 'idle' | 'loading' | 'success' | 'error';

export function QuickActionsStrip() {
  const { actions, addQuickAction, removeQuickAction } = useQuickActions();
  const { apply: applyScene } = useScenes();
  const { run: runScript } = useScripts();
  const { run: runAutomation } = useAutomation();
  const [editing, setEditing] = useState(false);
  const [pickerOpen, setPickerOpen] = useState(false);
  const [chipStates, setChipStates] = useState<Record<string, ChipState>>({});
  const [chipErrors, setChipErrors] = useState<Record<string, string>>({});

  const setChipState = (id: string, state: ChipState) =>
    setChipStates(prev => ({ ...prev, [id]: state }));

  const handleRun = async (action: QuickAction) => {
    if (chipStates[action.id] === 'loading') return;
    setChipState(action.id, 'loading');
    setChipErrors(prev => ({ ...prev, [action.id]: '' }));
    try {
      if (action.type === 'scene')      await applyScene(action.id);
      else if (action.type === 'script') await runScript(action.id);
      else                               await runAutomation();
      setChipState(action.id, 'success');
      setTimeout(() => setChipState(action.id, 'idle'), 1800);
    } catch (err) {
      setChipState(action.id, 'error');
      setChipErrors(prev => ({ ...prev, [action.id]: err instanceof Error ? err.message : 'Failed' }));
      setTimeout(() => setChipState(action.id, 'idle'), 3000);
    }
  };

  return (
    <section className="surface-card px-4 py-3">
      <div className="mb-2 flex items-center justify-between">
        <p className="text-xs font-semibold uppercase tracking-[0.08em] text-[color:var(--ink-muted)]">
          Quick Actions
        </p>
        {actions.length > 0 && (
          <button
            type="button"
            onClick={() => setEditing(e => !e)}
            className="text-xs text-[color:var(--ink-faint)] transition hover:text-[color:var(--ink-muted)]"
          >
            {editing ? 'Done' : 'Edit'}
          </button>
        )}
      </div>

      <div className="flex gap-2 overflow-x-auto pb-1 scrollbar-none">
        {actions.length === 0 ? (
          <button
            type="button"
            onClick={() => setPickerOpen(true)}
            className="flex shrink-0 items-center gap-1.5 rounded-full border border-dashed border-[var(--line-strong)] px-4 py-2 text-xs text-[color:var(--ink-faint)] transition hover:border-[color:var(--accent)] hover:text-[color:var(--accent)]"
          >
            <span>+</span> Pin an action
          </button>
        ) : (
          <>
            {actions.map(action => {
              const state = chipStates[action.id] ?? 'idle';
              const errorMsg = chipErrors[action.id];
              return (
                <div key={action.id} className="relative flex shrink-0 flex-col items-center">
                  <button
                    type="button"
                    onClick={() => editing ? removeQuickAction(action.id) : handleRun(action)}
                    disabled={state === 'loading'}
                    className={`flex items-center gap-1.5 rounded-full border px-4 py-2 text-xs font-medium transition-all duration-200 ${
                      editing
                        ? 'border-[color:var(--danger)] bg-[var(--danger-soft)] text-[color:var(--danger)]'
                        : state === 'success'
                        ? 'border-[color:var(--success)] bg-[var(--success-soft)] text-[color:var(--success)]'
                        : state === 'error'
                        ? 'border-[color:var(--danger)] bg-[var(--danger-soft)] text-[color:var(--danger)]'
                        : 'border-[var(--line-strong)] bg-[var(--surface)] text-[color:var(--ink-strong)] hover:bg-[var(--surface-hover)]'
                    }`}
                  >
                    {editing ? (
                      <span>✕</span>
                    ) : state === 'loading' ? (
                      <span className="inline-block h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent" />
                    ) : state === 'success' ? (
                      <span>✓</span>
                    ) : state === 'error' ? (
                      <span>⚠</span>
                    ) : (
                      <span>{TYPE_ICON[action.type]}</span>
                    )}
                    {action.label}
                  </button>
                  {state === 'error' && errorMsg && (
                    <p className="absolute top-full mt-1 max-w-[10rem] rounded bg-[var(--bg-modal)] px-2 py-1 text-center text-[0.65rem] text-[color:var(--danger)] shadow-[var(--shadow-modal)]">
                      {errorMsg}
                    </p>
                  )}
                </div>
              );
            })}
            <button
              type="button"
              onClick={() => setPickerOpen(true)}
              className="flex shrink-0 items-center gap-1 rounded-full border border-dashed border-[var(--line-strong)] px-3 py-2 text-xs text-[color:var(--ink-faint)] transition hover:border-[color:var(--accent)] hover:text-[color:var(--accent)]"
            >
              + Pin
            </button>
          </>
        )}
      </div>

      <PinPickerModal
        open={pickerOpen}
        onClose={() => setPickerOpen(false)}
        onPin={addQuickAction}
      />
    </section>
  );
}
