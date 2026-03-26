'use client';
import useSWR from 'swr';
import { listDashboards, createDashboard, deleteDashboard, addView, deleteView, addCard, deleteCard } from '@/lib/api/dashboards';
import type { CreateDashboardRequest, CreateViewRequest, CreateCardRequest } from '@/lib/api/types';

export function useDashboards() {
  const { data, error, isLoading, mutate } = useSWR('/api/dashboards', listDashboards, { refreshInterval: 5000 });

  const create = async (req: CreateDashboardRequest) => { await createDashboard(req); mutate(); };
  const remove = async (id: string) => { await deleteDashboard(id); mutate(); };
  const addViewFn = async (id: string, req: CreateViewRequest) => { await addView(id, req); mutate(); };
  const removeView = async (id: string, viewId: string) => { await deleteView(id, viewId); mutate(); };
  const addCardFn = async (id: string, viewId: string, req: CreateCardRequest) => { await addCard(id, viewId, req); mutate(); };
  const removeCard = async (id: string, viewId: string, cardId: string) => { await deleteCard(id, viewId, cardId); mutate(); };

  return { dashboards: data ?? [], error, isLoading, create, remove, addView: addViewFn, removeView, addCard: addCardFn, removeCard };
}
