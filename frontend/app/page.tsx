'use client';
import { useRef } from 'react';
import Link from 'next/link';
import { useDevices } from '@/lib/hooks/use-devices';
import { useAutomation } from '@/lib/hooks/use-automation';
import { useAreas } from '@/lib/hooks/use-areas';
import { downloadBackup, restoreBackup } from '@/lib/api/system';
import { QuickActionsStrip } from '@/components/dashboard/quick-actions-strip';

// ── Metric definitions ────────────────────────────────────────────────────────

const METRICS = [
  {
    key: 'total' as const,
    label: 'Total Devices',
    icon: '⬡',
    accent: '#818cf8',
    glow: 'rgba(129,140,248,0.22)',
    bg: 'rgba(129,140,248,0.08)',
    border: 'rgba(129,140,248,0.2)',
  },
  {
    key: 'online' as const,
    label: 'Online',
    icon: '●',
    accent: '#34d399',
    glow: 'rgba(52,211,153,0.22)',
    bg: 'rgba(52,211,153,0.08)',
    border: 'rgba(52,211,153,0.2)',
  },
  {
    key: 'active' as const,
    label: 'Active Now',
    icon: '⚡',
    accent: '#fbbf24',
    glow: 'rgba(251,191,36,0.22)',
    bg: 'rgba(251,191,36,0.08)',
    border: 'rgba(251,191,36,0.2)',
  },
  {
    key: 'rules' as const,
    label: 'Automation Rules',
    icon: '⟳',
    accent: '#22d3ee',
    glow: 'rgba(34,211,238,0.22)',
    bg: 'rgba(34,211,238,0.08)',
    border: 'rgba(34,211,238,0.2)',
  },
] as const;

// ── Page ─────────────────────────────────────────────────────────────────────

export default function DashboardPage() {
  const { devices } = useDevices();
  const { rules } = useAutomation();
  const { areas } = useAreas();
  const restoreInputRef = useRef<HTMLInputElement>(null);

  const handleRestore = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    try {
      const counts = await restoreBackup(file);
      alert(`Restored: ${Object.entries(counts).map(([k, v]) => `${v} ${k}`).join(', ')}`);
    } catch (err) {
      alert(`Restore failed: ${err instanceof Error ? err.message : String(err)}`);
    }
    e.target.value = '';
  };

  const online  = devices.filter(d => d.connected).length;
  const active  = devices.filter(d => d.state === 'on').length;
  const errors  = devices.filter(d => d.last_error).length;
  const enabledRules = rules.filter(r => r.enabled).length;
  const onlinePct = devices.length === 0 ? 100 : Math.round((online / devices.length) * 100);

  const values: Record<string, number> = {
    total: devices.length,
    online,
    active,
    rules: rules.length,
  };

  const subtexts: Record<string, string> = {
    total:  devices.length === 0 ? 'No devices added' : `${active} active, ${devices.length - active} idle`,
    online: devices.length === 0 ? '—' : `${onlinePct}% connectivity`,
    active: active === 0 ? 'All devices idle' : `${active} of ${devices.length} running`,
    rules:  rules.length === 0 ? 'No rules configured' : `${enabledRules} enabled`,
  };

  const allClear = errors === 0 && devices.length > 0;

  return (
    <div className="space-y-5">

      {/* ── Status bar ─────────────────────────────────────── */}
      <section className="surface-card px-5 py-4 sm:px-6">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div className="flex items-center gap-3">
            <div>
              <p className="section-kicker">Smart Home</p>
              <h1 className="mt-0.5 text-xl font-semibold text-[color:var(--ink-strong)]">Dashboard</h1>
            </div>
            <div
              className="hidden items-center gap-1.5 rounded-full border px-3 py-1 text-xs font-semibold sm:flex"
              style={allClear
                ? { borderColor: 'var(--success-soft)', background: 'var(--success-soft)', color: 'var(--success)' }
                : { borderColor: 'var(--danger-soft)', background: 'var(--danger-soft)', color: 'var(--danger)' }
              }
            >
              <span>{allClear ? '✓' : '⚠'}</span>
              {allClear ? 'All clear' : `${errors} alert${errors > 1 ? 's' : ''}`}
            </div>
            {areas.length > 0 && (
              <Link
                href="/rooms"
                className="hidden items-center gap-1 rounded-full border border-[var(--line-strong)] px-3 py-1 text-xs font-medium text-[color:var(--ink-muted)] transition hover:border-[color:var(--accent)] hover:text-[color:var(--accent)] sm:flex"
              >
                Browse by room →
              </Link>
            )}
          </div>

          {/* System tools */}
          <div className="flex items-center gap-2">
            <button
              onClick={() => downloadBackup().catch(console.error)}
              className="inline-flex items-center gap-1.5 rounded-lg border border-[color:var(--line-strong)] bg-[var(--surface)] px-3 py-1.5 text-xs font-medium text-[color:var(--ink-muted)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[color:var(--ink-strong)]"
            >
              ↓ Backup
            </button>
            <button
              onClick={() => restoreInputRef.current?.click()}
              className="inline-flex items-center gap-1.5 rounded-lg border border-[color:var(--line-strong)] bg-[var(--surface)] px-3 py-1.5 text-xs font-medium text-[color:var(--ink-muted)] transition-colors hover:bg-[var(--surface-hover)] hover:text-[color:var(--ink-strong)]"
            >
              ↑ Restore
            </button>
            <input ref={restoreInputRef} type="file" accept="application/json" className="hidden" onChange={handleRestore} />
          </div>
        </div>
      </section>

      {/* ── Quick Actions Strip ─────────────────────────────── */}
      <QuickActionsStrip />

      {/* ── Metric cards ───────────────────────────────────── */}
      <section className="grid grid-cols-2 gap-4 xl:grid-cols-4">
        {METRICS.map(({ key, label, icon, accent, glow, bg, border }) => (
          <article
            key={key}
            className="metric-card p-5 hover:-translate-y-0.5 transition-transform duration-200"
            style={{ borderColor: border, background: bg, boxShadow: `0 0 28px ${glow}` }}
          >
            {/* Decorative blob */}
            <div
              className="pointer-events-none absolute -right-4 -top-4 h-20 w-20 rounded-full opacity-25 blur-2xl"
              style={{ background: accent }}
              aria-hidden
            />
            <div className="relative">
              <div className="flex items-center justify-between">
                <p className="text-xs uppercase tracking-[0.1em] text-[color:var(--ink-muted)]">{label}</p>
                <span className="text-sm" style={{ color: accent }}>{icon}</span>
              </div>
              <p className="mt-3 text-4xl font-bold leading-none" style={{ color: accent }}>
                {values[key]}
              </p>
              <p className="mt-2 text-[0.72rem] text-[color:var(--ink-faint)]">{subtexts[key]}</p>
              {key === 'online' && devices.length > 0 && (
                <div className="mt-3 h-1 w-full overflow-hidden rounded-full bg-[rgba(255,255,255,0.08)]">
                  <div
                    className="h-full rounded-full transition-all duration-700"
                    style={{ width: `${onlinePct}%`, background: accent }}
                  />
                </div>
              )}
            </div>
          </article>
        ))}
      </section>

    </div>
  );
}
