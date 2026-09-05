// The seven e2e principals, their roles, their sessions, and the users the
// traffic dataset attributes to.
//
// Roles live in users.roles — the minted JWT's own role claim is ignored by the
// server — so this file is what decides what every authorization assertion in
// the suite sees.
import type { Client } from 'pg';
import { hasTable } from './kit';

// Why: core's session middleware attests that a cookie names a session the
// server issued *to that user*; a token carrying an unknown session_id is
// treated as stale and silently replaced with an anonymous one. Each principal
// therefore needs a real user_sessions row whose id the minted JWT repeats.
export const E2E_SESSIONS: Record<string, string> = {
  'e2e-platform-admin': 'e2e-session-platform-admin',
  'e2e-admin': 'e2e-session-admin',
  'e2e-developer': 'e2e-session-developer',
  'e2e-business-user': 'e2e-session-business-user',
  'e2e-unassigned': 'e2e-session-unassigned',
  'e2e-pm': 'e2e-session-pm',
  'e2e-user': 'e2e-session-user',
};

export const E2E = {
  admin: { id: 'e2e-admin', email: 'e2e-admin@e2e.local' },
  platformAdmin: { id: 'e2e-platform-admin', email: 'e2e-platform-admin@e2e.local' },
  developer: { id: 'e2e-developer', email: 'e2e-developer@e2e.local' },
  businessUser: { id: 'e2e-business-user', email: 'e2e-business-user@e2e.local' },
  unassigned: { id: 'e2e-unassigned', email: 'e2e-unassigned@e2e.local' },
  pm: { id: 'e2e-pm', email: 'e2e-pm@e2e.local' },
  user: { id: 'e2e-user', email: 'e2e-user@e2e.local' },
  members: [1, 2, 3, 4, 5, 6, 7].map((n) => ({
    id: `e2e-member-${n}`,
    email: `e2e-member-${n}@e2e.local`,
  })),
  victim: { id: 'e2e-victim', email: 'e2e-victim@e2e.local' },
  groups: {
    europe: 'europe-devs',
    india: 'india-devs',
    business: 'business-users',
  },
  projects: {
    storefront: 'storefront',
    checkout: 'checkout',
    payments: 'payments',
    platform: 'platform',
  },
  marketplaces: {
    europe: 'astound-europe-dev',
    india: 'astound-india-dev',
    cowork: 'astound-cowork',
    baseline: 'enterprise-demo',
  },
};

// The twelve users the declarative traffic dataset attributes rows to, in a
// fixed order: request n belongs to ACTORS[n % 12], which is what makes the
// per-user totals on the People pages exact rather than approximate.
export const ACTORS: string[] = [
  E2E.developer.id,
  E2E.members[0].id,
  E2E.members[1].id,
  E2E.members[2].id,
  E2E.pm.id,
  E2E.members[3].id,
  E2E.members[4].id,
  E2E.businessUser.id,
  E2E.members[5].id,
  E2E.members[6].id,
  E2E.user.id,
  E2E.unassigned.id,
];

const PRINCIPALS: { id: string; email: string; roles: string[]; manualRoles: string[] }[] = [
  { ...E2E.platformAdmin, roles: ['platform_admin', 'admin', 'user'], manualRoles: ['platform_admin'] },
  { ...E2E.admin, roles: ['admin', 'user'], manualRoles: [] },
  { ...E2E.developer, roles: ['developer', 'user'], manualRoles: [] },
  { ...E2E.businessUser, roles: ['user'], manualRoles: [] },
  { ...E2E.unassigned, roles: ['user'], manualRoles: [] },
  { ...E2E.pm, roles: ['project_manager', 'user'], manualRoles: ['project_manager'] },
  { ...E2E.user, roles: ['user'], manualRoles: [] },
  { ...E2E.victim, roles: ['user'], manualRoles: [] },
  ...E2E.members.map((m) => ({ ...m, roles: ['user'], manualRoles: [] as string[] })),
];

