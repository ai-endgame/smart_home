'use client';
import { Device } from '@/lib/api/types';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';

interface DeviceCardProps {
  device: Device;
  onToggle: () => void;
  onDelete: () => void;
}

const typeIcon: Record<string, string> = {
  light: '💡',
  thermostat: '🌡️',
  lock: '🔒',
  switch: '🔁',
  sensor: '📡',
};

export function DeviceCard({ device, onToggle, onDelete }: DeviceCardProps) {
  return (
    <article className="surface-card flex h-full flex-col gap-4 p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-start gap-3">
          <span className="inline-flex h-10 w-10 items-center justify-center rounded-xl bg-[linear-gradient(145deg,#f0f9f4,#d6f0e4)] text-lg shadow-[inset_0_0_0_1px_rgba(16,33,29,0.08)]">
            {typeIcon[device.device_type] ?? '📦'}
          </span>
          <div>
            <p className="font-semibold text-[color:var(--ink-strong)]">{device.name}</p>
            <p className="mt-0.5 text-xs capitalize text-[color:var(--ink-muted)]">{device.device_type}</p>
          </div>
        </div>
        <Badge
          label={device.state}
          variant={device.state === 'on' ? 'success' : device.state === 'off' ? 'default' : 'warning'}
        />
      </div>

      <div className="grid grid-cols-2 gap-2 text-sm">
        <div className="rounded-xl border border-[color:var(--line)] bg-white/70 px-3 py-2">
          <p className="text-[0.68rem] uppercase tracking-[0.08em] text-[color:var(--ink-muted)]">Connection</p>
          <p className="mt-1 font-semibold text-[color:var(--ink-strong)]">{device.connected ? 'Online' : 'Offline'}</p>
        </div>
        <div className="rounded-xl border border-[color:var(--line)] bg-white/70 px-3 py-2">
          <p className="text-[0.68rem] uppercase tracking-[0.08em] text-[color:var(--ink-muted)]">State</p>
          <p className="mt-1 font-semibold capitalize text-[color:var(--ink-strong)]">{device.state}</p>
        </div>
      </div>

      <div className="flex flex-wrap gap-2 text-sm text-[color:var(--ink-muted)]">
        {device.temperature != null && (
          <span className="rounded-full border border-[color:var(--line)] bg-white px-2.5 py-1">{device.temperature} degC</span>
        )}
        {device.brightness != null && (
          <span className="rounded-full border border-[color:var(--line)] bg-white px-2.5 py-1">{device.brightness}% brightness</span>
        )}
      </div>

      {device.last_error && (
        <p className="rounded-xl border border-[#e5a0a6] bg-[#fff1f3] px-3 py-2 text-xs text-[#8f2f38]">{device.last_error}</p>
      )}

      <div className="mt-auto flex gap-2">
        <Button size="sm" variant={device.state === 'on' ? 'secondary' : 'primary'} onClick={onToggle}>
          {device.state === 'on' ? 'Turn Off' : 'Turn On'}
        </Button>
        <Button size="sm" variant="danger" onClick={onDelete}>
          Delete
        </Button>
      </div>
    </article>
  );
}
