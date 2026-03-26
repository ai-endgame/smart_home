'use client';
import { useState } from 'react';
import Link from 'next/link';
import { useDashboards } from '@/lib/hooks/use-dashboards';
import { Button } from '@/components/ui/button';
import { Modal } from '@/components/ui/modal';
import { Field } from '@/components/ui/field';
import type { CreateCardRequest, Dashboard, View } from '@/lib/api/types';

type CardTypeKey = 'entity_card' | 'gauge_card' | 'button_card' | 'stat_card' | 'history_card';

const CARD_TYPES: { key: CardTypeKey; label: string }[] = [
  { key: 'entity_card', label: 'Entity' },
  { key: 'gauge_card', label: 'Gauge' },
  { key: 'button_card', label: 'Button' },
  { key: 'stat_card', label: 'Stat' },
  { key: 'history_card', label: 'History' },
];

function buildCardRequest(cardType: CardTypeKey, fields: Record<string, string>): CreateCardRequest | null {
  switch (cardType) {
    case 'entity_card':
      if (!fields.entity_id) return null;
      return { card_type: 'entity_card', entity_id: fields.entity_id };
    case 'gauge_card':
      if (!fields.entity_id) return null;
      return { card_type: 'gauge_card', entity_id: fields.entity_id, min: parseFloat(fields.min ?? '0'), max: parseFloat(fields.max ?? '100'), unit: fields.unit || undefined };
    case 'button_card':
      if (!fields.entity_id) return null;
      return { card_type: 'button_card', entity_id: fields.entity_id, action: fields.action || 'toggle' };
    case 'stat_card':
      if (!fields.title || !fields.entity_ids) return null;
      return { card_type: 'stat_card', title: fields.title, entity_ids: fields.entity_ids.split(',').map(s => s.trim()).filter(Boolean), aggregation: fields.aggregation || 'count' };
    case 'history_card':
      if (!fields.entity_id) return null;
      return { card_type: 'history_card', entity_id: fields.entity_id, hours: parseInt(fields.hours ?? '24', 10) };
    default:
      return null;
  }
}

