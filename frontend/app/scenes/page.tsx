'use client';
import { useState } from 'react';
import { useScenes } from '@/lib/hooks/use-scenes';
import { useDevices } from '@/lib/hooks/use-devices';
import { Button } from '@/components/ui/button';
import { AddSceneModal } from '@/components/scenes/add-scene-modal';
import type { ApplySceneResponse, CreateSceneRequest } from '@/lib/api/types';

export default function ScenesPage() {
  const { scenes, isLoading, add, remove, apply } = useScenes();
  const { devices } = useDevices();
  const [open, setOpen] = useState(false);
  const [applyResult, setApplyResult] = useState<{ name: string; res: ApplySceneResponse } | null>(null);

  const handleApply = async (id: string, name: string) => {
    const res = await apply(id);
    setApplyResult({ name, res });
  };

  return (
    <div className="space-y-5">
      {/* Header */}
      <section className="surface-card p-5 sm:p-6">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="section-kicker">Scene Registry</p>
            <h1 className="section-title">Saved Scenes</h1>
            <p className="section-subtitle">
              Snapshots of device states. Apply a scene to instantly restore a lighting or comfort preset.
            </p>
          </div>
          <Button onClick={() => setOpen(true)}>+ Add Scene</Button>
        </div>
      </section>

      {applyResult && (
        <section className="surface-card p-4">
          <div className="flex items-center justify-between">
            <p className="text-sm text-[color:var(--ink-strong)]">
              Scene <strong>{applyResult.name}</strong> applied — {applyResult.res.applied} device(s)
              {applyResult.res.errors.length > 0 && <span className="text-[#fb7185]">, {applyResult.res.errors.length} error(s)</span>}
            </p>
            <button className="text-xs text-[color:var(--ink-faint)]" onClick={() => setApplyResult(null)}>✕</button>
          </div>
          {applyResult.res.errors.length > 0 && (
            <ul className="mt-1 text-xs text-[#fb7185]">{applyResult.res.errors.map((e, i) => <li key={i}>{e}</li>)}</ul>
          )}
        </section>
      )}

      {isLoading ? (
        <section className="surface-card p-6">
          <p className="text-sm text-[color:var(--ink-muted)]">Loading scenes…</p>
        </section>
      ) : scenes.length === 0 ? (
        <section className="surface-card flex flex-col items-center gap-3 p-12 text-center">
          <span className="text-4xl opacity-30">🎬</span>
          <p className="font-medium text-[color:var(--ink-strong)]">No scenes yet</p>
          <p className="mb-2 text-sm text-[color:var(--ink-muted)]">Save a set of device states as a scene for one-click activation.</p>
          <Button onClick={() => setOpen(true)}>+ Add Scene</Button>
        </section>
      ) : (
        <section className="space-y-3">
          {scenes.map(scene => {
            const deviceCount = Object.keys(scene.states).length;
            return (
              <article key={scene.id} className="surface-card flex flex-col gap-3 p-5 md:flex-row md:items-center md:justify-between">
                <div className="min-w-0 flex-1">
                  <p className="font-semibold text-[color:var(--ink-strong)]">{scene.name}</p>
                  <p className="mt-0.5 text-xs text-[color:var(--ink-muted)]">{deviceCount} device{deviceCount !== 1 ? 's' : ''}</p>
                  <div className="mt-2 flex flex-wrap gap-1">
                    {Object.entries(scene.states).map(([id, s]) => {
                      const device = devices.find(d => d.id === id);
                      return (
                        <span key={id} className="rounded-md bg-[rgba(6,182,212,0.08)] px-2 py-0.5 text-[10px] text-[#22d3ee]">
                          {device?.name ?? id.slice(0, 8)} → {s.state ?? '–'}
                        </span>
                      );
                    })}
                  </div>
                </div>
                <div className="flex shrink-0 gap-2">
                  <Button size="sm" variant="secondary" onClick={() => handleApply(scene.id, scene.name)}>▶ Apply</Button>
                  <Button size="sm" variant="danger" onClick={() => remove(scene.id)}>Delete</Button>
                </div>
              </article>
            );
          })}
        </section>
      )}

      <AddSceneModal
        open={open}
        onClose={() => setOpen(false)}
        onAdd={async req => { await add(req as CreateSceneRequest); setOpen(false); }}
        devices={devices}
      />
    </div>
  );
}
