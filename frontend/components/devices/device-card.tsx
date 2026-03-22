'use client';
import { Device } from '@/lib/api/types';
import { SparkLine } from '@/components/ui/spark-line';
import { useDeviceHistory } from '@/lib/hooks/use-device-history';

interface DeviceCardProps {
  device: Device;
  onToggle: () => void;
  onDelete: () => void;
  onControl: () => void;
}

const TYPE_META: Record<string, { icon: string; accent: string; glow: string; bg: string }> = {
  light:        { icon: '💡', accent: '#fbbf24', glow: 'rgba(251,191,36,0.22)',  bg: 'rgba(251,191,36,0.10)' },
  thermostat:   { icon: '🌡️', accent: '#fb923c', glow: 'rgba(251,146,60,0.22)',  bg: 'rgba(251,146,60,0.10)' },
  fan:          { icon: '🌀', accent: '#67e8f9', glow: 'rgba(103,232,249,0.22)', bg: 'rgba(103,232,249,0.10)' },
  lock:         { icon: '🔒', accent: '#818cf8', glow: 'rgba(129,140,248,0.22)', bg: 'rgba(129,140,248,0.10)' },
  switch:       { icon: '⚡', accent: '#22d3ee', glow: 'rgba(34,211,238,0.22)',  bg: 'rgba(34,211,238,0.10)' },
  outlet:       { icon: '🔌', accent: '#a78bfa', glow: 'rgba(167,139,250,0.22)', bg: 'rgba(167,139,250,0.10)' },
  tv:           { icon: '📺', accent: '#60a5fa', glow: 'rgba(96,165,250,0.22)',  bg: 'rgba(96,165,250,0.10)' },
  speaker:      { icon: '🔊', accent: '#c084fc', glow: 'rgba(192,132,252,0.22)', bg: 'rgba(192,132,252,0.10)' },
  media_player: { icon: '▶️', accent: '#f472b6', glow: 'rgba(244,114,182,0.22)', bg: 'rgba(244,114,182,0.10)' },
  sensor:       { icon: '📡', accent: '#34d399', glow: 'rgba(52,211,153,0.22)',  bg: 'rgba(52,211,153,0.10)' },
  camera:       { icon: '📷', accent: '#94a3b8', glow: 'rgba(148,163,184,0.22)', bg: 'rgba(148,163,184,0.10)' },
  alarm:        { icon: '🚨', accent: '#f87171', glow: 'rgba(248,113,113,0.22)', bg: 'rgba(248,113,113,0.10)' },
  cover:        { icon: '🪟', accent: '#86efac', glow: 'rgba(134,239,172,0.22)', bg: 'rgba(134,239,172,0.10)' },
  hub:          { icon: '🔗', accent: '#fdba74', glow: 'rgba(253,186,116,0.22)', bg: 'rgba(253,186,116,0.10)' },
};

const DEFAULT_META = { icon: '📦', accent: '#7880a8', glow: 'rgba(120,128,168,0.22)', bg: 'rgba(120,128,168,0.10)' };

function levelLabel(device: Device): string | null {
  const type = device.device_type;
  if ((type === 'light' || type === 'cover') && device.brightness != null)
    return type === 'cover' ? `${device.brightness}% open` : `${device.brightness}%`;
  if ((type === 'tv' || type === 'speaker' || type === 'media_player') && device.brightness != null)
    return `Vol ${device.brightness}%`;
  if (type === 'fan' && device.brightness != null)
    return `Speed ${device.brightness}%`;
  return null;
}

