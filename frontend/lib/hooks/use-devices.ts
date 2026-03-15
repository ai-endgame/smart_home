'use client';
import useSWR from 'swr';
import { listDevices, createDevice, deleteDevice, setDeviceState, connectDevice, disconnectDevice } from '@/lib/api/devices';
import type { CreateDeviceRequest } from '@/lib/api/types';

export function useDevices() {
  const { data, error, isLoading, mutate } = useSWR('/api/devices', listDevices, { refreshInterval: 5000 });

  const add = async (req: CreateDeviceRequest) => {
    await createDevice(req);
    mutate();
  };

  const remove = async (name: string) => {
    await deleteDevice(name);
    mutate();
  };

  const setState = async (name: string, state: string) => {
    await setDeviceState(name, state);
    mutate();
  };

  const connect = async (name: string) => { await connectDevice(name); mutate(); };
  const disconnect = async (name: string) => { await disconnectDevice(name); mutate(); };

  return { devices: data ?? [], error, isLoading, add, remove, setState, connect, disconnect };
}
