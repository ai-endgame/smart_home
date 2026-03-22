'use client';
import { useState } from 'react';
import { useEntity } from '@/lib/hooks/use-entity';

interface Props { entityId: string; action: string; title?: string; }

export function ButtonCard({ entityId, action, title }: Props) {
  const { entity, isLoading } = useEntity(entityId);
  const [busy, setBusy] = useState(false);

  const handleClick = async () => {
    if (busy) return;
    setBusy(true);
    try {
      if (action.startsWith('script:')) {
        const scriptName = action.slice('script:'.length);
        await fetch(`/api/scripts/${encodeURIComponent(scriptName)}/run`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ args: {} }) });
      } else {
        // toggle: flip on/off
        const newState = entity?.state === 'on' ? 'off' : 'on';
        const deviceName = entityId.split('.')[1] ?? entityId;
        await fetch(`/api/devices/${encodeURIComponent(deviceName)}/state`, { method: 'PATCH', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ state: newState }) });
      }
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="surface-card flex flex-col gap-3 p-4">
      <p className="text-[10px] font-semibold uppercase tracking-[0.1em] text-[color:var(--ink-muted)]">{title ?? entityId}</p>
      {isLoading ? (
        <p className="text-sm text-[color:var(--ink-faint)]">Loading…</p>
      ) : (
        <button
          onClick={handleClick}
          disabled={busy}
          className="w-full rounded-xl bg-[color:var(--accent)] py-2 text-sm font-semibold text-white transition hover:opacity-80 disabled:opacity-40"
        >
          {busy ? '…' : action === 'toggle' ? (entity?.state === 'on' ? 'Turn Off' : 'Turn On') : action}
        </button>
      )}
      {entity && <p className="text-xs text-[color:var(--ink-muted)]">State: {entity.state}</p>}
    </div>
  );
}
