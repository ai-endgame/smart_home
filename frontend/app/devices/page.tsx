'use client';
import { useState } from 'react';
import { useDevices } from '@/lib/hooks/use-devices';
import { DeviceCard } from '@/components/devices/device-card';
import { AddDeviceModal } from '@/components/devices/add-device-modal';
import { Button } from '@/components/ui/button';
import { DeviceType } from '@/lib/api/types';

export default function DevicesPage() {
  const { devices, isLoading, add, remove, setState } = useDevices();
  const [open, setOpen] = useState(false);

  const handleAdd = async (name: string, type: DeviceType) => {
    await add({ name, device_type: type });
  };

  if (isLoading) {
    return (
      <div className="surface-card p-6">
        <p className="text-sm text-[color:var(--ink-muted)]">Loading devices...</p>
      </div>
    );
  }

  return (
    <div className="space-y-5">
      <section className="surface-card p-5 sm:p-6">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="section-kicker">Device Manager</p>
            <h1 className="section-title">Rooms and hardware at a glance</h1>
            <p className="section-subtitle">Track state, power, and health for every connected device.</p>
          </div>
          <Button onClick={() => setOpen(true)}>+ Add Device</Button>
        </div>
      </section>

      {devices.length === 0 ? (
        <section className="surface-card p-6">
          <p className="text-sm text-[color:var(--ink-muted)]">No devices yet. Add one to get started.</p>
        </section>
      ) : (
        <section className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
          {devices.map(d => (
            <DeviceCard
              key={d.id}
              device={d}
              onToggle={() => setState(d.name, d.state === 'on' ? 'off' : 'on')}
              onDelete={() => remove(d.name)}
            />
          ))}
        </section>
      )}

      <AddDeviceModal open={open} onClose={() => setOpen(false)} onAdd={handleAdd} />
    </div>
  );
}
