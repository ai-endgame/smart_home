'use client';
import { useCallback } from 'react';
import { useRouter } from 'next/navigation';

const TOKEN_KEY = 'smart_home.auth_token';

export function useAuth() {
  const router = useRouter();

  const isAuthenticated = typeof window !== 'undefined'
    ? Boolean(localStorage.getItem(TOKEN_KEY))
    : false;

  const logout = useCallback(() => {
    if (typeof window !== 'undefined') {
      localStorage.removeItem(TOKEN_KEY);
    }
    router.push('/login');
  }, [router]);

  return { isAuthenticated, logout };
}
