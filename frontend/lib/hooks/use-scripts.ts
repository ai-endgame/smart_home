'use client';
import useSWR from 'swr';
import { listScripts, createScript, deleteScript, runScript } from '@/lib/api/scripts';
import type { CreateScriptRequest, RunScriptRequest } from '@/lib/api/types';

export function useScripts() {
  const { data, error, isLoading, mutate } = useSWR('/api/scripts', listScripts, { refreshInterval: 0 });

  const add = async (req: CreateScriptRequest) => {
    await createScript(req);
    mutate();
  };

  const remove = async (id: string) => {
    await deleteScript(id);
    mutate();
  };

  const run = async (id: string, req?: RunScriptRequest) => {
    await runScript(id, req);
  };

  return { scripts: data ?? [], error, isLoading, add, remove, run };
}
