// Idempotent e2e seed: the seven principals, the five bridge principals and
// their tokens, the access graph, and a
// declarative demo dataset placed at fixed offsets from the instant the seed
// started.
//
// Every row this script owns carries an `e2e-` id prefix or an `@e2e.local`
// email; `--reset` deletes exactly those rows (children before parents) and
// nothing else — never TRUNCATE, never a developer's data. Safe to run
// repeatedly: every statement upserts.
//
// The dataset itself lives in setup/seed/*.ts; this file is the connection, the
// order, and the reset. Column shapes mirror tests/contract/admin/src/seed.rs
// and the declarative schema under extensions/web/schema/ — if an insert
// breaks, diff against those files first.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { Client } from 'pg';
import { seedBridge } from './seed/bridge';
import { seedGovernance } from './seed/governance';
import { seedAnalyticsTrail, seedGroupTraffic } from './seed/legacy';
import { seedPats } from './seed/pats';
import {
  E2E_USER_IDS,
  seedAccessControl,
  seedGroupsAndProjects,
  recomputeScopeDefaults,
  requireDeclaredMarketplaceAccess,
  seedMemberships,
  seedSalesforceLinks,
} from './seed/people';
import { requireGroupsSchema, seedPrincipals } from './seed/principals';
import type { MarketplaceAccessReport } from './seed/people';
import { seedTraffic } from './seed/traffic';

const REPO = join(__dirname, '..', '..');

export { BRIDGE_IDS, E2E, E2E_SESSIONS } from './seed/principals';
export { PATS_FILE } from './seed/pats';
export { T0 } from './seed/kit';

export function databaseUrl(): string {
  if (process.env.E2E_DATABASE_URL) return process.env.E2E_DATABASE_URL;
  const secrets = JSON.parse(
    readFileSync(join(REPO, '.systemprompt', 'profiles', 'local', 'secrets.json'), 'utf8'),
  );
  if (!secrets.database_url) throw new Error('no database_url in local profile secrets.json');
  return secrets.database_url;
}

// Children before parents; every predicate is anchored to the e2e prefix.
//
// The `groups` and `projects` rows themselves are deliberately NOT deleted: a
// developer may have hand-assigned a real user to europe-devs, and the
// membership rows (all `e2e-%`) are what this seed actually owns. The group
// rows are upserted on every run, so they cannot drift.
//
// A table this seed writes but that has no ownership column gets a scoped
// predicate instead, named below with the reason.
//
// The bridge principals are keyed by UUID, not by an `e2e-` id, so every
// child row they own — and every row a spec created under their token: real
// PATs, exchange codes, gateway requests — is found through their
// `@e2e.local` email (E2E_USER_IDS) rather than the id prefix.
const E2E_REQUEST_IDS = `SELECT id FROM ai_requests WHERE user_id IN (${E2E_USER_IDS})`;

