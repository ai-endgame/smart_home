import { apiFetch } from './client';
import type { AutomationRule, CreateRuleRequest } from './types';

export const listRules = () => apiFetch<AutomationRule[]>('/api/automation/rules');
export const createRule = (body: CreateRuleRequest) =>
  apiFetch<AutomationRule>('/api/automation/rules', { method: 'POST', body: JSON.stringify(body) });
export const deleteRule = (name: string) =>
  apiFetch<void>(`/api/automation/rules/${name}`, { method: 'DELETE' });
export const toggleRule = (name: string) =>
  apiFetch<AutomationRule>(`/api/automation/rules/${name}/toggle`, { method: 'POST' });
export const runAutomation = () =>
  apiFetch('/api/automation/run', { method: 'POST' });
export const toggleSafeMode = (name: string) =>
  apiFetch<{ name: string; safe_mode: boolean }>(`/api/automation/rules/${name}/safe-mode`, { method: 'POST' });
