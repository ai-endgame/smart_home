'use client';
import useSWR from 'swr';
import { getAreas } from '@/lib/api/areas';

export function useAreas() {
  const { data, error, isLoading } = useSWR('/api/areas', getAreas, { refreshInterval: 10000 });
  return { areas: data ?? [], error, isLoading };
}
