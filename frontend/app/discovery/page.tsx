'use client';
import { useDiscovery } from '@/lib/hooks/use-discovery';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';

export default function DiscoveryPage() {
  const { discovered, isLoading, addToHome, refresh } = useDiscovery();

  return (
    <div className="space-y-5">
      <section className="surface-card p-5 sm:p-6">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="section-kicker">Network Discovery</p>
            <h1 className="section-title">Detect devices on your network</h1>
            <p className="section-subtitle">Scan mDNS services and add discovered hardware into your home graph.</p>
          </div>
          <Button variant="secondary" onClick={() => refresh()}>
            Refresh Scan
          </Button>
        </div>
      </section>

      {isLoading ? (
        <section className="surface-card p-6">
          <p className="text-sm text-[color:var(--ink-muted)]">Scanning network...</p>
        </section>
      ) : discovered.length === 0 ? (
        <section className="surface-card p-6">
          <p className="text-sm text-[color:var(--ink-muted)]">No devices discovered on the network.</p>
        </section>
      ) : (
        <section className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
          {discovered.map(d => (
            <article key={d.id} className="surface-card flex flex-col gap-3 p-4">
              <div className="flex items-start justify-between gap-3">
                <div>
                  <p className="font-semibold text-[color:var(--ink-strong)]">{d.name}</p>
                  <p className="mt-0.5 text-xs text-[color:var(--ink-muted)]">
                    {d.host}:{d.port}
                  </p>
                </div>
                <Badge label={d.service_type} variant="info" />
              </div>

              {Object.entries(d.properties).length > 0 && (
                <dl className="max-h-28 space-y-1 overflow-auto rounded-xl border border-[color:var(--line)] bg-white/75 p-2 text-xs text-[color:var(--ink-muted)]">
                  {Object.entries(d.properties).map(([k, v]) => (
                    <div key={k} className="flex items-start gap-1">
                      <dt className="font-semibold text-[color:var(--ink-strong)]">{k}:</dt>
                      <dd className="break-all">{v}</dd>
                    </div>
                  ))}
                </dl>
              )}

              <Button
                size="sm"
                onClick={() =>
                  addToHome({
                    discovered_id: d.id,
                    name: d.name.replace(/[^a-z0-9_]/gi, '_').toLowerCase(),
                    device_type: d.suggested_type,
                  })
                }
              >
                Add to Home
              </Button>
            </article>
          ))}
        </section>
      )}
    </div>
  );
}
