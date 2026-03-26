import Link from 'next/link';
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'More — Smart Home',
};

const MORE_LINKS = [
  { href: '/discovery',  label: 'Discovery',  icon: '📡', description: 'Find new devices on your network' },
  { href: '/scripts',    label: 'Scripts',    icon: '📜', description: 'Multi-step automation scripts' },
  { href: '/ecosystem',  label: 'Ecosystem',  icon: '🌐', description: 'Protocol overview and mesh stats' },
  { href: '/dashboards', label: 'Dashboards', icon: '📊', description: 'Custom dashboard builder' },
  { href: '/areas',      label: 'Areas',      icon: '📍', description: 'Manage rooms and area assignments' },
  { href: '/energy',     label: 'Energy',     icon: '⚡', description: 'Power consumption and energy stats' },
  { href: '/settings',   label: 'Settings',   icon: '⚙️', description: 'Theme, notifications, and preferences' },
];

export default function MorePage() {
  return (
    <div className="space-y-5">
      <section className="surface-card p-5 sm:p-6">
        <p className="section-kicker">Navigation</p>
        <h1 className="section-title">More pages</h1>
        <p className="section-subtitle">Additional tools and settings for your smart home.</p>
      </section>

      <section className="surface-card divide-y divide-[color:var(--line)] overflow-hidden">
        {MORE_LINKS.map(({ href, label, icon, description }) => (
          <Link
            key={href}
            href={href}
            className="flex items-center gap-4 px-5 py-4 transition hover:bg-[var(--surface-hover)]"
          >
            <span className="text-2xl leading-none">{icon}</span>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-[color:var(--ink-strong)]">{label}</p>
              <p className="text-xs text-[color:var(--ink-muted)]">{description}</p>
            </div>
            <span className="text-[color:var(--ink-faint)]">→</span>
          </Link>
        ))}
      </section>
    </div>
  );
}
