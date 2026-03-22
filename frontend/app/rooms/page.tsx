'use client';
import { useEffect } from 'react';
import { useState } from 'react';
import { useDevices } from '@/lib/hooks/use-devices';
import { useAreas } from '@/lib/hooks/use-areas';
import { DeviceCard } from '@/components/devices/device-card';
import { DeviceControlModal } from '@/components/devices/device-control-modal';
import { DeviceCardSkeleton } from '@/components/ui/skeleton';
import { EmptyState } from '@/components/ui/empty-state';
import { AreaHeader } from '@/components/rooms/area-header';
import type { Device } from '@/lib/api/types';

export default function RoomsPage() {
  const { devices, isLoading: devicesLoading, setState, setBrightness, setTemperature, connect, disconnect, remove } = useDevices();
  const { areas, isLoading: areasLoading } = useAreas();
  const [controlling, setControlling] = useState<Device | null>(null);

  useEffect(() => { document.title = 'Rooms — Smart Home'; }, []);

  const isLoading = devicesLoading || areasLoading;

  if (isLoading) {
    return (
      <div className="space-y-5">
        <section className="surface-card p-5 sm:p-6">
          <p className="section-kicker">Room View</p>
          <h1 className="section-title">Browse by room</h1>
        </section>
        <section className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
          {Array.from({ length: 6 }).map((_, i) => <DeviceCardSkeleton key={i} />)}
        </section>
      </div>
    );
  }

  if (devices.length === 0) {
    return (
      <div className="space-y-5">
        <section className="surface-card p-5 sm:p-6">
          <p className="section-kicker">Room View</p>
          <h1 className="section-title">Browse by room</h1>
        </section>
        <EmptyState
          icon="🏠"
          title="No devices or areas yet"
          subtitle="Add devices and create areas to browse your home by room."
        />
      </div>
    );
  }

  // Group devices by area
  const devicesByArea = new Map<string, Device[]>();
  const unassigned: Device[] = [];

  for (const device of devices) {
    if (device.room) {
      const area = areas.find(a => a.name === device.room);
      const areaId = area?.area_id ?? device.room;
      if (!devicesByArea.has(areaId)) devicesByArea.set(areaId, []);
      devicesByArea.get(areaId)!.push(device);
    } else {
      unassigned.push(device);
    }
  }

  const handleToggle = (d: Device) => setState(d.name, d.state === 'on' ? 'off' : 'on');

  return (
    <div className="space-y-5">
      {/* Header */}
      <section className="surface-card p-5 sm:p-6">
        <div>
          <p className="section-kicker">Room View</p>
          <h1 className="section-title">Browse by room</h1>
          <p className="section-subtitle">Devices organized by area — control everything in context.</p>
        </div>
      </section>

      {/* Areas with devices */}
      {areas.filter(area => devicesByArea.has(area.area_id)).map(area => {
        const areaDevices = devicesByArea.get(area.area_id) ?? [];
        return (
          <section key={area.area_id} className="space-y-3">
            <AreaHeader name={area.name} icon={area.icon} devices={areaDevices} />
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
              {areaDevices.map(d => (
                <DeviceCard
                  key={d.id}
                  device={d}
                  onToggle={() => handleToggle(d)}
                  onDelete={() => remove(d.name)}
                  onControl={() => setControlling(d)}
                />
              ))}
            </div>
          </section>
        );
      })}

      {/* Unassigned devices */}
      {unassigned.length > 0 && (
        <section className="space-y-3">
          <AreaHeader name="Unassigned" devices={unassigned} />
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
            {unassigned.map(d => (
              <DeviceCard
                key={d.id}
                device={d}
                onToggle={() => handleToggle(d)}
                onDelete={() => remove(d.name)}
                onControl={() => setControlling(d)}
              />
            ))}
          </div>
        </section>
      )}

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
