// Groups, projects, memberships and the marketplace entitlement rules the
// access surfaces are asserted against.
import type { Client } from 'pg';
import { marketplaceAccess } from '../../tests/support/services-yaml';
import { E2E } from './principals';

// The UUID-keyed bridge principals have no `e2e-` id prefix, so the membership
// rows they own are found through their `@e2e.local` email instead.
export const E2E_USER_IDS = `SELECT id FROM users WHERE email LIKE '%@e2e.local'`;

type Source = 'adfs' | 'manual';

// `id` doubles as the slug the admin URLs use (/admin/groups/europe-devs).
// Groups are labelled by the content they work on, as in
// services/web/config/groups.yaml: europe-devs is the Commerce Cloud estate
// (Systemprompt-Commerce), india-devs the Salesforce Core build
// (Systemprompt-Core).
const GROUPS: { id: string; name: string; description: string; adGroups: string[] }[] = [
  {
    id: E2E.groups.europe,
    name: 'Europe devs',
    description: 'Commerce Cloud storefront and checkout engineers.',
    adGroups: ['Systemprompt-Commerce', 'Systemprompt-Commerce-Dev'],
  },
  {
    id: E2E.groups.india,
    name: 'India devs',
    description: 'Salesforce Core engineering, payments platform.',
    adGroups: ['Systemprompt-Core'],
  },
  {
    id: E2E.groups.business,
    name: 'Business users',
    description: 'Non-engineering staff; knowledge workers on the Cowork workspace.',
    adGroups: ['Systemprompt-Business'],
  },
];

const PROJECTS: { id: string; name: string; adGroups: string[] }[] = [
  { id: 'storefront', name: 'Storefront', adGroups: ['Systemprompt-Project-Storefront'] },
  { id: 'checkout', name: 'Checkout', adGroups: ['Systemprompt-Project-Checkout'] },
  { id: 'payments', name: 'Payments', adGroups: ['Systemprompt-Project-Payments'] },
  { id: 'platform', name: 'Platform', adGroups: ['Systemprompt-Project-Platform'] },
];

// A user absent from this table has no group_members row and is therefore in
// the DERIVED `unassigned` group (the user_groups view in migration 041), which
// is what the Unassigned page and the assign-to-group flow are built on.
//
// e2e-member-3 is deliberately in TWO groups: without one such person, nothing
// distinguishes "counted once against a primary group" from "counted in full in
// every group", which is the whole point of user_scope_defaults. It is member-3
// rather than a principal because the group-detail specs pin exactly which
// principals each group holds.
const GROUP_MEMBERSHIPS: { user: string; group: string; source: Source; adGroup: string | null }[] = [
  { user: 'e2e-developer', group: 'europe-devs', source: 'adfs', adGroup: 'Systemprompt-Commerce' },
  { user: 'e2e-user', group: 'europe-devs', source: 'manual', adGroup: null },
  { user: 'e2e-member-1', group: 'europe-devs', source: 'adfs', adGroup: 'Systemprompt-Commerce' },
  { user: 'e2e-member-2', group: 'europe-devs', source: 'adfs', adGroup: 'Systemprompt-Commerce' },
  { user: 'e2e-member-3', group: 'europe-devs', source: 'adfs', adGroup: 'Systemprompt-Commerce-Dev' },
  { user: 'e2e-member-3', group: 'india-devs', source: 'adfs', adGroup: 'Systemprompt-Core' },
  { user: 'e2e-pm', group: 'india-devs', source: 'manual', adGroup: null },
  { user: 'e2e-member-4', group: 'india-devs', source: 'adfs', adGroup: 'Systemprompt-Core' },
  { user: 'e2e-member-5', group: 'india-devs', source: 'adfs', adGroup: 'Systemprompt-Core' },
  { user: 'e2e-business-user', group: 'business-users', source: 'adfs', adGroup: 'Systemprompt-Business' },
  { user: 'e2e-member-6', group: 'business-users', source: 'adfs', adGroup: 'Systemprompt-Business' },
  { user: 'e2e-member-7', group: 'business-users', source: 'manual', adGroup: null },
  // The bridge principals sit in the groups their marketplace is granted to
  // (tests/bridge-manifest.spec.ts). The knowledge worker, the unassigned user
  // and the admin are deliberately in no group: their entitlement is a role or
  // nothing, and a stray membership would hide a regression in that band.
  { user: E2E.bridge.europe.id, group: 'europe-devs', source: 'adfs', adGroup: 'Systemprompt-Commerce' },
  { user: E2E.bridge.india.id, group: 'india-devs', source: 'adfs', adGroup: 'Systemprompt-Core' },
];

