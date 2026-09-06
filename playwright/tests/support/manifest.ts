// Signed-manifest helpers for the bridge principals.
//
// The seed mints one deterministic PAT per bridge principal (setup/seed/pats.ts)
// and writes the secrets to playwright/.auth/pats.json, so a spec never has to
// reach the console's PAT form — which a developer or knowledge worker cannot
// open anyway. `fetchManifest` verifies the Ed25519 signature against the key
// `/v1/bridge/pubkey` publishes before it hands the payload back: an unsigned
// or mis-signed bundle is a failure here, never a payload a spec reads.
import { createPublicKey, verify } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { PATS_FILE } from '../../setup/seed';
import type { BridgeRole } from '../../setup/seed/principals';
import type { SeededPat } from '../../setup/seed/pats';

export type { BridgeRole } from '../../setup/seed/principals';

export interface ManifestEnvelope {
  payload: string;
  signature: string;
}

export interface ManifestPayload {
  user_id: string;
  issued_at?: string;
  not_before?: string;
  plugins: { id: string; version: string; files: { path: string }[] }[];
  skills: { id: string; name: string; plugins?: string[] }[];
  agents: { id: string }[];
  managed_mcp_servers: { id: string; name: string; url: string }[];
  artifacts: { id: string }[];
  marketplaces?: { id: string; name: string; plugin_ids: string[] }[];
  [key: string]: unknown;
}

/** The seeded PAT for a bridge principal. */
export function seededPat(role: BridgeRole): SeededPat {
  const book = JSON.parse(readFileSync(PATS_FILE, 'utf8')) as Record<BridgeRole, SeededPat>;
  const pat = book[role];
  if (!pat) throw new Error(`no seeded PAT for ${role} in ${PATS_FILE}; run the seed`);
  return pat;
}

// A raw 32-byte Ed25519 public key is not something node:crypto imports on its
// own; wrapped in the fixed SPKI prefix for the curve it is.
const ED25519_SPKI_PREFIX = Buffer.from('302a300506032b6570032100', 'hex');

export async function bridgePubkey(baseURL: string): Promise<ReturnType<typeof createPublicKey>> {
  const res = await fetch(`${baseURL}/v1/bridge/pubkey`);
  if (!res.ok) throw new Error(`/v1/bridge/pubkey returned ${res.status}`);
  const { pubkey } = (await res.json()) as { pubkey: string };
  const raw = Buffer.from(pubkey, 'base64');
  if (raw.length !== 32) throw new Error(`bridge pubkey is ${raw.length} bytes, expected 32`);
  return createPublicKey({
    key: Buffer.concat([ED25519_SPKI_PREFIX, raw]),
    format: 'der',
    type: 'spki',
  });
}

/** Throws unless `signature` is the gateway's Ed25519 signature over `payload`. */
export async function assertSigned(baseURL: string, envelope: ManifestEnvelope): Promise<void> {
  const key = await bridgePubkey(baseURL);
  const ok = verify(
    null,
    Buffer.from(envelope.payload, 'utf8'),
    key,
    Buffer.from(envelope.signature, 'base64'),
  );
  if (!ok) throw new Error('manifest signature does not verify against /v1/bridge/pubkey');
}

export interface FetchedManifest {
  envelope: ManifestEnvelope;
  payload: ManifestPayload;
}

const exchanged = new Map<string, string>();

/**
 * The bridge JWT for a credential. A PAT (`sp-live-…`) is exchanged once at
 * /v1/auth/bridge/pat, exactly as the bridge does after enrolment — the
 * manifest and plugin-file routes accept the JWT, not the raw PAT. Anything
 * else is assumed to be a JWT already.
 */
export async function bridgeToken(baseURL: string, credential: string): Promise<string> {
  if (!credential.startsWith('sp-live-')) return credential;
  const cached = exchanged.get(credential);
  if (cached) return cached;
  const res = await fetch(`${baseURL}/v1/auth/bridge/pat`, {
    method: 'POST',
    headers: { authorization: `Bearer ${credential}`, 'content-type': 'application/json' },
    body: '{}',
  });
  if (!res.ok) {
    throw new Error(`/v1/auth/bridge/pat returned ${res.status}: ${await res.text()}`);
  }
  const { token } = (await res.json()) as { token: string };
  exchanged.set(credential, token);
  return token;
}

/** The manifest for a PAT or bridge JWT, signature verified, payload parsed. */
export async function fetchManifest(
  baseURL: string,
  bearer: string,
  headers: Record<string, string> = {},
): Promise<FetchedManifest> {
  const res = await fetch(`${baseURL}/v1/bridge/manifest`, {
    headers: { authorization: `Bearer ${await bridgeToken(baseURL, bearer)}`, ...headers },
  });
  if (!res.ok) {
    throw new Error(`/v1/bridge/manifest returned ${res.status}: ${await res.text()}`);
  }
  const envelope = (await res.json()) as ManifestEnvelope;
  await assertSigned(baseURL, envelope);
  return { envelope, payload: JSON.parse(envelope.payload) as ManifestPayload };
}

/** Set comparison that names both sides in the failure. */
export function sortedUnique(values: Iterable<string>): string[] {
  return [...new Set(values)].sort();
}

// Timestamps and anything expiry-like differ between two fetches of the same
// bundle; everything else must not.
const VOLATILE = /^(issued_at|not_before|expires?|expires_at|expiry|expires_in.*)$|^manifest_version$/;

export function stable(payload: ManifestPayload): Record<string, unknown> {
  return Object.fromEntries(Object.entries(payload).filter(([k]) => !VOLATILE.test(k)));
}
