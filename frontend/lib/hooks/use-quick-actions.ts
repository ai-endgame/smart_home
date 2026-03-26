'use client';
import { useState, useEffect, useCallback } from 'react';

export type QuickActionType = 'scene' | 'script' | 'automation';

export interface QuickAction {
  type: QuickActionType;
  id: string;
  label: string;
}

const STORAGE_KEY = 'smart_home.quick_actions';

function loadActions(): QuickAction[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed;
  } catch {
    return [];
  }
}

export function useQuickActions() {
  const [actions, setActions] = useState<QuickAction[]>([]);

  useEffect(() => {
    setActions(loadActions());
  }, []);

  const addQuickAction = useCallback((action: QuickAction) => {
    setActions(prev => {
      const updated = [...prev, action];
      localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
      return updated;
    });
  }, []);

  const removeQuickAction = useCallback((id: string) => {
    setActions(prev => {
      const updated = prev.filter(a => a.id !== id);
      localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
      return updated;
    });
  }, []);

  return { actions, addQuickAction, removeQuickAction };
}
