import { apiFetch } from './client';
import type { MqttStatus } from './types';

export async function getMqttStatus(): Promise<MqttStatus> {
  return apiFetch<MqttStatus>('/api/mqtt/status');
}
