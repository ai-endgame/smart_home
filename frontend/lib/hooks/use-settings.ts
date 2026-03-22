'use client';
import { useState, useEffect, useCallback } from 'react';

const SETTINGS_KEY = 'smart_home.settings';

export interface AppSettings {
  theme: 'system' | 'dark' | 'light';
  soundEnabled: boolean;
  landingPage: string;
}

const DEFAULTS: AppSettings = {
  theme: 'system',
  soundEnabled: false,
  landingPage: '/',
};

function loadSettings(): AppSettings {
  if (typeof window === 'undefined') return DEFAULTS;
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (!raw) return DEFAULTS;
    return { ...DEFAULTS, ...JSON.parse(raw) };
  } catch {
    return DEFAULTS;
  }
}

function applyTheme(theme: AppSettings['theme']) {
  if (typeof document === 'undefined') return;
  if (theme === 'system') {
    document.documentElement.removeAttribute('data-theme');
  } else {
    document.documentElement.setAttribute('data-theme', theme);
  }
}

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULTS);

  useEffect(() => {
    const loaded = loadSettings();
    setSettings(loaded);
    applyTheme(loaded.theme);
  }, []);

  const update = useCallback((patch: Partial<AppSettings>) => {
    setSettings(prev => {
      const next = { ...prev, ...patch };
      localStorage.setItem(SETTINGS_KEY, JSON.stringify(next));
      if (patch.theme !== undefined) applyTheme(next.theme);
      return next;
    });
  }, []);

  return { settings, update };
}