export default function DashboardBuilderPage() {
  const { dashboards, isLoading, create, remove, addView, removeView, addCard, removeCard } = useDashboards();

  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selectedViewId, setSelectedViewId] = useState<string | null>(null);

  // New dashboard modal
  const [dashModal, setDashModal] = useState(false);
  const [dashName, setDashName] = useState('');
  const [dashIcon, setDashIcon] = useState('');
  const [dashError, setDashError] = useState('');
  const [dashNameError, setDashNameError] = useState('');
  const [dashLoading, setDashLoading] = useState(false);

  // New view modal
  const [viewModal, setViewModal] = useState(false);
  const [viewTitle, setViewTitle] = useState('');
  const [viewIcon, setViewIcon] = useState('');
  const [viewTitleError, setViewTitleError] = useState('');
  const [viewLoading, setViewLoading] = useState(false);

  // New card modal
  const [cardModal, setCardModal] = useState(false);
  const [cardType, setCardType] = useState<CardTypeKey>('entity_card');
  const [cardFields, setCardFields] = useState<Record<string, string>>({});
  const [cardLoading, setCardLoading] = useState(false);
  const [cardError, setCardError] = useState('');
  const [cardFieldErrors, setCardFieldErrors] = useState<Record<string, string | undefined>>({});

  const activeDash: Dashboard | null =
    (selectedId ? dashboards.find(d => d.id === selectedId) : dashboards[0]) ?? null;

  const activeView: View | null =
    (selectedViewId ? activeDash?.views.find(v => v.id === selectedViewId) : activeDash?.views[0]) ?? null;

  const handleCreateDash = async (e: React.FormEvent) => {
    e.preventDefault();
    setDashNameError('');
    if (!dashName.trim()) { setDashNameError('Dashboard name is required.'); return; }
    setDashError('');
    setDashLoading(true);
    try {
      await create({ name: dashName.trim(), icon: dashIcon.trim() || undefined });
      setDashName(''); setDashIcon('');
      setDashModal(false);
    } catch (err) {
      setDashError(err instanceof Error ? err.message : 'Failed');
    } finally { setDashLoading(false); }
  };

  const handleAddView = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!activeDash) return;
    setViewTitleError('');
    if (!viewTitle.trim()) { setViewTitleError('Title is required.'); return; }
    setViewLoading(true);
    try {
      await addView(activeDash.id, { title: viewTitle.trim(), icon: viewIcon.trim() || undefined });
      setViewTitle(''); setViewIcon('');
      setViewModal(false);
    } finally { setViewLoading(false); }
  };

  const handleAddCard = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!activeDash || !activeView) return;
    setCardError(''); setCardFieldErrors({});
    const errs: Record<string, string> = {};
    const needsEntity = cardType === 'entity_card' || cardType === 'gauge_card' || cardType === 'button_card' || cardType === 'history_card';
    if (needsEntity && !cardFields.entity_id?.trim()) errs.entity_id = 'Entity ID is required.';
    if (cardType === 'stat_card' && !cardFields.title?.trim()) errs.title = 'Title is required.';
    if (cardType === 'stat_card' && !cardFields.entity_ids?.trim()) errs.entity_ids = 'At least one entity ID is required.';
    if (Object.keys(errs).length > 0) { setCardFieldErrors(errs); return; }
    const req = buildCardRequest(cardType, cardFields);
    if (!req) { setCardError('Please fill in all required fields.'); return; }
    setCardLoading(true);
    try {
      await addCard(activeDash.id, activeView.id, req);
      setCardFields({});
      setCardModal(false);
    } catch (err) {
      setCardError(err instanceof Error ? err.message : 'Failed');
    } finally { setCardLoading(false); }
  };

  if (isLoading) {
    return <div className="surface-card p-6"><p className="text-sm text-[color:var(--ink-muted)]">Loading…</p></div>;
  }

  return (
    <div className="space-y-5">
      {/* Header */}
      <section className="surface-card p-5 sm:p-6">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="section-kicker">Dashboards</p>
            <h1 className="section-title">Builder</h1>
            <p className="section-subtitle">Create dashboards, views, and cards.</p>
          </div>
          <div className="flex gap-2">
            <Link href="/dashboards"><Button variant="secondary">← Viewer</Button></Link>
            <Button onClick={() => setDashModal(true)}>+ Dashboard</Button>
          </div>
        </div>
      </section>

      {/* Dashboard list */}
      {dashboards.length > 0 && (
        <section className="surface-card p-4">
          <p className="mb-3 text-xs font-semibold uppercase tracking-[0.08em] text-[color:var(--ink-muted)]">Dashboards</p>
          <div className="flex flex-wrap gap-2">
            {dashboards.map(d => (
              <div key={d.id} className="flex items-center gap-1.5">
                <button
                  onClick={() => { setSelectedId(d.id); setSelectedViewId(null); }}
                  className={[
                    'rounded-lg px-3 py-1.5 text-sm font-medium transition',
                    activeDash?.id === d.id
                      ? 'bg-[color:var(--accent)] text-white'
                      : 'text-[color:var(--ink-muted)] hover:text-[color:var(--ink-strong)] hover:bg-[rgba(255,255,255,0.06)]',
                  ].join(' ')}
                >
                  {d.icon && <span className="mr-1">{d.icon}</span>}{d.name}
                </button>
                <button
                  onClick={() => remove(d.id)}
                  className="rounded-md px-1.5 py-0.5 text-xs text-[color:var(--ink-faint)] hover:text-red-400 hover:bg-[rgba(244,63,94,0.1)] transition"
                  title="Delete dashboard"
                >✕</button>
              </div>
            ))}
          </div>
        </section>
      )}

      {activeDash && (
        <>
          {/* Views */}
          <section className="surface-card p-4">
            <div className="mb-3 flex items-center justify-between">
              <p className="text-xs font-semibold uppercase tracking-[0.08em] text-[color:var(--ink-muted)]">Views — {activeDash.name}</p>
              <Button size="sm" onClick={() => setViewModal(true)}>+ View</Button>
            </div>
            {activeDash.views.length === 0 ? (
              <p className="text-sm text-[color:var(--ink-faint)]">No views yet.</p>
            ) : (
              <div className="flex flex-wrap gap-2">
                {activeDash.views.map(v => (
                  <div key={v.id} className="flex items-center gap-1.5">
                    <button
                      onClick={() => setSelectedViewId(v.id)}
                      className={[
                        'rounded-lg px-3 py-1.5 text-sm font-medium transition',
                        activeView?.id === v.id
                          ? 'bg-[rgba(99,102,241,0.2)] text-[color:var(--accent)]'
                          : 'text-[color:var(--ink-muted)] hover:text-[color:var(--ink-strong)] hover:bg-[rgba(255,255,255,0.06)]',
                      ].join(' ')}
                    >
                      {v.icon && <span className="mr-1">{v.icon}</span>}{v.title}
                    </button>
                    <button
                      onClick={() => removeView(activeDash.id, v.id)}
                      className="rounded-md px-1.5 py-0.5 text-xs text-[color:var(--ink-faint)] hover:text-red-400 hover:bg-[rgba(244,63,94,0.1)] transition"
                      title="Delete view"
                    >✕</button>
                  </div>
                ))}
              </div>
            )}
          </section>

          {/* Cards in active view */}
          {activeView && (
            <section className="surface-card p-4">
              <div className="mb-3 flex items-center justify-between">
                <p className="text-xs font-semibold uppercase tracking-[0.08em] text-[color:var(--ink-muted)]">Cards — {activeView.title}</p>
                <Button size="sm" onClick={() => setCardModal(true)}>+ Card</Button>
              </div>
              {activeView.cards.length === 0 ? (
                <p className="text-sm text-[color:var(--ink-faint)]">No cards yet.</p>
              ) : (
                <div className="space-y-2">
                  {activeView.cards.map(c => (
                    <div key={c.id} className="flex items-center justify-between rounded-xl border border-[rgba(148,155,200,0.1)] bg-[rgba(148,155,200,0.04)] px-3 py-2">
                      <div>
                        <span className="text-xs font-medium text-[color:var(--ink-strong)]">{c.card_type}</span>
                        {c.title && <span className="ml-2 text-xs text-[color:var(--ink-muted)]">{c.title}</span>}
                        {'entity_id' in c && <span className="ml-2 text-xs text-[color:var(--ink-faint)]">{c.entity_id}</span>}
                      </div>
                      <button
                        onClick={() => removeCard(activeDash.id, activeView.id, c.id)}
                        className="rounded-md px-1.5 py-0.5 text-xs text-[color:var(--ink-faint)] hover:text-red-400 hover:bg-[rgba(244,63,94,0.1)] transition"
                      >✕</button>
                    </div>
                  ))}
                </div>
              )}
            </section>
          )}
        </>
      )}

      {/* New Dashboard Modal */}
      <Modal title="New Dashboard" open={dashModal} onClose={() => setDashModal(false)}>
        <form onSubmit={handleCreateDash} noValidate className="flex flex-col gap-4">
          <Field label="Name" hint="The display name for this dashboard. Shown in the tab and page heading." error={dashNameError}>
            <input value={dashName} onChange={e => { setDashName(e.target.value); if (dashNameError) setDashNameError(''); }} placeholder="e.g. Home" />
          </Field>
          <Field label="Icon (emoji, optional)" hint="A single emoji shown next to the dashboard name. Leave blank for no icon.">
            <input value={dashIcon} onChange={e => setDashIcon(e.target.value)} placeholder="e.g. 🏠" />
          </Field>
          {dashError && <p className="rounded-xl border border-[rgba(244,63,94,0.3)] bg-[rgba(244,63,94,0.1)] px-3 py-2 text-sm text-[#fb7185]">{dashError}</p>}
          <div className="flex justify-end gap-2">
            <Button type="button" variant="ghost" onClick={() => setDashModal(false)}>Cancel</Button>
            <Button type="submit" disabled={dashLoading}>{dashLoading ? 'Creating…' : 'Create'}</Button>
          </div>
        </form>
      </Modal>

      {/* New View Modal */}
      <Modal title="Add View" open={viewModal} onClose={() => setViewModal(false)}>
        <form onSubmit={handleAddView} noValidate className="flex flex-col gap-4">
          <Field label="Title" hint="The name of this view — shown as a tab within the dashboard, e.g. 'Living Room' or 'Upstairs'." error={viewTitleError}>
            <input value={viewTitle} onChange={e => { setViewTitle(e.target.value); if (viewTitleError) setViewTitleError(''); }} placeholder="e.g. Living Room" />
          </Field>
          <Field label="Icon (optional)" hint="An emoji shown next to the view title in the tab bar.">
            <input value={viewIcon} onChange={e => setViewIcon(e.target.value)} placeholder="e.g. 🛋" />
          </Field>
          <div className="flex justify-end gap-2">
            <Button type="button" variant="ghost" onClick={() => setViewModal(false)}>Cancel</Button>
            <Button type="submit" disabled={viewLoading}>{viewLoading ? 'Adding…' : 'Add View'}</Button>
          </div>
        </form>
      </Modal>

      {/* New Card Modal */}
      <Modal title="Add Card" open={cardModal} onClose={() => setCardModal(false)}>
        <form onSubmit={handleAddCard} noValidate className="flex flex-col gap-4">
          <Field label="Card Type" hint="The visual style of this card. Each type displays different data — entity state, a gauge, a button, a stat roll-up, or a history chart.">
            <select
              value={cardType}
              onChange={e => { setCardType(e.target.value as CardTypeKey); setCardFields({}); }}
              className="w-full rounded-xl border border-[rgba(148,155,200,0.15)] bg-[rgba(148,155,200,0.06)] px-3 py-2 text-sm text-[color:var(--ink-strong)]"
            >
              {CARD_TYPES.map(t => <option key={t.key} value={t.key}>{t.label}</option>)}
            </select>
          </Field>

          {/* Dynamic fields per card type */}
          {(cardType === 'entity_card' || cardType === 'gauge_card' || cardType === 'button_card' || cardType === 'history_card') && (
            <Field label="Entity ID" hint="The entity to display on this card, e.g. device.lamp.switch. Must match an entity registered in your home." error={cardFieldErrors.entity_id}>
              <input value={cardFields.entity_id ?? ''} onChange={e => { setCardFields(f => ({ ...f, entity_id: e.target.value })); setCardFieldErrors(fe => ({ ...fe, entity_id: undefined })); }} placeholder="e.g. device.lamp.switch" />
            </Field>
          )}
          {cardType === 'gauge_card' && (
            <>
              <div className="grid grid-cols-2 gap-2">
                <Field label="Min">
                  <input type="number" value={cardFields.min ?? '0'} onChange={e => setCardFields(f => ({ ...f, min: e.target.value }))} />
                </Field>
                <Field label="Max">
                  <input type="number" value={cardFields.max ?? '100'} onChange={e => setCardFields(f => ({ ...f, max: e.target.value }))} />
                </Field>
              </div>
              <Field label="Unit (optional)" hint="The unit suffix shown on the gauge, e.g. °C or %.">
                <input value={cardFields.unit ?? ''} onChange={e => setCardFields(f => ({ ...f, unit: e.target.value }))} placeholder="e.g. °C" />
              </Field>
            </>
          )}
          {cardType === 'button_card' && (
            <Field label="Action" hint="What happens when the button is tapped. Use 'toggle' to flip the entity state, or 'script:name' to run a script.">
              <input value={cardFields.action ?? 'toggle'} onChange={e => setCardFields(f => ({ ...f, action: e.target.value }))} placeholder="toggle or script:name" />
            </Field>
          )}
          {cardType === 'stat_card' && (
            <>
              <Field label="Title" hint="The heading displayed on this stat card." error={cardFieldErrors.title}>
                <input value={cardFields.title ?? ''} onChange={e => { setCardFields(f => ({ ...f, title: e.target.value })); setCardFieldErrors(fe => ({ ...fe, title: undefined })); }} placeholder="e.g. Lights On" />
              </Field>
              <Field label="Entity IDs (comma-separated)" hint="The entities to aggregate. Use the full entity ID, separated by commas." error={cardFieldErrors.entity_ids}>
                <input value={cardFields.entity_ids ?? ''} onChange={e => { setCardFields(f => ({ ...f, entity_ids: e.target.value })); setCardFieldErrors(fe => ({ ...fe, entity_ids: undefined })); }} placeholder="device.lamp.switch, device.fan.switch" />
              </Field>
              <Field label="Aggregation" hint="How to roll up values across the selected entities.">
                <select value={cardFields.aggregation ?? 'count'} onChange={e => setCardFields(f => ({ ...f, aggregation: e.target.value }))} className="w-full rounded-xl border border-[rgba(148,155,200,0.15)] bg-[rgba(148,155,200,0.06)] px-3 py-2 text-sm text-[color:var(--ink-strong)]">
                  <option value="count">Count</option>
                  <option value="sum">Sum</option>
                  <option value="avg">Average</option>
                </select>
              </Field>
            </>
          )}
          {cardType === 'history_card' && (
            <Field label="Hours" hint="How many hours of history to display on the chart. Defaults to 24.">
              <input type="number" value={cardFields.hours ?? '24'} onChange={e => setCardFields(f => ({ ...f, hours: e.target.value }))} min={1} />
            </Field>
          )}

          {cardError && <p className="rounded-xl border border-[rgba(244,63,94,0.3)] bg-[rgba(244,63,94,0.1)] px-3 py-2 text-sm text-[#fb7185]">{cardError}</p>}
          <div className="flex justify-end gap-2">
            <Button type="button" variant="ghost" onClick={() => setCardModal(false)}>Cancel</Button>
            <Button type="submit" disabled={cardLoading}>{cardLoading ? 'Adding…' : 'Add Card'}</Button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
