'use client';
import { useState } from 'react';
import { useDevices } from '@/lib/hooks/use-devices';
import { DeviceCard } from '@/components/devices/device-card';
import { AddDeviceModal } from '@/components/devices/add-device-modal';
import { DeviceControlModal } from '@/components/devices/device-control-modal';
import { Button } from '@/components/ui/button';
import { DeviceCardSkeleton } from '@/components/ui/skeleton';
import { EmptyState } from '@/components/ui/empty-state';
import type { Device, DeviceType } from '@/lib/api/types';

export default function DevicesPage() {
  const { devices, isLoading, add, remove, setState, setBrightness, setTemperature, connect, disconnect } = useDevices();
  const [addOpen, setAddOpen] = useState(false);
  const [controlling, setControlling] = useState<Device | null>(null);

  const handleAdd = async (name: string, type: DeviceType) => {
    await add({ name, device_type: type });
  };

  return (
    <div className="space-y-5">
      {/* Header */}
      <section className="surface-card p-5 sm:p-6">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="section-kicker">Device Manager</p>
            <h1 className="section-title">Rooms and hardware at a glance</h1>
            <p className="section-subtitle">
              Track state, power, and health for every connected device.
            </p>
          </div>
          <Button onClick={() => setAddOpen(true)}>+ Add Device</Button>
        </div>
      </section>

      {/* Content */}
      {isLoading ? (
        <section className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
          {Array.from({ length: 6 }).map((_, i) => <DeviceCardSkeleton key={i} />)}
        </section>
      ) : devices.length === 0 ? (
        <EmptyState
          icon="💡"
          title="No devices yet"
          subtitle="Add your first device to start monitoring and controlling your home."
          action={<Button onClick={() => setAddOpen(true)}>+ Add Device</Button>}
        />
      ) : (
        <section className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
          {devices.map(d => (
            <DeviceCard
              key={d.id}
              device={d}
              onToggle={() => setState(d.name, d.state === 'on' ? 'off' : 'on')}
              onDelete={() => remove(d.name)}
              onControl={() => setControlling(d)}
            />
          ))}
        </section>
      )}

      <AddDeviceModal open={addOpen} onClose={() => setAddOpen(false)} onAdd={handleAdd} />

      <DeviceControlModal
        device={controlling}
        open={controlling !== null}
        onClose={() => setControlling(null)}
        onSetState={setState}
        onSetBrightness={setBrightness}
        onSetTemperature={setTemperature}
        onConnect={connect}
        onDisconnect={disconnect}
      />
    </div>
  );
}