const PROJECT_MEMBERSHIPS: { user: string; project: string; source: Source; adGroup: string | null }[] = [
  { user: 'e2e-developer', project: 'storefront', source: 'adfs', adGroup: 'Systemprompt-Project-Storefront' },
  { user: 'e2e-member-1', project: 'storefront', source: 'adfs', adGroup: 'Systemprompt-Project-Storefront' },
  { user: 'e2e-member-2', project: 'storefront', source: 'adfs', adGroup: 'Systemprompt-Project-Storefront' },
  { user: 'e2e-member-3', project: 'checkout', source: 'adfs', adGroup: 'Systemprompt-Project-Checkout' },
  { user: 'e2e-pm', project: 'payments', source: 'manual', adGroup: null },
  { user: 'e2e-member-4', project: 'payments', source: 'adfs', adGroup: 'Systemprompt-Project-Payments' },
  { user: 'e2e-member-5', project: 'payments', source: 'adfs', adGroup: 'Systemprompt-Project-Payments' },
  { user: 'e2e-member-6', project: 'platform', source: 'manual', adGroup: null },
  { user: 'e2e-business-user', project: 'platform', source: 'adfs', adGroup: 'Systemprompt-Project-Platform' },
  { user: E2E.bridge.europe.id, project: 'storefront', source: 'adfs', adGroup: 'Systemprompt-Project-Storefront' },
];

// The marketplace entitlements themselves are NOT seeded. Every rule the
// suite relies on for the three private workspaces — europe-devs and the
// storefront project on astound-europe-dev, india-devs on astound-india-dev,
// the knowledge_worker role on astound-cowork — is declared in
// services/marketplaces/<id>/config.yaml (`access:`) and projected into
// access_control_rules by core's `ingest_marketplace_access` at every boot,
// keyed on (entity_type, entity_id, rule_type, rule_value) and preserving an
// existing row's id. An earlier version of this seed mirrored those rules with
// `e2e-rule-*` ids; the ingestion then updated onto the seed's rows, so the
// production-declared grants ended up carrying seed-owned ids and `--reset`
// deleted them from the shared database — every europe and india developer
// dropped to the baseline marketplace until the next boot.
//
// The seed therefore writes only rules that exist for the tests and nowhere in
// services/, and asserts the declared ones are present instead
// (requireDeclaredMarketplaceAccess). No rule here may target a marketplace,
// which is the whole surface the ingestion owns; seedAccessControl enforces
// that. The `plugin` entity rows are ingested from
// services/access-control/roles.yaml, so no entity row is written either.
const ACL_RULES: {
  id: string;
  type: 'plugin';
  entity: string;
  ruleType: string;
  ruleValue: string;
  access: string;
}[] = [
  // The admin plugin is denied to business users outright — a deny in the
  // group band on top of roles.yaml's admin-only role grant, so the deny path
  // of the resolver is exercised by a principal who holds no admin role anyway.
  { id: 'e2e-rule-admin-business-deny', type: 'plugin', entity: 'astound-admin', ruleType: 'group', ruleValue: E2E.groups.business, access: 'deny' },
];

export async function seedGroupsAndProjects(db: Client) {
  for (const g of GROUPS) {
    await db.query(
      `INSERT INTO groups (id, name, description, is_system, source)
       VALUES ($1, $2, $3, false, 'dashboard')
       ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, description = EXCLUDED.description`,
      [g.id, g.name, g.description],
    );
    for (const ad of g.adGroups) {
      await db.query(
        `INSERT INTO group_ad_mappings (ad_group, group_id, source)
         VALUES ($1, $2, 'dashboard') ON CONFLICT DO NOTHING`,
        [ad, g.id],
      );
    }
  }
  for (const p of PROJECTS) {
    await db.query(
      `INSERT INTO projects (id, name, source) VALUES ($1, $2, 'dashboard')
       ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name`,
      [p.id, p.name],
    );
    for (const ad of p.adGroups) {
      await db.query(
        `INSERT INTO project_ad_mappings (ad_group, project_id, source)
         VALUES ($1, $2, 'dashboard') ON CONFLICT DO NOTHING`,
        [ad, p.id],
      );
    }
  }
}

export async function seedMemberships(db: Client) {
  // Delete first so a re-run after an in-test reassignment (the assign-group
  // spec moves e2e-unassigned into business-users) restores the seeded shape.
  await db.query(`DELETE FROM group_members WHERE user_id LIKE 'e2e-%' OR user_id IN (${E2E_USER_IDS})`);
  await db.query(`DELETE FROM project_members WHERE user_id LIKE 'e2e-%' OR user_id IN (${E2E_USER_IDS})`);
  for (const m of GROUP_MEMBERSHIPS) {
    await db.query(
      `INSERT INTO group_members (group_id, user_id, source, source_ad_group)
       VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING`,
      [m.group, m.user, m.source, m.adGroup],
    );
  }
  for (const m of PROJECT_MEMBERSHIPS) {
    await db.query(
      `INSERT INTO project_members (project_id, user_id, source, source_ad_group)
       VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING`,
      [m.project, m.user, m.source, m.adGroup],
    );
  }
}

