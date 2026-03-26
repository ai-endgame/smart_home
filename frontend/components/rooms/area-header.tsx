'use client';
import type { Device } from '@/lib/api/types';

interface AreaHeaderProps {
  name: string;
  icon?: string;
  devices: Device[];
}

export function AreaHeader({ name, icon, devices }: AreaHeaderProps) {
  const onlineCount = devices.filter(d => d.connected).length;
  const activeCount = devices.filter(d => d.state === 'on').length;
  const errorCount  = devices.filter(d => d.last_error).length;

  return (
    <div className="surface-card flex items-center gap-4 px-4 py-3">
      <span className="text-2xl leading-none">{icon ?? '🏠'}</span>
      <div className="flex-1 min-w-0">
        <h2 className="text-sm font-semibold text-[color:var(--ink-strong)] truncate">{name}</h2>
        <p className="text-xs text-[color:var(--ink-muted)]">
          {devices.length} device{devices.length !== 1 ? 's' : ''} ·{' '}
          <span style={{ color: 'var(--success)' }}>{onlineCount} online</span> ·{' '}
          {activeCount} active
        </p>
      </div>
      {errorCount > 0 && (
        <span
          className="flex shrink-0 items-center gap-1 rounded-full px-2 py-0.5 text-[0.68rem] font-semibold"
          style={{ background: 'var(--danger-soft)', color: 'var(--danger)' }}
        >
          ⚠ {errorCount}
        </span>
      )}
    </div>
  );
}
