import { apiFetch } from './client';
import type { CommissionJobResponse, FabricResponse, MatterDeviceResponse, MatterStatus } from './types';

export async function getMatterStatus(): Promise<MatterStatus> {
  return apiFetch<MatterStatus>('/api/matter/status');
}

export async function getMatterDevices(): Promise<MatterDeviceResponse[]> {
  return apiFetch<MatterDeviceResponse[]>('/api/matter/devices');
}

export async function getMatterFabrics(): Promise<FabricResponse[]> {
  return apiFetch<FabricResponse[]>('/api/matter/fabrics');
}

export async function startCommission(body: { setup_code: string; node_id: number }): Promise<CommissionJobResponse> {
  return apiFetch<CommissionJobResponse>('/api/matter/commission', {
    method: 'POST',
    body: JSON.stringify(body),
  });
}

export async function pollCommissionJob(jobId: string): Promise<CommissionJobResponse> {
  return apiFetch<CommissionJobResponse>(`/api/matter/commission/${jobId}`);
}
