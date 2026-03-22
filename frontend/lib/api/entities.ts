import { apiFetch } from './client';
import { Entity, EntityKind } from './types';

export async function getEntities(kind?: EntityKind): Promise<Entity[]> {
  const url = kind ? `/api/entities?kind=${encodeURIComponent(kind)}` : '/api/entities';
  const res = await fetch(url);
  if (!res.ok) throw new Error('Failed to fetch entities');
  return res.json();
}

export async function getDeviceEntities(name: string): Promise<Entity[]> {
  const res = await fetch(`/api/devices/${encodeURIComponent(name)}/entities`);
  if (!res.ok) throw new Error(`Failed to fetch entities for device ${name}`);
  return res.json();
}

export const getEntity = (entityId: string) =>
  apiFetch<Entity>(`/api/entities/${encodeURIComponent(entityId)}`);
