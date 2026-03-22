'use client';
import { useEntity } from '@/lib/hooks/use-entity';

interface Props { title: string; entityIds: string[]; aggregation: string; }

function EntityValue({ entityId }: { entityId: string }) {
  const { entity } = useEntity(entityId);
  return entity ? parseFloat(entity.state) : NaN;
}

export function StatCard({ title, entityIds, aggregation }: Props) {
  const results = entityIds.map(id => {
    // eslint-disable-next-line react-hooks/rules-of-hooks
    const { entity } = useEntity(id);
    return entity;
  });

  const values = results.map(e => (e ? parseFloat(e.state) : NaN)).filter(v => !isNaN(v));

  let display = '—';
  if (values.length > 0) {
    if (aggregation === 'count') display = String(entityIds.length);
    else if (aggregation === 'sum') display = String(values.reduce((a, b) => a + b, 0));
    else if (aggregation === 'avg') display = (values.reduce((a, b) => a + b, 0) / values.length).toFixed(1);
  } else if (aggregation === 'count') {
    display = String(entityIds.length);
  }

  return (
    <div className="surface-card flex flex-col gap-1.5 p-4">
      <p className="text-[10px] font-semibold uppercase tracking-[0.1em] text-[color:var(--ink-muted)]">{title}</p>
      <p className="text-2xl font-bold text-[color:var(--ink-strong)]">{display}</p>
      <p className="text-xs text-[color:var(--ink-faint)]">{aggregation} · {entityIds.length} entities</p>
    </div>
  );
}
