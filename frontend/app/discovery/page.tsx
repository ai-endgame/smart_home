'use client';
import { useState } from 'react';
import { useDiscovery } from '@/lib/hooks/use-discovery';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { CommissionModal } from '@/components/devices/commission-modal';

// ── Human-readable labels for known DNS-SD / mDNS TXT record keys ──────────
const PROP_LABELS: Record<string, string> = {
  // General
  fn: 'Friendly Name', n: 'Name', md: 'Model', manufacturer: 'Manufacturer',
  model: 'Model', serial: 'Serial', fw: 'Firmware', hw: 'Hardware',
  version: 'Version', id: 'Device ID', ip: 'IP Address', mac: 'MAC Address',
  // ESPHome
  board: 'Board', project_name: 'Project', project_version: 'Project Version',
  network: 'Network',
  // Google Cast
  ve: 'Version', ca: 'Capabilities', rs: 'Status', bs: 'Bluetooth',
  st: 'Stage',
  // HomeKit (show only non-cryptic ones)
  pv: 'Protocol Version', ci: 'Category',
  // WLED
  wled: 'WLED Version',
};

// Keys that are too cryptic/binary to show to a user
const PROP_BLOCKLIST = new Set(['sf', 'sh', 'ff', 's#', 'c#', 'pk', 'flags', 'type']);

// Human-readable service type labels
const SERVICE_LABELS: Record<string, string> = {
  '_hap._tcp.local.':         'HomeKit',
  '_googlecast._tcp.local.':  'Google Cast',
  '_esphomelib._tcp.local.':  'ESPHome',
  '_wled._tcp.local.':        'WLED LED',
  '_shelly._tcp.local.':      'Shelly',
  '_hue._tcp.local.':         'Philips Hue',
  '_arduino._tcp.local.':     'Arduino',
  '_smartthings._tcp.local.': 'SmartThings',
  '_matter._tcp.local.':      'Matter',
};

const TYPE_ICONS: Record<string, string> = {
  light: '💡', thermostat: '🌡️', fan: '🌀',
  lock: '🔒', switch: '⚡', outlet: '🔌',
  tv: '📺', speaker: '🔊', media_player: '▶️',
  sensor: '📡', camera: '📷', alarm: '🚨',
  cover: '🪟', hub: '🔗',
};

function serviceLabel(raw: string) {
  return SERVICE_LABELS[raw] ?? raw.replace(/^_/, '').replace(/\._tcp\.local\.$/, '').replace(/_/g, ' ');
}

function readableProps(props: Record<string, string>) {
  return Object.entries(props)
    .filter(([k]) => !PROP_BLOCKLIST.has(k) && PROP_LABELS[k])
    .map(([k, v]) => ({ label: PROP_LABELS[k], value: v }))
    .filter(({ value }) => value.trim().length > 0);
}

export default function DiscoveryPage() {
  const { discovered, isLoading, addToHome, refresh } = useDiscovery();
  const [commissioning, setCommissioning] = useState<{ name: string; id: string } | null>(null);

  return (
    <div className="space-y-5">
      {/* Header */}
      <section className="surface-card p-5 sm:p-6">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="section-kicker">Network Discovery</p>
            <h1 className="section-title">Detect devices on your network</h1>
            <p className="section-subtitle">
              Scan mDNS services and add discovered hardware into your home graph.
            </p>
          </div>
          <Button variant="secondary" onClick={() => refresh()}>
            ↺ Refresh Scan
          </Button>
        </div>
      </section>

      {/* States */}
      {isLoading ? (
        <section className="surface-card p-6">
          <p className="text-sm text-[color:var(--ink-muted)]">Scanning network…</p>
        </section>
      ) : discovered.length === 0 ? (
        <section className="surface-card flex flex-col items-center gap-3 p-12 text-center">
          <span className="text-4xl opacity-30">📡</span>
          <p className="font-medium text-[color:var(--ink-strong)]">No devices found</p>
          <p className="text-sm text-[color:var(--ink-muted)]">
            Make sure mDNS-compatible devices are on the same network and discovery is enabled.
          </p>
        </section>
      ) : (
        <section className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
          {discovered.map(d => {
            const icon = TYPE_ICONS[d.suggested_type] ?? '📦';
            const svcLabel = serviceLabel(d.service_type);
            const props = readableProps(d.properties);

            return (
              <article key={d.id} className="surface-card flex flex-col gap-4 p-5">
                {/* Top: icon + name + protocol badge */}
                <div className="flex items-start justify-between gap-3">
                  <div className="flex items-start gap-3">
                    <span className="inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-[rgba(6,182,212,0.1)] text-xl">
                      {icon}
                    </span>
                    <div>
                      <p className="font-semibold leading-snug text-[color:var(--ink-strong)]">{d.name}</p>
                      <p className="mt-0.5 text-xs capitalize text-[color:var(--ink-muted)]">{d.suggested_type}</p>
                    </div>
                  </div>
                  <Badge label={svcLabel} variant="info" />
                </div>

                {/* Network info */}
                <div className="rounded-xl border border-[rgba(6,182,212,0.15)] bg-[rgba(6,182,212,0.05)] px-3 py-2.5 text-xs">
                  <div className="flex items-center justify-between gap-2">
                    <span className="text-[color:var(--ink-muted)]">Host</span>
                    <span className="font-mono font-medium text-[#22d3ee]">
                      {d.host}:{d.port}
                    </span>
                  </div>
                  {d.addresses.length > 0 && (
                    <div className="mt-1.5 flex flex-wrap items-center justify-between gap-2">
                      <span className="text-[color:var(--ink-muted)]">
                        {d.addresses.length === 1 ? 'Address' : 'Addresses'}
                      </span>
                      <span className="font-mono font-medium text-[#22d3ee]">
                        {d.addresses.slice(0, 2).join(', ')}
                        {d.addresses.length > 2 && ` +${d.addresses.length - 2}`}
                      </span>
                    </div>
                  )}
                </div>

                {/* Readable device properties */}
                {props.length > 0 && (
                  <dl className="grid grid-cols-1 gap-1 text-xs">
                    {props.map(({ label, value }) => (
                      <div key={label} className="flex items-center justify-between gap-3">
                        <dt className="text-[color:var(--ink-muted)]">{label}</dt>
                        <dd className="truncate font-medium text-[color:var(--ink-strong)]" title={value}>
                          {value}
                        </dd>
                      </div>
                    ))}
                  </dl>
                )}

                {/* Actions */}
                <div className="mt-auto flex gap-2">
                  <Button
                    size="sm"
                    className="flex-1"
                    onClick={() =>
                      addToHome({
                        discovered_id: d.id,
                        name: d.name.replace(/[^a-z0-9_]/gi, '_').toLowerCase(),
                        device_type: d.suggested_type,
                      })
                    }
                  >
                    + Add to Home
                  </Button>
                  {d.protocol === 'matter' && (
                    <Button
                      size="sm"
                      variant="secondary"
                      onClick={() => setCommissioning({ name: d.name, id: d.id })}
                    >
                      Commission
                    </Button>
                  )}
                </div>
              </article>
            );
          })}
        </section>
      )}

      {commissioning && (
        <CommissionModal
          initialDeviceName={commissioning.name}
          onClose={() => setCommissioning(null)}
        />
      )}
    </div>
  );
}
