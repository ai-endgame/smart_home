'use client';
import { useEffect } from 'react';

const SETTINGS_KEY = 'smart_home.settings';

export function ThemeApplier() {
  useEffect(() => {
    try {
      const raw = localStorage.getItem(SETTINGS_KEY);
      if (!raw) return;
      const settings = JSON.parse(raw) as { theme?: string };
      const theme = settings.theme;
      if (theme && theme !== 'system') {
        document.documentElement.setAttribute('data-theme', theme);
      }
    } catch {
      // ignore corrupt settings
    }
  }, []);

  return null;
}
