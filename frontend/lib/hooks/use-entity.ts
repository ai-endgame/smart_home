'use client';
import useSWR from 'swr';
import { getEntity } from '@/lib/api/entities';

export function useEntity(entityId: string | null) {
  const { data, error, isLoading } = useSWR(
    entityId ? `/api/entities/${entityId}` : null,
    entityId ? () => getEntity(entityId) : null,
    { refreshInterval: 3000 },
  );
  return { entity: data ?? null, error, isLoading };
}
