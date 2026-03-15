'use client';
import { useDevices } from '@/lib/hooks/use-devices';
import { useAutomation } from '@/lib/hooks/use-automation';

const metricStyle = [
  'from-[#effaf4] to-[#dff4ea] border-[#b8e5d1]',
  'from-[#ecf6ff] to-[#dceefe] border-[#b7d7f1]',
  'from-[#fff8e9] to-[#fdf1cc] border-[#eedaa0]',
  'from-[#f8f0ff] to-[#eee0ff] border-[#d9c0f4]',
];

export default function DashboardPage() {
  const { devices } = useDevices();
  const { rules } = useAutomation();

  const online = devices.filter(d => d.connected).length;
  const on = devices.filter(d => d.state === 'on').length;
  const errorCount = devices.filter(d => d.last_error).length;

  const metrics = [
    { label: 'Total Devices', value: devices.length },
    { label: 'Online', value: online },
    { label: 'Active Now', value: on },
    { label: 'Automation Rules', value: rules.length },
  ];

  return (
    <div className="space-y-6">
      <section className="surface-card overflow-hidden p-5 sm:p-6">
        <div className="grid gap-5 lg:grid-cols-[1.35fr_1fr]">
          <div>
            <p className="section-kicker">Smart Home Dashboard</p>
            <h1 className="section-title">Keep every room in sync.</h1>
            <p className="section-subtitle">
              Monitor device health, run automations, and react to issues from one place.
            </p>
          </div>
          <div className="grid grid-cols-2 gap-3">
            <div className="rounded-2xl border border-[color:var(--line)] bg-white/80 p-3">
              <p className="text-[0.72rem] uppercase tracking-[0.09em] text-[color:var(--ink-muted)]">Online Ratio</p>
              <p className="mt-1 text-2xl font-semibold text-[color:var(--ink-strong)]">
                {devices.length === 0 ? '0%' : `${Math.round((online / devices.length) * 100)}%`}
              </p>
            </div>
            <div className="rounded-2xl border border-[color:var(--line)] bg-white/80 p-3">
              <p className="text-[0.72rem] uppercase tracking-[0.09em] text-[color:var(--ink-muted)]">Alerts</p>
              <p className="mt-1 text-2xl font-semibold text-[color:var(--ink-strong)]">{errorCount}</p>
            </div>
            <div className="rounded-2xl border border-[color:var(--line)] bg-white/80 p-3 col-span-2">
              <p className="text-[0.72rem] uppercase tracking-[0.09em] text-[color:var(--ink-muted)]">Quick Status</p>
              <p className="mt-1 text-sm text-[color:var(--ink-muted)]">
                {errorCount > 0
                  ? `${errorCount} device${errorCount > 1 ? 's' : ''} need attention.`
                  : 'All monitored devices are operating normally.'}
              </p>
            </div>
          </div>
        </div>
      </section>

      <section className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
        {metrics.map(({ label, value }, index) => (
          <article
            key={label}
            className={`metric-card border bg-gradient-to-br p-4 ${metricStyle[index % metricStyle.length]}`}
          >
            <p className="text-xs uppercase tracking-[0.09em] text-[color:var(--ink-muted)]">{label}</p>
            <p className="mt-2 text-3xl font-semibold text-[color:var(--ink-strong)]">{value}</p>
          </article>
        ))}
      </section>
    </div>
  );
}
