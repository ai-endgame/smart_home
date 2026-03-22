export async function restoreBackup(file: File): Promise<Record<string, number>> {
  const text = await file.text();
  const res = await fetch('/api/restore', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: text,
  });
  if (!res.ok) throw new Error('Failed to restore backup');
  const json = await res.json();
  return json.restored as Record<string, number>;
}

export async function downloadBackup(): Promise<void> {
  const res = await fetch('/api/backup');
  if (!res.ok) throw new Error('Failed to fetch backup');
  const blob = await res.blob();
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `smart-home-backup-${new Date().toISOString().slice(0, 10)}.json`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