// Why the retry: several agents share one database on this clone, and two
// seeds running at once both insert the same principal. ON CONFLICT arbitrates
// on `id`, so a peer's in-flight row can still surface as a violation of the
// `email` or `name` unique index instead. The row the peer is writing is byte
// for byte the row we want, so one retry after its commit is the whole fix.
async function upsertUser(db: Client, id: string, email: string, roles: string[]) {
  for (let attempt = 0; attempt < 2; attempt += 1) {
    try {
      const r = await db.query(
        `INSERT INTO users (id, name, email, display_name, status, email_verified, roles)
         VALUES ($1, $2, $2, $3, 'active', true, $4)
         ON CONFLICT (id) DO UPDATE SET roles = EXCLUDED.roles, status = 'active' RETURNING id`,
        [id, email, email.split('@')[0], roles],
      );
      if (r.rowCount !== 1) throw new Error(`upsertUser(${id}) affected ${r.rowCount} rows`);
      return;
    } catch (e) {
      if ((e as { code?: string }).code !== '23505' || attempt === 1) throw e;
      await new Promise((resolve) => setTimeout(resolve, 250));
    }
  }
}

// Migration 041 replaced user_profile_ext.project and the adfs_group_memberships
// side table with groups / group_members / projects / project_members. Missing
// tables mean an unmigrated DB, and every group assertion downstream would fail
// obscurely, so fail here with the migration named.
export async function requireGroupsSchema(db: Client) {
  const tables = [
    'groups',
    'group_members',
    'group_ad_mappings',
    'projects',
    'project_members',
    'project_ad_mappings',
    'user_manual_roles',
    'user_groups',
  ];
  const { rows } = await db.query<{ present: string }>(
    `SELECT c.relname AS present FROM pg_class c
      JOIN pg_namespace n ON n.oid = c.relnamespace
      WHERE n.nspname = 'public' AND c.relname = ANY($1)`,
    [tables],
  );
  const present = new Set(rows.map((r) => r.present));
  const missing = tables.filter((t) => !present.has(t));
  if (missing.length > 0) {
    throw new Error(
      `e2e seed: migration 041_groups_projects.sql has not been applied — missing ${missing.join(', ')}. ` +
        `Build and restart the server (migrations run at startup), then re-run the seed.`,
    );
  }
}

// user_scope_defaults (migration 042) is written by the recomputation job, not
// by this seed: a seeded row would be indistinguishable from a computed one and
// would hide the job failing to run. The seed only asserts it can tolerate the
// table being absent on an older DB.
export async function scopeDefaultsPresent(db: Client): Promise<boolean> {
  return hasTable(db, 'user_scope_defaults');
}

export async function seedPrincipals(db: Client) {
  for (const p of PRINCIPALS) {
    await upsertUser(db, p.id, p.email, p.roles);
    await db.query(
      `INSERT INTO user_profile_ext (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING`,
      [p.id],
    );
  }
  await db.query(`DELETE FROM user_manual_roles WHERE user_id LIKE 'e2e-%'`);
  for (const p of PRINCIPALS) {
    for (const role of p.manualRoles) {
      await db.query(
        `INSERT INTO user_manual_roles (user_id, role) VALUES ($1, $2) ON CONFLICT DO NOTHING`,
        [p.id, role],
      );
    }
  }
  for (const [userId, sessionId] of Object.entries(E2E_SESSIONS)) {
    await db.query(
      `INSERT INTO user_sessions (session_id, user_id, user_type, expires_at, last_activity_at)
       VALUES ($1, $2, 'registered', NOW() + INTERVAL '7 days', NOW())
       ON CONFLICT (session_id) DO UPDATE
          SET expires_at = NOW() + INTERVAL '7 days',
              last_activity_at = NOW(),
              ended_at = NULL`,
      [sessionId, userId],
    );
  }
}
