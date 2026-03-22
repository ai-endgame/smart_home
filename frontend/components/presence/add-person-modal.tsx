'use client';
import { useState } from 'react';
import { Modal } from '@/components/ui/modal';
import { Button } from '@/components/ui/button';
import { Field } from '@/components/ui/field';
import type { CreatePersonRequest } from '@/lib/api/types';

interface AddPersonModalProps {
  open: boolean;
  onClose: () => void;
  onAdd: (req: CreatePersonRequest) => Promise<void>;
}

export function AddPersonModal({ open, onClose, onAdd }: AddPersonModalProps) {
  const [name, setName] = useState('');
  const [grace, setGrace] = useState(120);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [nameError, setNameError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setNameError('');
    if (!name.trim()) { setNameError('Name is required.'); return; }
    setError('');
    setLoading(true);
    try {
      await onAdd({ name: name.trim(), grace_period_secs: grace });
      setName('');
      setGrace(120);
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create person');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal title="Add Person" open={open} onClose={onClose}>
      <form onSubmit={handleSubmit} noValidate className="flex flex-col gap-4">
        <Field label="Name" hint="The person's display name — used to identify them in automations and presence tracking." error={nameError}>
          <input value={name} onChange={e => { setName(e.target.value); if (nameError) setNameError(''); }} placeholder="e.g. Alice" />
        </Field>

        <Field
          label="Grace Period (seconds)"
          hint="How long the system waits after a source reports 'away' before marking the person as away. Prevents brief signal drops from triggering automations. Default is 120 s (2 min)."
        >
          <input
            type="number"
            value={grace}
            onChange={e => setGrace(parseInt(e.target.value, 10) || 0)}
            min={0}
            max={3600}
          />
        </Field>

        {error && (
          <p className="rounded-xl border border-[rgba(244,63,94,0.3)] bg-[rgba(244,63,94,0.1)] px-3 py-2 text-sm text-[#fb7185]">{error}</p>
        )}

        <div className="flex justify-end gap-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" disabled={loading}>{loading ? 'Adding…' : 'Add Person'}</Button>
        </div>
      </form>
    </Modal>
  );
}
