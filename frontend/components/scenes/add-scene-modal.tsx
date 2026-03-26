'use client';
import { useState } from 'react';
import { Modal } from '@/components/ui/modal';
import { Button } from '@/components/ui/button';
import { Field } from '@/components/ui/field';
import type { CreateSceneRequest, SceneState, Device } from '@/lib/api/types';

interface AddSceneModalProps {
  open: boolean;
  onClose: () => void;
  onAdd: (req: CreateSceneRequest) => Promise<void>;
  devices: Device[];
}

export function AddSceneModal({ open, onClose, onAdd, devices }: AddSceneModalProps) {
  const [name, setName] = useState('');
  const [selected, setSelected] = useState<Record<string, boolean>>({});
  const [overrides, setOverrides] = useState<Record<string, SceneState>>({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [nameError, setNameError] = useState('');

  const toggle = (id: string) => setSelected(s => ({ ...s, [id]: !s[id] }));

  const setOverride = (id: string, patch: Partial<SceneState>) =>
    setOverrides(o => ({ ...o, [id]: { ...o[id], ...patch } }));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setNameError('');
    if (!name.trim()) { setNameError('Scene name is required.'); return; }
    const deviceIds = Object.keys(selected).filter(id => selected[id]);
    if (deviceIds.length === 0) { setError('Select at least one device.'); return; }
    const states: Record<string, SceneState> = {};
    for (const id of deviceIds) {
      const device = devices.find(d => d.id === id);
      if (!device) continue;
      states[id] = overrides[id] ?? { state: device.state, brightness: device.brightness };
    }
    setError('');
    setLoading(true);
    try {
      await onAdd({ name: name.trim(), states });
      setName(''); setSelected({}); setOverrides({});
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create scene');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal title="Add Scene" open={open} onClose={onClose}>
      <form onSubmit={handleSubmit} noValidate className="flex flex-col gap-4">
        <Field label="Scene Name" hint="A descriptive name for this scene — e.g. 'Evening Relax' or 'Movie Mode'. Shown in the scenes list and automation actions." error={nameError}>
          <input value={name} onChange={e => { setName(e.target.value); if (nameError) setNameError(''); }} placeholder="e.g. Evening Relax" />
        </Field>

        <div>
          <p className="mb-2 text-xs font-semibold uppercase tracking-[0.08em] text-[color:var(--ink-muted)]">Select Devices</p>
          <div className="space-y-2 max-h-64 overflow-y-auto">
            {devices.map(device => (
              <div key={device.id} className="flex items-center gap-3 rounded-xl border border-[rgba(255,255,255,0.06)] bg-[rgba(255,255,255,0.02)] p-3">
                <input type="checkbox" id={`d-${device.id}`} checked={!!selected[device.id]} onChange={() => toggle(device.id)} className="accent-[color:var(--accent)]" />
                <label htmlFor={`d-${device.id}`} className="flex-1 cursor-pointer">
                  <p className="text-sm font-medium text-[color:var(--ink-strong)]">{device.name}</p>
                  <p className="text-xs text-[color:var(--ink-muted)]">{device.device_type} · {device.state}</p>
                </label>
                {selected[device.id] && (
                  <select
                    value={overrides[device.id]?.state ?? device.state}
                    onChange={e => setOverride(device.id, { state: e.target.value as 'on' | 'off' })}
                    className="text-xs"
                  >
                    <option value="on">on</option>
                    <option value="off">off</option>
                  </select>
                )}
              </div>
            ))}
          </div>
        </div>

        {error && (
          <p className="rounded-xl border border-[rgba(244,63,94,0.3)] bg-[rgba(244,63,94,0.1)] px-3 py-2 text-sm text-[#fb7185]">{error}</p>
        )}

        <div className="flex justify-end gap-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" disabled={loading}>{loading ? 'Creating…' : 'Create Scene'}</Button>
        </div>
      </form>
    </Modal>
  );
}
