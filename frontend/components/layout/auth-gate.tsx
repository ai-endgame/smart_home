'use client';
import { useEffect, useState, type ReactNode } from 'react';
import { useRouter, usePathname } from 'next/navigation';

export function AuthGate({ children }: { children: ReactNode }) {
  const router = useRouter();
  const pathname = usePathname();
  const [ready, setReady] = useState(false);

  useEffect(() => {
    // Skip gate on the login page itself
    if (pathname === '/login') { setReady(true); return; }

    const check = async () => {
      try {
        const res = await fetch('/api/auth/status');
        const { auth_enabled } = await res.json();
        if (!auth_enabled) { setReady(true); return; }

        const token = localStorage.getItem('smart_home.auth_token');
        if (!token) { router.replace('/login'); return; }
        setReady(true);
      } catch {
        // Server unreachable — let the app render (API calls will fail individually)
        setReady(true);
      }
    };
    check();
  }, [pathname, router]);

  if (!ready) return null;
  return <>{children}</>;
}
