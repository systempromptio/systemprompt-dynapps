// One deterministic personal access token per bridge principal.
//
// The secret is derived from the seeded PRNG, so every run of the seed writes
// the same `user_api_keys` row and the same secret; a spec that reads
// playwright/.auth/pats.json after any seed holds a token the gateway accepts.
// The hash is the real one — sha256 over the whole presented secret, hex — so
// these rows are verified by the same code path as a token the console issued
// (core: ApiKeyService::verify). The fake-hash rows in bridge.ts exist to fill
// the devices table and are deliberately NOT usable credentials.
import { createHash } from 'node:crypto';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Client } from 'pg';
import { prng } from './kit';
import { BRIDGE_IDS, E2E, type BridgeRole } from './principals';

export const PATS_FILE = join(__dirname, '..', '..', '.auth', 'pats.json');

const API_KEY_PREFIX = 'sp-live-';
const PREFIX_ID_HEX = 12;
const SECRET_HEX = 52;

export interface SeededPat {
  userId: string;
  email: string;
  keyId: string;
  secret: string;
}

export type PatBook = Record<BridgeRole, SeededPat>;

function hexFrom(rand: () => number, chars: number): string {
  let out = '';
  while (out.length < chars) out += Math.floor(rand() * 16).toString(16);
  return out;
}

// sp-live-<12 hex>.<52 hex>: the prefix (up to the dot) is the lookup key and
// the whole string is what is hashed, mirroring core's generate_secret.
export function patFor(role: BridgeRole): SeededPat {
  const principal = E2E.bridge[role];
  const rand = prng(0x5eed0000 + Object.keys(BRIDGE_IDS).indexOf(role));
  const prefix = `${API_KEY_PREFIX}${hexFrom(rand, PREFIX_ID_HEX)}`;
  return {
    userId: principal.id,
    email: principal.email,
    keyId: `e2e-dpat-${role}`,
    secret: `${prefix}.${hexFrom(rand, SECRET_HEX)}`,
  };
}

export function patBook(): PatBook {
  const roles = Object.keys(BRIDGE_IDS) as BridgeRole[];
  return Object.fromEntries(roles.map((r) => [r, patFor(r)])) as PatBook;
}

export async function seedPats(db: Client) {
  const book = patBook();
  for (const pat of Object.values(book)) {
    const keyPrefix = pat.secret.slice(0, pat.secret.indexOf('.'));
    const keyHash = createHash('sha256').update(pat.secret).digest('hex');
    // A re-run must revive a token a spec revoked, so revoked_at is cleared.
    await db.query(
      `INSERT INTO user_api_keys (id, user_id, name, key_prefix, key_hash, created_at, last_used_at, revoked_at)
       VALUES ($1, $2, $3, $4, $5, NOW(), NULL, NULL)
       ON CONFLICT (id) DO UPDATE
          SET key_prefix = EXCLUDED.key_prefix, key_hash = EXCLUDED.key_hash, revoked_at = NULL`,
      [pat.keyId, pat.userId, `e2e bridge (${pat.email})`, keyPrefix, keyHash],
    );
  }
  mkdirSync(join(PATS_FILE, '..'), { recursive: true });
  writeFileSync(PATS_FILE, JSON.stringify(book, null, 2));
}
