import { apiFetch } from './client';
import type { Person, CreatePersonRequest, UpdateSourceRequest } from './types';

export const listPersons = () => apiFetch<Person[]>('/api/presence/persons');
export const createPerson = (body: CreatePersonRequest) =>
  apiFetch<Person>('/api/presence/persons', { method: 'POST', body: JSON.stringify(body) });
export const deletePerson = (id: string) =>
  apiFetch<void>(`/api/presence/persons/${id}`, { method: 'DELETE' });
export const updateSource = (id: string, source: string, body: UpdateSourceRequest) =>
  apiFetch<Person>(`/api/presence/persons/${id}/sources/${source}`, {
    method: 'PATCH',
    body: JSON.stringify(body),
  });
