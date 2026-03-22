'use client';
import { useState } from 'react';
import Link from 'next/link';
import { useDashboards } from '@/lib/hooks/use-dashboards';
import { CardRenderer } from '@/components/dashboard/CardRenderer';
import { Button } from '@/components/ui/button';
import type { Dashboard, View } from '@/lib/api/types';

export default function DashboardsPage() {
  const { dashboards, isLoading } = useDashboards();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selectedViewId, setSelectedViewId] = useState<string | null>(null);

  const activeDash: Dashboard | null =
    (selectedId ? dashboards.find(d => d.id === selectedId) : dashboards[0]) ?? null;

  const activeView: View | null =
    (selectedViewId ? activeDash?.views.find(v => v.id === selectedViewId) : activeDash?.views[0]) ?? null;

  if (isLoading) {
    return (
      <div className="space-y-5">
        <section className="surface-card p-6">
          <p className="text-sm text-[color:var(--ink-muted)]">Loading dashboards…</p>
        </section>
      </div>
    );
  }

  if (dashboards.length === 0) {
    return (
      <div className="space-y-5">
        <section className="surface-card flex flex-col items-center gap-3 p-12 text-center">
          <span className="text-4xl opacity-30">📊</span>
          <p className="font-medium text-[color:var(--ink-strong)]">No dashboards yet</p>
          <p className="mb-2 text-sm text-[color:var(--ink-muted)]">Build your first dashboard to visualise your home.</p>
          <Link href="/dashboards/builder"><Button>+ New Dashboard</Button></Link>
        </section>
      </div>
    );
  }

  return (
    <div className="space-y-5">
      {/* Header */}
      <section className="surface-card p-5 sm:p-6">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="section-kicker">Control Center</p>
            <h1 className="section-title">Dashboards</h1>
            <p className="section-subtitle">Live views of your smart home entities.</p>
          </div>
          <Link href="/dashboards/builder"><Button variant="secondary">Edit</Button></Link>
        </div>
      </section>

      {/* Dashboard selector (when multiple) */}
      {dashboards.length > 1 && (
        <section className="surface-card p-4">
          <div className="flex flex-wrap gap-2">
            {dashboards.map(d => (
              <button
                key={d.id}
                onClick={() => { setSelectedId(d.id); setSelectedViewId(null); }}
                className={[
                  'rounded-lg px-3 py-1.5 text-sm font-medium transition',
                  activeDash?.id === d.id
                    ? 'bg-[color:var(--accent)] text-white'
                    : 'text-[color:var(--ink-muted)] hover:text-[color:var(--ink-strong)] hover:bg-[rgba(255,255,255,0.06)]',
                ].join(' ')}
              >
                {d.icon && <span className="mr-1">{d.icon}</span>}{d.name}
              </button>
            ))}
          </div>
        </section>
      )}

      {activeDash && (
        <>
          {/* View tabs */}
          {activeDash.views.length === 0 ? (
            <section className="surface-card flex flex-col items-center gap-3 p-10 text-center">
              <p className="text-sm text-[color:var(--ink-muted)]">No views yet.</p>
              <Link href="/dashboards/builder"><Button size="sm" variant="secondary">Add View</Button></Link>
            </section>
          ) : (
            <>
              <section className="surface-card p-3">
                <div className="flex flex-wrap gap-1.5">
                  {activeDash.views.map(v => (
                    <button
                      key={v.id}
                      onClick={() => setSelectedViewId(v.id)}
                      className={[
                        'rounded-lg px-3 py-1.5 text-sm font-medium transition',
                        activeView?.id === v.id
                          ? 'bg-[color:var(--accent)] text-white'
                          : 'text-[color:var(--ink-muted)] hover:text-[color:var(--ink-strong)] hover:bg-[rgba(255,255,255,0.06)]',
                      ].join(' ')}
                    >
                      {v.icon && <span className="mr-1">{v.icon}</span>}{v.title}
                    </button>
                  ))}
                </div>
              </section>

              {/* Cards grid */}
              {activeView && (
                activeView.cards.length === 0 ? (
                  <section className="surface-card flex flex-col items-center gap-3 p-10 text-center">
                    <p className="text-sm text-[color:var(--ink-muted)]">No cards in this view yet.</p>
                    <Link href="/dashboards/builder"><Button size="sm" variant="secondary">Add Card</Button></Link>
                  </section>
                ) : (
                  <section className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                    {activeView.cards.map(card => (
                      <CardRenderer key={card.id} card={card} />
                    ))}
                  </section>
                )
              )}
            </>
          )}
        </>
      )}
    </div>
  );
}
