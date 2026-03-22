'use client';
import useSWR from 'swr';
import { useEcosystem } from '@/lib/hooks/use-ecosystem';
import { getMqttStatus } from '@/lib/api/mqtt';
import { getMatterStatus, getMatterDevices } from '@/lib/api/matter';
import type { MatterStatus, MqttStatus, ProtocolEntry } from '@/lib/api/types';

function MqttBadge() {
  const { data } = useSWR<MqttStatus>('/api/mqtt/status', () => getMqttStatus(), { refreshInterval: 5000 });
  if (!data) return null;
  return (
    <span
      className="inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-medium"
      style={{
        background: data.connected ? 'rgba(52,211,153,0.12)' : 'rgba(148,163,184,0.1)',
        color: data.connected ? '#34d399' : '#94a3b8',
        border: `1px solid ${data.connected ? 'rgba(52,211,153,0.3)' : 'rgba(148,163,184,0.2)'}`,
      }}
    >
      <span
        className="inline-block h-1.5 w-1.5 rounded-full"
        style={{ background: data.connected ? '#34d399' : '#94a3b8' }}
      />
      MQTT {data.connected ? `connected · ${data.topics_received} msgs` : 'not connected'}
    </span>
  );
}

function MatterBadge() {
  const { data: status } = useSWR<MatterStatus>('/api/matter/status', () => getMatterStatus(), { refreshInterval: 10000 });
  const { data: devices } = useSWR('/api/matter/devices', () => getMatterDevices(), { refreshInterval: 10000 });
  if (!status) return null;
  const count = devices?.length ?? status.devices_seen;
  const active = count > 0;
  return (
    <span
      className="inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-medium"
      style={{
        background: active ? 'rgba(52,211,153,0.12)' : 'rgba(148,163,184,0.1)',
        color: active ? '#34d399' : '#94a3b8',
        border: `1px solid ${active ? 'rgba(52,211,153,0.3)' : 'rgba(148,163,184,0.2)'}`,
      }}
    >
      <span
        className="inline-block h-1.5 w-1.5 rounded-full"
        style={{ background: active ? '#34d399' : '#94a3b8' }}
      />
      Matter {active ? `· ${count} device${count !== 1 ? 's' : ''}` : '· scanning'}
    </span>
  );
}

const PROTOCOL_COLORS: Record<string, { accent: string; bg: string; border: string }> = {
  zigbee:  { accent: '#a78bfa', bg: 'rgba(167,139,250,0.08)', border: 'rgba(167,139,250,0.2)' },
  z_wave:  { accent: '#60a5fa', bg: 'rgba(96,165,250,0.08)',  border: 'rgba(96,165,250,0.2)' },
  matter:  { accent: '#34d399', bg: 'rgba(52,211,153,0.08)',  border: 'rgba(52,211,153,0.2)' },
  thread:  { accent: '#6ee7b7', bg: 'rgba(110,231,183,0.08)', border: 'rgba(110,231,183,0.2)' },
  wifi:    { accent: '#fbbf24', bg: 'rgba(251,191,36,0.08)',  border: 'rgba(251,191,36,0.2)' },
  shelly:  { accent: '#f97316', bg: 'rgba(249,115,22,0.08)',  border: 'rgba(249,115,22,0.2)' },
  tasmota: { accent: '#22d3ee', bg: 'rgba(34,211,238,0.08)',  border: 'rgba(34,211,238,0.2)' },
  esphome: { accent: '#818cf8', bg: 'rgba(129,140,248,0.08)', border: 'rgba(129,140,248,0.2)' },
  wled:    { accent: '#fb7185', bg: 'rgba(251,113,133,0.08)', border: 'rgba(251,113,133,0.2)' },
  unknown: { accent: '#94a3b8', bg: 'rgba(148,163,184,0.08)', border: 'rgba(148,163,184,0.2)' },
};

