'use client';
import useSWR from 'swr';
import { listPersons, createPerson, deletePerson, updateSource } from '@/lib/api/presence';
import type { CreatePersonRequest, SourceState } from '@/lib/api/types';

export function usePresence() {
  const { data, error, isLoading, mutate } = useSWR('/api/presence/persons', listPersons, { refreshInterval: 5000 });

  const add = async (req: CreatePersonRequest) => {
    await createPerson(req);
    mutate();
  };

  const remove = async (id: string) => {
    await deletePerson(id);
    mutate();
  };

  const setSource = async (id: string, source: string, state: SourceState) => {
    await updateSource(id, source, { state });
    mutate();
  };

  return { persons: data ?? [], error, isLoading, add, remove, setSource };
}
