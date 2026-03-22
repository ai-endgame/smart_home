'use client';
import useSWR from 'swr';
import { listScenes, createScene, deleteScene, applyScene, snapshotScene } from '@/lib/api/scenes';
import type { CreateSceneRequest, SnapshotSceneRequest } from '@/lib/api/types';

export function useScenes() {
  const { data, error, isLoading, mutate } = useSWR('/api/scenes', listScenes, { refreshInterval: 0 });

  const add = async (req: CreateSceneRequest) => {
    await createScene(req);
    mutate();
  };

  const snapshot = async (req: SnapshotSceneRequest) => {
    await snapshotScene(req);
    mutate();
  };

  const remove = async (id: string) => {
    await deleteScene(id);
    mutate();
  };

  const apply = async (id: string) => {
    return applyScene(id);
  };

  return { scenes: data ?? [], error, isLoading, add, snapshot, remove, apply };
}