function ProtocolCard({ entry }: { entry: ProtocolEntry }) {
  const colors = PROTOCOL_COLORS[entry.id] ?? PROTOCOL_COLORS.unknown;
  return (
    <article
      className="relative overflow-hidden rounded-2xl border p-5"
      style={{ borderColor: colors.border, background: colors.bg }}
    >
      <div
        className="pointer-events-none absolute -right-4 -top-4 h-20 w-20 rounded-full opacity-20 blur-2xl"
        style={{ background: colors.accent }}
        aria-hidden
      />
      <div className="relative space-y-3">
        {/* Header */}
        <div className="flex items-start justify-between gap-2">
          <div>
            <p className="text-xs uppercase tracking-[0.1em] text-[color:var(--ink-muted)]">{entry.transport}</p>
            <h3 className="mt-0.5 text-lg font-semibold" style={{ color: colors.accent }}>
              {entry.id}
            </h3>
          </div>
          <div className="text-right">
            <p className="text-3xl font-bold" style={{ color: colors.accent }}>{entry.device_count}</p>
            <p className="text-[0.65rem] uppercase tracking-wider text-[color:var(--ink-muted)]">devices</p>
          </div>
        </div>

        {/* Badges */}
        <div className="flex gap-2">
          <span
            className="rounded-full px-2 py-0.5 text-[0.65rem] font-medium uppercase tracking-wider"
            style={{
              background: entry.local_only ? 'rgba(52,211,153,0.15)' : 'rgba(251,113,133,0.15)',
              color: entry.local_only ? '#34d399' : '#fb7185',
            }}
          >
            {entry.local_only ? 'local' : 'cloud'}
          </span>
          {entry.mesh && (
            <span className="rounded-full bg-[rgba(167,139,250,0.15)] px-2 py-0.5 text-[0.65rem] font-medium uppercase tracking-wider text-[#a78bfa]">
              mesh
            </span>
          )}
        </div>

        {/* Description */}
        <p className="text-xs leading-relaxed text-[color:var(--ink-muted)]">{entry.description}</p>
      </div>
    </article>
  );
}

export default function EcosystemPage() {
  const { ecosystem, isLoading, error } = useEcosystem();

  if (isLoading) {
    return (
      <div className="flex h-40 items-center justify-center text-[color:var(--ink-muted)]">
        Loading ecosystem…
      </div>
    );
  }

  if (error || !ecosystem) {
    return (
      <div className="flex h-40 items-center justify-center text-[#fb7185]">
        Failed to load ecosystem data.
      </div>
    );
  }

  const localPct = ecosystem.total_devices === 0
    ? 0
    : Math.round((ecosystem.layers.local_devices / ecosystem.total_devices) * 100);

  return (
    <div className="space-y-6">
      {/* Hero */}
      <section className="surface-card p-6 sm:p-8">
        <div className="mb-3 flex items-center gap-3">
          <p className="section-kicker">Protocol Ecosystem</p>
          <MqttBadge />
          <MatterBadge />
        </div>
        <h1 className="section-title">
          Your home&apos;s{' '}
          <span style={{ color: 'var(--accent)' }}>connectivity stack.</span>
        </h1>
        <p className="section-subtitle">
          Protocols in use, local vs cloud breakdown, and device distribution across network layers.
        </p>
      </section>

      {/* Layer summary bar */}
      <section className="surface-card p-5">
        <p className="mb-3 text-xs uppercase tracking-[0.1em] text-[color:var(--ink-muted)]">Network Layer Distribution</p>
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
          <Stat label="Total Devices"    value={ecosystem.total_devices}           color="#818cf8" />
          <Stat label="Connected"        value={ecosystem.connected_count}          color="#34d399" />
          <Stat label="Local Protocol"   value={ecosystem.layers.local_devices}     color="#a78bfa" />
          <Stat label="Cloud Protocol"   value={ecosystem.layers.cloud_devices}     color="#fb7185" />
        </div>

        {/* Local ratio bar */}
        {ecosystem.total_devices > 0 && (
          <div className="mt-4">
            <div className="mb-1 flex justify-between text-xs text-[color:var(--ink-muted)]">
              <span>Local control</span>
              <span>{localPct}%</span>
            </div>
            <div className="h-2 overflow-hidden rounded-full bg-[rgba(255,255,255,0.06)]">
              <div
                className="h-full rounded-full transition-all duration-500"
                style={{
                  width: `${localPct}%`,
                  background: 'linear-gradient(90deg,#a78bfa,#34d399)',
                }}
              />
            </div>
          </div>
        )}

        {ecosystem.unprotocolled_devices > 0 && (
          <p className="mt-3 text-xs text-[color:var(--ink-muted)]">
            ⚠ {ecosystem.unprotocolled_devices} device{ecosystem.unprotocolled_devices > 1 ? 's' : ''} have no protocol assigned.
          </p>
        )}
      </section>

      {/* Protocol cards */}
      {ecosystem.protocols.length === 0 ? (
        <section className="surface-card flex flex-col items-center justify-center gap-2 p-12 text-center">
          <p className="text-2xl">⬡</p>
          <p className="text-sm text-[color:var(--ink-muted)]">
            No protocols detected yet. Add devices with a <code>control_protocol</code> to see the ecosystem map.
          </p>
        </section>
      ) : (
        <section className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
          {ecosystem.protocols.map(entry => (
            <ProtocolCard key={entry.id} entry={entry} />
          ))}
        </section>
      )}
    </div>
  );
}

function Stat({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div
      className="rounded-xl border p-3"
      style={{ borderColor: `${color}33`, background: `${color}0d` }}
    >
      <p className="text-[0.65rem] uppercase tracking-[0.1em] text-[color:var(--ink-muted)]">{label}</p>
      <p className="mt-1 text-2xl font-bold" style={{ color }}>{value}</p>
    </div>
  );
}
