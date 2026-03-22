'use client';
import { useEffect } from 'react';
import { createEventSource } from '@/lib/api/events';
import type { ServerEvent } from '@/lib/api/types';

export function useSseEvents(onEvent: (ev: ServerEvent) => void) {
  useEffect(() => {
    const es = createEventSource();
    es.onmessage = (e) => {
      try {
        const ev: ServerEvent = JSON.parse(e.data);
        onEvent(ev);
      } catch {
        // ignore malformed events
      }
    };
    return () => es.close();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
}
