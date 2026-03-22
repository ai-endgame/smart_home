'use client';
import { useState, useEffect, useRef } from 'react';
import { useRouter } from 'next/navigation';
import type { CommissionJobResponse } from '@/lib/api/types';
import { startCommission, pollCommissionJob } from '@/lib/api/matter';
import { Field } from '@/components/ui/field';

interface Props {
  initialDeviceName?: string;
  initialNodeId?: number;
  onClose: () => void;
}

type Step = 'input' | 'progress' | 'success' | 'failure';

export function CommissionModal({ initialDeviceName = '', initialNodeId, onClose }: Props) {
  const router = useRouter();
  const [step, setStep] = useState<Step>('input');
  const [setupCode, setSetupCode] = useState('');
  const [nodeId, setNodeId] = useState(initialNodeId?.toString() ?? '1');
  const [job, setJob] = useState<CommissionJobResponse | null>(null);
  const [countdown, setCountdown] = useState(60);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const countdownRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const codeValid = setupCode.length === 11 && /^\d{11}$/.test(setupCode);
  const nodeValid = /^\d+$/.test(nodeId) && Number(nodeId) > 0;

  function cleanup() {
    if (pollRef.current) clearInterval(pollRef.current);
    if (countdownRef.current) clearInterval(countdownRef.current);
  }

  useEffect(() => () => cleanup(), []);

  async function handleStart() {
    if (!codeValid || !nodeValid) return;
    try {
      const j = await startCommission({ setup_code: setupCode, node_id: Number(nodeId) });
      setJob(j);
      setStep('progress');
      setCountdown(60);

      // Countdown
      countdownRef.current = setInterval(() => {
        setCountdown(c => Math.max(0, c - 1));
      }, 1000);

      // Poll every 2s
      pollRef.current = setInterval(async () => {
        try {
          const updated = await pollCommissionJob(j.job_id);
          setJob(updated);
          if (updated.status === 'done') {
            cleanup();
            setStep('success');
          } else if (updated.status === 'failed') {
            cleanup();
            setStep('failure');
          }
        } catch {
          // keep polling
        }
      }, 2000);
    } catch (e) {
      setJob({ job_id: '', status: 'failed', message: 'Failed to start commissioning', device_id: null, error: String(e) });
      setStep('failure');
    }
  }

  function handleRetry() {
    cleanup();
    setStep('input');
    setSetupCode('');
    setCountdown(60);
    setJob(null);
  }

  return (
    <div className="fixed inset-0 z-50 flex items-end justify-center p-4 sm:items-center">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-[rgba(5,7,15,0.75)] backdrop-blur-sm" onClick={onClose} />

      <div className="relative w-full max-w-sm rounded-2xl border border-[var(--line-strong)] bg-[var(--bg-modal)] shadow-[var(--shadow-modal)]">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-[var(--line)] px-5 py-4">
          <div>
            <p className="font-semibold text-[color:var(--ink-strong)]">Commission Matter Device</p>
            {initialDeviceName && <p className="text-xs text-[color:var(--ink-muted)]">{initialDeviceName}</p>}
          </div>
          <button
            type="button" onClick={onClose}
            className="flex h-7 w-7 items-center justify-center rounded-lg text-lg text-[color:var(--ink-muted)] transition hover:bg-[rgba(255,255,255,0.08)]"
          >×</button>
        </div>

        <div className="px-5 py-5 space-y-5">

          {/* Step indicator */}
          <div className="flex items-center gap-2">
            {(['input', 'progress', 'success'] as Step[]).map((s, i) => (
              <div key={s} className="flex items-center gap-2">
                <div
                  className="flex h-6 w-6 items-center justify-center rounded-full text-xs font-bold"
                  style={{
                    background: step === s ? '#818cf8' : ['success', 'done'].includes(step) && i < 2 ? 'rgba(52,211,153,0.2)' : 'rgba(255,255,255,0.06)',
                    color: step === s ? '#fff' : '#94a3b8',
                    border: step === s ? 'none' : '1px solid rgba(148,155,200,0.15)',
                  }}
                >{i + 1}</div>
                {i < 2 && <div className="h-px w-6 bg-[rgba(148,155,200,0.15)]" />}
              </div>
            ))}
            <span className="ml-1 text-xs text-[color:var(--ink-muted)]">
              {step === 'input' ? 'Enter code' : step === 'progress' ? 'Commissioning…' : step === 'success' ? 'Done!' : 'Failed'}
            </span>
          </div>

          {/* Step 1: input */}
          {step === 'input' && (
            <div className="space-y-4">
              <Field label="Setup Code (11 digits)" hint="The 11-digit pairing code printed on the device or shown in its app. Digits only — no dashes.">
                <input
                  type="text"
                  inputMode="numeric"
                  maxLength={11}
                  value={setupCode}
                  onChange={e => setSetupCode(e.target.value.replace(/\D/g, ''))}
                  placeholder="34970112332"
                  className="w-full rounded-xl border border-[rgba(148,155,200,0.15)] bg-[rgba(255,255,255,0.04)] px-4 py-2.5 font-mono text-sm text-[color:var(--ink-strong)] placeholder-[color:var(--ink-faint)] outline-none focus:border-[#818cf8]"
                />
                {setupCode.length > 0 && !codeValid && (
                  <p className="text-[0.65rem] text-[#fb7185]">Must be exactly 11 digits</p>
                )}
              </Field>
              <Field label="Node ID" hint="A unique numeric identifier assigned to this device on the Matter fabric. Increment for each new device.">
                <input
                  type="number"
                  min={1}
                  value={nodeId}
                  onChange={e => setNodeId(e.target.value)}
                  className="w-full rounded-xl border border-[rgba(148,155,200,0.15)] bg-[rgba(255,255,255,0.04)] px-4 py-2.5 text-sm text-[color:var(--ink-strong)] outline-none focus:border-[#818cf8]"
                />
              </Field>
              <button
                type="button"
                disabled={!codeValid || !nodeValid}
                onClick={handleStart}
                className="w-full rounded-xl bg-[#818cf8] py-2.5 text-sm font-semibold text-white transition hover:bg-[#6366f1] disabled:opacity-40 disabled:cursor-not-allowed"
              >
                Start Commissioning
              </button>
            </div>
          )}

          {/* Step 2: progress */}
          {step === 'progress' && (
            <div className="space-y-4 text-center">
              <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-full border border-[rgba(129,140,248,0.3)] bg-[rgba(129,140,248,0.1)]">
                <span className="animate-spin text-2xl">⟳</span>
              </div>
              <div>
                <p className="font-semibold text-[color:var(--ink-strong)]">
                  {job?.message ?? 'Commissioning in progress…'}
                </p>
                <p className="mt-1 text-xs text-[color:var(--ink-muted)]">
                  This may take up to 60 seconds
                </p>
              </div>
              <div className="rounded-xl border border-[rgba(148,155,200,0.1)] bg-[rgba(255,255,255,0.03)] px-4 py-3">
                <p className="text-xs text-[color:var(--ink-muted)]">Time remaining</p>
                <p className="text-2xl font-bold" style={{ color: countdown < 10 ? '#fb7185' : '#818cf8' }}>
                  {countdown}s
                </p>
              </div>
            </div>
          )}

          {/* Step 3: success */}
          {step === 'success' && (
            <div className="space-y-4 text-center">
              <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-full border border-[rgba(52,211,153,0.3)] bg-[rgba(52,211,153,0.1)]">
                <span className="text-2xl">✓</span>
              </div>
              <div>
                <p className="font-semibold text-[color:var(--ink-strong)]">Device added!</p>
                {job?.device_id && (
                  <p className="mt-1 text-xs text-[color:var(--ink-muted)]">Device ID: {job.device_id}</p>
                )}
              </div>
              <button
                type="button"
                onClick={() => { onClose(); router.push('/devices'); }}
                className="w-full rounded-xl bg-[rgba(52,211,153,0.15)] py-2.5 text-sm font-semibold text-[#34d399] transition hover:bg-[rgba(52,211,153,0.25)]"
              >
                Go to Devices
              </button>
            </div>
          )}

          {/* Step 3: failure */}
          {step === 'failure' && (
            <div className="space-y-4 text-center">
              <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-full border border-[rgba(248,113,113,0.3)] bg-[rgba(248,113,113,0.1)]">
                <span className="text-2xl">✗</span>
              </div>
              <div>
                <p className="font-semibold text-[color:var(--ink-strong)]">Commissioning failed</p>
                {job?.error && (
                  <p className="mt-1 text-xs text-[#fb7185]">{job.error}</p>
                )}
              </div>
              <button
                type="button"
                onClick={handleRetry}
                className="w-full rounded-xl border border-[rgba(148,155,200,0.15)] py-2.5 text-sm font-semibold text-[color:var(--ink-muted)] transition hover:bg-[rgba(255,255,255,0.06)]"
              >
                Try Again
              </button>
            </div>
          )}

        </div>
      </div>
    </div>
  );
}