// The Salesforce Hosted MCP server is narrowed to linked users by the
// `salesforce: linked` dimension (services/access-control/salesforce.yaml), so
// a knowledge worker who has not linked a Salesforce Username never sees it in
// the manifest. Both knowledge workers are linked here; the developers and the
// admin are not, which is what keeps `salesforce` out of their entitlements.
export async function seedSalesforceLinks(db: Client) {
  for (const user of [E2E.bridge.knowledgeWorker.id, E2E.businessUser.id]) {
    await db.query(
      `INSERT INTO salesforce_user_identities (user_id, sf_username)
       VALUES ($1, $2) ON CONFLICT (user_id) DO UPDATE SET sf_username = EXCLUDED.sf_username`,
      [user, `${user}@e2e.local.salesforce`],
    );
  }
}

// The guard that keeps this seed honest about what it does not own: every
// rule a marketplace descriptor declares must already be in the database, with
// an id the ingestion minted rather than one of ours. A fresh database is
// deterministic because boot is the step that writes these rows; a seed that
// wrote them itself would only re-create the ownership defect above.
// Why this exists rather than a fixed sentence: "boot the server so the blocks
// are ingested" reads as "your stack has not started yet", and sends someone
// whose stack is up and has booted many times to restart it again. The two
// causes are distinguishable from the data. If no rule of this rule_type exists
// for ANY marketplace, nothing has ever ingested that shape and the fault is in
// the ingestion, not in this database's freshness.
async function ingestionAdvice(
  db: Client,
  missing: string[],
): Promise<{ text: string; neverIngested: boolean }> {
  const types = [...new Set(missing.map((m) => m.split(': ')[1]?.split('=')[0]).filter(Boolean))];
  const absent: string[] = [];
  for (const ruleType of types) {
    const { rows } = await db.query<{ n: string }>(
      `SELECT count(*)::text AS n FROM access_control_rules
        WHERE entity_type = 'marketplace' AND rule_type = $1`,
      [ruleType],
    );
    if (rows[0]?.n === '0') absent.push(ruleType);
  }
  if (absent.length > 0) {
    return {
      neverIngested: true,
      text:
      `No marketplace rule of type ${absent.join('/')} exists for ANY marketplace, so this is not ` +
      `a stale database: nothing has ever ingested that rule shape. Check whether the startup ` +
      `ingestion handles it — rebooting will not help. Note the declaring configs set ` +
      `default_included: false, so while these rules are absent those workspaces are gated by the ` +
      `entity fallthrough alone, which is worth establishing before shipping.`,
    };
  }
  return {
    neverIngested: false,
    text: `Boot the server so the marketplace access blocks are ingested (core ingests them at startup), then re-run the seed.`,
  };
}

/** What the marketplace-access precondition found, for the caller to act on. */
export interface MarketplaceAccessReport {
  /** True when every declared block is present and none is seed-owned. */
  satisfied: boolean;
  /** Set when the rule shape exists for no marketplace at all — an ingestion gap. */
  neverIngested: boolean;
  detail: string;
}

