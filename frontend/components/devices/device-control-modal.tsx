'use client';
import { useState, useEffect } from 'react';
import type { Device, DeviceType } from '@/lib/api/types';
import { snapshotScene } from '@/lib/api/scenes';
import { useDeviceHistory } from '@/lib/hooks/use-device-history';
import { AreaChart, Area, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts';

interface DeviceControlModalProps {
  device: Device | null;
  open: boolean;
  onClose: () => void;
  onSetState: (name: string, state: 'on' | 'off') => Promise<void>;
  onSetBrightness: (name: string, value: number) => Promise<void>;
  onSetTemperature: (name: string, value: number) => Promise<void>;
  onConnect: (name: string) => Promise<void>;
  onDisconnect: (name: string) => Promise<void>;
}

// ── Type metadata ─────────────────────────────────────────────────────────────

const TYPE_META: Record<string, { icon: string; accent: string; bg: string; label: string }> = {
  light:        { icon: '💡', accent: '#fbbf24', bg: 'rgba(251,191,36,0.12)',  label: 'Light' },
  thermostat:   { icon: '🌡️', accent: '#fb923c', bg: 'rgba(251,146,60,0.12)',  label: 'Thermostat' },
  fan:          { icon: '🌀', accent: '#67e8f9', bg: 'rgba(103,232,249,0.12)', label: 'Fan' },
  lock:         { icon: '🔒', accent: '#818cf8', bg: 'rgba(129,140,248,0.12)', label: 'Lock' },
  switch:       { icon: '⚡', accent: '#22d3ee', bg: 'rgba(34,211,238,0.12)',  label: 'Switch' },
  outlet:       { icon: '🔌', accent: '#a78bfa', bg: 'rgba(167,139,250,0.12)', label: 'Outlet' },
  tv:           { icon: '📺', accent: '#60a5fa', bg: 'rgba(96,165,250,0.12)',  label: 'TV' },
  speaker:      { icon: '🔊', accent: '#c084fc', bg: 'rgba(192,132,252,0.12)', label: 'Speaker' },
  media_player: { icon: '▶️', accent: '#f472b6', bg: 'rgba(244,114,182,0.12)', label: 'Media Player' },
  sensor:       { icon: '📡', accent: '#34d399', bg: 'rgba(52,211,153,0.12)',  label: 'Sensor' },
  camera:       { icon: '📷', accent: '#94a3b8', bg: 'rgba(148,163,184,0.12)', label: 'Camera' },
  alarm:        { icon: '🚨', accent: '#f87171', bg: 'rgba(248,113,113,0.12)', label: 'Alarm' },
  cover:        { icon: '🪟', accent: '#86efac', bg: 'rgba(134,239,172,0.12)', label: 'Cover' },
  hub:          { icon: '🔗', accent: '#fdba74', bg: 'rgba(253,186,116,0.12)', label: 'Hub' },
};
const DEFAULT_META = { icon: '📦', accent: '#818cf8', bg: 'rgba(129,140,248,0.12)', label: 'Device' };

// ── Control config per type ───────────────────────────────────────────────────

type ControlConfig = {
  hasToggle: boolean;
  toggleLabels: [string, string];  // [on label, off label]
  levelControl?: { label: string; unit: string; min: number; max: number; step: number; field: 'brightness' | 'temperature' };
  tempControl?: boolean;
  readonly?: boolean;
};

function getControlConfig(type: DeviceType | string): ControlConfig {
  switch (type) {
    case 'light':
      return { hasToggle: true, toggleLabels: ['On', 'Off'], levelControl: { label: 'Brightness', unit: '%', min: 0, max: 100, step: 1, field: 'brightness' } };
    case 'thermostat':
      return { hasToggle: true, toggleLabels: ['On', 'Off'], levelControl: { label: 'Temperature', unit: '°C', min: 10, max: 35, step: 0.5, field: 'temperature' } };
    case 'fan':
      return { hasToggle: true, toggleLabels: ['On', 'Off'], levelControl: { label: 'Speed', unit: '%', min: 0, max: 100, step: 10, field: 'brightness' } };
    case 'cover':
      return { hasToggle: true, toggleLabels: ['Open', 'Close'], levelControl: { label: 'Position', unit: '% open', min: 0, max: 100, step: 5, field: 'brightness' } };
    case 'tv':
      return { hasToggle: true, toggleLabels: ['Power On', 'Power Off'], levelControl: { label: 'Volume', unit: '%', min: 0, max: 100, step: 5, field: 'brightness' } };
    case 'speaker':
      return { hasToggle: true, toggleLabels: ['On', 'Off'], levelControl: { label: 'Volume', unit: '%', min: 0, max: 100, step: 5, field: 'brightness' } };
    case 'media_player':
      return { hasToggle: true, toggleLabels: ['Play', 'Stop'], levelControl: { label: 'Volume', unit: '%', min: 0, max: 100, step: 5, field: 'brightness' } };
    case 'lock':
      return { hasToggle: true, toggleLabels: ['Lock', 'Unlock'] };
    case 'alarm':
      return { hasToggle: true, toggleLabels: ['Arm', 'Disarm'] };
    case 'camera':
      return { hasToggle: true, toggleLabels: ['Enable', 'Disable'] };
    case 'sensor':
      return { hasToggle: false, toggleLabels: ['On', 'Off'], readonly: true };
    default:
      return { hasToggle: true, toggleLabels: ['On', 'Off'] };
  }
}

// ── Slider component ──────────────────────────────────────────────────────────

function LevelSlider({
  value, min, max, step, label, unit, accent, onChange,
}: {
  value: number; min: number; max: number; step: number;
  label: string; unit: string; accent: string;
  onChange: (v: number) => void;
}) {
  const pct = ((value - min) / (max - min)) * 100;
  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium text-[color:var(--ink-muted)]">{label}</span>
        <span className="text-lg font-bold" style={{ color: accent }}>
          {typeof value === 'number' && !Number.isInteger(value) ? value.toFixed(1) : value}{unit}
        </span>
      </div>
      <div className="relative h-2 w-full rounded-full bg-[rgba(255,255,255,0.08)]">
        <div
          className="absolute left-0 top-0 h-2 rounded-full transition-all"
          style={{ width: `${pct}%`, background: accent }}
        />
        <input
          type="range"
          min={min} max={max} step={step}
          value={value}
          onChange={e => onChange(parseFloat(e.target.value))}
          className="absolute inset-0 h-full w-full cursor-pointer opacity-0"
          style={{ zIndex: 1 }}
        />
        {/* Thumb */}
        <div
          className="pointer-events-none absolute top-1/2 h-5 w-5 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-[#0b0e1a] shadow-lg transition-all"
          style={{ left: `${pct}%`, background: accent }}
        />
      </div>
      <div className="flex justify-between text-xs text-[color:var(--ink-faint)]">
        <span>{min}{unit}</span>
        <span>{max}{unit}</span>
      </div>
    </div>
  );
}

