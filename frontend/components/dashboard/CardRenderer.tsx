'use client';
import type { Card } from '@/lib/api/types';
import { EntityCard } from './EntityCard';
import { GaugeCard } from './GaugeCard';
import { ButtonCard } from './ButtonCard';
import { StatCard } from './StatCard';
import { HistoryCard } from './HistoryCard';

interface Props { card: Card; }

export function CardRenderer({ card }: Props) {
  switch (card.card_type) {
    case 'entity_card':
      return <EntityCard entityId={card.entity_id} title={card.title} />;
    case 'gauge_card':
      return <GaugeCard entityId={card.entity_id} min={card.min} max={card.max} unit={card.unit} title={card.title} />;
    case 'button_card':
      return <ButtonCard entityId={card.entity_id} action={card.action} title={card.title} />;
    case 'stat_card':
      return <StatCard title={card.title ?? card.title} entityIds={card.entity_ids} aggregation={card.aggregation} />;
    case 'history_card':
      return <HistoryCard entityId={card.entity_id} hours={card.hours} title={card.title} />;
    default:
      return <div className="surface-card p-4 text-xs text-[color:var(--ink-faint)]">Unknown card type</div>;
  }
}
