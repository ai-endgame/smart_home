'use client';
import { useState, useEffect, useRef } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { clsx } from 'clsx';
import { useNotifications } from '@/lib/context/notification-context';

// All nav links (desktop shows all)
const allLinks = [
  { href: '/',           label: 'Dashboard',  icon: '⬡' },
  { href: '/devices',    label: 'Devices',    icon: '💡' },
  { href: '/rooms',      label: 'Rooms',      icon: '🏠' },
  { href: '/automation', label: 'Automation', icon: '⚡' },
  { href: '/scenes',     label: 'Scenes',     icon: '🎬' },
  { href: '/presence',   label: 'Presence',   icon: '👤' },
  { href: '/scripts',    label: 'Scripts',    icon: '📜' },
  { href: '/ecosystem',  label: 'Ecosystem',  icon: '🌐' },
  { href: '/areas',      label: 'Areas',      icon: '📍' },
  { href: '/dashboards', label: 'Dashboards', icon: '📊' },
  { href: '/discovery',  label: 'Discovery',  icon: '📡' },
];

// Mobile drawer: 6 primary + "More"
const mobileLinks = allLinks.slice(0, 6);

export function Nav() {
  const pathname = usePathname();
  const [open, setOpen] = useState(false);
  const drawerRef = useRef<HTMLDivElement>(null);
  const { unreadCount, open: openNotifications, close: closeNotifications, isOpen: notifOpen } = useNotifications();

  // Close mobile drawer on outside click
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (drawerRef.current && !drawerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [open]);

  // Close mobile drawer on route change
  useEffect(() => { setOpen(false); }, [pathname]);

  // Close notification drawer on route change
  useEffect(() => { closeNotifications(); }, [pathname, closeNotifications]);

  const activeLink = allLinks.find(l => l.href === pathname);

  const handleNavLinkClick = () => {
    setOpen(false);
    closeNotifications();
  };

  return (
    <nav className="sticky top-3 z-20 px-4 sm:px-6 lg:px-8" ref={drawerRef}>
      <div className="mx-auto w-full max-w-7xl">
        {/* ── Main bar ─────────────────────────────────── */}
        <div className="flex items-center justify-between gap-4 rounded-2xl border border-[var(--line)] bg-[var(--bg-nav)] px-4 py-2.5 shadow-[var(--shadow-nav)] backdrop-blur-2xl sm:px-5">
          {/* Logo */}
          <div className="flex items-center gap-3">
            <span className="inline-flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-xl bg-[linear-gradient(140deg,#6366f1,#06b6d4)] text-sm font-bold text-white shadow-[0_0_20px_rgba(99,102,241,0.45)]">
              SH
            </span>
            <div className="hidden sm:block">
              <p className="text-[0.65rem] uppercase tracking-[0.14em] text-[color:var(--ink-faint)]">Control Center</p>
              <p className="text-sm font-semibold leading-tight text-[color:var(--ink-strong)]">Smart Home</p>
            </div>
            {/* Active page pill (mobile only) */}
            <span className="sm:hidden text-sm font-semibold text-[color:var(--ink-strong)]">
              {activeLink?.label ?? 'Smart Home'}
            </span>
          </div>

          {/* Desktop links */}
          <div className="hidden lg:flex items-center gap-1 rounded-xl bg-[var(--surface)] p-1">
            {allLinks.map(({ href, label }) => {
              const active = pathname === href;
              return (
                <Link
                  key={href}
                  href={href}
                  onClick={handleNavLinkClick}
                  className={clsx(
                    'rounded-lg px-3 py-1.5 text-sm font-medium transition-all duration-200',
                    active
                      ? 'bg-[color:var(--accent)] text-white shadow-[0_4px_14px_var(--accent-glow)]'
                      : 'text-[color:var(--ink-muted)] hover:bg-[var(--surface-hover)] hover:text-[color:var(--ink-strong)]'
                  )}
                >
                  {label}
                </Link>
              );
            })}
          </div>

          {/* Right side: bell + burger */}
          <div className="flex items-center gap-2">
            {/* Bell icon button */}
            <button
              type="button"
              onClick={() => notifOpen ? closeNotifications() : openNotifications()}
              aria-label="Notifications"
              className="relative inline-flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-xl border border-[var(--line-strong)] bg-[var(--surface)] text-[color:var(--ink-muted)] transition hover:bg-[var(--surface-hover)] hover:text-[color:var(--ink-strong)] focus:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--accent)]"
            >
              <svg viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
                <path d="M10 2a6 6 0 00-6 6v2.586l-.707.707A1 1 0 004 13h12a1 1 0 00.707-1.707L16 10.586V8a6 6 0 00-6-6zm0 16a2 2 0 01-2-2h4a2 2 0 01-2 2z" />
              </svg>
              {unreadCount > 0 && (
                <span className="absolute -right-1 -top-1 flex h-4 w-4 items-center justify-center rounded-full bg-[color:var(--danger)] text-[0.6rem] font-bold text-white">
                  {unreadCount > 9 ? '9+' : unreadCount}
                </span>
              )}
            </button>

            {/* Burger button (hidden on lg+) */}
            <button
              type="button"
              onClick={() => setOpen(o => !o)}
              aria-label={open ? 'Close menu' : 'Open menu'}
              aria-expanded={open}
              className="lg:hidden inline-flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-xl border border-[var(--line-strong)] bg-[var(--surface)] text-[color:var(--ink-muted)] transition hover:bg-[var(--surface-hover)] hover:text-[color:var(--ink-strong)] focus:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--accent)]"
            >
              <svg
                className="h-4 w-4 transition-all duration-200"
                viewBox="0 0 20 20"
                fill="none"
                stroke="currentColor"
                strokeWidth={1.8}
                strokeLinecap="round"
              >
                {open ? (
                  <>
                    <line x1="4" y1="4" x2="16" y2="16" />
                    <line x1="16" y1="4" x2="4" y2="16" />
                  </>
                ) : (
                  <>
                    <line x1="3" y1="6" x2="17" y2="6" />
                    <line x1="3" y1="10" x2="17" y2="10" />
                    <line x1="3" y1="14" x2="17" y2="14" />
                  </>
                )}
              </svg>
            </button>
          </div>
        </div>

        {/* ── Mobile drawer ─────────────────────────────── */}
        <div
          className={clsx(
            'lg:hidden overflow-hidden transition-all duration-300 ease-in-out',
            open ? 'max-h-[32rem] opacity-100' : 'max-h-0 opacity-0'
          )}
        >
          <div className="mt-2 rounded-2xl border border-[var(--line)] bg-[var(--bg-nav-drawer)] p-2 shadow-[var(--shadow-modal)] backdrop-blur-2xl">
            <div className="grid grid-cols-2 gap-1 sm:grid-cols-3">
              {mobileLinks.map(({ href, label, icon }) => {
                const active = pathname === href;
                return (
                  <Link
                    key={href}
                    href={href}
                    onClick={handleNavLinkClick}
                    className={clsx(
                      'flex items-center gap-2.5 rounded-xl px-3 py-2.5 text-sm font-medium transition-all duration-150',
                      active
                        ? 'bg-[color:var(--accent)] text-white shadow-[0_4px_14px_var(--accent-glow)]'
                        : 'text-[color:var(--ink-muted)] hover:bg-[var(--surface-hover)] hover:text-[color:var(--ink-strong)]'
                    )}
                  >
                    <span className="text-base leading-none">{icon}</span>
                    {label}
                  </Link>
                );
              })}
              {/* More link */}
              <Link
                href="/more"
                onClick={handleNavLinkClick}
                className={clsx(
                  'flex items-center gap-2.5 rounded-xl px-3 py-2.5 text-sm font-medium transition-all duration-150',
                  pathname === '/more'
                    ? 'bg-[color:var(--accent)] text-white shadow-[0_4px_14px_var(--accent-glow)]'
                    : 'text-[color:var(--ink-muted)] hover:bg-[var(--surface-hover)] hover:text-[color:var(--ink-strong)]'
                )}
              >
                <span className="text-base leading-none">···</span>
                More →
              </Link>
            </div>
          </div>
        </div>
      </div>
    </nav>
  );
}
