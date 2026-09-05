// Bridge and device fleet: live bridge sessions, enrolled desktop apps, device
// certificates, personal access keys, and two device links still waiting to be
// claimed.
import type { Client } from 'pg';
import { ID, ago, minutesAgo } from './kit';
import { ACTORS } from './principals';

const BRIDGE_COUNT = 8;
const PENDING_LINKS = 2;

const HOSTS: { os: string; hostname: string; version: string }[] = [
  { os: 'windows', hostname: 'AST-WIN-0142', version: '0.44.1' },
  { os: 'macos', hostname: 'ast-mbp-lopez', version: '0.44.1' },
  { os: 'linux', hostname: 'ast-wsl-dev01', version: '0.44.0' },
  { os: 'macos', hostname: 'ast-mbp-ncube', version: '0.43.2' },
];

const PLATFORMS = ['windows', 'macos', 'linux'] as const;

async function seedBridgeSessions(db: Client) {
  for (let n = 0; n < BRIDGE_COUNT; n += 1) {
    const host = HOSTS[n % HOSTS.length];
    // Two of the eight last heartbeated over a day ago, so "active bridges"
    // and "enrolled bridges" are different numbers on the fleet page.
    const stale = n >= BRIDGE_COUNT - 2;
    const heartbeat = stale ? ago(2, n) : minutesAgo(2 + n * 7);
    await db.query(
      `INSERT INTO bridge_sessions
           (session_id, user_id, bridge_version, os, hostname, started_at,
            last_heartbeat_at, last_activity_at, forwarded_total, tokens_in_total, tokens_out_total)
       VALUES ($1, $2, $3, $4, $5, $6::timestamptz, $7::timestamptz, $7::timestamptz, $8, $9, $10)
       ON CONFLICT (session_id) DO UPDATE SET last_heartbeat_at = EXCLUDED.last_heartbeat_at`,
      [
        ID.bridge(n),
        ACTORS[n % ACTORS.length],
        host.version,
        host.os,
        `${host.hostname}-${n}`,
        ago(n % 9, 8),
        heartbeat,
        40 + n * 13,
        120_000 + n * 4_100,
        18_000 + n * 900,
      ],
    );
  }
}

async function seedDevices(db: Client) {
  for (let n = 0; n < 6; n += 1) {
    const host = HOSTS[n % HOSTS.length];
    await db.query(
      `INSERT INTO device_app_links
           (device_id, user_id, app_platform, app_version, hostname, last_seen_at, enrolled_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7)
       ON CONFLICT (device_id) DO UPDATE SET last_seen_at = EXCLUDED.last_seen_at`,
      [
        ID.device(n),
        ACTORS[n % ACTORS.length],
        PLATFORMS[n % PLATFORMS.length],
        host.version,
        `${host.hostname}-${n}`,
        minutesAgo(5 + n * 30),
        ago(20 + n, 3),
      ],
    );
    await db.query(
      `INSERT INTO user_device_certs (id, user_id, fingerprint, label, enrolled_at, revoked_at)
       VALUES ($1, $2, $3, $4, $5, $6)
       ON CONFLICT (fingerprint) DO NOTHING`,
      [
        ID.cert(n),
        ACTORS[n % ACTORS.length],
        `e2e:${'ab12cd34'.repeat(2)}:${n}`,
        `${host.hostname}-${n}`,
        ago(20 + n, 3),
        // One revoked cert, so the revoked filter on the devices table is not
        // a permanently empty view.
        n === 5 ? ago(1, 2) : null,
      ],
    );
  }
  for (let n = 0; n < 3; n += 1) {
    await db.query(
      `INSERT INTO user_api_keys (id, user_id, name, key_prefix, key_hash, created_at, last_used_at, revoked_at)
       VALUES ($1, $2, $3, $4, 'e2e-not-a-real-hash', $5, $6, $7)
       ON CONFLICT (key_prefix) DO NOTHING`,
      [
        ID.apiKey(n),
        ACTORS[n],
        ['CI runner', 'Laptop', 'Retired token'][n],
        `e2e_pat_${n}`,
        ago(30 + n, 1),
        n === 2 ? null : minutesAgo(60 + n * 45),
        n === 2 ? ago(3, 1) : null,
      ],
    );
  }
}

// A device link is pending while its exchange code is unconsumed and unexpired;
// the enrolment page lists exactly those, so two of them keep that list honest.
async function seedPendingLinks(db: Client) {
  for (let n = 0; n < PENDING_LINKS; n += 1) {
    await db.query(
      `INSERT INTO bridge_exchange_codes (code_hash, user_id, created_at, expires_at, consumed_at)
       VALUES ($1, $2, $3, $4, NULL)
       ON CONFLICT (code_hash) DO UPDATE SET expires_at = EXCLUDED.expires_at, consumed_at = NULL`,
      [
        `e2e-dcode-${n}`,
        ACTORS[n],
        minutesAgo(3 + n),
        new Date(Date.now() + 600_000),
      ],
    );
  }
}

export async function seedBridge(db: Client) {
  await seedBridgeSessions(db);
  await seedDevices(db);
  await seedPendingLinks(db);
}
