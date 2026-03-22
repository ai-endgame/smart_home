'use client';
import { useDeviceHistory } from '@/lib/hooks/use-device-history';
import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts';

interface Props { entityId: string; hours: number; title?: string; }

export function HistoryCard({ entityId, hours, title }: Props) {
  const limit = hours * 6; // ~one point per 10 min
  const { history, isLoading } = useDeviceHistory(entityId, limit);

  return (
    <div className="surface-card flex flex-col gap-2 p-4">
      <p className="text-[10px] font-semibold uppercase tracking-[0.1em] text-[color:var(--ink-muted)]">
        {title ?? entityId}
      </p>

      {isLoading ? (
        <div className="h-20 animate-pulse rounded-xl bg-[var(--surface-hover)]" />
      ) : history.length < 2 ? (
        <div className="flex h-20 items-center justify-center rounded-xl border border-dashed border-[rgba(148,155,200,0.15)] bg-[rgba(148,155,200,0.04)]">
          <p className="text-xs text-[color:var(--ink-faint)]">No history</p>
        </div>
      ) : (
        <ResponsiveContainer width="100%" height={80}>
          <LineChart data={history.map(h => ({ ts: h.ts.slice(11, 16), val: h.state === 'on' ? 1 : 0 }))}>
            <XAxis dataKey="ts" tick={{ fontSize: 9, fill: 'var(--ink-faint)' }} interval="preserveStartEnd" />
            <YAxis domain={[0, 1]} hide />
            <Tooltip formatter={(v) => [Number(v) === 1 ? 'On' : 'Off', 'State']} />
            <Line type="stepAfter" dataKey="val" stroke="var(--accent)" strokeWidth={1.5} dot={false} />
          </LineChart>
        </ResponsiveContainer>
      )}

      <p className="text-[10px] text-[color:var(--ink-faint)]">{entityId} · last {hours}h</p>
    </div>
  );
}
