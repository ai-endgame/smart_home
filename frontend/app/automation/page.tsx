'use client';
import { useState } from 'react';
import { useAutomation } from '@/lib/hooks/use-automation';
import { useDevices } from '@/lib/hooks/use-devices';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { AddRuleModal } from '@/components/automation/add-rule-modal';
import { RuleRowSkeleton } from '@/components/ui/skeleton';
import { EmptyState } from '@/components/ui/empty-state';
import type { CreateRuleRequest, Trigger, Action } from '@/lib/api/types';

// ── Display helpers ────────────────────────────────────────────────────────────

function triggerSummary(trigger: Trigger): string {
  switch (trigger.type) {
    case 'state_change':
      return `${trigger.device_name} → ${trigger.target_state ?? 'any'}`;
    case 'temperature_above':
      return `${trigger.device_name} temp > ${trigger.threshold}°C`;
    case 'temperature_below':
      return `${trigger.device_name} temp < ${trigger.threshold}°C`;
    case 'time':
      return `At ${trigger.time}`;
    case 'sun':
      return trigger.offset_minutes && trigger.offset_minutes !== 0
        ? `${trigger.event} ${trigger.offset_minutes > 0 ? '+' : ''}${trigger.offset_minutes}min`
        : String(trigger.event);
    case 'numeric_state_above':
      return `${trigger.device_name} ${trigger.attribute} > ${trigger.threshold}`;
    case 'numeric_state_below':
      return `${trigger.device_name} ${trigger.attribute} < ${trigger.threshold}`;
    case 'webhook':
      return `Webhook: ${trigger.id}`;
    default:
      return `${trigger.type}`;
  }
}

function actionSummary(action: Action): string {
  switch (action.type) {
    case 'state':
      return `${action.device_name} → ${action.state}`;
    case 'brightness':
      return `${action.device_name} brightness → ${action.brightness}%`;
    case 'temperature':
      return `${action.device_name} temp → ${action.temperature}°C`;
    case 'notify':
      return `Notify: ${action.message}`;
    default:
      return `${action.type}`;
  }
}

// ── Starter templates ──────────────────────────────────────────────────────────

interface TemplateRule {
  name: string;
  description: string;
  icon: string;
  rule: CreateRuleRequest;
}

const STARTER_TEMPLATES: TemplateRule[] = [
  {
    name: 'Motion Light',
    description: 'Turn on a light when a motion sensor activates',
    icon: '💡',
    rule: {
      name: 'motion_light',
      trigger: { type: 'state_change', device_name: 'motion_sensor', target_state: 'on' },
      action:  { type: 'state', device_name: 'hallway_light', state: 'on' },
    },
  },
  {
    name: 'Sunrise Blinds',
    description: 'Open covers automatically at sunrise',
    icon: '🌅',
    rule: {
      name: 'sunrise_blinds',
      trigger: { type: 'sun', event: 'sunrise', offset_minutes: 0 },
      action:  { type: 'state', device_name: 'living_room_cover', state: 'on' },
    },
  },
  {
    name: 'Low Battery Alert',
    description: 'Get notified when a sensor brightness drops below 20%',
    icon: '🔋',
    rule: {
      name: 'low_battery_alert',
      trigger: { type: 'numeric_state_below', device_name: 'door_sensor', attribute: 'brightness', threshold: 20 },
      action:  { type: 'notify', message: 'Low battery on door_sensor!' },
    },
  },
];

// ── Page ───────────────────────────────────────────────────────────────────────

