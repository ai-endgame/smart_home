'use client';
import { useState } from 'react';
import { useScripts } from '@/lib/hooks/use-scripts';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { AddScriptModal } from '@/components/scripts/add-script-modal';
import type { CreateScriptRequest } from '@/lib/api/types';

export default function ScriptsPage() {
  const { scripts, isLoading, add, remove, run } = useScripts();
  const [open, setOpen] = useState(false);
  const [runningId, setRunningId] = useState<string | null>(null);

  const handleRun = async (id: string) => {
    setRunningId(id);
    try { await run(id); } finally { setRunningId(null); }
  };

  return (
    <div className="space-y-5">
      {/* Header */}
      <section className="surface-card p-5 sm:p-6">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="section-kicker">Script Registry</p>
            <h1 className="section-title">Automation Scripts</h1>
            <p className="section-subtitle">
              Named, parameterised action sequences. Reusable from automations or on-demand.
            </p>
          </div>
          <Button onClick={() => setOpen(true)}>+ Add Script</Button>
        </div>
      </section>

      {isLoading ? (
        <section className="surface-card p-6">
          <p className="text-sm text-[color:var(--ink-muted)]">Loading scripts…</p>
        </section>
      ) : scripts.length === 0 ? (
        <section className="surface-card flex flex-col items-center gap-3 p-12 text-center">
          <span className="text-4xl opacity-30">⟳</span>
          <p className="font-medium text-[color:var(--ink-strong)]">No scripts yet</p>
          <p className="mb-2 text-sm text-[color:var(--ink-muted)]">Create a script to automate multi-step device sequences.</p>
          <Button onClick={() => setOpen(true)}>+ Add Script</Button>
        </section>
      ) : (
        <section className="space-y-3">
          {scripts.map(script => (
            <article key={script.id} className="surface-card flex flex-col gap-3 p-5 md:flex-row md:items-center md:justify-between">
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-2">
                  <p className="font-semibold text-[color:var(--ink-strong)]">{script.name}</p>
                  <Badge label={`${script.steps.length} step${script.steps.length !== 1 ? 's' : ''}`} variant="default" />
                </div>
                {script.description && (
                  <p className="mt-1 text-xs text-[color:var(--ink-muted)]">{script.description}</p>
                )}
                <div className="mt-2 flex flex-wrap gap-1">
                  {script.steps.map((step, i) => (
                    <span key={i} className="rounded-md bg-[rgba(99,102,241,0.1)] px-2 py-0.5 text-[10px] text-[#818cf8]">
                      {step.type}
                    </span>
                  ))}
                </div>
              </div>
              <div className="flex shrink-0 gap-2">
                <Button size="sm" variant="secondary" onClick={() => handleRun(script.id)} disabled={runningId === script.id}>
                  {runningId === script.id ? '…' : '▶ Run'}
                </Button>
                <Button size="sm" variant="danger" onClick={() => remove(script.id)}>Delete</Button>
              </div>
            </article>
          ))}
        </section>
      )}

      <AddScriptModal open={open} onClose={() => setOpen(false)} onAdd={async req => { await add(req as CreateScriptRequest); setOpen(false); }} />
    </div>
  );
}
