'use client';
import { useState } from 'react';
import { Modal } from '@/components/ui/modal';
import { Button } from '@/components/ui/button';
import { Field } from '@/components/ui/field';
import type { CreateScriptRequest, ScriptStep, ScriptStepType } from '@/lib/api/types';

interface AddScriptModalProps {
  open: boolean;
  onClose: () => void;
  onAdd: (req: CreateScriptRequest) => Promise<void>;
}

const STEP_TYPES: { value: ScriptStepType; label: string }[] = [
  { value: 'set_state',       label: 'Set device state' },
  { value: 'set_brightness',  label: 'Set brightness' },
  { value: 'set_temperature', label: 'Set temperature' },
  { value: 'delay',           label: 'Delay (ms)' },
  { value: 'apply_scene',     label: 'Apply scene' },
  { value: 'call_script',     label: 'Call script' },
];

function emptyStep(type: ScriptStepType): ScriptStep {
  switch (type) {
    case 'set_state':       return { type, device_name: '', state: 'on' };
    case 'set_brightness':  return { type, device_name: '', brightness: 80 };
    case 'set_temperature': return { type, device_name: '', temperature: 22 };
    case 'delay':           return { type, milliseconds: 1000 };
    case 'apply_scene':     return { type, scene_name: '' };
    case 'call_script':     return { type, script_name: '' };
  }
}

export function AddScriptModal({ open, onClose, onAdd }: AddScriptModalProps) {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [steps, setSteps] = useState<ScriptStep[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [nameError, setNameError] = useState('');

  const addStep = (type: ScriptStepType) => setSteps(s => [...s, emptyStep(type)]);

  const updateStep = (i: number, patch: Partial<ScriptStep>) =>
    setSteps(s => s.map((step, idx) => idx === i ? { ...step, ...patch } : step));

  const removeStep = (i: number) => setSteps(s => s.filter((_, idx) => idx !== i));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setNameError('');
    if (!name.trim()) { setNameError('Script name is required.'); return; }
    if (steps.length === 0) { setError('Add at least one step.'); return; }
    setError('');
    setLoading(true);
    try {
      await onAdd({ name: name.trim(), description, steps });
      setName(''); setDescription(''); setSteps([]);
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create script');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal title="Add Script" open={open} onClose={onClose}>
      <form onSubmit={handleSubmit} noValidate className="flex flex-col gap-4">
        <Field label="Name" hint="A unique slug for this script — used when calling it from automations or other scripts. Lowercase with underscores." error={nameError}>
          <input value={name} onChange={e => { setName(e.target.value); if (nameError) setNameError(''); }} placeholder="e.g. dim_all" />
        </Field>
        <Field label="Description (optional)" hint="A short explanation of what this script does. Shown in the scripts list.">
          <input value={description} onChange={e => setDescription(e.target.value)} placeholder="What this script does" />
        </Field>

        <div>
          <p className="mb-2 text-xs font-semibold uppercase tracking-[0.08em] text-[color:var(--ink-muted)]">Steps</p>
          <div className="space-y-2">
            {steps.map((step, i) => (
              <div key={i} className="flex items-start gap-2 rounded-xl border border-[rgba(255,255,255,0.06)] bg-[rgba(255,255,255,0.02)] p-3">
                <div className="flex-1 space-y-2">
                  <p className="text-xs text-[color:var(--ink-muted)]">{i + 1}. {STEP_TYPES.find(t => t.value === step.type)?.label ?? step.type}</p>
                  {(step.type === 'set_state' || step.type === 'set_brightness' || step.type === 'set_temperature') && (
                    <input value={step.device_name ?? ''} onChange={e => updateStep(i, { device_name: e.target.value })} placeholder="Device name" />
                  )}
                  {step.type === 'set_state' && (
                    <select value={step.state ?? 'on'} onChange={e => updateStep(i, { state: e.target.value })}>
                      <option value="on">on</option>
                      <option value="off">off</option>
                    </select>
                  )}
                  {step.type === 'set_brightness' && (
                    <input type="number" value={Number(step.brightness ?? 80)} onChange={e => updateStep(i, { brightness: parseInt(e.target.value, 10) })} min={0} max={100} />
                  )}
                  {step.type === 'set_temperature' && (
                    <input type="number" value={Number(step.temperature ?? 22)} onChange={e => updateStep(i, { temperature: parseFloat(e.target.value) })} step={0.5} />
                  )}
                  {step.type === 'delay' && (
                    <input type="number" value={step.milliseconds ?? 1000} onChange={e => updateStep(i, { milliseconds: parseInt(e.target.value, 10) })} min={0} max={60000} placeholder="ms (max 60000)" />
                  )}
                  {step.type === 'apply_scene' && (
                    <input value={step.scene_name ?? ''} onChange={e => updateStep(i, { scene_name: e.target.value })} placeholder="Scene name" />
                  )}
                  {step.type === 'call_script' && (
                    <input value={step.script_name ?? ''} onChange={e => updateStep(i, { script_name: e.target.value })} placeholder="Script name" />
                  )}
                </div>
                <button type="button" onClick={() => removeStep(i)} className="mt-1 text-xs text-[color:var(--ink-faint)] hover:text-[#f43f5e]">✕</button>
              </div>
            ))}
          </div>
          <div className="mt-2 flex flex-wrap gap-1.5">
            {STEP_TYPES.map(t => (
              <button key={t.value} type="button" onClick={() => addStep(t.value)}
                className="rounded-lg border border-[rgba(255,255,255,0.08)] bg-[rgba(255,255,255,0.04)] px-2 py-1 text-xs text-[color:var(--ink-muted)] hover:border-[rgba(6,182,212,0.3)] hover:text-[color:var(--ink-strong)]">
                + {t.label}
              </button>
            ))}
          </div>
        </div>

        {error && (
          <p className="rounded-xl border border-[rgba(244,63,94,0.3)] bg-[rgba(244,63,94,0.1)] px-3 py-2 text-sm text-[#fb7185]">{error}</p>
        )}

        <div className="flex justify-end gap-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" disabled={loading}>{loading ? 'Creating…' : 'Create Script'}</Button>
        </div>
      </form>
    </Modal>
  );
}

