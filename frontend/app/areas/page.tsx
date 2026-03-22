'use client';
import { useAreas } from '@/lib/hooks/use-areas';
import type { AreaResponse } from '@/lib/api/types';

const FLOOR_LABELS: Record<number, string> = { 0: 'Ground', 1: '1st', 2: '2nd', 3: '3rd' };
const floorLabel = (f?: number) => f == null ? null : (FLOOR_LABELS[f] ?? `${f}th`) + ' floor';

function AreaCard({ area }: { area: AreaResponse }) {
  return (
    <article className="relative overflow-hidden rounded-2xl border border-[rgba(148,155,200,0.14)] bg-[rgba(255,255,255,0.03)] p-5 transition-all hover:border-[rgba(129,140,248,0.3)] hover:bg-[rgba(129,140,248,0.05)]">
      <div className="pointer-events-none absolute -right-4 -top-4 h-20 w-20 rounded-full bg-[#818cf8] opacity-10 blur-2xl" aria-hidden />
      <div className="relative space-y-3">
        <div className="flex items-start justify-between gap-2">
          <div>
            {area.icon && (
              <p className="mb-1 text-xl">{area.icon}</p>
            )}
            <h3 className="text-base font-semibold text-[color:var(--ink-strong)]">{area.name}</h3>
            <p className="text-xs text-[color:var(--ink-muted)]">{area.area_id}</p>
          </div>
          <div className="text-right">
            <p className="text-3xl font-bold text-[#818cf8]">{area.device_count}</p>
            <p className="text-[0.65rem] uppercase tracking-wider text-[color:var(--ink-muted)]">devices</p>
          </div>
        </div>
        {floorLabel(area.floor) && (
          <span className="inline-block rounded-full bg-[rgba(129,140,248,0.12)] px-2 py-0.5 text-[0.65rem] font-medium text-[#818cf8]">
            {floorLabel(area.floor)}
          </span>
        )}
      </div>
    </article>
  );
}

export default function AreasPage() {
  const { areas, isLoading, error } = useAreas();

  if (isLoading) return (
    <div className="flex h-40 items-center justify-center text-[color:var(--ink-muted)]">Loading areas…</div>
  );
  if (error) return (
    <div className="flex h-40 items-center justify-center text-[#fb7185]">Failed to load areas.</div>
  );

  return (
    <div className="space-y-6">
      <section className="surface-card p-6 sm:p-8">
        <p className="section-kicker">Area Registry</p>
        <h1 className="section-title">
          Your home&apos;s{' '}
          <span style={{ color: 'var(--accent)' }}>rooms &amp; zones.</span>
        </h1>
        <p className="section-subtitle">
          Areas group devices by physical location — the HA equivalent of rooms, with floor and icon metadata.
        </p>
      </section>

      {areas.length === 0 ? (
        <section className="surface-card flex flex-col items-center justify-center gap-2 p-12 text-center">
          <p className="text-2xl">⬡</p>
          <p className="text-sm text-[color:var(--ink-muted)]">
            No areas yet. Assign devices to rooms to see the area registry.
          </p>
        </section>
      ) : (
        <section className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
          {areas.map(area => <AreaCard key={area.area_id} area={area} />)}
        </section>
      )}
    </div>
  );
}
