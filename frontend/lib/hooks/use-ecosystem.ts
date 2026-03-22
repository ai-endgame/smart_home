'use client';
import useSWR from 'swr';
import { getEcosystem } from '@/lib/api/ecosystem';

export function useEcosystem() {
  const { data, error, isLoading } = useSWR('/api/ecosystem', getEcosystem, { refreshInterval: 10000 });
  return { ecosystem: data ?? null, error, isLoading };
}
