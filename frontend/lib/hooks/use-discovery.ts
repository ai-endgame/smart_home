'use client';
import useSWR from 'swr';
import { listDiscovered, addDiscoveredDevice } from '@/lib/api/discovery';
import type { AddDiscoveredDeviceRequest } from '@/lib/api/types';

export function useDiscovery() {
  const { data, error, isLoading, mutate } = useSWR('/api/discovery/devices', listDiscovered, { refreshInterval: 10000 });

  const addToHome = async (req: AddDiscoveredDeviceRequest) => {
    await addDiscoveredDevice(req);
    mutate();
  };

  return { discovered: data ?? [], error, isLoading, addToHome, refresh: mutate };
}
