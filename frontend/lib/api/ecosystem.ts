import { EcosystemResponse, ProtocolInfo } from './types';

export async function getEcosystem(): Promise<EcosystemResponse> {
  const res = await fetch('/api/ecosystem');
  if (!res.ok) throw new Error('Failed to fetch ecosystem');
  return res.json();
}

export async function getProtocols(): Promise<ProtocolInfo[]> {
  const res = await fetch('/api/protocols');
  if (!res.ok) throw new Error('Failed to fetch protocols');
  return res.json();
}
