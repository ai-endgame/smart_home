'use client';
import { useState } from 'react';
import { Modal } from '@/components/ui/modal';
import { Button } from '@/components/ui/button';
import { Field } from '@/components/ui/field';
import type { CreateRuleRequest, DeviceState, Trigger, Action } from '@/lib/api/types';

interface AddRuleModalProps {
  open: boolean;
  onClose: () => void;
  onAdd: (req: CreateRuleRequest) => Promise<void>;
  deviceNames: string[];
  initialValues?: Partial<CreateRuleRequest>;
}

type TriggerType = Trigger['type'];
type ActionType  = Action['type'];

const TRIGGER_TYPES: { value: TriggerType; label: string }[] = [
  { value: 'state_change',          label: 'Device state changes' },
  { value: 'temperature_above',     label: 'Temperature rises above' },
  { value: 'temperature_below',     label: 'Temperature drops below' },
  { value: 'time',                  label: 'At a specific time' },
  { value: 'sun',                   label: 'Sunrise / Sunset' },
  { value: 'numeric_state_above',   label: 'Brightness / Temp rises above' },
  { value: 'numeric_state_below',   label: 'Brightness / Temp drops below' },
  { value: 'webhook',               label: 'Webhook (external trigger)' },
];

const ACTION_TYPES: { value: ActionType; label: string }[] = [
  { value: 'state',       label: 'Set device state' },
  { value: 'brightness',  label: 'Set brightness' },
  { value: 'temperature', label: 'Set temperature' },
  { value: 'notify',      label: 'Send notification' },
];

const STATES: DeviceState[] = ['on', 'off'];

const DEFAULT: CreateRuleRequest = {
  name: '',
  trigger: { type: 'state_change', device_name: '', target_state: 'on' },
  action:  { type: 'state', device_name: '', state: 'on' },
};

type FieldErrors = Partial<Record<'name' | 'trigger_device' | 'webhook_id' | 'action_device' | 'notify_message', string>>;

