import { apiFetch } from './client';
import type { Device, CreateDeviceRequest, UpdateDeviceRequest } from './types';

export const listDevices = () => apiFetch<Device[]>('/api/devices');
export const getDevice = (name: string) => apiFetch<Device>(`/api/devices/${name}`);
export const createDevice = (body: CreateDeviceRequest) =>
  apiFetch<Device>('/api/devices', { method: 'POST', body: JSON.stringify(body) });
export const updateDevice = (name: string, body: UpdateDeviceRequest) =>
  apiFetch<Device>(`/api/devices/${name}`, { method: 'PATCH', body: JSON.stringify(body) });
export const deleteDevice = (name: string) =>
  apiFetch<void>(`/api/devices/${name}`, { method: 'DELETE' });
export const setDeviceState = (name: string, state: string) =>
  apiFetch<Device>(`/api/devices/${name}/state`, { method: 'PATCH', body: JSON.stringify({ state }) });
export const connectDevice = (name: string) =>
  apiFetch<Device>(`/api/devices/${name}/connect`, { method: 'POST' });
export const disconnectDevice = (name: string) =>
  apiFetch<Device>(`/api/devices/${name}/disconnect`, { method: 'POST' });
