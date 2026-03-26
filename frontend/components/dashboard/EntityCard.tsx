'use client';
import { useEntity } from '@/lib/hooks/use-entity';

interface Props { entityId: string; title?: string; }

export function EntityCard({ entityId, title }: Props) {
  const { entity, isLoading } = useEntity(entityId);
  return (
    <div className="surface-card flex flex-col gap-1.5 p-4">
      <p className="text-[10px] font-semibold uppercase tracking-[0.1em] text-[color:var(--ink-muted)]">{title ?? entityId}</p>
      {isLoading ? (
        <p className="text-sm text-[color:var(--ink-faint)]">Loading…</p>
      ) : entity ? (
        <>
          <p className="text-xl font-bold text-[color:var(--ink-strong)]">{entity.state}</p>
          <p className="text-xs text-[color:var(--ink-muted)]">{entity.kind}</p>
        </>
      ) : (
        <p className="text-sm text-[color:var(--ink-faint)]">Unavailable</p>
      )}
    </div>
  );
}