export function AddRuleModal({ open, onClose, onAdd, deviceNames, initialValues }: AddRuleModalProps) {
  const [form, setForm] = useState<CreateRuleRequest>({ ...DEFAULT, ...initialValues });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [fieldErrors, setFieldErrors] = useState<FieldErrors>({});
  const [showTimeRange, setShowTimeRange] = useState(false);

  const clearFieldError = (key: keyof FieldErrors) =>
    setFieldErrors(e => ({ ...e, [key]: undefined }));

  const setTrigger = (patch: Partial<Trigger>) =>
    setForm(f => ({ ...f, trigger: { ...f.trigger, ...patch } as Trigger }));

  const setAction = (patch: Partial<Action>) =>
    setForm(f => ({ ...f, action: { ...f.action, ...patch } as Action }));

  const handleTriggerType = (t: TriggerType) => {
    const base = { type: t, device_name: form.trigger.device_name ?? '' };
    switch (t) {
      case 'state_change':       setForm(f => ({ ...f, trigger: { ...base, target_state: 'on' } })); break;
      case 'temperature_above':
      case 'temperature_below':  setForm(f => ({ ...f, trigger: { ...base, threshold: 25 } })); break;
      case 'time':               setForm(f => ({ ...f, trigger: { type: t, time: '08:00' } })); break;
      case 'sun':                setForm(f => ({ ...f, trigger: { type: t, event: 'sunrise', offset_minutes: 0 } })); break;
      case 'numeric_state_above':
      case 'numeric_state_below': setForm(f => ({ ...f, trigger: { ...base, attribute: 'brightness', threshold: 50 } })); break;
      case 'webhook':            setForm(f => ({ ...f, trigger: { type: t, id: '' } })); break;
      default:                   setForm(f => ({ ...f, trigger: { ...base } as Trigger }));
    }
  };

  const handleActionType = (t: ActionType) => {
    const base = { type: t, device_name: form.action.device_name ?? '' };
    switch (t) {
      case 'state':       setForm(f => ({ ...f, action: { ...base, state: 'on' } })); break;
      case 'brightness':  setForm(f => ({ ...f, action: { ...base, brightness: 80 } })); break;
      case 'temperature': setForm(f => ({ ...f, action: { ...base, temperature: 22 } })); break;
      case 'notify':      setForm(f => ({ ...f, action: { type: t, message: '' } })); break;
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const errs: FieldErrors = {};
    if (!form.name.trim()) errs.name = 'Rule name is required.';
    if (needsDevice && !form.trigger.device_name?.trim()) errs.trigger_device = 'Select or enter a device.';
    if (form.trigger.type === 'webhook' && !(form.trigger as { id?: string }).id?.trim()) errs.webhook_id = 'Webhook ID is required.';
    if (actionNeedsDevice && !form.action.device_name?.trim()) errs.action_device = 'Select or enter a device.';
    if (form.action.type === 'notify' && !(form.action as { message?: string }).message?.trim()) errs.notify_message = 'Message is required.';
    if (Object.keys(errs).length > 0) { setFieldErrors(errs); return; }
    setFieldErrors({});
    setError('');
    setLoading(true);
    try {
      const payload: CreateRuleRequest = { ...form };
      if (!showTimeRange) delete payload.time_range;
      await onAdd(payload);
      setForm(DEFAULT);
      setShowTimeRange(false);
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create rule');
    } finally {
      setLoading(false);
    }
  };

  const needsDevice = !['time', 'sun', 'webhook'].includes(form.trigger.type);
  const actionNeedsDevice = form.action.type !== 'notify';

  return (
    <Modal title="Add Automation Rule" open={open} onClose={onClose}>
      <form onSubmit={handleSubmit} noValidate className="flex flex-col gap-5">

        {/* Rule name */}
        <Field label="Rule Name" hint="A short, memorable name for this automation rule. Must be unique across all rules." error={fieldErrors.name}>
          <input
            value={form.name}
            onChange={e => { setForm(f => ({ ...f, name: e.target.value })); clearFieldError('name'); }}
            placeholder="e.g. Night mode"
          />
        </Field>

        {/* ── Trigger ─────────────────────────────────────── */}
        <fieldset className="flex flex-col gap-3 rounded-xl border border-[rgba(251,191,36,0.2)] bg-[rgba(251,191,36,0.05)] p-4">
          <legend className="px-1 text-[0.7rem] font-semibold uppercase tracking-widest text-[#fbbf24]">
            ⚡ Trigger — When…
          </legend>

          <Field label="Condition" hint="The event that will start this automation. Choose a trigger type, then configure its details below.">
            <select
              value={form.trigger.type}
              onChange={e => handleTriggerType(e.target.value as TriggerType)}
            >
              {TRIGGER_TYPES.map(t => (
                <option key={t.value} value={t.value}>{t.label}</option>
              ))}
            </select>
          </Field>

          {/* Device selector — only for device-based triggers */}
          {needsDevice && (
            <Field label="Device" hint="The device this trigger will watch. Only devices already registered in your home are listed." error={fieldErrors.trigger_device}>
              {deviceNames.length > 0 ? (
                <select
                  value={form.trigger.device_name ?? ''}
                  onChange={e => { setTrigger({ device_name: e.target.value }); clearFieldError('trigger_device'); }}
                >
                  <option value="">— select device —</option>
                  {deviceNames.map(n => <option key={n} value={n}>{n}</option>)}
                </select>
              ) : (
                <input
                  value={form.trigger.device_name ?? ''}
                  onChange={e => { setTrigger({ device_name: e.target.value }); clearFieldError('trigger_device'); }}
                  placeholder="Device name"
                />
              )}
            </Field>
          )}

          {form.trigger.type === 'state_change' && (
            <Field label="Target State">
              <select
                value={form.trigger.target_state ?? 'on'}
                onChange={e => setTrigger({ target_state: e.target.value as DeviceState })}
              >
                {STATES.map(s => <option key={s} value={s}>{s}</option>)}
              </select>
            </Field>
          )}

          {(form.trigger.type === 'temperature_above' || form.trigger.type === 'temperature_below') && (
            <Field label="Threshold (°C)">
              <input type="number" value={form.trigger.threshold ?? 25}
                onChange={e => setTrigger({ threshold: parseFloat(e.target.value) })}
                min={-40} max={100} step={0.5} />
            </Field>
          )}

          {form.trigger.type === 'time' && (
            <Field label="Time (HH:MM)">
              <input type="time" value={form.trigger.time ?? '08:00'}
                onChange={e => setTrigger({ time: e.target.value })} />
            </Field>
          )}

          {form.trigger.type === 'sun' && (
            <>
              <Field label="Event">
                <select value={form.trigger.event ?? 'sunrise'}
                  onChange={e => setTrigger({ event: e.target.value as 'sunrise' | 'sunset' })}>
                  <option value="sunrise">Sunrise</option>
                  <option value="sunset">Sunset</option>
                </select>
              </Field>
              <Field label="Offset (minutes, + after / − before)">
                <input type="number" value={form.trigger.offset_minutes ?? 0}
                  onChange={e => setTrigger({ offset_minutes: parseInt(e.target.value, 10) })}
                  min={-120} max={120} />
              </Field>
            </>
          )}

          {(form.trigger.type === 'numeric_state_above' || form.trigger.type === 'numeric_state_below') && (
            <>
              <Field label="Attribute">
                <select value={form.trigger.attribute ?? 'brightness'}
                  onChange={e => setTrigger({ attribute: e.target.value as 'brightness' | 'temperature' })}>
                  <option value="brightness">Brightness (%)</option>
                  <option value="temperature">Temperature (°C)</option>
                </select>
              </Field>
              <Field label="Threshold">
                <input type="number" value={form.trigger.threshold ?? 50}
                  onChange={e => setTrigger({ threshold: parseFloat(e.target.value) })}
                  step={0.5} />
              </Field>
            </>
          )}

          {form.trigger.type === 'webhook' && (
            <Field label="Webhook ID" hint="A unique slug used in the webhook URL: POST /api/automations/webhook/{id}. Share this with the external service that will fire this rule." error={fieldErrors.webhook_id}>
              <input value={form.trigger.id ?? ''}
                onChange={e => { setTrigger({ id: e.target.value }); clearFieldError('webhook_id'); }}
                placeholder="my-webhook-id" />
            </Field>
          )}
        </fieldset>

        {/* ── Action ──────────────────────────────────────── */}
        <fieldset className="flex flex-col gap-3 rounded-xl border border-[rgba(129,140,248,0.2)] bg-[rgba(129,140,248,0.05)] p-4">
          <legend className="px-1 text-[0.7rem] font-semibold uppercase tracking-widest text-[#818cf8]">
            ↪ Action — Then…
          </legend>

          <Field label="Action" hint="What should happen when the trigger fires. Set a device state, adjust brightness or temperature, or send a notification.">
            <select value={form.action.type}
              onChange={e => handleActionType(e.target.value as ActionType)}>
              {ACTION_TYPES.map(t => (
                <option key={t.value} value={t.value}>{t.label}</option>
              ))}
            </select>
          </Field>

          {actionNeedsDevice && (
            <Field label="Target Device" error={fieldErrors.action_device}>
              {deviceNames.length > 0 ? (
                <select value={form.action.device_name ?? ''}
                  onChange={e => { setAction({ device_name: e.target.value }); clearFieldError('action_device'); }}>
                  <option value="">— select device —</option>
                  {deviceNames.map(n => <option key={n} value={n}>{n}</option>)}
                </select>
              ) : (
                <input value={form.action.device_name ?? ''}
                  onChange={e => { setAction({ device_name: e.target.value }); clearFieldError('action_device'); }}
                  placeholder="Device name" />
              )}
            </Field>
          )}

          {form.action.type === 'state' && (
            <Field label="State">
              <select value={form.action.state ?? 'on'}
                onChange={e => setAction({ state: e.target.value as DeviceState })}>
                {STATES.map(s => <option key={s} value={s}>{s}</option>)}
              </select>
            </Field>
          )}

          {form.action.type === 'brightness' && (
            <Field label="Brightness (0–100%)">
              <input type="number" value={form.action.brightness ?? 80}
                onChange={e => setAction({ brightness: parseInt(e.target.value, 10) })}
                min={0} max={100} />
            </Field>
          )}

          {form.action.type === 'temperature' && (
            <Field label="Temperature (°C)">
              <input type="number" value={form.action.temperature ?? 22}
                onChange={e => setAction({ temperature: parseFloat(e.target.value) })}
                min={-40} max={100} step={0.5} />
            </Field>
          )}

          {form.action.type === 'notify' && (
            <Field label="Message" error={fieldErrors.notify_message}>
              <textarea
                value={form.action.message ?? ''}
                onChange={e => { setAction({ message: e.target.value }); clearFieldError('notify_message'); }}
                placeholder="e.g. Motion detected in kitchen"
                rows={2}
                className="resize-none"
              />
            </Field>
          )}
        </fieldset>

        {/* ── Time Range (optional) ────────────────────────── */}
        <div>
          <button
            type="button"
            onClick={() => setShowTimeRange(v => !v)}
            className="text-xs text-[color:var(--ink-muted)] hover:text-[color:var(--ink-strong)] transition-colors"
          >
            {showTimeRange ? '▾ Hide time window' : '▸ Add time window (optional)'}
          </button>
          {showTimeRange && (
            <div className="mt-3 flex gap-3">
              <Field label="Active from" hint="The rule will only fire when the current time falls within this window.">
                <input type="time" value={form.time_range?.from ?? '08:00'}
                  onChange={e => setForm(f => ({ ...f, time_range: { from: e.target.value, to: f.time_range?.to ?? '22:00' } }))} />
              </Field>
              <Field label="Active until">
                <input type="time" value={form.time_range?.to ?? '22:00'}
                  onChange={e => setForm(f => ({ ...f, time_range: { from: f.time_range?.from ?? '08:00', to: e.target.value } }))} />
              </Field>
            </div>
          )}
        </div>

        {error && (
          <p className="rounded-xl border border-[rgba(244,63,94,0.3)] bg-[rgba(244,63,94,0.1)] px-3 py-2 text-sm text-[#fb7185]">
            {error}
          </p>
        )}

        <div className="flex justify-end gap-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" disabled={loading}>
            {loading ? 'Creating…' : 'Create Rule'}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

