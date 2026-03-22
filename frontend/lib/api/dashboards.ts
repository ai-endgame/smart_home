import { apiFetch } from './client';
import type { Dashboard, CreateDashboardRequest, CreateViewRequest, CreateCardRequest } from './types';

export const listDashboards = () => apiFetch<Dashboard[]>('/api/dashboards');
export const createDashboard = (body: CreateDashboardRequest) =>
  apiFetch<Dashboard>('/api/dashboards', { method: 'POST', body: JSON.stringify(body) });
export const deleteDashboard = (id: string) =>
  apiFetch<void>(`/api/dashboards/${id}`, { method: 'DELETE' });
export const addView = (id: string, body: CreateViewRequest) =>
  apiFetch<Dashboard>(`/api/dashboards/${id}/views`, { method: 'POST', body: JSON.stringify(body) });
export const deleteView = (id: string, viewId: string) =>
  apiFetch<Dashboard>(`/api/dashboards/${id}/views/${viewId}`, { method: 'DELETE' });
export const addCard = (id: string, viewId: string, body: CreateCardRequest) =>
  apiFetch<Dashboard>(`/api/dashboards/${id}/views/${viewId}/cards`, { method: 'POST', body: JSON.stringify(body) });
export const deleteCard = (id: string, viewId: string, cardId: string) =>
  apiFetch<Dashboard>(`/api/dashboards/${id}/views/${viewId}/cards/${cardId}`, { method: 'DELETE' });
