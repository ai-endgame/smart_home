'use client';
import { useState } from 'react';
import { Modal } from '@/components/ui/modal';
import { Button } from '@/components/ui/button';
import { DeviceType } from '@/lib/api/types';

interface AddDeviceModalProps {
  open: boolean;
  onClose: () => void;
  onAdd: (name: string, type: DeviceType) => Promise<void>;
}

const DEVICE_TYPES: DeviceType[] = ['light', 'thermostat', 'lock', 'switch', 'sensor'];

export function AddDeviceModal({ open, onClose, onAdd }: AddDeviceModalProps) {
  const [name, setName] = useState('');
  const [type, setType] = useState<DeviceType>('light');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
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
      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        <div>
          <label className="mb-1.5 block text-xs font-semibold uppercase tracking-[0.07em] text-[color:var(--ink-muted)]">
            Device Name
          </label>
          <input
            value={name}
            onChange={e => setName(e.target.value)}
            required
            placeholder="e.g. living_room_light"
          />
        </div>
        <div>
          <label className="mb-1.5 block text-xs font-semibold uppercase tracking-[0.07em] text-[color:var(--ink-muted)]">
            Device Type
          </label>
          <select value={type} onChange={e => setType(e.target.value as DeviceType)}>
            {DEVICE_TYPES.map(t => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </div>
        {error && (
          <p className="rounded-xl border border-[#e5a0a6] bg-[#fff1f3] px-3 py-2 text-sm text-[#8f2f38]">{error}</p>
        )}
        <div className="mt-2 flex justify-end gap-2">
          <Button type="button" variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" disabled={loading}>
            {loading ? 'Adding...' : 'Add Device'}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
