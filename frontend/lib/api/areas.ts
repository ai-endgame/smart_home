import { AreaDetailResponse, AreaResponse } from './types';

export async function getAreas(): Promise<AreaResponse[]> {
  const res = await fetch('/api/areas');
  if (!res.ok) throw new Error('Failed to fetch areas');
  return res.json();
}

export async function getArea(areaId: string): Promise<AreaDetailResponse> {
  const res = await fetch(`/api/areas/${encodeURIComponent(areaId)}`);
  if (!res.ok) throw new Error(`Failed to fetch area ${areaId}`);
  return res.json();
}
