'use client';
import useSWR from 'swr';
import { listRules, createRule, deleteRule, toggleRule, runAutomation, toggleSafeMode } from '@/lib/api/automation';
import type { CreateRuleRequest } from '@/lib/api/types';
import { useSseEvents } from '@/lib/hooks/use-sse-events';

export function useAutomation() {
  const { data, error, isLoading, mutate } = useSWR('/api/automation/rules', listRules, { refreshInterval: 30000 });

  useSseEvents((ev) => {
    if (ev.kind === 'automation') {
      mutate();
    }
  });

  const add = async (req: CreateRuleRequest) => { await createRule(req); mutate(); };
  const remove = async (name: string) => { await deleteRule(name); mutate(); };
  const toggle = async (name: string) => { await toggleRule(name); mutate(); };
  const run = async () => { await runAutomation(); };
  const toggleSafe = async (name: string) => { await toggleSafeMode(name); mutate(); };

  return { rules: data ?? [], error, isLoading, add, remove, toggle, run, toggleSafe };
}