export default function AutomationPage() {
  const { rules, isLoading, add, remove, toggle, run, toggleSafe } = useAutomation();
  const { devices } = useDevices();
  const [open, setOpen] = useState(false);
  const [templateInit, setTemplateInit] = useState<Partial<CreateRuleRequest> | undefined>(undefined);
  const [templatesExpanded, setTemplatesExpanded] = useState(true);

  const deviceNames = devices.map(d => d.name);

  const handleAdd = async (req: CreateRuleRequest) => {
    await add(req);
  };

  const openWithTemplate = (t: TemplateRule) => {
    setTemplateInit(t.rule);
    setOpen(true);
  };

  return (
    <div className="space-y-5">
      {/* Header */}
      <section className="surface-card p-5 sm:p-6">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="section-kicker">Automation Hub</p>
            <h1 className="section-title">Rules that run your home</h1>
            <p className="section-subtitle">
              Triggers, conditions, time windows, and on-demand webhook execution.
            </p>
          </div>
          <div className="flex gap-2">
            <Button variant="secondary" onClick={run}>▶ Run All</Button>
            <Button onClick={() => { setTemplateInit(undefined); setOpen(true); }}>+ Add Rule</Button>
          </div>
        </div>
      </section>

      {/* ── Starter Templates ──────────────────────────────── */}
      <section className="surface-card p-5">
        <button
          onClick={() => setTemplatesExpanded(v => !v)}
          className="flex w-full items-center justify-between text-left"
        >
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.08em] text-[color:var(--ink-muted)]">
              Starter Templates
            </p>
            <p className="mt-0.5 text-sm text-[color:var(--ink-muted)]">
              Click a template to pre-fill the rule form
            </p>
          </div>
          <span className="text-[color:var(--ink-faint)]">{templatesExpanded ? '▾' : '▸'}</span>
        </button>

        {templatesExpanded && (
          <div className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-3">
            {STARTER_TEMPLATES.map(t => (
              <button
                key={t.name}
                onClick={() => openWithTemplate(t)}
                className="flex items-start gap-3 rounded-xl border border-[color:var(--line)] bg-[var(--surface)] p-4 text-left transition-all duration-150 hover:border-[color:var(--accent)] hover:bg-[var(--surface-hover)] hover:-translate-y-0.5"
              >
                <span className="text-2xl">{t.icon}</span>
                <div>
                  <p className="font-semibold text-[color:var(--ink-strong)]">{t.name}</p>
                  <p className="mt-0.5 text-xs text-[color:var(--ink-muted)]">{t.description}</p>
                  <div className="mt-2 flex flex-wrap gap-1.5">
                    <span className="rounded-md bg-[var(--warn-soft)] px-2 py-0.5 text-[10px] text-[color:var(--warn)]">
                      {triggerSummary(t.rule.trigger)}
                    </span>
                    <span className="rounded-md bg-[var(--accent-soft)] px-2 py-0.5 text-[10px] text-[color:var(--accent)]">
                      {actionSummary(t.rule.action)}
                    </span>
                  </div>
                </div>
              </button>
            ))}
          </div>
        )}
      </section>

      {/* ── Rules list ─────────────────────────────────────── */}
      {isLoading ? (
        <section className="space-y-3">
          {Array.from({ length: 3 }).map((_, i) => <RuleRowSkeleton key={i} />)}
        </section>
      ) : rules.length === 0 ? (
        <EmptyState
          icon="⚡"
          title="No automation rules yet"
          subtitle="Use a starter template above or create a custom rule to automate your home."
          action={<Button onClick={() => { setTemplateInit(undefined); setOpen(true); }}>+ Add Rule</Button>}
        />
      ) : (
        <section className="space-y-3">
          {rules.map(rule => (
            <article
              key={rule.name}
              className="surface-card flex flex-col gap-4 p-5 md:flex-row md:items-center md:justify-between"
            >
              {/* Rule info */}
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-2">
                  <p className="font-semibold text-[color:var(--ink-strong)]">{rule.name}</p>
                  <Badge
                    label={rule.enabled ? 'enabled' : 'disabled'}
                    variant={rule.enabled ? 'success' : 'default'}
                  />
                  {rule.safe_mode && (
                    <span className="rounded-md bg-[var(--warn-soft)] px-2 py-0.5 text-[10px] font-semibold text-[color:var(--warn)]">
                      SAFE
                    </span>
                  )}
                  {rule.time_range && (
                    <span className="rounded-md bg-[var(--info-soft)] px-2 py-0.5 text-[10px] text-[color:var(--info)]">
                      🕐 {rule.time_range.from}–{rule.time_range.to}
                    </span>
                  )}
                </div>

                {/* Trigger → Action flow */}
                <div className="mt-3 flex flex-wrap items-center gap-2 text-xs">
                  <div className="flex items-center gap-1.5 rounded-lg border border-[color:var(--warn-soft)] bg-[var(--warn-soft)] px-2.5 py-1.5">
                    <span className="text-[color:var(--warn)]">⚡</span>
                    <span className="text-[color:var(--ink-muted)]">When</span>
                    <span className="font-medium text-[color:var(--ink-strong)]">{triggerSummary(rule.trigger)}</span>
                  </div>
                  <span className="text-[color:var(--ink-faint)]">→</span>
                  <div className="flex items-center gap-1.5 rounded-lg border border-[color:var(--accent-soft)] bg-[var(--accent-soft)] px-2.5 py-1.5">
                    <span className="text-[color:var(--accent)]">↪</span>
                    <span className="text-[color:var(--ink-muted)]">Then</span>
                    <span className="font-medium text-[color:var(--ink-strong)]">{actionSummary(rule.action)}</span>
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div className="flex shrink-0 gap-2">
                <Button size="sm" variant="secondary" onClick={() => toggleSafe(rule.name)}>
                  {rule.safe_mode ? 'Unsafe' : 'Safe'}
                </Button>
                <Button size="sm" variant="secondary" onClick={() => toggle(rule.name)}>
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

      <AddRuleModal
        open={open}
        onClose={() => { setOpen(false); setTemplateInit(undefined); }}
        onAdd={handleAdd}
        deviceNames={deviceNames}
        initialValues={templateInit}
      />
    </div>
  );
}
