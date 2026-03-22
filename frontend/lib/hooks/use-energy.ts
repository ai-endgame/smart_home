'use client';
import useSWR from 'swr';
import { apiFetch } from '@/lib/api/client';
import type { EnergySummaryResponse } from '@/lib/api/types';

export function useEnergy() {
  const { data, isLoading } = useSWR<EnergySummaryResponse>(
    '/api/energy/summary',
    (url: string) => apiFetch<EnergySummaryResponse>(url),
    { refreshInterval: 30000 },
  );
  return { summary: data ?? null, isLoading };
}