export function DeviceCard({ device, onToggle, onDelete, onControl }: DeviceCardProps) {
  const meta = TYPE_META[device.device_type] ?? DEFAULT_META;
  const isOn = device.state === 'on';
  const level = levelLabel(device);
  const { history } = useDeviceHistory(device.name, 20);

  return (
    <article
      className="surface-card relative flex h-full flex-col cursor-pointer overflow-hidden hover:-translate-y-0.5"
      style={{ boxShadow: isOn ? `0 4px 32px ${meta.glow}` : undefined }}
      onClick={onControl}
    >
      {/* Body */}
      <div className="flex flex-1 flex-col gap-3 p-4">

        {/* Header */}
        <div className="flex items-start justify-between gap-3">
          <div className="flex items-center gap-3">
            <span
              className="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-xl text-xl transition-all duration-300"
              style={{
                background: meta.bg,
                boxShadow: isOn ? `0 0 16px ${meta.glow}` : 'none',
              }}
            >
              {meta.icon}
            </span>
            <div>
              <p className="font-semibold leading-tight text-[color:var(--ink-strong)]">{device.name}</p>
              <p className="mt-0.5 text-xs capitalize text-[color:var(--ink-muted)]">
                {device.device_type.replace('_', ' ')}
              </p>
            </div>
          </div>

          {/* State dot — pulses when on */}
          <span
            className={isOn ? 'state-dot-on' : ''}
            style={{
              display: 'inline-block',
              marginTop: '4px',
              height: '8px',
              width: '8px',
              borderRadius: '9999px',
              flexShrink: 0,
              background: isOn ? '#34d399' : 'var(--ink-faint)',
              color: '#34d399',
              outline: !device.connected ? '2px solid #fbbf24' : undefined,
              outlineOffset: '2px',
            }}
          />
        </div>

        {/* Single-line status */}
        <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-[color:var(--ink-muted)]">
          <span className="flex items-center gap-1.5">
            <span
              className="inline-block h-1.5 w-1.5 rounded-full"
              style={{ background: device.connected ? 'var(--success)' : 'var(--ink-faint)' }}
            />
            {device.connected ? 'Online' : 'Offline'}
          </span>
          {level && (
            <span
              className="rounded-full px-2 py-0.5 text-[0.68rem] font-medium"
              style={{ background: meta.bg, color: meta.accent }}
            >
              {level}
            </span>
          )}
          {device.temperature != null && (
            <span className="rounded-full bg-[rgba(251,146,60,0.1)] px-2 py-0.5 text-[0.68rem] font-medium text-[#fb923c]">
              {device.temperature.toFixed(1)}°C
            </span>
          )}
          {device.power_w != null && device.power_w > 0 && (
            <span className="rounded-full bg-[rgba(34,197,94,0.12)] px-2 py-0.5 text-[0.68rem] font-medium text-[#22c55e]">
              {device.power_w.toFixed(1)} W
            </span>
          )}
        </div>

        {/* Spark line */}
        {history.length >= 2 && (
          <div className="px-0.5">
            <SparkLine points={history} width={120} height={18} />
          </div>
        )}

        {/* Error */}
        {device.last_error && (
          <p className="rounded-lg border border-[color:var(--danger-soft)] bg-[var(--danger-soft)] px-2.5 py-1.5 text-xs text-[color:var(--danger)]">
            ⚠ {device.last_error}
          </p>
        )}
      </div>

      {/* Action bar */}
      <div
        className="flex items-center gap-2 border-t border-[color:var(--line)] px-4 py-3"
        onClick={e => e.stopPropagation()}
      >
        {/* Pill toggle switch */}
        <button
          role="switch"
          aria-checked={isOn}
          onClick={onToggle}
          className="relative inline-flex h-6 w-11 shrink-0 items-center rounded-full transition-colors duration-250 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--accent)]"
          style={{ background: isOn ? meta.accent : 'var(--line-strong)' }}
          title={isOn ? 'Turn off' : 'Turn on'}
        >
          <span
            className="inline-block h-[18px] w-[18px] rounded-full bg-white shadow-sm transition-transform duration-250"
            style={{ transform: isOn ? 'translateX(21px)' : 'translateX(3px)' }}
          />
        </button>

        <span className="flex-1 text-xs font-medium text-[color:var(--ink-muted)]">
          {isOn ? 'On' : 'Off'}
        </span>

        <button
          onClick={onControl}
          className="inline-flex h-7 w-7 items-center justify-center rounded-lg text-sm text-[color:var(--ink-faint)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[color:var(--ink-strong)]"
          title="Open controls"
        >
          ⚙
        </button>

        <button
          onClick={onDelete}
          className="inline-flex h-7 w-7 items-center justify-center rounded-lg text-sm text-[color:var(--ink-faint)] transition-colors hover:bg-[var(--danger-soft)] hover:text-[color:var(--danger)]"
          title="Delete device"
        >
          ✕
        </button>
      </div>

      {/* Bottom status strip */}
      <div
        className="h-[3px] w-full transition-colors duration-300"
        style={{ background: isOn ? meta.accent : 'transparent' }}
      />
    </article>
  );
}