export async function requireDeclaredMarketplaceAccess(db: Client): Promise<MarketplaceAccessReport> {
  const missing: string[] = [];
  const seedOwned: string[] = [];
  for (const [marketplace, access] of marketplaceAccess()) {
    const expected: { ruleType: string; ruleValue: string; access: string }[] = [
      ...access.roles.map((role) => ({ ruleType: 'role', ruleValue: role, access: 'allow' })),
      ...access.rules.flatMap((r) => r.values.map((v) => ({ ruleType: r.ruleType, ruleValue: v, access: r.access }))),
    ];
    const entity = await db.query<{ default_included: boolean }>(
      `SELECT default_included FROM access_control_entities
        WHERE entity_type = 'marketplace' AND entity_id = $1`,
      [marketplace],
    );
    if (entity.rowCount === 0 || entity.rows[0].default_included !== access.defaultIncluded) {
      missing.push(`${marketplace}: entity row (default_included=${access.defaultIncluded})`);
    }
    for (const e of expected) {
      const { rows } = await db.query<{ id: string }>(
        `SELECT id FROM access_control_rules
          WHERE entity_type = 'marketplace' AND entity_id = $1
            AND rule_type = $2 AND rule_value = $3 AND access = $4`,
        [marketplace, e.ruleType, e.ruleValue, e.access],
      );
      const label = `${marketplace}: ${e.ruleType}=${e.ruleValue} (${e.access})`;
      if (rows.length === 0) missing.push(label);
      else if (rows[0].id.startsWith('e2e-')) seedOwned.push(`${label} id=${rows[0].id}`);
    }
  }
  // Why the seedOwned branch below still throws and this one may not: they have
  // different owners and different risks. A production-declared grant carrying an
  // e2e- id is a fault the seed caused and can cause again, where the next
  // --reset silently deletes real access — stopping the run is proportionate.
  //
  // A rule shape that exists for NO marketplace was not caused by the seed, cannot
  // be fixed by it, and is not made worse by the rest of the suite running. What a
  // hard stop makes worse is that nobody can test anything, including whoever is
  // fixing the ingestion. So that case reports and the entitlement-dependent specs
  // skip themselves; every other shortfall is drift and still stops the run.
  if (missing.length > 0) {
    const advice = await ingestionAdvice(db, missing);
    const detail =
      `the marketplace access blocks declared in services/marketplaces/*/config.yaml are not in ` +
      `the database — missing ${missing.join('; ')}. ${advice.text}`;
    if (!advice.neverIngested) throw new Error(`e2e seed: ${detail}`);
    return { satisfied: false, neverIngested: true, detail };
  }
  if (seedOwned.length > 0) {
    throw new Error(
      `e2e seed: YAML-declared marketplace rules carry seed-owned ids — ${seedOwned.join('; ')}. ` +
        `Delete those rows and reboot the server so the ingestion re-creates them with its own ids; the seed will not adopt them.`,
    );
  }
  return { satisfied: true, neverIngested: false, detail: '' };
}

export async function seedAccessControl(db: Client) {
  for (const r of ACL_RULES) {
    // Belt and braces for the comment above ACL_RULES: a marketplace-scoped
    // rule here would be adopted by the boot ingestion and deleted by --reset.
    if ((r.type as string) === 'marketplace') {
      throw new Error(`e2e seed: ${r.id} targets a marketplace; those rules belong to services/marketplaces`);
    }
    await db.query(
      `INSERT INTO access_control_rules
           (id, entity_type, entity_id, rule_type, rule_value, access, justification)
       VALUES ($1, $2, $3, $4, $5, $6, 'e2e fixture')
       ON CONFLICT (entity_type, entity_id, rule_type, rule_value)
       DO UPDATE SET access = EXCLUDED.access`,
      [r.id, r.type, r.entity, r.ruleType, r.ruleValue, r.access],
    );
  }
}

// Why the seed runs this itself: memberships are written here with direct SQL,
// which is the one path into group_members that does not go through the code
// that recomputes attribution. Left alone, user_scope_defaults stays empty and
// every Exclusive total on the dashboards renders 0 with 100% unattributed —
// a wrong number, not a missing one, which is the worse failure.
//
// The statement is recompute_scope_defaults verbatim from
// extensions/web/admin/src/repositories/scope/defaults.rs. Copying it is a
// deliberate duplication: the seed must produce exactly what the job produces,
// so a paraphrase that drifts would make the suite agree with itself and
// disagree with production. Its final WHERE clause is what preserves an
// operator's source='manual' row.
export async function recomputeScopeDefaults(db: Client): Promise<number> {
  const r = await db.query(
    `INSERT INTO user_scope_defaults
         (user_id, primary_group_id, primary_project_id, source, updated_at)
     SELECT u.id,
            (SELECT ug.group_id
               FROM user_groups ug
              WHERE ug.user_id = u.id
              ORDER BY EXISTS (
                           SELECT 1 FROM group_members gm
                           WHERE gm.group_id = ug.group_id
                             AND gm.user_id = ug.user_id
                             AND gm.source = 'adfs') DESC,
                       (SELECT COUNT(DISTINCT x.user_id)
                          FROM user_groups x
                         WHERE x.group_id = ug.group_id) DESC,
                       ug.group_id
              LIMIT 1),
            (SELECT pm.project_id
               FROM project_members pm
              WHERE pm.user_id = u.id
              ORDER BY (pm.source = 'adfs') DESC,
                       (SELECT COUNT(DISTINCT y.user_id)
                          FROM project_members y
                         WHERE y.project_id = pm.project_id) DESC,
                       pm.project_id
              LIMIT 1),
            'auto', NOW()
     FROM users u
     WHERE NOT ('anonymous' = ANY(u.roles))
     ON CONFLICT (user_id) DO UPDATE SET
         primary_group_id = EXCLUDED.primary_group_id,
         primary_project_id = EXCLUDED.primary_project_id,
         updated_at = NOW()
     WHERE user_scope_defaults.source = 'auto'`,
  );
  return r.rowCount ?? 0;
}

export const PEOPLE_DATASET = { GROUPS, PROJECTS, GROUP_MEMBERSHIPS, PROJECT_MEMBERSHIPS, ACL_RULES };
