'use client';
import { useState } from 'react';
import { Modal } from '@/components/ui/modal';
import { Button } from '@/components/ui/button';
import { Field } from '@/components/ui/field';
import { DeviceType } from '@/lib/api/types';

interface AddDeviceModalProps {
  open: boolean;
  onClose: () => void;
  onAdd: (name: string, type: DeviceType) => Promise<void>;
}

const DEVICE_TYPES: { value: DeviceType; label: string; icon: string; group: string }[] = [
  { value: 'light',        label: 'Light',         icon: '💡', group: 'Lighting' },
  { value: 'thermostat',   label: 'Thermostat',    icon: '🌡️', group: 'Climate' },
  { value: 'fan',          label: 'Fan',            icon: '🌀', group: 'Climate' },
  { value: 'lock',         label: 'Lock',           icon: '🔒', group: 'Access' },
  { value: 'switch',       label: 'Switch',         icon: '⚡', group: 'Power' },
  { value: 'outlet',       label: 'Smart Outlet',   icon: '🔌', group: 'Power' },
  { value: 'tv',           label: 'TV',             icon: '📺', group: 'Media' },
  { value: 'speaker',      label: 'Speaker',        icon: '🔊', group: 'Media' },
  { value: 'media_player', label: 'Media Player',   icon: '▶️', group: 'Media' },
  { value: 'sensor',       label: 'Sensor',         icon: '📡', group: 'Safety' },
  { value: 'camera',       label: 'Camera',         icon: '📷', group: 'Safety' },
  { value: 'alarm',        label: 'Alarm',          icon: '🚨', group: 'Safety' },
  { value: 'cover',        label: 'Blinds / Cover', icon: '🪟', group: 'Covers' },
  { value: 'hub',          label: 'Hub / Bridge',   icon: '🔗', group: 'Infrastructure' },
];

export function AddDeviceModal({ open, onClose, onAdd }: AddDeviceModalProps) {
  const [name, setName] = useState('');
  const [type, setType] = useState<DeviceType>('light');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [nameError, setNameError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setNameError('');
    if (!name.trim()) { setNameError('Device name is required.'); return; }
    setError('');
    setLoading(true);
    try {
      await onAdd(name.trim(), type);
      setName('');
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add device');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal title="Add Device" open={open} onClose={onClose}>
      <form onSubmit={handleSubmit} noValidate className="flex flex-col gap-4">
        <Field
          label="Device Name"
          hint="A unique, lowercase identifier for this device. Use underscores instead of spaces — e.g. living_room_light."
          error={nameError}
        >
          <input
            value={name}
            onChange={e => { setName(e.target.value); if (nameError) setNameError(''); }}
            placeholder="e.g. living_room_light"
          />
        </Field>

        <Field
          label="Device Type"
          hint="The category that best describes this device. It controls which attributes (brightness, temperature, etc.) are available."
        >
          <select value={type} onChange={e => setType(e.target.value as DeviceType)}>
            {Array.from(new Set(DEVICE_TYPES.map(t => t.group))).map(group => (
              <optgroup key={group} label={group}>
                {DEVICE_TYPES.filter(t => t.group === group).map(t => (
                  <option key={t.value} value={t.value}>
                    {t.icon} {t.label}
                  </option>
                ))}
              </optgroup>
            ))}
          </select>
        </Field>

        {error && (
          <p className="rounded-xl border border-[rgba(244,63,94,0.3)] bg-[rgba(244,63,94,0.1)] px-3 py-2 text-sm text-[#fb7185]">
            {error}
          </p>
        )}

        <div className="mt-1 flex justify-end gap-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" disabled={loading}>
            {loading ? 'Adding…' : 'Add Device'}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
