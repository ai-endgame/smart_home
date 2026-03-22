'use client';
import { useRef, useEffect } from 'react';
import type { HistoryEntry } from '@/lib/api/types';

interface SparkLineProps {
  points: HistoryEntry[];
  width?: number;
  height?: number;
}

export function SparkLine({ points, width = 80, height = 20 }: SparkLineProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || points.length < 2) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    ctx.scale(dpr, dpr);

    const accentColor = getComputedStyle(document.documentElement)
      .getPropertyValue('--accent').trim() || '#6366f1';
    const mutedColor = getComputedStyle(document.documentElement)
      .getPropertyValue('--line-strong').trim() || '#3f3f5a';

    const segW = width / (points.length - 1);

    for (let i = 0; i < points.length - 1; i++) {
      const isOn = points[i].state === 'on';
      const x1 = i * segW;
      const x2 = (i + 1) * segW;
      const y = height / 2;

      ctx.beginPath();
      ctx.moveTo(x1, y);
      ctx.lineTo(x2, y);
      ctx.strokeStyle = isOn ? accentColor : mutedColor;
      ctx.lineWidth = 2;
      ctx.stroke();
    }
  }, [points, width, height]);

  if (points.length < 2) return null;

  return (
    <canvas
      ref={canvasRef}
      style={{ width, height }}
      aria-hidden="true"
    />
  );
}
