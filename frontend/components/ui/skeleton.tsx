import { clsx } from 'clsx';

interface SkeletonProps {
  className?: string;
}

export function Skeleton({ className }: SkeletonProps) {
  return <div className={clsx('skeleton', className)} aria-hidden />;
}

/** Pre-built skeleton that mimics a DeviceCard */
export function DeviceCardSkeleton() {
  return (
    <div className="surface-card flex flex-col overflow-hidden">
      <div className="flex flex-1 flex-col gap-3 p-4">
        <div className="flex items-start justify-between gap-3">
          <div className="flex items-center gap-3">
            <Skeleton className="h-10 w-10 rounded-xl" />
            <div className="space-y-2">
              <Skeleton className="h-4 w-28" />
              <Skeleton className="h-3 w-16" />
            </div>
          </div>
          <Skeleton className="h-5 w-14 rounded-full" />
        </div>
        <Skeleton className="h-3 w-40" />
      </div>
      <div className="flex items-center gap-3 border-t border-[color:var(--line)] px-4 py-3">
        <Skeleton className="h-6 w-11 rounded-full" />
        <Skeleton className="h-3 w-6" />
        <div className="ml-auto flex gap-2">
          <Skeleton className="h-7 w-7 rounded-lg" />
          <Skeleton className="h-7 w-7 rounded-lg" />
        </div>
      </div>
      <div className="h-1 bg-[var(--surface-hover)]" />
    </div>
  );
}

/** Pre-built skeleton that mimics an automation rule row */
export function RuleRowSkeleton() {
  return (
    <div className="surface-card flex flex-col gap-4 p-5 md:flex-row md:items-center md:justify-between">
      <div className="flex-1 space-y-3">
        <div className="flex items-center gap-2">
          <Skeleton className="h-4 w-32" />
          <Skeleton className="h-5 w-16 rounded-full" />
        </div>
        <div className="flex items-center gap-2">
          <Skeleton className="h-7 w-44 rounded-lg" />
          <Skeleton className="h-3 w-4" />
          <Skeleton className="h-7 w-44 rounded-lg" />
        </div>
      </div>
      <div className="flex gap-2">
        <Skeleton className="h-8 w-16 rounded-xl" />
        <Skeleton className="h-8 w-20 rounded-xl" />
        <Skeleton className="h-8 w-16 rounded-xl" />
      </div>
    </div>
  );
}
