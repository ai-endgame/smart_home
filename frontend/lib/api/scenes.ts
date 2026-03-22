import { apiFetch } from './client';
import type { Scene, CreateSceneRequest, SnapshotSceneRequest, ApplySceneResponse } from './types';

export const listScenes = () => apiFetch<Scene[]>('/api/scenes');
export const getScene = (id: string) => apiFetch<Scene>(`/api/scenes/${id}`);
export const createScene = (body: CreateSceneRequest) =>
  apiFetch<Scene>('/api/scenes', { method: 'POST', body: JSON.stringify(body) });
export const updateScene = (id: string, states: Scene['states']) =>
  apiFetch<Scene>(`/api/scenes/${id}`, { method: 'PUT', body: JSON.stringify({ states }) });
export const deleteScene = (id: string) =>
  apiFetch<void>(`/api/scenes/${id}`, { method: 'DELETE' });
export const applyScene = (id: string) =>
  apiFetch<ApplySceneResponse>(`/api/scenes/${id}/apply`, { method: 'POST' });
export const snapshotScene = (body: SnapshotSceneRequest) =>
  apiFetch<Scene>('/api/scenes/snapshot', { method: 'POST', body: JSON.stringify(body) });
