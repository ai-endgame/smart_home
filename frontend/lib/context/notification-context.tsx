'use client';
import { createContext, useContext, useState, useCallback, type ReactNode } from 'react';
import { useSseEvents } from '@/lib/hooks/use-sse-events';
import type { ServerEvent } from '@/lib/api/types';

interface NotificationContextValue {
  events: ServerEvent[];
  unreadCount: number;
  isOpen: boolean;
  open: () => void;
  close: () => void;
  markRead: () => void;
}

const NotificationContext = createContext<NotificationContextValue | null>(null);

export function NotificationProvider({ children }: { children: ReactNode }) {
  const [events, setEvents] = useState<ServerEvent[]>([]);
  const [unreadCount, setUnreadCount] = useState(0);
  const [isOpen, setIsOpen] = useState(false);

  useSseEvents((ev) => {
    setEvents(prev => [ev, ...prev].slice(0, 50));
    setUnreadCount(prev => prev + 1);
  });

  const open = useCallback(() => {
    setIsOpen(true);
    setUnreadCount(0);
  }, []);

  const close = useCallback(() => setIsOpen(false), []);
  const markRead = useCallback(() => setUnreadCount(0), []);

  return (
    <NotificationContext.Provider value={{ events, unreadCount, isOpen, open, close, markRead }}>
      {children}
    </NotificationContext.Provider>
  );
}

export function useNotifications() {
  const ctx = useContext(NotificationContext);
  if (!ctx) throw new Error('useNotifications must be used within NotificationProvider');
  return ctx;
}