// ── Temperature stepper ───────────────────────────────────────────────────────

function TempStepper({
  value, min, max, step, accent, onChange,
}: {
  value: number; min: number; max: number; step: number; accent: string;
  onChange: (v: number) => void;
}) {
  const dec = () => onChange(Math.max(min, parseFloat((value - step).toFixed(1))));
  const inc = () => onChange(Math.min(max, parseFloat((value + step).toFixed(1))));
  return (
    <div className="flex items-center justify-center gap-4">
      <button
        type="button"
        onClick={dec}
        className="flex h-12 w-12 items-center justify-center rounded-full border border-[rgba(255,255,255,0.1)] bg-[rgba(255,255,255,0.06)] text-xl font-bold text-[color:var(--ink-strong)] transition hover:bg-[rgba(255,255,255,0.12)] active:scale-95"
      >
        −
      </button>
      <div className="text-center">
        <p className="text-4xl font-bold" style={{ color: accent }}>{value.toFixed(1)}</p>
        <p className="text-xs text-[color:var(--ink-muted)]">°C</p>
      </div>
      <button
        type="button"
        onClick={inc}
        className="flex h-12 w-12 items-center justify-center rounded-full border border-[rgba(255,255,255,0.1)] bg-[rgba(255,255,255,0.06)] text-xl font-bold text-[color:var(--ink-strong)] transition hover:bg-[rgba(255,255,255,0.12)] active:scale-95"
      >
        +
      </button>
    </div>
  );
}

// ── Main modal ────────────────────────────────────────────────────────────────

