'use client';
import useSWR from 'swr';
import { apiFetch } from '@/lib/api/client';
import type { HistoryEntry } from '@/lib/api/types';

export function useDeviceHistory(name: string, limit = 20) {
  const { data, isLoading } = useSWR<HistoryEntry[]>(
    name ? `/api/devices/${encodeURIComponent(name)}/history?limit=${limit}` : null,
    (url: string) => apiFetch<HistoryEntry[]>(url),
    { refreshInterval: 60000 },
  );
  return { history: data ?? [], isLoading };
}
