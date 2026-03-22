'use client';
import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';

export default function LoginPage() {
  const [pin, setPin] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const router = useRouter();

  // Redirect if already authenticated
  useEffect(() => {
    const token = localStorage.getItem('smart_home.auth_token');
    if (token) router.replace('/');
  }, [router]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!pin) { setError('Please enter your PIN'); return; }
    setLoading(true);
    setError('');
    try {
      const res = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ pin }),
      });
      if (res.status === 401) { setError('Incorrect PIN'); return; }
      if (!res.ok) { setError('Login failed'); return; }
      const { token } = await res.json();
      localStorage.setItem('smart_home.auth_token', token);
      router.replace('/');
    } catch {
      setError('Could not connect to server');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center px-4">
      <div className="w-full max-w-sm surface-card p-8 space-y-6">
        <div className="text-center">
          <span className="inline-flex h-12 w-12 items-center justify-center rounded-2xl bg-[linear-gradient(140deg,#6366f1,#06b6d4)] text-xl font-bold text-white shadow-[0_0_24px_rgba(99,102,241,0.45)]">
            SH
          </span>
          <h1 className="mt-4 text-xl font-semibold text-[color:var(--ink-strong)]">Smart Home</h1>
          <p className="mt-1 text-sm text-[color:var(--ink-muted)]">Enter your PIN to continue</p>
        </div>

        <form onSubmit={handleSubmit} noValidate className="space-y-4">
          <div>
            <input
              type="password"
              inputMode="numeric"
              value={pin}
              onChange={e => { setPin(e.target.value); setError(''); }}
              placeholder="PIN"
              autoFocus
              className="w-full rounded-xl border border-[var(--line-strong)] bg-[var(--input-bg)] px-4 py-3 text-center text-2xl tracking-[0.5em] text-[color:var(--ink-strong)] placeholder:text-[color:var(--ink-faint)] placeholder:tracking-normal focus:border-[color:var(--accent)] focus:outline-none focus:ring-1 focus:ring-[color:var(--accent)]"
            />
            {error && (
              <p className="mt-2 text-center text-sm text-[color:var(--danger)]">{error}</p>
            )}
          </div>

          <button
            type="submit"
            disabled={loading}
            className="w-full rounded-xl py-3 text-sm font-semibold text-white transition disabled:opacity-60"
            style={{ background: 'var(--btn-primary-bg)', boxShadow: 'var(--btn-primary-shadow)' }}
          >
            {loading ? 'Signing in…' : 'Sign in'}
          </button>
        </form>
      </div>
    </div>
  );
}
