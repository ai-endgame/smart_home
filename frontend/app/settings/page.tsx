'use client';
import type { Metadata } from 'next';
import { useSettings } from '@/lib/hooks/use-settings';
import { useAuth } from '@/lib/hooks/use-auth';

export default function SettingsPage() {
  const { settings, update } = useSettings();
  const { logout, isAuthenticated } = useAuth();

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-[color:var(--ink-strong)]">Settings</h1>

      {/* Theme */}
      <div className="surface-card divide-y divide-[color:var(--line)] overflow-hidden">
        <div className="px-5 py-4">
          <p className="text-xs font-semibold uppercase tracking-wider text-[color:var(--ink-muted)]">Appearance</p>
        </div>
        <div className="px-5 py-4">
          <p className="mb-3 text-sm font-medium text-[color:var(--ink-strong)]">Theme</p>
          <div className="flex gap-3">
            {(['system', 'dark', 'light'] as const).map(t => (
              <button
                key={t}
                type="button"
                onClick={() => update({ theme: t })}
                className="flex-1 rounded-xl border py-2.5 text-sm font-medium capitalize transition"
                style={{
                  borderColor: settings.theme === t ? 'var(--accent)' : 'var(--line-strong)',
                  background: settings.theme === t ? 'var(--accent-soft)' : 'transparent',
                  color: settings.theme === t ? 'var(--accent)' : 'var(--ink-muted)',
                }}
              >
                {t}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Notifications */}
      <div className="surface-card divide-y divide-[color:var(--line)] overflow-hidden">
        <div className="px-5 py-4">
          <p className="text-xs font-semibold uppercase tracking-wider text-[color:var(--ink-muted)]">Notifications</p>
        </div>
        <div className="flex items-center justify-between px-5 py-4">
          <div>
            <p className="text-sm font-medium text-[color:var(--ink-strong)]">Notification Sound</p>
            <p className="text-xs text-[color:var(--ink-muted)]">Play a sound for alerts</p>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={settings.soundEnabled}
            onClick={() => update({ soundEnabled: !settings.soundEnabled })}
            className="relative inline-flex h-6 w-11 shrink-0 rounded-full transition-colors"
            style={{ background: settings.soundEnabled ? 'var(--accent)' : 'var(--line-strong)' }}
          >
            <span
              className="inline-block h-[18px] w-[18px] rounded-full bg-white shadow-sm transition-transform duration-200"
              style={{
                transform: settings.soundEnabled ? 'translate(21px, 3px)' : 'translate(3px, 3px)',
              }}
            />
          </button>
        </div>
      </div>

      {/* Navigation */}
      <div className="surface-card divide-y divide-[color:var(--line)] overflow-hidden">
        <div className="px-5 py-4">
          <p className="text-xs font-semibold uppercase tracking-wider text-[color:var(--ink-muted)]">Navigation</p>
        </div>
        <div className="px-5 py-4">
          <label htmlFor="landing-page" className="mb-2 block text-sm font-medium text-[color:var(--ink-strong)]">
            Landing Page
          </label>
          <select
            id="landing-page"
            value={settings.landingPage}
            onChange={e => update({ landingPage: e.target.value })}
            className="w-full rounded-xl border border-[var(--line-strong)] bg-[var(--input-bg)] px-3 py-2 text-sm text-[color:var(--ink-strong)] focus:border-[color:var(--accent)] focus:outline-none"
          >
            <option value="/">Dashboard</option>
            <option value="/devices">Devices</option>
            <option value="/automation">Automation</option>
            <option value="/rooms">Rooms</option>
            <option value="/energy">Energy</option>
          </select>
        </div>
      </div>

      {/* Account */}
      {isAuthenticated && (
        <div className="surface-card divide-y divide-[color:var(--line)] overflow-hidden">
          <div className="px-5 py-4">
            <p className="text-xs font-semibold uppercase tracking-wider text-[color:var(--ink-muted)]">Account</p>
          </div>
          <div className="px-5 py-4">
            <button
              type="button"
              onClick={logout}
              className="w-full rounded-xl border border-[rgba(244,63,94,0.3)] bg-[rgba(244,63,94,0.06)] py-2.5 text-sm font-medium text-[color:var(--danger)] transition hover:bg-[rgba(244,63,94,0.12)]"
            >
              Sign out
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