const RESET_STATEMENTS = [
  `DELETE FROM salesforce_user_identities WHERE user_id IN (${E2E_USER_IDS})`,
  `DELETE FROM bridge_exchange_codes WHERE user_id IN (${E2E_USER_IDS})`,
  `DELETE FROM bridge_sessions WHERE user_id IN (${E2E_USER_IDS})`,
  `DELETE FROM user_api_keys WHERE user_id IN (${E2E_USER_IDS})`,
  `DELETE FROM user_sessions WHERE user_id IN (${E2E_USER_IDS})`,
  `DELETE FROM user_manual_roles WHERE user_id IN (${E2E_USER_IDS})`,
  `DELETE FROM user_profile_ext WHERE user_id IN (${E2E_USER_IDS})`,
  `DELETE FROM ai_request_tool_calls WHERE id LIKE 'e2e-%'`,
  `DELETE FROM ai_safety_findings WHERE id LIKE 'e2e-%'`,
  `DELETE FROM approval_requests WHERE call_id LIKE 'e2e-%'`,
  `DELETE FROM mcp_tool_executions WHERE mcp_execution_id LIKE 'e2e-%'`,
  `DELETE FROM mcp_sessions WHERE session_id LIKE 'e2e-%'`,
  // Only rules the seed minted itself. The marketplace grants declared in
  // services/marketplaces/*/config.yaml are ingested at boot and never carry
  // an e2e- id (requireDeclaredMarketplaceAccess refuses to run if one does),
  // so this cannot revoke a workspace from a real developer.
  `DELETE FROM access_control_rules WHERE id LIKE 'e2e-rule-%'`,
  `DELETE FROM group_members WHERE user_id LIKE 'e2e-%'`,
  `DELETE FROM project_members WHERE user_id LIKE 'e2e-%'`,
  `DELETE FROM user_manual_roles WHERE user_id LIKE 'e2e-%'`,
  `DELETE FROM session_cost_snapshots WHERE session_id LIKE 'e2e-%'`,
  `DELETE FROM session_analyses WHERE session_id LIKE 'e2e-%'`,
  `DELETE FROM session_ratings WHERE session_id LIKE 'e2e-%'`,
  `DELETE FROM session_entity_links WHERE session_id LIKE 'e2e-%'`,
  `DELETE FROM skill_ratings WHERE id LIKE 'e2e-%'`,
  `DELETE FROM plugin_usage_daily WHERE id LIKE 'e2e-%'`,
  `DELETE FROM admin_usage_daily_rollups WHERE user_id LIKE 'e2e-%'`,
  `DELETE FROM plugin_session_summaries WHERE id LIKE 'e2e-%'`,
  `DELETE FROM plugin_usage_events WHERE id LIKE 'e2e-%'`,
  `DELETE FROM governance_decisions WHERE id LIKE 'e2e-%'`,
  `DELETE FROM ai_requests WHERE id LIKE 'e2e-%'`,
  `DELETE FROM ai_request_tool_calls WHERE request_id IN (${E2E_REQUEST_IDS})`,
  `DELETE FROM ai_request_messages WHERE request_id IN (${E2E_REQUEST_IDS})`,
  `DELETE FROM ai_request_payloads WHERE ai_request_id IN (${E2E_REQUEST_IDS})`,
  `DELETE FROM ai_safety_findings WHERE ai_request_id IN (${E2E_REQUEST_IDS})`,
  `DELETE FROM ai_requests WHERE user_id IN (${E2E_USER_IDS})`,
  `DELETE FROM user_contexts WHERE context_id LIKE 'e2e-%'`,
  `DELETE FROM user_commits WHERE user_id LIKE 'e2e-%'`,
  `DELETE FROM bridge_exchange_codes WHERE code_hash LIKE 'e2e-%'`,
  `DELETE FROM bridge_sessions WHERE session_id LIKE 'e2e-%'`,
  `DELETE FROM device_app_links WHERE device_id LIKE 'e2e-%'`,
  `DELETE FROM user_device_certs WHERE id LIKE 'e2e-%'`,
  `DELETE FROM user_api_keys WHERE id LIKE 'e2e-%'`,
  // usage_anomalies is a job-derived rollup keyed (metric, window_start) with
  // no ownership column. The seed claims the last day of it and nothing older.
  `DELETE FROM usage_anomalies WHERE window_start >= date_trunc('hour', NOW()) - INTERVAL '24 hours'`,
  `DELETE FROM user_sessions WHERE session_id LIKE 'e2e-%'`,
  `DELETE FROM user_profile_ext WHERE user_id LIKE 'e2e-%'`,
  `DELETE FROM users WHERE email LIKE '%@e2e.local'`,
];

async function reset(db: Client) {
  for (const stmt of RESET_STATEMENTS) {
    // secret_audit_log and a few side tables are optional on older databases;
    // a missing table must not abort a reset that has already deleted rows.
    try {
      await db.query(stmt);
    } catch (e) {
      if ((e as { code?: string }).code !== '42P01') throw e;
    }
  }
  try {
    await db.query(`DELETE FROM secret_audit_log WHERE id LIKE 'e2e-%'`);
  } catch (e) {
    if ((e as { code?: string }).code !== '42P01') throw e;
  }
}

export async function seed(opts: { reset?: boolean } = {}): Promise<MarketplaceAccessReport> {
  const db = new Client({ connectionString: databaseUrl() });
  await db.connect();
  try {
    await requireGroupsSchema(db);
    const marketplaceAccess = await requireDeclaredMarketplaceAccess(db);
    if (opts.reset) await reset(db);
    await seedPrincipals(db);
    await seedGroupsAndProjects(db);
    await seedMemberships(db);
    await recomputeScopeDefaults(db);
    await seedAccessControl(db);
    await seedSalesforceLinks(db);
    await seedPats(db);
    await seedAnalyticsTrail(db);
    await seedGroupTraffic(db);
    await seedTraffic(db);
    await seedGovernance(db);
    await seedBridge(db);
    return marketplaceAccess;
  } finally {
    await db.end();
  }
}

if (require.main === module) {
  seed({ reset: process.argv.includes('--reset') })
    .then(() => console.log('e2e seed complete'))
    .catch((e) => {
      console.error('e2e seed failed:', e.message);
      console.error(e.stack);
      process.exit(1);
    });
}
