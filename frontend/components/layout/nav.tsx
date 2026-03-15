'use client';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { clsx } from 'clsx';

const links = [
  { href: '/', label: 'Dashboard' },
  { href: '/devices', label: 'Devices' },
  { href: '/discovery', label: 'Discovery' },
  { href: '/automation', label: 'Automation' },
];

export function Nav() {
  const pathname = usePathname();

  return (
    <nav className="sticky top-3 z-20 px-4 sm:px-6 lg:px-8">
      <div className="mx-auto flex w-full max-w-7xl items-center justify-between gap-4 rounded-2xl border border-[color:var(--line)] bg-[color:var(--surface)] px-4 py-3 shadow-[0_12px_28px_rgba(13,28,24,0.12)] backdrop-blur-xl sm:px-5">
        <div className="flex items-center gap-3">
          <span className="inline-flex h-9 w-9 items-center justify-center rounded-xl bg-[linear-gradient(160deg,#1f8f6a,#2c6da6)] text-base font-semibold text-white shadow-[0_8px_20px_rgba(32,91,74,0.35)]">
            SH
          </span>
          <div>
            <p className="text-[0.68rem] uppercase tracking-[0.14em] text-[color:var(--ink-muted)]">Control Center</p>
            <p className="font-semibold leading-tight text-[color:var(--ink-strong)]">Smart Home</p>
          </div>
        </div>

        <div className="flex flex-wrap items-center justify-end gap-1 rounded-xl bg-white/65 p-1">
          {links.map(({ href, label }) => {
            const active = pathname === href;
            return (
              <Link
                key={href}
                href={href}
                className={clsx(
                  'rounded-lg px-3 py-1.5 text-sm font-medium transition-all duration-200',
                  active
                    ? 'bg-[color:var(--accent)] text-white shadow-[0_8px_18px_rgba(31,143,106,0.35)]'
                    : 'text-[color:var(--ink-muted)] hover:bg-white hover:text-[color:var(--ink-strong)]'
                )}
              >
                {label}
              </Link>
            );
          })}
        </div>
      </div>
    </nav>
  );
}
