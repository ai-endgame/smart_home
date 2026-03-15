'use client';
import { useAutomation } from '@/lib/hooks/use-automation';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';

export default function AutomationPage() {
  const { rules, isLoading, remove, toggle, run } = useAutomation();

  return (
    <div className="space-y-5">
      <section className="surface-card p-5 sm:p-6">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="section-kicker">Automation Hub</p>
            <h1 className="section-title">Rules that run your home</h1>
            <p className="section-subtitle">Review triggers, action chains, and execute all enabled rules on demand.</p>
          </div>
          <Button onClick={run}>Run Rules</Button>
        </div>
      </section>

      {isLoading ? (
        <section className="surface-card p-6">
          <p className="text-sm text-[color:var(--ink-muted)]">Loading rules...</p>
        </section>
      ) : rules.length === 0 ? (
        <section className="surface-card p-6">
          <p className="text-sm text-[color:var(--ink-muted)]">No automation rules defined. Use the backend API to add rules.</p>
        </section>
      ) : (
        <section className="space-y-3">
          {rules.map(rule => (
            <article
              key={rule.name}
              className="surface-card flex flex-col gap-4 p-4 md:flex-row md:items-start md:justify-between"
            >
              <div>
                <div className="flex flex-wrap items-center gap-2">
                  <p className="font-semibold text-[color:var(--ink-strong)]">{rule.name}</p>
                  <Badge label={rule.enabled ? 'enabled' : 'disabled'} variant={rule.enabled ? 'success' : 'default'} />
                </div>
                <div className="mt-2 grid gap-2 text-xs text-[color:var(--ink-muted)] sm:grid-cols-2">
                  <p className="rounded-xl border border-[color:var(--line)] bg-white/70 px-3 py-2">
                    Trigger: {rule.trigger.type} on {rule.trigger.device_name}
                    {rule.trigger.threshold != null ? ` (${rule.trigger.threshold})` : ''}
                    {rule.trigger.target_state ? ` -> ${rule.trigger.target_state}` : ''}
                  </p>
                  <p className="rounded-xl border border-[color:var(--line)] bg-white/70 px-3 py-2">
                    Action: {rule.action.type} on {rule.action.device_name}
                  </p>
                </div>
              </div>
              <div className="flex shrink-0 gap-2">
                <Button size="sm" variant="ghost" onClick={() => toggle(rule.name)}>
                  {rule.enabled ? 'Disable' : 'Enable'}
                </Button>
                <Button size="sm" variant="danger" onClick={() => remove(rule.name)}>
                  Delete
                </Button>
              </div>
            </article>
          ))}
        </section>
      )}
    </div>
  );
}