export function DeviceControlModal({
  device, open, onClose,
  onSetState, onSetBrightness, onSetTemperature,
  onConnect, onDisconnect,
}: DeviceControlModalProps) {
  // Local copies so controls feel instant; synced to real device on open
  const [localBrightness, setLocalBrightness] = useState(0);
  const [localTemp, setLocalTemp] = useState(22.0);
  const [busy, setBusy] = useState(false);
  const [activeTab, setActiveTab] = useState<'controls' | 'history'>('controls');
  const { history, isLoading: histLoading } = useDeviceHistory(device?.name ?? '', 100);

  useEffect(() => {
    if (device) {
      setLocalBrightness(device.brightness ?? 0);
      setLocalTemp(device.temperature ?? 22.0);
    }
  }, [device]);

  useEffect(() => {
    if (!open) return;
    const h = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); };
    document.addEventListener('keydown', h);
    return () => document.removeEventListener('keydown', h);
  }, [open, onClose]);

  if (!open || !device) return null;

  const meta    = TYPE_META[device.device_type] ?? DEFAULT_META;
  const cfg     = getControlConfig(device.device_type);
  const isOn    = device.state === 'on';
  const [onLabel, offLabel] = cfg.toggleLabels;

  const run = async (fn: () => Promise<void>) => {
    setBusy(true);
    try { await fn(); } finally { setBusy(false); }
  };

  const commitBrightness = (v: number) => {
    setLocalBrightness(v);
    run(() => onSetBrightness(device.name, v));
  };

  const commitTemperature = (v: number) => {
    setLocalTemp(v);
    run(() => onSetTemperature(device.name, v));
  };

  return (
    <div className="fixed inset-0 z-50 flex items-end justify-center p-4 sm:items-center">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-[rgba(5,7,15,0.75)] backdrop-blur-sm" onClick={onClose} />

      {/* Panel */}
      <div className="relative w-full max-w-sm rounded-2xl border border-[var(--line-strong)] bg-[var(--bg-modal)] shadow-[var(--shadow-modal)]">

        {/* Header */}
        <div
          className="flex items-center gap-4 rounded-t-2xl px-5 py-4"
          style={{ background: meta.bg }}
        >
          <span
            className="flex h-13 w-13 shrink-0 items-center justify-center rounded-xl text-2xl"
            style={{ background: meta.bg, boxShadow: `0 0 0 1px ${meta.accent}30` }}
          >
            {meta.icon}
          </span>
          <div className="min-w-0 flex-1">
            <p className="truncate font-bold text-[color:var(--ink-strong)]">{device.name}</p>
            <p className="text-xs text-[color:var(--ink-muted)]">{meta.label}</p>
          </div>
          {/* Connection pill */}
          <div className="flex items-center gap-1.5 rounded-full border border-[rgba(148,155,200,0.15)] bg-[rgba(0,0,0,0.25)] px-2.5 py-1 text-xs">
            <span
              className="h-1.5 w-1.5 rounded-full"
              style={{ background: device.connected ? '#34d399' : '#4d5480' }}
            />
            <span className="text-[color:var(--ink-muted)]">{device.connected ? 'Online' : 'Offline'}</span>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="ml-1 flex h-7 w-7 items-center justify-center rounded-lg text-lg text-[color:var(--ink-muted)] transition hover:bg-[rgba(255,255,255,0.08)] hover:text-[color:var(--ink-strong)]"
          >
            ×
          </button>
        </div>

        {/* Tab navigation */}
        <div className="flex border-b border-[rgba(148,155,200,0.1)] px-5">
          {(['controls', 'history'] as const).map(tab => (
            <button
              key={tab}
              type="button"
              onClick={() => setActiveTab(tab)}
              className="mr-4 py-3 text-sm font-medium capitalize transition-colors"
              style={{
                color: activeTab === tab ? 'var(--accent)' : 'var(--ink-muted)',
                borderBottom: activeTab === tab ? '2px solid var(--accent)' : '2px solid transparent',
              }}
            >
              {tab}
            </button>
          ))}
        </div>

        {/* History tab */}
        {activeTab === 'history' && (
          <div className="px-5 py-5">
            {histLoading ? (
              <div className="flex h-48 items-center justify-center text-sm text-[color:var(--ink-muted)]">
                Loading…
              </div>
            ) : history.length === 0 ? (
              <div className="flex h-48 flex-col items-center justify-center gap-2 text-center">
                <span className="text-3xl">📊</span>
                <p className="text-sm font-medium text-[color:var(--ink-strong)]">No history</p>
                <p className="text-xs text-[color:var(--ink-muted)]">State changes will appear here</p>
              </div>
            ) : (
              <ResponsiveContainer width="100%" height={180}>
                <AreaChart data={history.map(h => ({ ts: h.ts.slice(11, 16), val: h.state === 'on' ? 1 : 0 }))}>
                  <XAxis dataKey="ts" tick={{ fontSize: 10, fill: 'var(--ink-muted)' }} interval="preserveStartEnd" />
                  <YAxis domain={[0, 1]} ticks={[0, 1]} tick={{ fontSize: 10, fill: 'var(--ink-muted)' }} tickFormatter={v => v === 1 ? 'On' : 'Off'} />
                  <Tooltip formatter={(v) => [Number(v) === 1 ? 'On' : 'Off', 'State']} labelFormatter={l => `Time: ${l}`} />
                  <Area type="stepAfter" dataKey="val" stroke={meta.accent} fill={`${meta.accent}22`} strokeWidth={2} dot={false} />
                </AreaChart>
              </ResponsiveContainer>
            )}
          </div>
        )}

        <div className={`flex flex-col gap-5 px-5 py-5 ${activeTab !== 'controls' ? 'hidden' : ''}`}>

          {/* ── Power toggle ──────────────────────────────────── */}
          {cfg.hasToggle && (
            <div className="flex items-center justify-between gap-4 rounded-xl border border-[rgba(148,155,200,0.1)] bg-[rgba(255,255,255,0.03)] px-4 py-3">
              <div>
                <p className="font-semibold text-[color:var(--ink-strong)]">
                  {isOn ? onLabel : offLabel}
                </p>
                <p className="text-xs text-[color:var(--ink-muted)]">
                  Currently {isOn ? 'on' : 'off'}
                </p>
              </div>
              {/* Toggle switch */}
              <button
                type="button"
                disabled={busy}
                onClick={() => run(() => onSetState(device.name, isOn ? 'off' : 'on'))}
                className="relative h-7 w-14 rounded-full transition-all duration-300 focus:outline-none disabled:opacity-50"
                style={{ background: isOn ? meta.accent : 'rgba(255,255,255,0.1)' }}
                aria-label={isOn ? 'Turn off' : 'Turn on'}
              >
                <span
                  className="absolute top-0.5 h-6 w-6 rounded-full bg-white shadow-md transition-all duration-300"
                  style={{ left: isOn ? 'calc(100% - 26px)' : '2px' }}
                />
              </button>
            </div>
          )}

          {/* ── Level slider (brightness / volume / speed / position) ── */}
          {cfg.levelControl && cfg.levelControl.field === 'brightness' && (
            <div className="rounded-xl border border-[rgba(148,155,200,0.1)] bg-[rgba(255,255,255,0.03)] px-4 py-4">
              <LevelSlider
                value={localBrightness}
                min={cfg.levelControl.min}
                max={cfg.levelControl.max}
                step={cfg.levelControl.step}
                label={cfg.levelControl.label}
                unit={cfg.levelControl.unit}
                accent={meta.accent}
                onChange={commitBrightness}
              />
            </div>
          )}

          {/* ── Temperature stepper ───────────────────────────── */}
          {cfg.levelControl && cfg.levelControl.field === 'temperature' && (
            <div className="rounded-xl border border-[rgba(148,155,200,0.1)] bg-[rgba(255,255,255,0.03)] px-4 py-5">
              <p className="mb-4 text-center text-sm font-medium text-[color:var(--ink-muted)]">
                {cfg.levelControl.label}
              </p>
              <TempStepper
                value={localTemp}
                min={cfg.levelControl.min}
                max={cfg.levelControl.max}
                step={cfg.levelControl.step}
                accent={meta.accent}
                onChange={commitTemperature}
              />
              {/* Also a fine-grained slider below */}
              <div className="mt-4">
                <LevelSlider
                  value={localTemp}
                  min={cfg.levelControl.min}
                  max={cfg.levelControl.max}
                  step={cfg.levelControl.step}
                  label=""
                  unit="°C"
                  accent={meta.accent}
                  onChange={commitTemperature}
                />
              </div>
            </div>
          )}

          {/* ── Sensor read-only display ─────────────────────── */}
          {cfg.readonly && (
            <div className="grid grid-cols-2 gap-3">
              <div className="rounded-xl border border-[rgba(148,155,200,0.1)] bg-[rgba(255,255,255,0.03)] px-3 py-3 text-center">
                <p className="text-[0.65rem] uppercase tracking-wider text-[color:var(--ink-muted)]">State</p>
                <p className="mt-1 font-semibold capitalize text-[color:var(--ink-strong)]">{device.state}</p>
              </div>
              {device.temperature != null && (
                <div className="rounded-xl border border-[rgba(148,155,200,0.1)] bg-[rgba(255,255,255,0.03)] px-3 py-3 text-center">
                  <p className="text-[0.65rem] uppercase tracking-wider text-[color:var(--ink-muted)]">Temperature</p>
                  <p className="mt-1 font-semibold" style={{ color: meta.accent }}>{device.temperature.toFixed(1)}°C</p>
                </div>
              )}
            </div>
          )}

          {/* ── Attributes panel ──────────────────────────────── */}
          {device.attributes && typeof device.attributes === 'object' && !Array.isArray(device.attributes) &&
            Object.entries(device.attributes as Record<string, unknown>).filter(([, v]) => v != null && v !== '').length > 0 && (
            <div className="rounded-xl border border-[rgba(148,155,200,0.1)] bg-[rgba(255,255,255,0.03)] overflow-hidden">
              <p className="border-b border-[rgba(148,155,200,0.08)] px-4 py-2 text-[0.65rem] uppercase tracking-[0.1em] text-[color:var(--ink-muted)]">Attributes</p>
              <div className="divide-y divide-[rgba(148,155,200,0.06)]">
                {Object.entries(device.attributes as Record<string, unknown>)
                  .filter(([, v]) => v != null && v !== '')
                  .map(([k, v]) => (
                    <div key={k} className="flex justify-between px-4 py-2 text-xs">
                      <span className="text-[color:var(--ink-muted)]">{k}</span>
                      <span className="font-mono text-[color:var(--ink-strong)]">{String(v)}</span>
                    </div>
                  ))}
              </div>
            </div>
          )}

          {/* ── Error ─────────────────────────────────────────── */}
          {device.last_error && (
            <p className="rounded-xl border border-[rgba(244,63,94,0.25)] bg-[rgba(244,63,94,0.08)] px-3 py-2 text-xs text-[#fb7185]">
              ⚠ {device.last_error}
            </p>
          )}

          {/* ── Connection control ────────────────────────────── */}
          <div className="flex gap-2 border-t border-[rgba(148,155,200,0.08)] pt-3">
            <button
              type="button"
              disabled={busy || device.connected}
              onClick={() => run(() => onConnect(device.name))}
              className="flex-1 rounded-xl border border-[rgba(52,211,153,0.25)] bg-[rgba(52,211,153,0.08)] py-2 text-sm font-medium text-[#34d399] transition hover:bg-[rgba(52,211,153,0.14)] disabled:opacity-30"
            >
              Connect
            </button>
            <button
              type="button"
              disabled={busy || !device.connected}
              onClick={() => run(() => onDisconnect(device.name))}
              className="flex-1 rounded-xl border border-[rgba(148,155,200,0.15)] bg-[rgba(255,255,255,0.04)] py-2 text-sm font-medium text-[color:var(--ink-muted)] transition hover:bg-[rgba(255,255,255,0.08)] disabled:opacity-30"
            >
              Disconnect
            </button>
          </div>

          <div className="border-t border-[rgba(148,155,200,0.08)] pt-3">
            <button
              type="button"
              disabled={busy}
              onClick={() => run(() => snapshotScene({ name: `${device.name} scene`, device_ids: [device.id] }).then(() => undefined))}
              className="w-full rounded-xl border border-[rgba(6,182,212,0.2)] bg-[rgba(6,182,212,0.06)] py-2 text-sm font-medium text-[#22d3ee] transition hover:bg-[rgba(6,182,212,0.12)] disabled:opacity-30"
            >
              Save as Scene
            </button>
          </div>

        </div>{/* end controls tab */}
      </div>
    </div>
  );
}
