import { apiFetch } from './client';
import type { Script, CreateScriptRequest, RunScriptRequest, RunScriptResponse } from './types';

export const listScripts = () => apiFetch<Script[]>('/api/scripts');
export const getScript = (id: string) => apiFetch<Script>(`/api/scripts/${id}`);
export const createScript = (body: CreateScriptRequest) =>
  apiFetch<Script>('/api/scripts', { method: 'POST', body: JSON.stringify(body) });
export const updateScript = (id: string, body: CreateScriptRequest) =>
  apiFetch<Script>(`/api/scripts/${id}`, { method: 'PUT', body: JSON.stringify(body) });
export const deleteScript = (id: string) =>
  apiFetch<void>(`/api/scripts/${id}`, { method: 'DELETE' });
export const runScript = (id: string, body?: RunScriptRequest) =>
  apiFetch<RunScriptResponse>(`/api/scripts/${id}/run`, { method: 'POST', body: body ? JSON.stringify(body) : undefined });
