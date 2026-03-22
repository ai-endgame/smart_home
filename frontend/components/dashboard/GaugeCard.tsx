'use client';
import { useEntity } from '@/lib/hooks/use-entity';

interface Props { entityId: string; min: number; max: number; unit?: string; title?: string; }

export function GaugeCard({ entityId, min, max, unit, title }: Props) {
  const { entity, isLoading } = useEntity(entityId);
  const value = entity ? parseFloat(entity.state) : NaN;
  const pct = isNaN(value) ? 0 : Math.min(100, Math.max(0, ((value - min) / (max - min)) * 100));
  return (
    <div className="surface-card flex flex-col gap-2 p-4">
      <p className="text-[10px] font-semibold uppercase tracking-[0.1em] text-[color:var(--ink-muted)]">{title ?? entityId}</p>
      {isLoading ? (
        <p className="text-sm text-[color:var(--ink-faint)]">Loading…</p>
      ) : (
        <>
          <p className="text-xl font-bold text-[color:var(--ink-strong)]">
            {isNaN(value) ? '—' : `${value}${unit ? ` ${unit}` : ''}`}
          </p>
          <div className="h-2 w-full overflow-hidden rounded-full bg-[rgba(148,155,200,0.12)]">
            <div
              className="h-full rounded-full bg-[color:var(--accent)] transition-all duration-500"
              style={{ width: `${pct}%` }}
            />
          </div>
          <div className="flex justify-between text-[10px] text-[color:var(--ink-faint)]">
            <span>{min}{unit ? ` ${unit}` : ''}</span>
            <span>{max}{unit ? ` ${unit}` : ''}</span>
          </div>
        </>
      )}
    </div>
  );
}
