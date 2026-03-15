'use client';
import useSWR from 'swr';
import { listRules, createRule, deleteRule, toggleRule, runAutomation } from '@/lib/api/automation';
import type { CreateRuleRequest } from '@/lib/api/types';

export function useAutomation() {
  const { data, error, isLoading, mutate } = useSWR('/api/automation/rules', listRules, { refreshInterval: 10000 });

  const add = async (req: CreateRuleRequest) => { await createRule(req); mutate(); };
  const remove = async (name: string) => { await deleteRule(name); mutate(); };
  const toggle = async (name: string) => { await toggleRule(name); mutate(); };
  const run = async () => { await runAutomation(); };

  return { rules: data ?? [], error, isLoading, add, remove, toggle, run };
}
