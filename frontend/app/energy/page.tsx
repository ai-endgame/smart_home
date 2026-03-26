'use client';
import { useEnergy } from '@/lib/hooks/use-energy';
import { useDevices } from '@/lib/hooks/use-devices';

export default function EnergyPage() {
  const { summary, isLoading: summaryLoading } = useEnergy();
  const { devices, isLoading: devicesLoading } = useDevices();

  const devicesWithPower = devices.filter(d => d.power_w != null && d.power_w > 0);

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-[color:var(--ink-strong)]">Energy</h1>

      {/* Summary card */}
      <div className="surface-card p-6">
        {summaryLoading ? (
          <div className="h-16 animate-pulse rounded-xl bg-[var(--surface-hover)]" />
        ) : (
          <div className="flex items-center gap-6">
            <div className="flex h-14 w-14 items-center justify-center rounded-2xl bg-[rgba(34,197,94,0.12)] text-2xl">
              ⚡
            </div>
            <div>
              <p className="text-3xl font-bold text-[color:var(--ink-strong)]">
                {summary ? summary.total_power_w.toFixed(1) : '0'} <span className="text-lg font-normal text-[color:var(--ink-muted)]">W</span>
              </p>
              <p className="text-sm text-[color:var(--ink-muted)]">
                {summary?.devices_reporting ?? 0} device{(summary?.devices_reporting ?? 0) !== 1 ? 's' : ''} reporting
              </p>
            </div>
          </div>
        )}
      </div>

      {/* Device list */}
      {devicesLoading ? (
        <div className="space-y-3">
          {[...Array(3)].map((_, i) => (
            <div key={i} className="h-16 animate-pulse rounded-xl bg-[var(--surface)]" />
          ))}
        </div>
      ) : devicesWithPower.length === 0 ? (
        <div className="flex flex-col items-center gap-3 py-16 text-center">
          <span className="text-5xl">🔌</span>
          <p className="text-lg font-semibold text-[color:var(--ink-strong)]">No energy data</p>
          <p className="text-sm text-[color:var(--ink-muted)]">
            Devices reporting power consumption will appear here
          </p>
        </div>
      ) : (
        <div className="space-y-2">
          <h2 className="text-sm font-medium text-[color:var(--ink-muted)]">By Device</h2>
          {devicesWithPower
            .sort((a, b) => (b.power_w ?? 0) - (a.power_w ?? 0))
            .map(device => (
              <div
                key={device.id}
                className="surface-card flex items-center justify-between px-4 py-3"
              >
                <div>
                  <p className="font-medium text-[color:var(--ink-strong)]">{device.name}</p>
                  <p className="text-xs capitalize text-[color:var(--ink-muted)]">
                    {device.device_type.replace('_', ' ')}
                  </p>
                </div>
                <div className="text-right">
                  <p className="font-bold text-[#22c55e]">{device.power_w!.toFixed(1)} W</p>
                  {device.energy_kwh != null && (
                    <p className="text-xs text-[color:var(--ink-muted)]">{device.energy_kwh.toFixed(3)} kWh</p>
                  )}
                </div>
              </div>
            ))}
        </div>
      )}
    </div>
  );
}
