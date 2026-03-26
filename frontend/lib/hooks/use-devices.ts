'use client';
import useSWR from 'swr';
import { listDevices, createDevice, deleteDevice, setDeviceState, setDeviceBrightness, setDeviceTemperature, connectDevice, disconnectDevice } from '@/lib/api/devices';
import type { CreateDeviceRequest } from '@/lib/api/types';
import { useSseEvents } from '@/lib/hooks/use-sse-events';

export function useDevices() {
  const { data, error, isLoading, mutate } = useSWR('/api/devices', listDevices, { refreshInterval: 30000 });

  useSseEvents((ev) => {
    if (ev.kind === 'device_updated' || ev.kind === 'device_connected' || ev.kind === 'device_disconnected' || ev.kind === 'device_error') {
      mutate();
    }
  });

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

  const setBrightness = async (name: string, brightness: number) => { await setDeviceBrightness(name, brightness); mutate(); };
  const setTemperature = async (name: string, temperature: number) => { await setDeviceTemperature(name, temperature); mutate(); };
  const connect = async (name: string) => { await connectDevice(name); mutate(); };
  const disconnect = async (name: string) => { await disconnectDevice(name); mutate(); };

  return { devices: data ?? [], error, isLoading, add, remove, setState, setBrightness, setTemperature, connect, disconnect };
}
