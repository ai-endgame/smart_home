import { apiFetch } from './client';
import type { AddDiscoveredDeviceRequest, DiscoveredDevice, Device } from './types';

export const listDiscovered = () => apiFetch<DiscoveredDevice[]>('/api/discovery/devices');
export const addDiscoveredDevice = (body: AddDiscoveredDeviceRequest) =>
  apiFetch<Device>('/api/discovery/devices/add', { method: 'POST', body: JSON.stringify(body) });
