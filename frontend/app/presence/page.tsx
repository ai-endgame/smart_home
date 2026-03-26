'use client';
import { useState } from 'react';
import { usePresence } from '@/lib/hooks/use-presence';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { AddPersonModal } from '@/components/presence/add-person-modal';
import type { CreatePersonRequest, PresenceState, SourceState } from '@/lib/api/types';

function stateBadgeVariant(state: PresenceState): 'success' | 'warning' | 'default' {
  if (state === 'home') return 'success';
  if (state === 'away') return 'default';
  return 'warning';
}

export default function PresencePage() {
  const { persons, isLoading, add, remove, setSource } = usePresence();
  const [open, setOpen] = useState(false);

  return (
    <div className="space-y-5">
      {/* Header */}
      <section className="surface-card p-5 sm:p-6">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="section-kicker">Presence Registry</p>
            <h1 className="section-title">Tracked Persons</h1>
            <p className="section-subtitle">
              Multi-source presence detection with grace-period debouncing. Any home source wins.
            </p>
          </div>
          <Button onClick={() => setOpen(true)}>+ Add Person</Button>
        </div>
      </section>

      {isLoading ? (
        <section className="surface-card p-6">
          <p className="text-sm text-[color:var(--ink-muted)]">Loading persons…</p>
        </section>
      ) : persons.length === 0 ? (
        <section className="surface-card flex flex-col items-center gap-3 p-12 text-center">
          <span className="text-4xl opacity-30">👤</span>
          <p className="font-medium text-[color:var(--ink-strong)]">No persons tracked yet</p>
          <p className="mb-2 text-sm text-[color:var(--ink-muted)]">Add a person to enable presence-based automations.</p>
          <Button onClick={() => setOpen(true)}>+ Add Person</Button>
        </section>
      ) : (
        <section className="space-y-3">
          {persons.map(person => (
            <article key={person.id} className="surface-card flex flex-col gap-4 p-5">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div className="flex items-center gap-3">
                  <p className="font-semibold text-[color:var(--ink-strong)]">{person.name}</p>
                  <Badge label={person.effective_state} variant={stateBadgeVariant(person.effective_state)} />
                </div>
                <div className="flex shrink-0 gap-2">
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => setSource(person.id, 'manual', 'home')}
                  >
                    Set Home
                  </Button>
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={() => setSource(person.id, 'manual', 'away')}
                  >
                    Set Away
                  </Button>
                  <Button size="sm" variant="danger" onClick={() => remove(person.id)}>Delete</Button>
                </div>
              </div>

              {/* Source breakdown */}
              {Object.keys(person.sources).length > 0 && (
                <div className="flex flex-wrap gap-2">
                  {Object.entries(person.sources).map(([src, state]) => (
                    <span
                      key={src}
                      className={[
                        'rounded-md px-2 py-0.5 text-[10px] font-medium',
                        state === 'home'
                          ? 'bg-[rgba(52,211,153,0.1)] text-[#34d399]'
                          : state === 'away'
                          ? 'bg-[rgba(148,155,200,0.1)] text-[color:var(--ink-muted)]'
                          : 'bg-[rgba(251,191,36,0.1)] text-[#fbbf24]',
                      ].join(' ')}
                    >
                      {src}: {state as SourceState}
                    </span>
                  ))}
                </div>
              )}

              <p className="text-[11px] text-[color:var(--ink-faint)]">
                Grace period: {person.grace_period_secs}s
              </p>
            </article>
          ))}
        </section>
      )}

      <AddPersonModal
        open={open}
        onClose={() => setOpen(false)}
        onAdd={async (req: CreatePersonRequest) => { await add(req); setOpen(false); }}
      />
    </div>
  );
}
